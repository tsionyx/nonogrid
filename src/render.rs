use super::board::{BlackBlock, Block, Board, Description};
use super::utils::{concat_vecs, pad_with, transpose};
use std::fmt::Display;
use std::rc::Rc;

pub trait Renderer {
    fn render(&self) -> String;
}

pub struct ShellRenderer {
    pub board: Rc<Board<BlackBlock>>,
}

impl Renderer for ShellRenderer {
    fn render(&self) -> String {
        let full_width = self.side_width() + self.board.width();

        let mut header = self.header_lines();
        for row in header.iter_mut() {
            pad_with(row, "#".to_string(), full_width, false);
        }
        let h_lines: Vec<String> = header.iter().map(|line| line.join(" ")).collect();

        let mut side = self.side_lines();
        //let s_lines: Vec<String> = side.iter().map(|line| line.join(" ")).collect();

        let grid = self.grid_lines();
        let grid: Vec<&Vec<String>> = side
            .iter_mut()
            .zip(grid)
            .map(|(s, g)| {
                s.extend(g);
                // convert into immutable, https://stackoverflow.com/a/41367094
                &*s
            })
            .collect();
        let g_lines: Vec<String> = grid.iter().map(|line| line.join(" ")).collect();

        let lines = vec![h_lines, g_lines];
        concat_vecs(lines).join("\n")
    }
}

impl ShellRenderer {
    fn header_height(&self) -> usize {
        Self::descriptions_width(&self.board.desc_cols)
    }

    fn side_width(&self) -> usize {
        Self::descriptions_width(&self.board.desc_rows)
    }

    fn descriptions_width<B: Block + Display>(descriptions: &[Description<B>]) -> usize {
        descriptions
            .iter()
            .map(|desc| desc.vec.len())
            .max()
            .unwrap_or(0)
    }

    fn desc_to_string<B: Block + Display>(desc: &Description<B>) -> Vec<String> {
        desc.vec.iter().map(|block| block.to_string()).collect()
    }

    fn descriptions_to_matrix<B: Block + Display>(
        descriptions: &[Description<B>],
    ) -> Vec<Vec<String>> {
        let mut rows: Vec<Vec<String>> = descriptions.iter().map(Self::desc_to_string).collect();

        let width = Self::descriptions_width(descriptions);

        for row in rows.iter_mut() {
            pad_with(row, " ".to_string(), width, false);
        }
        rows
    }

    fn side_lines(&self) -> Vec<Vec<String>> {
        Self::descriptions_to_matrix(&self.board.desc_rows)
    }

    fn header_lines(&self) -> Vec<Vec<String>> {
        transpose(&Self::descriptions_to_matrix(&self.board.desc_cols)).unwrap()
    }

    fn grid_lines(&self) -> Vec<Vec<String>> {
        self.board
            .cells
            .iter()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect())
            .collect()
    }
}
