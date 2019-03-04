use super::board::{Block, Board, Description};

use std::fmt::Debug;
use std::fs;
use std::marker::PhantomData;
use std::rc::Rc;

use self::sxd_xpath::nodeset::{Node, Nodeset};
use self::sxd_xpath::{evaluate_xpath, Value};

extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate toml;

#[derive(Debug, Deserialize)]
struct Clues {
    rows: String,
    columns: String,
}

#[derive(Debug, Deserialize)]
struct Colors {}

#[derive(Debug, Deserialize)]
pub struct MyFormat<B> {
    clues: Clues,
    colors: Option<Colors>,

    dummy: Option<PhantomData<B>>,
}

pub trait BoardParser {
    type BlockType: Block;
    fn read_board(resource_name: &str) -> Board<Self::BlockType>;
}

impl<B> MyFormat<B>
where
    B: Block + Default + PartialEq,
{
    pub fn from_file(file_name: &str) -> Result<Self, toml::de::Error> {
        let contents =
            fs::read_to_string(file_name).expect("Something went wrong reading the file");
        toml::from_str(&*contents)
    }

    fn parse_line(descriptions: &str) -> Option<Vec<Description<B>>> {
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
                .map(|row| {
                    let row: &str = row.trim().trim_matches(|c| c == '\'' || c == '"');
                    // dbg!(row);
                    Description::new(row.split_whitespace().map(B::from_str).collect())
                })
                .collect(),
        )
    }

    pub(in super::parser) fn parse_clues(descriptions: &str) -> Vec<Rc<Description<B>>> {
        descriptions
            .lines()
            .map(|line| Self::parse_line(line).unwrap_or_else(|| vec![]))
            .flatten()
            .map(Rc::new)
            .collect()
    }
}

impl<B> BoardParser for MyFormat<B>
where
    B: Block + Default + PartialEq,
    B::Color: Clone + Debug,
{
    type BlockType = B;

    fn read_board(resource_name: &str) -> Board<Self::BlockType> {
        let clues = Self::from_file(resource_name)
            .expect("Something wrong with format")
            .clues;
        Board::with_descriptions(
            Self::parse_clues(&clues.rows),
            Self::parse_clues(&clues.columns),
        )
    }
}

pub struct WebPbn<B> {
    dummy: PhantomData<B>,
}

impl<B> BoardParser for WebPbn<B>
where
    B: Block + Default + PartialEq,
    <B as Block>::Color: Clone + Debug,
{
    type BlockType = B;

    fn read_board(resource_name: &str) -> Board<Self::BlockType> {
        let package = Self::from_file(resource_name);
        Board::with_descriptions(
            Self::parse_clues(&package, "rows"),
            Self::parse_clues(&package, "columns"),
        )
    }
}

impl<B> WebPbn<B>
where
    B: Block + Default + PartialEq,
{
    pub fn from_file(file_name: &str) -> sxd_document::Package {
        let contents =
            fs::read_to_string(file_name).expect("Something went wrong reading the file");

        sxd_document::parser::parse(&contents).expect("failed to parse XML")
    }

    fn parse_line(description: &Node) -> Description<B> {
        Description::new(
            description
                .children()
                .iter()
                .map(|child| B::from_str(&child.string_value()))
                .collect(),
        )
    }

    pub(in super::parser) fn parse_clues(
        package: &sxd_document::Package,
        type_: &str,
    ) -> Vec<Rc<Description<B>>> {
        let document = package.as_document();
        let value = evaluate_xpath(&document, &format!(".//clues[@type='{}']/line", type_))
            .expect("XPath evaluation failed");

        if let Value::Nodeset(ns) = value {
            Self::get_clues(&ns)
        } else {
            vec![]
        }
    }

    fn get_clues(descriptions: &Nodeset) -> Vec<Rc<Description<B>>> {
        descriptions
            .document_order()
            .iter()
            .map(Self::parse_line)
            .map(Rc::new)
            .collect()
    }
}

impl<B> WebPbn<B>
where
    B: Block + Default + PartialEq,
    <B as Block>::Color: Clone + Debug,
{
    const BASE_URL: &'static str = "http://webpbn.com";

    fn get_puzzle_xml(id: &str) -> reqwest::Result<String> {
        let url = format!("{}/XMLpuz.cgi?id={}", Self::BASE_URL, id);
        info!("Requesting {} ...", &url);
        reqwest::get(url.as_str())?.text()
    }

    pub fn get_board(resource_name: &str) -> Board<B> {
        let xml_content = Self::get_puzzle_xml(resource_name).unwrap();
        let package = sxd_document::parser::parse(&xml_content).expect("failed to parse XML");

        Board::with_descriptions(
            Self::parse_clues(&package, "rows"),
            Self::parse_clues(&package, "columns"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::board::{BinaryBlock, Description};
    use super::MyFormat;
    use std::rc::Rc;

    fn block(n: usize) -> BinaryBlock {
        BinaryBlock(n)
    }

    #[test]
    fn parse_single() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1")),
            vec![Rc::new(Description::new(vec![block(1)]))]
        )
    }

    #[test]
    fn parse_two_lines() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1\n2")),
            vec![
                Rc::new(Description::new(vec![block(1)])),
                Rc::new(Description::new(vec![block(2)]))
            ]
        )
    }

    #[test]
    fn parse_two_rows_same_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1, 2")),
            vec![
                Rc::new(Description::new(vec![block(1)])),
                Rc::new(Description::new(vec![block(2)]))
            ]
        )
    }

    #[test]
    fn parse_two_blocks() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2")),
            vec![Rc::new(Description::new(vec![block(1), block(2)])),]
        )
    }

    #[test]
    fn parse_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("'1 2'")),
            vec![Rc::new(Description::new(vec![block(1), block(2)])),]
        )
    }

    #[test]
    fn parse_double_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2\n\"3 4\"\n")),
            vec![
                Rc::new(Description::new(vec![block(1), block(2)])),
                Rc::new(Description::new(vec![block(3), block(4)])),
            ]
        )
    }

    #[test]
    fn parse_comment_end_of_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  # the comment")),
            vec![Rc::new(Description::new(vec![block(1), block(2)])),]
        )
    }

    #[test]
    fn parse_comment_semicolon() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  ; another comment")),
            vec![Rc::new(Description::new(vec![block(1), block(2)])),]
        )
    }

    #[test]
    fn parse_comments_in_the_middle() {
        assert_eq!(
            MyFormat::parse_clues(&String::from(
                "1 2 \n # the multi-line \n # comment \n 3, 4"
            )),
            vec![
                Rc::new(Description::new(vec![block(1), block(2)])),
                Rc::new(Description::new(vec![block(3)])),
                Rc::new(Description::new(vec![block(4)])),
            ]
        )
    }
}
