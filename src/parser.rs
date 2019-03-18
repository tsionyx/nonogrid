use super::block::{Block, Description};
use super::board::Board;

use std::collections::HashMap;
use std::fs;

use self::sxd_xpath::nodeset::{Node, Nodeset};
use self::sxd_xpath::{evaluate_xpath, Value};

#[cfg(feature = "web")]
extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate toml;

pub trait LocalReader {
    fn read_local(file_name: &str) -> Result<String, String> {
        fs::read_to_string(file_name).map_err(|err| format!("{:?}", err))
    }
}

pub trait NetworkReader {
    fn read_remote(file_name: &str) -> Result<String, String>;

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

pub trait BoardParser<B>
where
    B: Block,
{
    //type BlockType: Block;
    fn parse(board_str: &str) -> Board<B>;
}

#[derive(Debug, PartialEq)]
pub enum PuzzleScheme {
    BlackAndWhite,
    MultiColor,
}

pub trait InferScheme {
    fn infer_scheme(board_str: &str) -> PuzzleScheme;
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
pub struct MyFormat {
    clues: Clues,
    colors: Option<Colors>,
}

impl LocalReader for MyFormat {}

impl<B> BoardParser<B> for MyFormat
where
    B: Block,
{
    fn parse(board_str: &str) -> Board<B> {
        let clues = Self::with_content(board_str)
            .expect("Something wrong with format")
            .clues;
        Board::with_descriptions(
            Self::parse_clues(&clues.rows),
            Self::parse_clues(&clues.columns),
        )
    }
}

impl InferScheme for MyFormat {
    fn infer_scheme(board_str: &str) -> PuzzleScheme {
        let this = Self::with_content(board_str).unwrap();
        if let Some(colors) = this.colors {
            let has_colors = colors.defs.map(|defs| !defs.is_empty()).unwrap_or(false);
            if has_colors {
                return PuzzleScheme::MultiColor;
            }
        }

        PuzzleScheme::BlackAndWhite
    }
}

impl MyFormat {
    pub fn with_content(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    fn parse_line<B>(descriptions: &str) -> Option<Vec<Description<B>>>
    where
        B: Block,
    {
        let descriptions = descriptions.trim();
        let parts: Vec<&str> = descriptions.split(|c| c == '#' || c == ';').collect();

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
                            row.split_whitespace().map(B::from_str).collect(),
                        ))
                    }
                })
                .collect(),
        )
    }

    pub(in super::parser) fn parse_clues<B>(descriptions: &str) -> Vec<Description<B>>
    where
        B: Block,
    {
        descriptions
            .lines()
            .map(|line| Self::parse_line(line).unwrap_or_else(|| vec![]))
            .flatten()
            .collect()
    }
}

pub struct WebPbn {}

impl LocalReader for WebPbn {}

impl NetworkReader for WebPbn {
    fn read_remote(file_name: &str) -> Result<String, String> {
        let url = format!("{}/XMLpuz.cgi?id={}", Self::BASE_URL, file_name);
        Self::http_content(url)
    }
}

impl<B> BoardParser<B> for WebPbn
where
    B: Block,
{
    fn parse(board_str: &str) -> Board<B> {
        let package = Self::xml_package(board_str);
        Board::with_descriptions(
            Self::parse_clues(&package, "rows"),
            Self::parse_clues(&package, "columns"),
        )
    }
}

impl InferScheme for WebPbn {
    fn infer_scheme(board_str: &str) -> PuzzleScheme {
        let colors = Self::get_colors(board_str);
        let mut names: Vec<_> = colors.keys().collect();
        names.sort();
        if names.is_empty() || names == ["black", "white"] {
            return PuzzleScheme::BlackAndWhite;
        }

        PuzzleScheme::MultiColor
    }
}

impl WebPbn {
    const BASE_URL: &'static str = "http://webpbn.com";

    pub fn xml_package(content: &str) -> sxd_document::Package {
        sxd_document::parser::parse(content).expect("failed to parse XML")
    }

    fn parse_line<B>(description: &Node) -> Description<B>
    where
        B: Block,
    {
        Description::new(
            description
                .children()
                .iter()
                .map(|child| B::from_str(&child.string_value()))
                .collect(),
        )
    }

    fn get_clues<B>(descriptions: &Nodeset) -> Vec<Description<B>>
    where
        B: Block,
    {
        descriptions
            .document_order()
            .iter()
            .map(Self::parse_line)
            .collect()
    }

    pub(in super::parser) fn parse_clues<B>(
        package: &sxd_document::Package,
        type_: &str,
    ) -> Vec<Description<B>>
    where
        B: Block,
    {
        let document = package.as_document();
        let value = evaluate_xpath(&document, &format!(".//clues[@type='{}']/line", type_))
            .expect("XPath evaluation failed");

        if let Value::Nodeset(ns) = value {
            Self::get_clues(&ns)
        } else {
            vec![]
        }
    }

    pub fn get_colors(board_str: &str) -> HashMap<String, (char, String)> {
        let package = Self::xml_package(board_str);
        let document = package.as_document();
        let value = evaluate_xpath(&document, ".//color").expect("XPath evaluation failed");

        if let Value::Nodeset(ns) = value {
            ns.iter()
                .filter_map(|color_node| {
                    let value = color_node.string_value();
                    if let Node::Element(e) = color_node {
                        let name = e.attribute("name").unwrap().value();
                        let symbol = e.attribute("char").unwrap().value();
                        let symbol: char = symbol.as_bytes()[0] as char;
                        Some((name.to_string(), (symbol, value)))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            HashMap::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::block::binary::BinaryBlock;
    use super::super::block::Description;
    use super::{InferScheme, MyFormat, PuzzleScheme};

    fn block(n: usize) -> BinaryBlock {
        BinaryBlock(n)
    }

    #[test]
    fn parse_single() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1")),
            vec![Description::new(vec![block(1)])]
        )
    }

    #[test]
    fn parse_two_lines() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1\n2")),
            vec![
                Description::new(vec![block(1)]),
                Description::new(vec![block(2)])
            ]
        )
    }

    #[test]
    fn parse_two_rows_same_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1, 2")),
            vec![
                Description::new(vec![block(1)]),
                Description::new(vec![block(2)])
            ]
        )
    }

    #[test]
    fn parse_two_rows_with_commas() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1, 2,\n3")),
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
            MyFormat::parse_clues(&String::from("1 2")),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("'1 2'")),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_double_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2\n\"3 4\"\n")),
            vec![
                Description::new(vec![block(1), block(2)]),
                Description::new(vec![block(3), block(4)]),
            ]
        )
    }

    #[test]
    fn parse_comment_end_of_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  # the comment")),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_comment_semicolon() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  ; another comment")),
            vec![Description::new(vec![block(1), block(2)]),]
        )
    }

    #[test]
    fn parse_comments_in_the_middle() {
        assert_eq!(
            MyFormat::parse_clues(&String::from(
                "1 2 \n # the multi-line \n # comment \n 3, 4"
            )),
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

        assert_eq!(MyFormat::infer_scheme(s), PuzzleScheme::BlackAndWhite)
    }

    #[test]
    fn infer_black_and_white_empty_colors_section() {
        let s = r"
        [clues]
        rows = '1'
        columns = '1'

        [colors]
        ";

        assert_eq!(MyFormat::infer_scheme(s), PuzzleScheme::BlackAndWhite)
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

        assert_eq!(MyFormat::infer_scheme(s), PuzzleScheme::BlackAndWhite)
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

        assert_eq!(MyFormat::infer_scheme(s), PuzzleScheme::MultiColor)
    }
}
