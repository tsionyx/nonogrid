use std::fs;

extern crate toml;

use super::board::{BinaryBlock, Block, Board, Description};

#[derive(Debug, Deserialize)]
struct Clues {
    rows: String,
    columns: String,
}

#[derive(Debug, Deserialize)]
struct Colors {}

#[derive(Debug, Deserialize)]
pub struct MyFormat {
    clues: Clues,
    colors: Option<Colors>,
}

impl MyFormat {
    //pub fn new(file_name: &str) -> Result<Self, ConfigError> {
    //    let mut s = Config::new();
    //
    //    let res = s.merge(File::with_name(file_name));
    //
    //    if let Err(e) = res {
    //        panic!(e.to_string())
    //    }
    //
    //    // You can deserialize (and thus freeze) the entire configuration as
    //    s.try_into()
    //}

    pub fn from_file(file_name: &str) -> Result<Self, toml::de::Error> {
        let contents =
            fs::read_to_string(file_name).expect("Something went wrong reading the file");
        toml::from_str(&*contents)
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
                .map(|row| {
                    let row: &str = row.trim().trim_matches(|c| c == '\'' || c == '"');
                    // dbg!(row);
                    Description::new(row.split_whitespace().map(B::from_str).collect())
                })
                .collect(),
        )
    }

    pub(in super::reader) fn parse_clues<B>(descriptions: &str) -> Vec<Description<B>>
    where
        B: Block,
    {
        descriptions
            .lines()
            .map(|line| Self::parse_line(line).unwrap_or_else(|| vec![]))
            .flatten()
            .collect()
    }

    pub fn read_board(file_name: &str) -> Board<BinaryBlock> {
        let clues = Self::from_file(file_name)
            .expect("Something wrong with format")
            .clues;
        Board::with_descriptions(
            Self::parse_clues(&clues.rows),
            Self::parse_clues(&clues.columns),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::MyFormat;
    use crate::board::BinaryBlock;
    use crate::board::Description;

    #[test]
    fn parse_single() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1")),
            vec![Description::new(vec![BinaryBlock(1)])]
        )
    }

    #[test]
    fn parse_two_lines() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1\n2")),
            vec![
                Description::new(vec![BinaryBlock(1)]),
                Description::new(vec![BinaryBlock(2)])
            ]
        )
    }

    #[test]
    fn parse_two_rows_same_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1, 2")),
            vec![
                Description::new(vec![BinaryBlock(1)]),
                Description::new(vec![BinaryBlock(2)])
            ]
        )
    }

    #[test]
    fn parse_two_blocks() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2")),
            vec![Description::new(vec![BinaryBlock(1), BinaryBlock(2)]),]
        )
    }

    #[test]
    fn parse_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("'1 2'")),
            vec![Description::new(vec![BinaryBlock(1), BinaryBlock(2)]),]
        )
    }

    #[test]
    fn parse_double_quotes() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2\n\"3 4\"\n")),
            vec![
                Description::new(vec![BinaryBlock(1), BinaryBlock(2)]),
                Description::new(vec![BinaryBlock(3), BinaryBlock(4)]),
            ]
        )
    }

    #[test]
    fn parse_comment_end_of_line() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  # the comment")),
            vec![Description::new(vec![BinaryBlock(1), BinaryBlock(2)]),]
        )
    }

    #[test]
    fn parse_comment_semicolon() {
        assert_eq!(
            MyFormat::parse_clues(&String::from("1 2  ; another comment")),
            vec![Description::new(vec![BinaryBlock(1), BinaryBlock(2)]),]
        )
    }

    #[test]
    fn parse_comments_in_the_middle() {
        assert_eq!(
            MyFormat::parse_clues(&String::from(
                "1 2 \n # the multi-line \n # comment \n 3, 4"
            )),
            vec![
                Description::new(vec![BinaryBlock(1), BinaryBlock(2)]),
                Description::new(vec![BinaryBlock(3)]),
                Description::new(vec![BinaryBlock(4)]),
            ]
        )
    }
}
