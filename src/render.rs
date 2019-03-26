use super::block::base::color::{ColorDesc, ColorId};
use super::block::{Block, Description};
use super::board::Board;
use super::utils::{pad, pad_with, transpose};

use std::cell::{Ref, RefCell};
use std::fmt::Display;
use std::rc::Rc;

use colored::{Color, ColoredString, Colorize};

pub trait Renderer<B>
where
    B: Block,
{
    fn with_board(board: Rc<RefCell<Board<B>>>) -> Self;
    fn render(&self) -> String;
}

pub struct ShellRenderer<B>
where
    B: Block,
{
    board: Rc<RefCell<Board<B>>>,
}

impl<B> Renderer<B> for ShellRenderer<B>
where
    B: Block + Display,
    B::Color: Display,
{
    fn with_board(board: Rc<RefCell<Board<B>>>) -> Self {
        Self { board }
    }

    fn render(&self) -> String {
        let full_width = self.side_width() + self.board().width();

        let mut header = self.header_lines();
        for row in &mut header {
            pad_with(row, "#".to_string(), full_width, false);
        }

        let header: Vec<_> = header
            .into_iter()
            .map(|row| row.into_iter().map(|s| s.normal()).collect::<Vec<_>>())
            .collect();

        let side = self.side_lines();
        let mut side: Vec<_> = side
            .into_iter()
            .map(|row| row.into_iter().map(|s| s.normal()).collect::<Vec<_>>())
            .collect();

        let grid = self.grid_lines();
        let grid: Vec<_> = side
            .iter_mut()
            .zip(grid)
            .map(|(s, g)| {
                s.extend(g);
                // convert into immutable, https://stackoverflow.com/a/41367094
                s.to_owned()
            })
            .collect();

        let lines = vec![header, grid];
        lines
            .concat()
            .iter()
            .map(|line| {
                line.iter()
                    .map(|symbol| pad(symbol, 2, true))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl<B> ShellRenderer<B>
where
    B: Block + Display,
{
    fn board(&self) -> Ref<Board<B>> {
        self.board.borrow()
    }
    //fn header_height(&self) -> usize {
    //    Self::descriptions_width(&self.board().desc_cols)
    //}

    fn side_width(&self) -> usize {
        Self::descriptions_width(&self.board().descriptions(true))
    }

    fn descriptions_width(descriptions: &[Rc<Description<B>>]) -> usize {
        descriptions
            .iter()
            .map(|desc| desc.vec.len())
            .max()
            .unwrap_or(0)
    }

    fn desc_to_string(desc: &Rc<Description<B>>) -> Vec<String> {
        (*desc).vec.iter().map(|block| block.to_string()).collect()
    }

    fn descriptions_to_matrix(descriptions: &[Rc<Description<B>>]) -> Vec<Vec<String>> {
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

    fn from_desc(color_desc: &ColorDesc) -> ColoredString {
        let symbol = color_desc.symbol();
        let color_res: Result<Color, _> = color_desc.name().parse();
        if let Ok(color) = color_res {
            //symbol.color(color)
            " ".on_color(color)
        } else {
            symbol.normal()
        }
    }

    fn cell_symbol(&self, cell: &B::Color) -> ColoredString
    where
        B::Color: Display,
    {
        let symbol = cell.to_string();
        let color_id = symbol.parse::<ColorId>();

        if let Ok(color_id) = color_id {
            let color_desc = self.board().desc_by_id(color_id);
            if let Some(color_desc) = &color_desc {
                return Self::from_desc(color_desc);
            }
        }

        symbol.normal()
    }

    fn grid_lines(&self) -> Vec<Vec<ColoredString>>
    where
        B::Color: Display,
    {
        self.board()
            .iter_rows()
            .map(|row| row.iter().map(|cell| self.cell_symbol(cell)).collect())
            .collect()
    }
}
