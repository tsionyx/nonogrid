use super::utils;

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::Sized;
use std::ops::{Add, Sub};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }
}

pub trait Color
where
    Self: Debug
        + PartialEq
        + Eq
        + Hash
        + Copy
        + Clone
        + Add<Output = Self>
        + Sub<Output = Result<Self, String>>,
{
    fn initial() -> Self;
    fn blank() -> Self;
    fn is_solved(&self) -> bool;
    fn solution_rate(&self) -> f64;
    fn is_updated_with(&self, new: &Self) -> Result<bool, String>;
    fn variants(&self) -> HashSet<Self>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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

    fn is_updated_with(&self, new: &Self) -> Result<bool, String> {
        if self == new {
            return Ok(false);
        }

        if self != &BinaryColor::Undefined {
            return Err("Can only update undefined".to_string());
        }
        if !new.is_solved() {
            return Err("Cannot update already solved".to_string());
        }

        Ok(true)
    }

    fn variants(&self) -> HashSet<Self> {
        if self.is_solved() {
            vec![*self]
        } else {
            vec![BinaryColor::White, BinaryColor::Black]
        }
        .into_iter()
        .collect()
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

impl Add for BinaryColor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        rhs
    }
}

impl Sub for BinaryColor {
    type Output = Result<Self, String>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_solved() {
            return Err(format!("Cannot unset already set cell {:?}", &self));
        }

        Ok(match rhs {
            BinaryColor::Black => BinaryColor::White,
            BinaryColor::White => BinaryColor::Black,
            _ => self,
        })
    }
}

pub trait Block
where
    Self: Debug + PartialEq + Eq + Hash + Default,
{
    type Color: Color;

    fn from_str(s: &str) -> Self;
    fn partial_sums(desc: &[Self]) -> Vec<usize>
    where
        Self: Sized;

    fn size(&self) -> usize;
    fn color(&self) -> Self::Color;
}

#[derive(Debug, PartialEq, Eq, Hash, Default)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Description<T: Block>
where
    T: Block,
{
    pub vec: Vec<T>,
}

impl<T> Description<T>
where
    T: Block,
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
    B::Color: Copy,
{
    pub fn with_descriptions(
        rows: Vec<Rc<Description<B>>>,
        columns: Vec<Rc<Description<B>>>,
    ) -> Board<B> {
        let height = rows.len();
        let width = columns.len();

        let init = B::Color::initial();

        let cells = (0..height)
            .map(|_| Rc::new(RefCell::new(vec![init; width])))
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

    pub fn get_row(&self, index: usize) -> Vec<B::Color> {
        self.cells[index].borrow().clone()
    }

    pub fn get_column(&self, index: usize) -> Vec<B::Color> {
        self.cells.iter().map(|row| row.borrow()[index]).collect()
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

    /// How many cells in the row with given index are known to be of particular color
    pub fn row_solution_rate(&self, index: usize) -> f64 {
        Self::line_solution_rate(&self.get_row(index))
    }

    /// How many cells in the column with given index are known to be of particular color
    pub fn column_solution_rate(&self, index: usize) -> f64 {
        Self::line_solution_rate(&self.get_column(index))
    }

    /// How many cells in the whole grid are known to be of particular color
    pub fn solution_rate(&self) -> f64 {
        self.cells
            .iter()
            .map(|row| Self::line_solution_rate(&row.borrow()))
            .sum::<f64>()
            / (self.height() as f64)
    }

    pub fn unsolved_cells(&self) -> Vec<Point> {
        self.cells
            .iter()
            .enumerate()
            .map(|(y, row)| {
                let row = row.borrow();
                row.iter()
                    .enumerate()
                    .filter_map(move |(x, cell)| {
                        if cell.is_solved() {
                            None
                        } else {
                            Some(Point::new(x, y))
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    pub fn cell(&self, point: &Point) -> B::Color {
        let Point { x, y } = *point;
        self.cells[y].borrow()[x]
    }

    /// For the given cell yield
    /// the four possible neighbour cells.
    /// When the given cell is on a border,
    /// that number can reduce to three or two.
    fn neighbours(&self, point: &Point) -> Vec<Point> {
        let Point { x, y } = *point;
        let mut res = vec![];
        if x > 0 {
            res.push(Point::new(x - 1, y));
        }
        if x < self.width() - 1 {
            res.push(Point::new(x + 1, y));
        }
        if y > 0 {
            res.push(Point::new(x, y - 1));
        }
        if y < self.height() - 1 {
            res.push(Point::new(x, y + 1));
        }
        res
    }

    /// For the given cell yield
    /// the neighbour cells
    /// that are not completely solved yet.
    pub fn unsolved_neighbours(&self, point: &Point) -> Vec<Point> {
        self.neighbours(&point)
            .iter()
            .filter_map(|n| {
                if self.cell(n).is_solved() {
                    None
                } else {
                    Some(*n)
                }
            })
            .collect()
    }
}

impl<B> Board<B>
where
    B: Block,
{
    /// Difference between two boards as coordinates of changed cells.
    /// Standard diff semantic as result:
    /// - first returned points which set in current board and unset in the other
    /// - second returned points which unset in current board and set in the other
    pub fn diff(&self, other: &Self) -> (Vec<Point>, Vec<Point>) {
        let mut removed = vec![];
        let mut added = vec![];

        for (y, (row, other_row)) in self.cells.iter().zip(&other.cells).enumerate() {
            let row = row.borrow();
            let other_row = other_row.borrow();

            for (x, (cell, other_cell)) in row.iter().zip(other_row.iter()).enumerate() {
                if cell != other_cell {
                    let p = Point::new(x, y);

                    if !cell.is_updated_with(other_cell).unwrap_or(false) {
                        removed.push(p);
                    }

                    if !other_cell.is_updated_with(cell).unwrap_or(false) {
                        added.push(p);
                    }
                }
            }
        }
        (removed, added)
    }
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Copy,
{
    pub fn set_color(&self, point: &Point, color: &B::Color) {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        let mut row = self.cells[y].borrow_mut();
        row[x] = old_value + *color;
    }

    pub fn unset_color(&self, point: &Point, color: &B::Color) -> Result<(), String> {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        let mut row = self.cells[y].borrow_mut();
        row[x] = (old_value - *color)?;
        Ok(())
    }
}

impl<B> Clone for Board<B>
where
    B: Block,
{
    fn clone(&self) -> Self {
        let cells = self
            .cells
            .iter()
            .map(|row| {
                let row = row.borrow().to_vec();
                Rc::new(RefCell::new(row))
            })
            .collect();

        Self {
            cells,
            desc_rows: self.desc_rows.clone(),
            desc_cols: self.desc_cols.clone(),
        }
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
