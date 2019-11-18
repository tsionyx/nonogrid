use std::fmt;
use std::ops::{Add, Sub};

use hashbrown::HashSet;

use crate::block::base::{
    color::{ColorId, ColorPalette},
    Block, Color,
};
use crate::utils::{from_two_powers, two_powers};

#[derive(Debug, PartialEq, Eq, Hash, Default, Copy, Clone, PartialOrd, Ord)]
pub struct MultiColor(pub ColorId);

impl Color for MultiColor {
    fn blank() -> Self {
        Self(ColorPalette::WHITE_ID)
    }

    fn is_solved(&self) -> bool {
        self.0.is_power_of_two()
    }

    fn memoize_rate() -> bool {
        true
    }
    /// Calculate the rate of the given cell.
    /// The formula is like that:
    ///   `rate = (N - n) / (N - 1)`, where
    ///    N = full puzzle color set
    ///    n = current color set for given cell,
    ///
    ///    in particular:
    ///    a) when the cell is completely unsolved
    ///       rate = (N - N) / (N - 1) = 0
    ///    b) when the cell is solved
    ///       rate = (N - 1) / (N - 1) = 1
    fn solution_rate(&self, all_colors: &[ColorId]) -> f64 {
        let all_colors: HashSet<_> = all_colors.iter().cloned().collect();
        let cell_colors = self.variants_as_ids();
        let current_size = cell_colors.intersection(&all_colors).count();

        if current_size == 0 {
            return 0.0;
        }
        if current_size == 1 {
            return 1.0;
        }

        let full_size = all_colors.len();
        let rate = full_size - current_size;
        let normalized_rate = rate as f64 / (full_size - 1) as f64;
        assert!(normalized_rate >= 0.0 && normalized_rate <= 1.0);

        normalized_rate
    }

    fn variants(&self) -> Vec<Self> {
        two_powers(self.0).map(Self).collect()
    }

    fn as_color_id(&self) -> Option<ColorId> {
        Some(self.0)
    }

    fn from_color_ids(ids: &[ColorId]) -> Self {
        Self(from_two_powers(ids))
    }
}

impl MultiColor {
    fn variants_as_ids(self) -> HashSet<ColorId> {
        two_powers(self.0).collect()
    }
}

impl Add for MultiColor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        rhs
    }
}

impl Sub for MultiColor {
    type Output = Result<Self, String>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_solved() {
            return Err(format!(
                "Cannot unset {:?} from already set cell {:?}",
                rhs, self
            ));
        }

        let colors = self.variants_as_ids();
        let bad_state = rhs.variants_as_ids();
        debug!("Previous state: {:?}", colors);
        debug!("Bad state: {:?}", bad_state);

        let new_value: HashSet<_> = colors.difference(&bad_state).cloned().collect();

        if !new_value.is_empty() && new_value.is_subset(&colors) {
            let new_state: Vec<_> = new_value.into_iter().collect();
            debug!("New state: {:?}", new_state);
            Ok(Self(from_two_powers(&new_state)))
        } else {
            Err(format!(
                "Cannot unset the colors {:?} from {:?}",
                bad_state, colors
            ))
        }
    }
}

impl fmt::Display for MultiColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let colors: Vec<_> = self.variants_as_ids().into_iter().collect();
        let symbol = if colors.len() == 1 {
            format!("{}", colors[0])
        } else {
            "?".to_string()
        };
        write!(f, "{}", symbol)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct ColoredBlock {
    size: usize,
    color: ColorId,
}

impl ColoredBlock {
    pub const fn from_size_and_color(size: usize, color: ColorId) -> Self {
        Self { size, color }
    }
}

impl Block for ColoredBlock {
    type Color = MultiColor;

    fn from_size_and_color(size: usize, color: Option<ColorId>) -> Self {
        let color = color.expect("Color not provided for ColoredBlock");
        Self { size, color }
    }

    fn partial_sums(desc: &[Self]) -> Vec<usize> {
        desc.iter()
            .scan(None, |prev: &mut Option<Self>, block| {
                let current = if let Some(ref prev) = prev {
                    let sum = prev.size() + block.size();
                    if prev.color() == block.color() {
                        sum + 1
                    } else {
                        sum
                    }
                } else {
                    block.size()
                };

                *prev = Some(Self::from_size_and_color(current, block.color));
                Some(current)
            })
            .collect()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn color(&self) -> Self::Color {
        MultiColor(self.color)
    }
}

impl fmt::Display for ColoredBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.size)
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{Block, Description};

    use super::ColoredBlock;

    #[test]
    fn partial_sums_empty() {
        let d = Description::new(vec![]);
        assert_eq!(ColoredBlock::partial_sums(&d.vec), Vec::<usize>::new());
    }

    #[test]
    fn partial_sums_single() {
        let d = Description::new(vec![ColoredBlock::from_size_and_color(5, 1)]);
        assert_eq!(ColoredBlock::partial_sums(&d.vec), vec![5]);
    }

    #[test]
    fn check_partial_sums() {
        let d = Description::new(vec![
            ColoredBlock::from_size_and_color(1, 1),
            ColoredBlock::from_size_and_color(2, 1),
            ColoredBlock::from_size_and_color(3, 2),
        ]);
        assert_eq!(ColoredBlock::partial_sums(&d.vec), vec![1, 4, 7]);
    }
}
