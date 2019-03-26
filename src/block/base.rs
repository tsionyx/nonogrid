use super::super::block::base::color::ColorId;
use super::super::utils;

use std::fmt::Debug;
use std::hash::Hash;
use std::marker::Sized;
use std::ops::{Add, Sub};

use hashbrown::HashSet;

pub trait Color
where
    Self: Debug
        + PartialEq
        + Eq
        + Hash
        + Default
        + Copy
        + Clone
        + PartialOrd
        + Ord
        + Add<Output = Self>
        + Sub<Output = Result<Self, String>>,
{
    fn blank() -> Self;
    fn is_solved(&self) -> bool;
    fn solution_rate(&self, all_colors: &[ColorId]) -> f64;
    fn is_updated_with(&self, new: &Self) -> Result<bool, String>;
    fn variants(&self) -> HashSet<Self>
    where
        Self: Sized;

    fn as_color_id(&self) -> ColorId;
    fn from_color_ids(ids: &[ColorId]) -> Self;
}

pub trait Block
where
    Self: Debug + PartialEq + Eq + Hash + Default + Clone,
{
    type Color: Color;

    fn from_str_and_color(s: &str, color: Option<ColorId>) -> Self;
    fn partial_sums(desc: &[Self]) -> Vec<usize>
    where
        Self: Sized;

    fn size(&self) -> usize;
    fn color(&self) -> Self::Color;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Description<T: Block>
where
    T: Block,
{
    pub vec: Vec<T>,
}

impl<T> Description<T>
where
    T: Block,
{
    pub fn new(mut vec: Vec<T>) -> Self {
        // remove zero blocks
        utils::remove(&mut vec, &T::default());
        Self { vec }
    }
}

pub mod color {
    use std::collections::{HashMap, HashSet};

    #[derive(Debug, PartialEq, Clone)] //, Eq, Hash, Copy, Clone, PartialOrd)]
    pub enum ColorValue {
        // "red", "blue", "pink"
        CommonName(String),
        // (0, 255, 0) for green
        RgbTriplet(u8, u8, u8),
        // 0xFF0 for yellow
        HexValue3(u16),
        // 0xFF00FF for magenta
        HexValue6(u32),
    }

    /// ```
    /// use nonogrid::block::base::color::ColorValue;
    ///
    /// assert_eq!(ColorValue::parse("0F0"), ColorValue::HexValue3(240));
    /// assert_eq!(ColorValue::parse("0000FF"), ColorValue::HexValue6(255));
    /// assert_eq!(ColorValue::parse("white"), ColorValue::CommonName("white".to_string()));
    /// assert_eq!(ColorValue::parse("200, 16,0  "), ColorValue::RgbTriplet(200, 16, 0));
    /// // invalid triplet: G component is not an u8
    /// assert_eq!(ColorValue::parse("200, X, 16"), ColorValue::CommonName("200, X, 16".to_string()));
    /// ```
    impl ColorValue {
        pub fn parse(value: &str) -> Self {
            if value.len() == 3 {
                let hex3 = u16::from_str_radix(value, 16);
                if let Ok(hex3) = hex3 {
                    return ColorValue::HexValue3(hex3);
                }
            }

            if value.len() == 6 {
                let hex6 = u32::from_str_radix(value, 16);
                if let Ok(hex6) = hex6 {
                    return ColorValue::HexValue6(hex6);
                }
            }

            let rgb: Vec<_> = value.split(',').collect();
            if rgb.len() == 3 {
                let rgb: Vec<_> = rgb
                    .iter()
                    .filter_map(|component| component.trim().parse::<u8>().ok())
                    .collect();

                if rgb.len() == 3 {
                    return ColorValue::RgbTriplet(rgb[0], rgb[1], rgb[2]);
                }
            }

            ColorValue::CommonName(value.to_string())
        }
    }

    pub type ColorId = u32;

    #[derive(Clone)]
    pub struct ColorDesc {
        id: ColorId,
        name: String,
        value: ColorValue,
        symbol: char,
    }

    impl ColorDesc {
        /// used in ShellRenderer
        pub fn symbol(&self) -> String {
            self.symbol.to_string()
        }

        pub fn name(&self) -> &str {
            self.name.as_str()
        }
    }

    #[derive(Clone)]
    pub struct ColorPalette {
        vec: HashMap<String, ColorDesc>,
        symbols: HashSet<char>,
        default_color: Option<String>,
    }

    impl ColorPalette {
        pub const WHITE_ID: ColorId = 1;

        pub fn with_white_and_black(white_name: &str, black_name: &str) -> Self {
            let mut this = Self::new();
            this.color_with_name_value_symbol_and_id(
                white_name,
                ColorValue::HexValue3(0xFFF),
                '.',
                Self::WHITE_ID,
            );
            this.color_with_name_value_and_symbol(black_name, ColorValue::HexValue3(0x000), 'X');
            this.set_default(black_name);

            this
        }

        fn new() -> Self {
            Self::with_colors(HashMap::new())
        }

        fn with_colors(colors: HashMap<String, ColorDesc>) -> Self {
            let symbols: HashSet<_> = (0_u8..0xFF)
                .filter_map(|x| {
                    let ch = x as char;
                    if ch.is_ascii_punctuation() {
                        Some(ch)
                    } else {
                        None
                    }
                })
                .collect();

            Self {
                vec: colors,
                symbols,
                default_color: None,
            }
        }

        pub fn set_default(&mut self, color_name: &str) -> bool {
            if self.vec.contains_key(color_name) {
                self.default_color = Some(color_name.to_string());
                return true;
            }

            false
        }

        pub fn get_default(&self) -> Option<String> {
            self.default_color.clone()
        }

        pub fn id_by_name(&self, name: &str) -> Option<ColorId> {
            self.vec.get(name).map(|desc| desc.id)
        }

        pub fn desc_by_id(&self, id: ColorId) -> Option<ColorDesc> {
            self.vec.iter().find_map(|(_name, color_desc)| {
                if color_desc.id == id {
                    Some(color_desc.clone())
                } else {
                    None
                }
            })
        }

        fn add(&mut self, color: ColorDesc) {
            self.vec.insert(color.name.clone(), color);
        }

        pub fn color_with_name_value_symbol_and_id(
            &mut self,
            name: &str,
            value: ColorValue,
            symbol: char,
            id: ColorId,
        ) {
            let new = ColorDesc {
                id,
                name: name.to_string(),
                value,
                symbol,
            };
            if !self.vec.contains_key(name) {
                self.add(new);
            }
        }

        pub fn color_with_name_value_and_symbol(
            &mut self,
            name: &str,
            value: ColorValue,
            symbol: char,
        ) {
            let current_max = self.vec.iter().map(|(_name, color)| color.id).max();
            let id = current_max.map_or(1, |val| val * 2);
            self.color_with_name_value_symbol_and_id(name, value, symbol, id)
        }

        #[allow(dead_code)]
        pub fn color_with_name_and_value(&mut self, name: &str, value: ColorValue) {
            let next_symbol = self
                .vec
                .iter()
                .filter_map(|(_name, color)| {
                    let symbol = color.symbol;
                    if self.symbols.contains(&symbol) {
                        None
                    } else {
                        Some(symbol)
                    }
                })
                .next()
                .expect("Cannot create color: No more symbols available.");

            self.color_with_name_value_and_symbol(name, value, next_symbol)
        }
    }
}
