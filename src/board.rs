use super::utils;

use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug)]
struct Point {
    x: usize,
    y: usize,
}

//struct Cell {
//    coord: Point,
//    color: Color,
//}

pub trait Color {
    fn initial() -> Self;
    fn blank() -> Self;
    fn is_solved(&self) -> bool;
    fn solution_rate(&self) -> f64;
    fn is_updated_with(&self, new: &Self) -> bool;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinaryColor {
    Undefined,
    White,
    Black,
    // especially for DynamicSolver
    BlackOrWhite,
}

impl Color for BinaryColor {
    fn initial() -> Self {
        BinaryColor::Undefined
    }
    fn blank() -> Self {
        BinaryColor::White
    }

    fn is_solved(&self) -> bool {
        self == &BinaryColor::Black || self == &BinaryColor::White
    }

    fn solution_rate(&self) -> f64 {
        if self.is_solved() {
            1.0
        } else {
            0.0
        }
    }

    fn is_updated_with(&self, new: &Self) -> bool {
        if self == new {
            return false;
        }

        assert_eq!(self, &BinaryColor::Undefined);
        assert!((new == &BinaryColor::White) || (new == &BinaryColor::Black));

        true
    }
}

impl fmt::Display for BinaryColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BinaryColor::*;

        let symbol = match self {
            Undefined => '?',
            White => '.',
            Black => '\u{2b1b}',
            BlackOrWhite => '?',
        };
        write!(f, "{}", symbol)
    }
}

pub trait Block {
    type Color: Color;

    fn from_str(s: &str) -> Self;
    fn partial_sums(desc: &[Self]) -> Vec<usize>
    where
        Self: Sized;

    fn size(&self) -> usize;
    fn color(&self) -> Self::Color;
}

#[derive(Debug, PartialEq, Default)]
pub struct BinaryBlock(pub usize);

impl Block for BinaryBlock {
    type Color = BinaryColor;

    fn from_str(s: &str) -> Self {
        Self(s.parse::<usize>().unwrap())
    }

    fn partial_sums(desc: &[Self]) -> Vec<usize> {
        if desc.is_empty() {
            return vec![];
        }

        desc.iter()
            .fold(Vec::with_capacity(desc.len()), |mut acc, block| {
                if acc.is_empty() {
                    vec![block.0]
                } else {
                    let last = acc.last().unwrap();
                    acc.push(last + block.0 + 1);
                    acc
                }
            })
    }

    fn size(&self) -> usize {
        self.0
    }

    fn color(&self) -> Self::Color {
        BinaryColor::Black
    }
}

impl fmt::Display for BinaryBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Description<T: Block>
where
    T: Block,
{
    pub vec: Vec<T>,
}

impl<T> Description<T>
where
    T: Block + Default + PartialEq,
{
    pub fn new(mut vec: Vec<T>) -> Description<T> {
        // remove zero blocks
        utils::remove(&mut vec, T::default());
        Description { vec }
    }
}

#[derive(Debug)]
pub struct Board<B>
where
    B: Block,
{
    pub cells: Vec<Rc<RefCell<Vec<B::Color>>>>,
    pub desc_rows: Vec<Rc<Description<B>>>,
    pub desc_cols: Vec<Rc<Description<B>>>,
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Clone + Color + Debug,
{
    pub fn with_descriptions(
        rows: Vec<Rc<Description<B>>>,
        columns: Vec<Rc<Description<B>>>,
    ) -> Board<B> {
        let height = rows.len();
        let width = columns.len();

        let init = B::Color::initial();

        let cells = (0..height)
            .map(|_| Rc::new(RefCell::new(vec![init.clone(); width])))
            .collect();

        Board {
            desc_rows: rows,
            desc_cols: columns,
            cells,
        }
    }

    pub fn height(&self) -> usize {
        self.desc_rows.len()
    }

    pub fn width(&self) -> usize {
        self.desc_cols.len()
    }

    pub fn is_solved_full(&self) -> bool {
        self.cells
            .iter()
            .all(|row| row.borrow().iter().all(|cell| cell.is_solved()))
    }

    pub fn get_row(&self, index: usize) -> Rc<RefCell<Vec<B::Color>>> {
        Rc::clone(&self.cells[index])
    }

    pub fn get_column(&self, index: usize) -> Rc<RefCell<Vec<B::Color>>> {
        Rc::new(RefCell::new(
            self.cells
                .iter()
                .map(|row| row.borrow()[index].clone())
                .collect(),
        ))
    }

    pub fn set_row(&mut self, index: usize, new: Vec<B::Color>) {
        self.cells[index] = Rc::new(RefCell::new(new));
    }

    pub fn set_column(&mut self, index: usize, new: Vec<B::Color>) {
        self.cells.iter().zip(new).for_each(|(row, new_cell)| {
            row.borrow_mut()[index] = new_cell;
        });
    }

    /// How many cells in a line are known to be of particular color
    pub fn line_solution_rate(line: &[B::Color]) -> f64 {
        let size = line.len();

        let solved: f64 = line.iter().map(|cell| cell.solution_rate()).sum();

        solved / size as f64
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryColor::Undefined;
    use super::{BinaryBlock, Block, Board, Description};
    use std::rc::Rc;

    #[test]
    fn u_letter() {
        // X   X
        // X   X
        // X X X
        let rows = vec![
            Description::new(vec![BinaryBlock(1), BinaryBlock(1)]),
            Description::new(vec![BinaryBlock(1), BinaryBlock(1)]),
            Description::new(vec![BinaryBlock(3)]),
        ];
        let columns = vec![
            Description::new(vec![BinaryBlock(3)]),
            Description::new(vec![BinaryBlock(1)]),
            Description::new(vec![BinaryBlock(3)]),
        ];

        let board = Board::with_descriptions(
            rows.into_iter().map(Rc::new).collect(),
            columns.into_iter().map(Rc::new).collect(),
        );
        assert_eq!(board.cells.len(), 3);
        assert_eq!(*board.cells[0].borrow(), [Undefined, Undefined, Undefined]);
    }

    #[test]
    fn check_partial_sums() {
        let d = Description::new(vec![BinaryBlock(1), BinaryBlock(2), BinaryBlock(3)]);
        assert_eq!(BinaryBlock::partial_sums(&d.vec), vec![1, 4, 8]);
    }

    #[test]
    fn i_letter() {
        // X
        //
        // X
        // X
        // X
        let rows = vec![
            Description::new(vec![BinaryBlock(1)]),
            Description::new(vec![BinaryBlock(0)]),
            Description::new(vec![BinaryBlock(1)]),
            Description::new(vec![BinaryBlock(1)]),
            Description::new(vec![BinaryBlock(1)]),
        ];
        let columns = vec![Description::new(vec![BinaryBlock(1), BinaryBlock(3)])];

        let board = Board::with_descriptions(
            rows.into_iter().map(Rc::new).collect(),
            columns.into_iter().map(Rc::new).collect(),
        );
        assert_eq!(board.cells.len(), 5);
        assert_eq!(*board.cells[0].borrow(), [Undefined]);
        assert_eq!(board.desc_rows[0].vec, vec![BinaryBlock(1)]);
        assert_eq!(board.desc_rows[1].vec, vec![]);
        assert_eq!(board.desc_rows[2].vec, vec![BinaryBlock(1)]);
    }
}
