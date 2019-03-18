use super::base::{Block, Color};

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd)]
pub enum BinaryColor {
    Undefined,
    White,
    Black,
    // especially for DynamicSolver
    BlackOrWhite,
}

impl Color for BinaryColor {
    fn initial() -> Self {
        BinaryColor::Undefined
    }
    fn blank() -> Self {
        BinaryColor::White
    }

    fn is_solved(&self) -> bool {
        self == &BinaryColor::Black || self == &BinaryColor::White
    }

    fn solution_rate(&self) -> f64 {
        if self.is_solved() {
            1.0
        } else {
            0.0
        }
    }

    fn is_updated_with(&self, new: &Self) -> Result<bool, String> {
        if self == new {
            return Ok(false);
        }

        if self != &BinaryColor::Undefined {
            return Err("Can only update undefined".to_string());
        }
        if !new.is_solved() {
            return Err("Cannot update already solved".to_string());
        }

        Ok(true)
    }

    fn variants(&self) -> HashSet<Self> {
        if self.is_solved() {
            vec![*self]
        } else {
            vec![BinaryColor::White, BinaryColor::Black]
        }
        .into_iter()
        .collect()
    }
}

impl fmt::Display for BinaryColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BinaryColor::*;

        let symbol = match self {
            Undefined => '?',
            White => '.',
            Black => '\u{2b1b}',
            BlackOrWhite => '?',
        };
        write!(f, "{}", symbol)
    }
}

impl BinaryColor {
    fn order(self) -> u8 {
        match self {
            BinaryColor::Undefined => 0,
            BinaryColor::White => 1,
            BinaryColor::Black => 2,
            _ => 3,
        }
    }
}

impl Ord for BinaryColor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order().cmp(&other.order())
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
            return Err(format!("Cannot unset already set cell {:?}", &self));
        }

        Ok(match rhs {
            BinaryColor::Black => BinaryColor::White,
            BinaryColor::White => BinaryColor::Black,
            _ => self,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct BinaryBlock(pub usize);

impl Block for BinaryBlock {
    type Color = BinaryColor;

    fn from_str(s: &str) -> Self {
        Self(s.parse::<usize>().unwrap())
    }

    fn partial_sums(desc: &[Self]) -> Vec<usize> {
        if desc.is_empty() {
            return vec![];
        }

        desc.iter()
            .fold(Vec::with_capacity(desc.len()), |mut acc, block| {
                if acc.is_empty() {
                    vec![block.0]
                } else {
                    let last = acc.last().unwrap();
                    acc.push(last + block.0 + 1);
                    acc
                }
            })
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
