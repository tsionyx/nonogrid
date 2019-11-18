use std::fmt;
use std::ops::{Add, Sub};

use crate::block::base::{
    color::{ColorId, ColorPalette},
    Block, Color,
};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum BinaryColor {
    Undefined,
    White,
    Black,
    // special value for DynamicSolver
    BlackOrWhite,
}

impl Default for BinaryColor {
    fn default() -> Self {
        Self::Undefined
    }
}

impl Color for BinaryColor {
    fn blank() -> Self {
        Self::White
    }

    fn is_solved(&self) -> bool {
        *self == Self::Black || *self == Self::White
    }

    fn solution_rate(&self, _all_colors: &[ColorId]) -> f64 {
        if self.is_solved() {
            1.0
        } else {
            0.0
        }
    }

    fn variants(&self) -> Vec<Self> {
        if self.is_solved() {
            vec![*self]
        } else {
            vec![Self::White, Self::Black]
        }
    }

    fn as_color_id(&self) -> Option<ColorId> {
        None
    }

    fn from_color_ids(ids: &[ColorId]) -> Self {
        if ids == [ColorPalette::WHITE_ID] {
            Self::Undefined
        } else {
            Self::Black
        }
    }
}

impl fmt::Display for BinaryColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BinaryColor::*;

        let symbol = match self {
            White => '.',
            Black => '\u{2b1b}',
            Undefined | BlackOrWhite => '?',
        };
        write!(f, "{}", symbol)
    }
}

impl Add for BinaryColor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        rhs
    }
}

impl Sub for BinaryColor {
    type Output = Result<Self, String>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_solved() {
            return Err(format!("Cannot unset already set cell {:?}", self));
        }

        Ok(match rhs {
            Self::Black => Self::White,
            Self::White => Self::Black,
            _ => self,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct BinaryBlock(pub usize);

impl Block for BinaryBlock {
    type Color = BinaryColor;

    fn from_size_and_color(size: usize, _color: Option<ColorId>) -> Self {
        Self(size)
    }

    fn partial_sums(desc: &[Self]) -> Vec<usize> {
        desc.iter()
            .scan(None, |prev, block| {
                let current = if let Some(ref prev_size) = prev {
                    prev_size + block.0 + 1
                } else {
                    block.0
                };
                *prev = Some(current);
                *prev
            })
            .collect()
    }

    fn size(&self) -> usize {
        self.0
    }

    fn color(&self) -> Self::Color {
        BinaryColor::Black
    }
}

impl fmt::Display for BinaryBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{Block, Description};

    use super::BinaryBlock;

    #[test]
    fn partial_sums_empty() {
        let d = Description::new(vec![]);
        assert_eq!(BinaryBlock::partial_sums(&d.vec), Vec::<usize>::new());
    }

    #[test]
    fn partial_sums_single() {
        let d = Description::new(vec![BinaryBlock(5)]);
        assert_eq!(BinaryBlock::partial_sums(&d.vec), vec![5]);
    }

    #[test]
    fn check_partial_sums() {
        let d = Description::new(vec![BinaryBlock(1), BinaryBlock(2), BinaryBlock(3)]);
        assert_eq!(BinaryBlock::partial_sums(&d.vec), vec![1, 4, 8]);
    }
}
