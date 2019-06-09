use crate::block::{
    base::{
        clues_from_solution,
        color::{ColorId, ColorPalette, ColorValue},
    },
    Block, Description,
};
use crate::board::Board;
use crate::utils::{
    iter::FindOk,
    product,
    rc::{mutate_ref, read_ref, InteriorMutableRef},
};

use std::fs;
use std::io;

use hashbrown::HashMap;
use sxd_xpath::{
    evaluate_xpath,
    nodeset::{Node, Nodeset},
    Value,
};

#[cfg(feature = "web")]
extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate toml;

#[derive(Debug)]
pub struct ParseError(pub String);

pub trait BoardParser {
    fn with_content(content: String) -> Result<Self, ParseError>
    where
        Self: Sized;

    fn parse<B>(&self) -> Board<B>
    where
        B: Block;

    fn infer_scheme(&self) -> PuzzleScheme;
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        Self(format!("{:?}", err))
    }
}

pub trait LocalReader: BoardParser {
    fn read_local(file_name: &str) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let content = Self::file_content(file_name)?;
        Self::with_content(content)
    }
    fn file_content(file_name: &str) -> io::Result<String> {
        fs::read_to_string(file_name)
    }
}

#[cfg(feature = "web")]
impl From<reqwest::Error> for ParseError {
    fn from(err: io::Error) -> String {
        Self(format!("{:?}", err))
    }
}

pub trait NetworkReader: BoardParser {
    fn read_remote(file_name: &str) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let url = file_name.to_string();
        let content = Self::http_content(url)?;
        Self::with_content(content)
    }

    #[cfg(feature = "web")]
    fn http_content(url: String) -> Result<String, reqwest::Error> {
        info!("Requesting {} ...", &url);
        let mut response = reqwest::get(url.as_str())?;
        response.text()
    }

    #[cfg(not(feature = "web"))]
    fn http_content(url: String) -> Result<String, ParseError> {
        info!("Requesting {} ...", &url);
        Err(ParseError(format!(
            "Cannot request url {}: no support for web client (hint: add --features=web)",
            url
        )))
    }
}

pub trait Paletted {
    fn get_colors(&self) -> Vec<(String, char, String)>;
    fn get_palette(&self) -> ColorPalette;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PuzzleScheme {
    BlackAndWhite,
    MultiColor,
}

#[derive(Debug, Deserialize)]
struct Clues {
    rows: String,
    columns: String,
}

#[derive(Debug, Deserialize)]
struct Colors {
    defs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct NonoToml {
    clues: Clues,
    colors: Option<Colors>,
}

pub struct MyFormat {
    structure: NonoToml,
    //board_str: String,
}

impl LocalReader for MyFormat {}

impl From<toml::de::Error> for ParseError {
    fn from(err: toml::de::Error) -> Self {
        Self(format!("{:?}", err))
    }
}

impl BoardParser for MyFormat {
    fn with_content(content: String) -> Result<Self, ParseError> {
        let nono = toml::from_str(&content)?;

        Ok(Self {
            structure: nono,
            //board_str: content,
        })
    }

    fn parse<B>(&self) -> Board<B>
    where
        B: Block,
    {
        let clues = &self.structure.clues;
        let palette = self.get_palette();
        Board::with_descriptions_and_palette(
            Self::parse_clues(&clues.rows, &palette),
            Self::parse_clues(&clues.columns, &palette),
            Some(palette),
        )
    }

    fn infer_scheme(&self) -> PuzzleScheme {
        if let Some(colors) = &self.structure.colors {
            if let Some(defs) = &colors.defs {
                if !defs.is_empty() {
                    return PuzzleScheme::MultiColor;
                }
            }
        }

        PuzzleScheme::BlackAndWhite
    }
}

impl MyFormat {
    fn parse_block<B>(block: &str, palette: &ColorPalette) -> B
    where
        B: Block,
    {
        let mut as_chars = block.chars();
        let value_color_pos = as_chars.position(|c| !c.is_digit(10));
        let (value, block_color) = if let Some(pos) = value_color_pos {
            let (value, color) = block.split_at(pos);
            (value, Some(color.to_string()))
        } else {
            (block, palette.get_default())
        };

        let color_id = if let Some(name) = &block_color {
            palette.id_by_name(name)
        } else {
            None
        };
        B::from_str_and_color(value, color_id)
    }

    fn parse_line<B>(descriptions: &str, palette: &ColorPalette) -> Option<Vec<Description<B>>>
    where
        B: Block,
    {
        let descriptions = descriptions.trim();
        let parts: Vec<_> = descriptions.split(|c| c == '#' || c == ';').collect();

        let non_comment = parts[0];
        // dbg!(&non_comment);

        if non_comment == "" {
            return None;
        }

        Some(
            non_comment
                .split(',')
                .filter_map(|row| {
                    let row: &str = row.trim().trim_matches(|c| c == '\'' || c == '"');
                    if row == "" {
                        None
                    } else {
                        Some(Description::new(
                            row.split_whitespace()
                                .map(|block| Self::parse_block(block, palette))
                                .collect(),
                        ))
                    }
                })
                .collect(),
        )
    }

    fn parse_clues<B>(descriptions: &str, palette: &ColorPalette) -> Vec<Description<B>>
    where
        B: Block,
    {
        descriptions
            .lines()
            .flat_map(|line| Self::parse_line(line, palette).unwrap_or_else(|| vec![]))
            .collect()
    }

    ///```
    /// use nonogrid::parser::MyFormat;
    ///
    /// let s = "b = (blue) *";
    /// let colors = MyFormat::parse_color_def(s);
    /// assert_eq!(colors, ("b".to_string(), '*', "blue".to_string()));
    /// ```
    pub fn parse_color_def(color_def: &str) -> (String, char, String) {
        let parts: Vec<_> = color_def.split('=').map(str::trim).collect();
        let name = parts[0];
        let mut desc = parts[1].to_string();
        let symbol = desc.pop().expect("Empty color description in definition");

        desc = desc
            .trim()
            .trim_matches(|c| c == '(' || c == ')')
            .to_string();
        (name.to_string(), symbol, desc)
    }
}

impl Paletted for MyFormat {
    fn get_colors(&self) -> Vec<(String, char, String)> {
        if let Some(colors) = &self.structure.colors {
            if let Some(defs) = &colors.defs {
                let mut colors: Vec<_> =
                    defs.iter().map(|def| Self::parse_color_def(def)).collect();
                colors.sort_unstable_by_key(|(name, ..)| name.clone());
                return colors;
            }
        }

        vec![]
    }

    fn get_palette(&self) -> ColorPalette {
        let mut palette = ColorPalette::with_white_and_black("W", "B");

        let colors = self.get_colors();
        colors.iter().for_each(|(name, symbol, value)| {
            let val = ColorValue::parse(value);
            palette.color_with_name_value_and_symbol(name, val, *symbol);
        });

        palette
    }
}

pub struct WebPbn {
    package: sxd_document::Package,
    cached_colors: InteriorMutableRef<Option<Vec<(String, char, String)>>>,
    cached_palette: InteriorMutableRef<Option<ColorPalette>>,
}

impl LocalReader for WebPbn {}

impl NetworkReader for WebPbn {
    fn read_remote(file_name: &str) -> Result<Self, ParseError> {
        let url = format!("{}/XMLpuz.cgi?id={}", Self::BASE_URL, file_name);

        let content = Self::http_content(url)?;
        Self::with_content(content)
    }
}

impl From<sxd_document::parser::Error> for ParseError {
    fn from(err: sxd_document::parser::Error) -> Self {
        Self(format!("{:?}", err))
    }
}

impl BoardParser for WebPbn {
    fn with_content(content: String) -> Result<Self, ParseError> {
        let package = sxd_document::parser::parse(&content)?;

        Ok(Self {
            package,
            cached_colors: InteriorMutableRef::new(None),
            cached_palette: InteriorMutableRef::new(None),
        })
    }

    fn parse<B>(&self) -> Board<B>
    where
        B: Block,
    {
        Board::with_descriptions_and_palette(
            self.parse_clues("rows"),
            self.parse_clues("columns"),
            Some(self.get_palette()),
        )
    }

    fn infer_scheme(&self) -> PuzzleScheme {
        let colors = self.get_colors();
        let mut names: Vec<_> = colors.iter().map(|(name, ..)| name).collect();
        names.sort_unstable();
        if names.is_empty() || names == ["black", "white"] {
            return PuzzleScheme::BlackAndWhite;
        }

        PuzzleScheme::MultiColor
    }
}

impl WebPbn {
    const BASE_URL: &'static str = "http://webpbn.com";

    fn parse_block<B>(block: &Node, palette: &ColorPalette) -> B
    where
        B: Block,
    {
        let value = &block.string_value();

        let block_color = if let Node::Element(e) = block {
            if let Some(color) = e.attribute("color") {
                Some(color.value().to_string())
            } else {
                palette.get_default()
            }
        } else {
            None
        };
        let color_id = if let Some(name) = &block_color {
            palette.id_by_name(name)
        } else {
            None
        };
        B::from_str_and_color(value, color_id)
    }

    fn parse_line<B>(description: &Node, palette: &ColorPalette) -> Description<B>
    where
        B: Block,
    {
        Description::new(
            description
                .children()
                .iter()
                .filter_map(|child| {
                    if let Node::Text(_text) = child {
                        // ignore newlines and whitespaces
                        None
                    } else {
                        Some(Self::parse_block(child, palette))
                    }
                })
                .collect(),
        )
    }

    fn get_clues<B>(descriptions: &Nodeset, palette: &ColorPalette) -> Vec<Description<B>>
    where
        B: Block,
    {
        descriptions
            .document_order()
            .iter()
            .map(|line_node| Self::parse_line(line_node, palette))
            .collect()
    }

    fn parse_clues<B>(&self, type_: &str) -> Vec<Description<B>>
    where
        B: Block,
    {
        let document = self.package.as_document();
        let value = evaluate_xpath(&document, &format!(".//clues[@type='{}']/line", type_))
            .expect("XPath evaluation failed");

        if let Value::Nodeset(ns) = value {
            Self::get_clues(&ns, &self.get_palette())
        } else {
            vec![]
        }
    }
}

impl WebPbn {
    fn _get_colors(&self) -> Vec<(String, char, String)> {
        let document = self.package.as_document();
        let value = evaluate_xpath(&document, ".//color").expect("XPath evaluation failed");

        if let Value::Nodeset(ns) = value {
            let mut colors: Vec<_> = ns
                .iter()
                .filter_map(|color_node| {
                    let value = color_node.string_value();
                    if let Node::Element(e) = color_node {
                        let name = e
                            .attribute("name")
                            .expect("Not found 'name' attribute in the 'color' element")
                            .value();
                        let symbol = e
                            .attribute("char")
                            .expect("Not found 'char' attribute in the 'color' element")
                            .value();
                        let symbol: char = symbol.as_bytes()[0] as char;
                        Some((name.to_string(), symbol, value))
                    } else {
                        None
                    }
                })
                .collect();
            colors.sort_unstable_by_key(|(name, ..)| name.clone());
            colors
        } else {
            vec![]
        }
    }

    fn _get_palette(&self) -> ColorPalette {
        let mut palette = ColorPalette::with_white_and_black("white", "black");

        let colors = self.get_colors();
        colors.iter().for_each(|(name, symbol, value)| {
            let val = ColorValue::parse(value);
            palette.color_with_name_value_and_symbol(name, val, *symbol);
        });

        let document = self.package.as_document();
        let value =
            evaluate_xpath(&document, ".//puzzle[@type='grid']").expect("XPath evaluation failed");
        if let Value::Nodeset(ns) = value {
            let first_node = ns.iter().next();
            if let Some(Node::Element(e)) = first_node {
                if let Some(default_color) = e.attribute("defaultcolor") {
                    palette.set_default(default_color.value());
                }
            }
        }
        palette
    }
}

impl Paletted for WebPbn {
    fn get_colors(&self) -> Vec<(String, char, String)> {
        if let Some(ref colors) = *read_ref(&self.cached_colors) {
            return colors.clone();
        }

        let result = self._get_colors();
        let mut cache = mutate_ref(&self.cached_colors);
        *cache = Some(result.clone());
        result
    }

    fn get_palette(&self) -> ColorPalette {
        if let Some(ref palette) = *read_ref(&self.cached_palette) {
            return palette.clone();
        }

        let result = self._get_palette();
        let mut cache = mutate_ref(&self.cached_palette);
        *cache = Some(result.clone());
        result
    }
}

type EncodedInt = u16;

pub struct NonogramsOrg {
    encoded: Vec<Vec<EncodedInt>>,
}

impl NonogramsOrg {
    const URLS: [&'static str; 2] = ["http://www.nonograms.ru/", "http://www.nonograms.org/"];
    const PATHS: [(PuzzleScheme, &'static str); 2] = [
        (PuzzleScheme::BlackAndWhite, "nonograms"),
        (PuzzleScheme::MultiColor, "nonograms2"),
    ];
    const CYPHER_PREFIX: &'static str = r"var d=";
    const CYPHER_SUFFIX: char = ';';

    fn extract_encoded_json(html: &str) -> Option<String> {
        html.lines().find_map(|line| {
            if line.starts_with(Self::CYPHER_PREFIX) {
                Some(
                    line.replace(Self::CYPHER_PREFIX, "")
                        .trim_end_matches(|c| c == Self::CYPHER_SUFFIX)
                        .to_string(),
                )
            } else {
                None
            }
        })
    }

    fn parse_line(line: &str) -> Vec<EncodedInt> {
        line.split(',')
            .map(|x| {
                x.parse::<EncodedInt>()
                    .expect("The items should be positive integers")
            })
            .collect()
    }

    fn parse_json(array: &str) -> Vec<Vec<EncodedInt>> {
        array
            .trim_start_matches(|x| x == '[')
            .trim_end_matches(|x| x == ']')
            .split("],[")
            .map(Self::parse_line)
            .collect()
    }

    /// Reverse engineered version of the part of the script
    /// http://www.nonograms.org/js/nonogram.min.059.js
    /// that produces a nonogram solution for the given cyphered solution
    /// (it can be found in puzzle HTML in the form 'var d=[...]').
    #[allow(clippy::shadow_unrelated)]
    pub fn decipher(&self) -> (Vec<String>, Vec<Vec<ColorId>>) {
        let cyphered = self.encoded();

        let x = &cyphered[1];
        let width = (x[0] % x[3] + x[1] % x[3] - x[2] % x[3]) as usize;

        let x = &cyphered[2];
        let height = (x[0] % x[3] + x[1] % x[3] - x[2] % x[3]) as usize;

        let x = &cyphered[3];
        let colors_number = (x[0] % x[3] + x[1] % x[3] - x[2] % x[3]) as usize;

        let x = &cyphered[4];
        let colors: Vec<_> = (0..colors_number)
            .map(|c| {
                let color_x = &cyphered[c + 5];
                let a = color_x[0] - x[1];
                let b = u32::from(color_x[1] - x[0]);
                let c = u32::from(color_x[2] - x[3]);
                let _unknown_flag = color_x[3] - a - x[2];
                let a = &format!("{:x}", a + 256)[1..];
                let b = &format!("{:x}", ((b + 256) << 8) + c)[1..];
                a.to_string() + b
            })
            .collect();

        let mut solution = vec![vec![0; width]; height];
        let z = colors_number + 5;
        let x = &cyphered[z];
        let solution_size = (x[0] % x[3] * (x[0] % x[3]) + x[1] % x[3] * 2 + x[2] % x[3]) as usize;

        let x = &cyphered[z + 1];
        for i in 0..solution_size {
            let y = &cyphered[z + 2 + i];
            let vv = y[0] - x[0] - 1;

            for j in 0..(y[1] - x[1]) {
                let v = (j + vv) as usize;
                let xx = y[3] - x[3] - 1;
                solution[xx as usize][v] = ColorId::from(y[2] - x[2]);
            }
        }

        (colors, solution)
    }

    pub fn encoded(&self) -> &Vec<Vec<EncodedInt>> {
        &self.encoded
    }

    fn get_solution_matrix(&self) -> Vec<Vec<ColorId>> {
        let (_colors, solution_matrix) = self.decipher();
        let palette = self.get_palette();

        let mut mapping_cache = HashMap::new();
        solution_matrix
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&item| {
                        *mapping_cache.entry(item).or_insert_with(|| {
                            palette
                                .id_by_name(&Self::color_name_by_id(item))
                                .unwrap_or(0)
                        })
                    })
                    .collect()
            })
            .collect()
    }

    fn color_name_by_id(id: ColorId) -> String {
        format!("color-{}", id)
    }
}

impl LocalReader for NonogramsOrg {}

impl Default for ParseError {
    fn default() -> Self {
        Self("Unknown parser error".to_string())
    }
}

impl NetworkReader for NonogramsOrg {
    fn read_remote(file_name: &str) -> Result<Self, ParseError> {
        product(&Self::URLS, &Self::PATHS)
            .iter()
            .first_ok(|(base_url, (_scheme, path))| {
                let url = format!("{}{}/i/{}", base_url, path, file_name);
                let content = Self::http_content(url)?;
                Self::with_content(content)
            })
    }
}

impl BoardParser for NonogramsOrg {
    fn with_content(content: String) -> Result<Self, ParseError> {
        let json = Self::extract_encoded_json(&content)
            .ok_or_else(|| ParseError("Not found cypher in HTML content".to_string()))?;

        Ok(Self {
            encoded: Self::parse_json(&json),
        })
    }

    fn parse<B>(&self) -> Board<B>
    where
        B: Block,
    {
        let solution_matrix = self.get_solution_matrix();
        let (columns, rows) = clues_from_solution(&solution_matrix, 0);

        Board::with_descriptions_and_palette(rows, columns, Some(self.get_palette()))
    }

    fn infer_scheme(&self) -> PuzzleScheme {
        let (colors, _solution) = self.decipher();
        if colors.len() == 1 {
            assert_eq!(colors, ["000000"]);
            return PuzzleScheme::BlackAndWhite;
        }

        PuzzleScheme::MultiColor
    }
}

impl Paletted for NonogramsOrg {
    fn get_colors(&self) -> Vec<(String, char, String)> {
        let (colors, _solution) = self.decipher();
        colors
            .into_iter()
            .enumerate()
            // enumerating starts with 1
            .map(|(i, rgb)| (Self::color_name_by_id((i + 1) as ColorId), '?', rgb))
            .collect()
    }

    fn get_palette(&self) -> ColorPalette {
        let mut palette = ColorPalette::with_white("W");

        let colors = self.get_colors();
        colors.iter().for_each(|(name, _dumb_symbol, value)| {
            let val = ColorValue::parse(value);
            palette.color_with_name_and_value(name, val);
        });

        palette
    }
}

#[cfg(test)]
mod tests {
    use super::{BoardParser, MyFormat, Paletted, PuzzleScheme};
    use crate::block::{base::color::ColorPalette, binary::BinaryBlock, Description};

    fn block(n: usize) -> BinaryBlock {
        BinaryBlock(n)
    }

    fn palette() -> ColorPalette {
        ColorPalette::with_white_and_black("W", "B")
    }

    #[test]
    fn parse_single() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1"), &palette()),
            vec![Description::new(vec![block(1)])]
        )
    }

    #[test]
    fn parse_two_lines() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1\n2"), &palette()),
            vec![
                Description::new(vec![block(1)]),
                Description::new(vec![block(2)])
            ]
        )
    }

    #[test]
    fn parse_two_rows_same_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1, 2"), &palette()),
            vec![
                Description::new(vec![block(1)]),
                Description::new(vec![block(2)])
            ]
        )
    }

    #[test]
    fn parse_two_rows_with_commas() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1, 2,\n3"), &palette()),
            vec![
                Description::new(vec![block(1)]),
                Description::new(vec![block(2)]),
                Description::new(vec![block(3)]),
            ]
        )
    }

    #[test]
    fn parse_two_blocks() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2"), &palette()),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("'1 2'"), &palette()),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_double_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2\n\"3 4\"\n"), &palette()),
            vec![
                Description::new(vec![block(1), block(2)]),
                Description::new(vec![block(3), block(4)]),
            ]
        )
    }

    #[test]
    fn parse_comment_end_of_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  # the comment"), &palette()),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_comment_semicolon() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  ; another comment"), &palette()),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_comments_in_the_middle() {
        assert_eq!(
            MyFormat::parse_clues(
                &String::from("1 2 \n # the multi-line \n # comment \n 3, 4"),
                &palette(),
            ),
            vec![
                Description::new(vec![block(1), block(2)]),
                Description::new(vec![block(3)]),
                Description::new(vec![block(4)]),
            ]
        )
    }

    #[test]
    fn infer_black_and_white_no_colors_section() {
        let s = r"
        [clues]
        rows = '1'
        columns = '1'
        ";

        assert_eq!(
            MyFormat::with_content(s.to_string())
                .unwrap()
                .infer_scheme(),
            PuzzleScheme::BlackAndWhite
        )
    }

    #[test]
    fn infer_black_and_white_empty_colors_section() {
        let s = r"
        [clues]
        rows = '1'
        columns = '1'

        [colors]
        ";

        assert_eq!(
            MyFormat::with_content(s.to_string())
                .unwrap()
                .infer_scheme(),
            PuzzleScheme::BlackAndWhite
        )
    }

    #[test]
    fn infer_black_and_white_empty_defs_in_colors_section() {
        let s = r"
        [clues]
        rows = '1'
        columns = '1'

        [colors]
        defs = []
        ";

        assert_eq!(
            MyFormat::with_content(s.to_string())
                .unwrap()
                .infer_scheme(),
            PuzzleScheme::BlackAndWhite
        )
    }

    #[test]
    fn infer_multi_color() {
        let s = r"
        [clues]
        rows = '1'
        columns = '1'

        [colors]
        defs = ['g=(0, 204, 0) %']
        ";

        assert_eq!(
            MyFormat::with_content(s.to_string())
                .unwrap()
                .infer_scheme(),
            PuzzleScheme::MultiColor
        )
    }

    #[test]
    fn parse_colors() {
        let s = r"
        [clues]
        rows = '1'
        columns = '1g'

        [colors]
        defs = ['g=(0, 204, 0) %']
        ";

        let f = MyFormat::with_content(s.to_string()).unwrap();
        let mut colors = vec![];
        colors.push(("g".to_string(), '%', "0, 204, 0".to_string()));

        assert_eq!(f.get_colors(), colors,)
    }
}
