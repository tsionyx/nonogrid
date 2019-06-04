use crate::block::base::color::ColorId;
use crate::utils;

use std::fmt::Debug;
use std::hash::Hash;
use std::marker::Sized;
use std::ops::{Add, Sub};

pub trait Color
where
    Self: Debug
        + Eq
        + Hash
        + Default
        + Copy
        + Send
        + Sync
        + Ord
        + Add<Output = Self>
        + Sub<Output = Result<Self, String>>,
{
    fn blank() -> Self;
    fn is_solved(&self) -> bool;
    fn memoize_rate() -> bool {
        false
    }
    fn solution_rate(&self, all_colors: &[ColorId]) -> f64;
    fn is_updated_with(&self, new: &Self) -> Result<bool, String>;
    fn variants(&self) -> Vec<Self>
    where
        Self: Sized;

    fn as_color_id(&self) -> Option<ColorId>;
    fn from_color_ids(ids: &[ColorId]) -> Self;
}

pub trait Block
where
    Self: Debug + Eq + Hash + Default + Clone + Send + Sync,
{
    type Color: Color;

    fn from_str_and_color(s: &str, color: Option<ColorId>) -> Self {
        let size = s.parse::<usize>().expect("Non-integer block size given");
        Self::from_size_and_color(size, color)
    }

    fn from_size_and_color(size: usize, color: Option<ColorId>) -> Self;
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

/// Generate clues for the given line of color codes
fn line_clues<B>(line: &[color::ColorId], blank_code: color::ColorId) -> Description<B>
where
    B: Block,
{
    let size = line.len();

    let mut description = vec![];

    let mut index = 0;
    while index < size {
        let block_begin = index;
        let color_number = line[index];

        while (index < size) && (line[index] == color_number) {
            index += 1;
        }

        let block_size = index - block_begin;
        if (block_size > 0) && (color_number != blank_code) {
            let block = B::from_size_and_color(block_size, Some(color_number));
            description.push(block);
        }
    }
    Description::new(description)
}

/// Generate nonogram description (columns and rows) from a solution matrix.
pub fn clues_from_solution<B>(
    solution_matrix: &[Vec<color::ColorId>],
    blank_code: color::ColorId,
) -> (Vec<Description<B>>, Vec<Description<B>>)
where
    B: Block,
{
    let height = solution_matrix.len();
    if height == 0 {
        return (vec![], vec![]);
    }

    let width = solution_matrix[0].len();
    if width == 0 {
        return (vec![], vec![]);
    }

    let columns = (0..width)
        .map(|col_index| {
            let column: Vec<_> = (0..height)
                .map(|row_index| solution_matrix[row_index][col_index])
                .collect();
            line_clues(&column, blank_code)
        })
        .collect();
    let rows = (0..height)
        .map(|row_index| {
            let row = &solution_matrix[row_index];
            line_clues(row, blank_code)
        })
        .collect();
    (columns, rows)
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

        /// ```
        /// use nonogrid::block::base::color::ColorValue;
        ///
        /// assert_eq!(ColorValue::parse("0F0").to_rgb(), (0, 255, 0));
        /// assert_eq!(ColorValue::parse("0000FF").to_rgb(), (0, 0, 255));
        /// assert_eq!(ColorValue::parse("red").to_rgb(), (255, 0, 0));
        /// assert_eq!(ColorValue::parse("YELLOW").to_rgb(), (255, 255, 0));
        /// assert_eq!(ColorValue::parse("teal").to_rgb(), (0, 128, 128));
        /// assert_eq!(ColorValue::parse("unknown").to_rgb(), (0, 0, 0));
        /// assert_eq!(ColorValue::parse("200, 16,0  ").to_rgb(), (200, 16, 0));
        /// // short form
        /// assert_eq!(ColorValue::parse("55bb88").to_rgb(), ColorValue::parse("5b8").to_rgb());
        /// ```
        pub fn to_rgb(&self) -> (u8, u8, u8) {
            match self {
                ColorValue::RgbTriplet(r, g, b) => (*r, *g, *b),
                ColorValue::HexValue3(hex3) => {
                    let (r, gb) = (hex3 / 256, hex3 % 256);
                    let (g, b) = (gb / 16, gb % 16);

                    ((r * 17) as u8, (g * 17) as u8, (b * 17) as u8)
                }
                ColorValue::HexValue6(hex6) => {
                    let (r, gb) = (hex6 / (1 << 16), hex6 % (1 << 16));
                    let (g, b) = (gb / 256, gb % 256);

                    (r as u8, g as u8, b as u8)
                }
                // https://www.rapidtables.com/web/color/RGB_Color.html#color-table
                ColorValue::CommonName(name) => match name.to_lowercase().as_str() {
                    "black" => (0, 0, 0),
                    "white" => (255, 255, 255),
                    "red" => (255, 0, 0),
                    "lime" => (0, 255, 0),
                    "blue" => (0, 0, 255),
                    "yellow" => (255, 255, 0),
                    "cyan" => (0, 255, 255),
                    "aqua" => (0, 255, 255),
                    "magenta" => (255, 0, 255),
                    "fuchsia" => (255, 0, 255),
                    "silver" => (192, 192, 192),
                    "gray" => (128, 128, 128),
                    "maroon" => (128, 0, 0),
                    "olive" => (128, 128, 0),
                    "green" => (0, 128, 0),
                    "purple" => (128, 0, 128),
                    "teal" => (0, 128, 128),
                    "navy" => (0, 0, 128),
                    _unknown_color => (0, 0, 0),
                },
            }
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

        pub fn rgb_value(&self) -> (u8, u8, u8) {
            self.value.to_rgb()
        }
    }

    #[derive(Clone)]
    pub struct ColorPalette {
        vec: HashMap<String, ColorDesc>,
        symbols: Vec<char>,
        default_color: Option<String>,
    }

    impl ColorPalette {
        pub const WHITE_ID: ColorId = 1;

        pub fn with_white_and_black(white_name: &str, black_name: &str) -> Self {
            let mut this = Self::with_white(white_name);
            this.color_with_name_value_and_symbol(black_name, ColorValue::HexValue3(0x000), 'X');
            this.set_default(black_name);
            this
        }

        pub fn with_white(white_name: &str) -> Self {
            let mut this = Self::new();
            this.color_with_name_value_symbol_and_id(
                white_name,
                ColorValue::HexValue3(0xFFF),
                '.',
                Self::WHITE_ID,
            );

            this
        }

        fn new() -> Self {
            Self::with_colors(HashMap::new())
        }

        fn with_colors(colors: HashMap<String, ColorDesc>) -> Self {
            let symbols: Vec<_> = (0_u8..0xFF)
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
            let occupied_symbols: HashSet<_> =
                self.vec.iter().map(|(_name, color)| color.symbol).collect();

            let next_symbol = self
                .symbols
                .iter()
                .find_map(|available_symbol| {
                    if occupied_symbols.contains(&available_symbol) {
                        None
                    } else {
                        Some(available_symbol)
                    }
                })
                .expect("Cannot create color: No more symbols available.");

            self.color_with_name_value_and_symbol(name, value, *next_symbol)
        }
    }
}
