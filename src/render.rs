use std::fmt::Display;

#[cfg(feature = "colored")]
use colored::{self, ColoredString, Colorize};
use hashbrown::HashMap;

use crate::block::{base::color::ColorDesc, Block, Color, Description};
use crate::board::Board;
use crate::utils::{
    pad, pad_with,
    rc::{MutRc, ReadRc, ReadRef},
    transpose,
};

#[cfg(not(feature = "colored"))]
type ColoredString = String;

pub trait Renderer<B>
where
    B: Block,
{
    fn with_board(board: MutRc<Board<B>>) -> Self;
    fn render(&self) -> String;
}

#[allow(missing_debug_implementations)]
pub struct ShellRenderer<B>
where
    B: Block,
{
    board: MutRc<Board<B>>,
}

impl<B> Renderer<B> for ShellRenderer<B>
where
    B: Block + Display,
    B::Color: Display,
{
    fn with_board(board: MutRc<Board<B>>) -> Self {
        Self { board }
    }

    fn render(&self) -> String {
        let full_width = self.side_width() + self.board().width();

        let mut header = self.header_lines();
        for row in &mut header {
            pad_with(row, "#".to_string(), full_width, false);
        }

        let header = header.into_iter().map(|row| {
            row.into_iter()
                .map(|s| ColoredString::from(s.as_str()))
                .collect()
        });

        let side = self.side_lines();
        let side: Vec<_> = side
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|s| ColoredString::from(s.as_str()))
                    .collect()
            })
            .collect();

        let grid = self.grid_lines();
        let grid = side
            .into_iter()
            .zip(grid)
            // https://users.rust-lang.org/t/how-to-concatenate-two-vectors/8324/4
            .map(|(s, g): (Vec<ColoredString>, _)| [&s[..], &g].concat());

        header
            .chain(grid)
            .map(|line| {
                line.iter()
                    .map(|symbol| pad(symbol, 2, true))
                    .collect::<Vec<_>>()
                    .concat()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl<B> ShellRenderer<B>
where
    B: Block + Display,
{
    fn board(&self) -> ReadRef<Board<B>> {
        self.board.read()
    }

    fn side_width(&self) -> usize {
        Self::descriptions_width(&self.board().descriptions(true))
    }

    fn descriptions_width(descriptions: &[ReadRc<Description<B>>]) -> usize {
        descriptions
            .iter()
            .map(|desc| desc.vec.len())
            .max()
            .unwrap_or(0)
    }

    fn desc_to_string(desc: &ReadRc<Description<B>>) -> Vec<String> {
        desc.vec.iter().map(ToString::to_string).collect()
    }

    fn descriptions_to_matrix(descriptions: &[ReadRc<Description<B>>]) -> Vec<Vec<String>> {
        let mut rows: Vec<_> = descriptions.iter().map(Self::desc_to_string).collect();

        let width = Self::descriptions_width(descriptions);

        for row in &mut rows {
            pad_with(row, " ".to_string(), width, false);
        }
        rows
    }

    fn side_lines(&self) -> Vec<Vec<String>> {
        Self::descriptions_to_matrix(&self.board().descriptions(true))
    }

    fn header_lines(&self) -> Vec<Vec<String>> {
        transpose(&Self::descriptions_to_matrix(
            &self.board().descriptions(false),
        ))
        .unwrap()
    }
}

#[cfg(feature = "colored")]
fn to_color_string(color_desc: &ColorDesc) -> ColoredString {
    let color_res: Result<colored::Color, _> = color_desc.name().parse();
    if let Ok(color) = color_res {
        //symbol.color(color)
        " ".on_color(color)
    } else {
        let symbol = color_desc.symbol();
        ColoredString::from(symbol.as_str())
    }
}

#[cfg(not(feature = "colored"))]
fn to_color_string(color_desc: &ColorDesc) -> ColoredString {
    color_desc.symbol()
}

impl<B> ShellRenderer<B>
where
    B: Block + Display,
    B::Color: Display,
{
    fn cell_symbol(&self, cell: &B::Color) -> ColoredString {
        let id = cell.as_color_id();

        id.and_then(|color_id| {
            self.board()
                .desc_by_id(color_id)
                .map(|color_desc| to_color_string(&color_desc))
        })
        .unwrap_or_else(|| cell.to_string().as_str().into())
    }

    fn grid_lines(&self) -> Vec<Vec<ColoredString>> {
        let mut color_cache = HashMap::new();
        self.board()
            .iter_rows()
            .map(|row| {
                row.iter()
                    .map(|cell| {
                        color_cache
                            .entry(cell)
                            .or_insert_with(|| self.cell_symbol(cell))
                            .clone()
                    })
                    .collect()
            })
            .collect()
    }
}
