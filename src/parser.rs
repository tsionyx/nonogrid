use super::block::base::color::{ColorPalette, ColorValue};
use super::block::{Block, Description};
use super::board::Board;

use std::fs;

use self::sxd_xpath::nodeset::{Node, Nodeset};
use self::sxd_xpath::{evaluate_xpath, Value};

#[cfg(feature = "web")]
extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate toml;

pub trait LocalReader {
    fn read_local(file_name: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let content = Self::file_content(file_name)?;
        Self::from_string(content)
    }
    fn file_content(file_name: &str) -> Result<String, String> {
        fs::read_to_string(file_name).map_err(|err| format!("{:?}", err))
    }

    fn from_string(content: String) -> Result<Self, String>
    where
        Self: Sized;
}

pub trait NetworkReader {
    fn read_remote(file_name: &str) -> Result<Self, String>
    where
        Self: Sized;

    #[cfg(feature = "web")]
    fn http_content(url: String) -> Result<String, String> {
        info!("Requesting {} ...", &url);
        let mut response = reqwest::get(url.as_str()).map_err(|err| format!("{:?}", err))?;
        response.text().map_err(|err| format!("{:?}", err))
    }

    #[cfg(not(feature = "web"))]
    fn http_content(url: String) -> Result<String, String> {
        info!("Requesting {} ...", &url);
        Err(format!(
            "Cannot request url {}: no support for web client (hint: add --features=web)",
            url
        ))
    }
}

pub trait BoardParser {
    fn parse<B>(&self) -> Board<B>
    where
        B: Block;

    fn infer_scheme(&self) -> PuzzleScheme;
}

pub trait Paletted {
    fn get_colors(&self) -> Vec<(String, char, String)>;
    fn get_palette(&self) -> ColorPalette;
}

#[derive(Debug, PartialEq)]
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

impl LocalReader for MyFormat {
    fn from_string(content: String) -> Result<Self, String>
    where
        Self: Sized,
    {
        let nono =
            toml::from_str(&content).map_err(|toml_de_error| format!("{:?}", toml_de_error))?;

        Ok(Self {
            structure: nono,
            //board_str: content,
        })
    }
}

impl BoardParser for MyFormat {
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

    pub(in super::parser) fn parse_clues<B>(
        descriptions: &str,
        palette: &ColorPalette,
    ) -> Vec<Description<B>>
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
        let parts: Vec<_> = color_def.split('=').map(|part| part.trim()).collect();
        let name = parts[0];
        let mut desc = parts[1].to_string();
        let symbol = desc.pop().unwrap();

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
                colors.sort_by_key(|(name, ..)| name.clone());
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
    //board_str: String,
}

impl LocalReader for WebPbn {
    fn from_string(content: String) -> Result<Self, String>
    where
        Self: Sized,
    {
        let package = sxd_document::parser::parse(&content)
            .map_err(|sxd_parser_error| format!("{:?}", sxd_parser_error))?;

        Ok(Self {
            package,
            //board_str: content,
        })
    }
}

impl NetworkReader for WebPbn {
    fn read_remote(file_name: &str) -> Result<Self, String> {
        let url = format!("{}/XMLpuz.cgi?id={}", Self::BASE_URL, file_name);

        let content = Self::http_content(url)?;
        let package = sxd_document::parser::parse(&content)
            .map_err(|sxd_parser_error| format!("{:?}", sxd_parser_error))?;

        Ok(Self {
            package,
            //board_str: content,
        })
    }
}

impl BoardParser for WebPbn {
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
        names.sort();
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

    pub(in super::parser) fn parse_clues<B>(&self, type_: &str) -> Vec<Description<B>>
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

impl Paletted for WebPbn {
    fn get_colors(&self) -> Vec<(String, char, String)> {
        let document = self.package.as_document();
        let value = evaluate_xpath(&document, ".//color").expect("XPath evaluation failed");

        if let Value::Nodeset(ns) = value {
            let mut colors: Vec<_> = ns
                .iter()
                .filter_map(|color_node| {
                    let value = color_node.string_value();
                    if let Node::Element(e) = color_node {
                        let name = e.attribute("name").unwrap().value();
                        let symbol = e.attribute("char").unwrap().value();
                        let symbol: char = symbol.as_bytes()[0] as char;
                        Some((name.to_string(), symbol, value))
                    } else {
                        None
                    }
                })
                .collect();
            colors.sort_by_key(|(name, ..)| name.clone());
            colors
        } else {
            vec![]
        }
    }

    fn get_palette(&self) -> ColorPalette {
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

#[cfg(test)]
mod tests {
    use super::super::block::base::color::ColorPalette;
    use super::super::block::binary::BinaryBlock;
    use super::super::block::Description;
    use super::{BoardParser, LocalReader, MyFormat, Paletted, PuzzleScheme};

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
                &palette()
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
            MyFormat::from_string(s.to_string()).unwrap().infer_scheme(),
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
            MyFormat::from_string(s.to_string()).unwrap().infer_scheme(),
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
            MyFormat::from_string(s.to_string()).unwrap().infer_scheme(),
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
            MyFormat::from_string(s.to_string()).unwrap().infer_scheme(),
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

        let f = MyFormat::from_string(s.to_string()).unwrap();
        let mut colors = vec![];
        colors.push(("g".to_string(), '%', "0, 204, 0".to_string()));

        assert_eq!(f.get_colors(), colors,)
    }
}
