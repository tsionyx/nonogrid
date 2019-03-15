use super::board::{BinaryBlock, Block, Board, Description};
use super::utils::{pad, pad_with, transpose};
use std::cell::{Ref, RefCell};
use std::fmt::Display;
use std::rc::Rc;

pub trait Renderer {
    fn with_board(board: Rc<RefCell<Board<BinaryBlock>>>) -> Self;
    fn render(&self) -> String;
}

pub struct ShellRenderer {
    board: Rc<RefCell<Board<BinaryBlock>>>,
}

impl Renderer for ShellRenderer {
    fn with_board(board: Rc<RefCell<Board<BinaryBlock>>>) -> Self {
        Self { board }
    }

    fn render(&self) -> String {
        let full_width = self.side_width() + self.board().width();

        let mut header = self.header_lines();
        for row in header.iter_mut() {
            pad_with(row, "#".to_string(), full_width, false);
        }

        let mut side = self.side_lines();
        let grid = self.grid_lines();
        let grid: Vec<Vec<String>> = side
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

impl ShellRenderer {
    fn board(&self) -> Ref<Board<BinaryBlock>> {
        self.board.borrow()
    }
    //fn header_height(&self) -> usize {
    //    Self::descriptions_width(&self.board().desc_cols)
    //}

    fn side_width(&self) -> usize {
        Self::descriptions_width(self.board().descriptions(true))
    }

    fn descriptions_width<B: Block + Display>(descriptions: &[Rc<Description<B>>]) -> usize {
        descriptions
            .iter()
            .map(|desc| desc.vec.len())
            .max()
            .unwrap_or(0)
    }

    fn desc_to_string<B: Block + Display>(desc: &Rc<Description<B>>) -> Vec<String> {
        desc.vec.iter().map(|block| block.to_string()).collect()
    }

    fn descriptions_to_matrix<B: Block + Display>(
        descriptions: &[Rc<Description<B>>],
    ) -> Vec<Vec<String>> {
        let mut rows: Vec<Vec<String>> = descriptions.iter().map(Self::desc_to_string).collect();

        let width = Self::descriptions_width(descriptions);

        for row in rows.iter_mut() {
            pad_with(row, " ".to_string(), width, false);
        }
        rows
    }

    fn side_lines(&self) -> Vec<Vec<String>> {
        Self::descriptions_to_matrix(self.board().descriptions(true))
    }

    fn header_lines(&self) -> Vec<Vec<String>> {
        transpose(&Self::descriptions_to_matrix(
            self.board().descriptions(false),
        ))
        .unwrap()
    }

    fn grid_lines(&self) -> Vec<Vec<String>> {
        self.board()
            .cells()
            .iter()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect())
            .collect()
    }
}
