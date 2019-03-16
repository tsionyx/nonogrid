use super::utils;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::Sized;
use std::ops::{Add, Sub};
use std::rc::Rc;

extern crate rulinalg;
use rulinalg::matrix::{BaseMatrix, BaseMatrixMut, Column, Matrix, Row, Rows};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
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
        + PartialOrd
        + Ord
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

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd)]
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

impl BinaryColor {
    fn order(self) -> u8 {
        match self {
            BinaryColor::Undefined => 0,
            BinaryColor::White => 1,
            BinaryColor::Black => 2,
            _ => 3,
        }
    }
}

impl Ord for BinaryColor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order().cmp(&other.order())
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
    Self: Debug + PartialEq + Eq + Hash + Default + Clone,
{
    type Color: Color;

    fn from_str(s: &str) -> Self;
    fn partial_sums(desc: &[Self]) -> Vec<usize>
    where
        Self: Sized;

    fn size(&self) -> usize;
    fn color(&self) -> Self::Color;
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    cells: Matrix<B::Color>,
    desc_rows: Vec<Rc<Description<B>>>,
    desc_cols: Vec<Rc<Description<B>>>,
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Copy,
{
    pub fn with_descriptions(rows: Vec<Description<B>>, columns: Vec<Description<B>>) -> Board<B> {
        let height = rows.len();
        let width = columns.len();

        let init = B::Color::initial();

        let cells = Matrix::new(height, width, vec![init; height * width]);

        Board {
            cells,
            desc_rows: rows.into_iter().map(Rc::new).collect(),
            desc_cols: columns.into_iter().map(Rc::new).collect(),
        }
    }

    pub fn cells(&self) -> Rows<B::Color> {
        self.cells.row_iter()
    }

    pub fn descriptions(&self, rows: bool) -> &Vec<Rc<Description<B>>> {
        if rows {
            &self.desc_rows
        } else {
            &self.desc_cols
        }
    }

    pub fn height(&self) -> usize {
        self.desc_rows.len()
    }

    pub fn width(&self) -> usize {
        self.desc_cols.len()
    }

    pub fn is_solved_full(&self) -> bool {
        self.cells()
            .all(|row| row.iter().all(|cell| cell.is_solved()))
    }

    pub fn get_row(&self, index: usize) -> Row<B::Color> {
        self.cells.row(index)
    }

    pub fn get_column(&self, index: usize) -> Column<B::Color> {
        self.cells.col(index)
    }

    pub fn set_row(&mut self, index: usize, new: Vec<B::Color>) {
        let mat_block = self.cells.sub_slice_mut([index, 0], 1, self.width());
        let new = Matrix::new(1, self.width(), new);
        mat_block.set_to(new);
    }

    pub fn set_column(&mut self, index: usize, new: Vec<B::Color>) {
        let mat_block = self.cells.sub_slice_mut([0, index], self.height(), 1);
        let new = Matrix::new(self.height(), 1, new);
        mat_block.set_to(new);
    }

    /// How many cells in a line are known to be of particular color
    pub fn line_solution_rate<M: BaseMatrix<B::Color>>(line: &M) -> f64 {
        let solved: f64 = line.iter().map(|cell| cell.solution_rate()).sum();
        let size = line.rows() * line.cols();

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
        Self::line_solution_rate(&self.cells)
    }

    pub fn unsolved_cells(&self) -> Vec<Point> {
        self.cells()
            .enumerate()
            .map(|(y, row)| {
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
        self.cells[[y, x]]
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
    pub fn diff(&self, other: &Matrix<B::Color>) -> (Vec<Point>, Vec<Point>) {
        let mut removed = vec![];
        let mut added = vec![];

        for (y, (row, other_row)) in self.cells().zip(other.row_iter()).enumerate() {
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

    pub fn make_snapshot(&self) -> Matrix<B::Color> {
        self.cells.clone()
    }

    pub fn restore(&mut self, cells: Matrix<B::Color>) {
        self.cells = cells;
    }
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Copy,
{
    pub fn set_color(&mut self, point: &Point, color: &B::Color) {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        self.cells[[y, x]] = old_value + *color;
    }

    pub fn unset_color(&mut self, point: &Point, color: &B::Color) -> Result<(), String> {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        self.cells[[y, x]] = (old_value - *color)?;
        Ok(())
    }
}

impl<B> Clone for Board<B>
where
    B: Block,
{
    fn clone(&self) -> Self {
        Self {
            cells: self.make_snapshot(),
            desc_rows: self.desc_rows.clone(),
            desc_cols: self.desc_cols.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryColor::Undefined;
    use super::{BinaryBlock, Block, Board, Description};

    use rulinalg::matrix::BaseMatrix;

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

        let board = Board::with_descriptions(rows, columns);
        assert_eq!(board.cells.rows(), 3);
        assert_eq!(
            *board.cells.row(0).iter().cloned().collect::<Vec<_>>(),
            [Undefined, Undefined, Undefined]
        );
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

        let board = Board::with_descriptions(rows, columns);
        assert_eq!(board.cells.rows(), 5);
        assert_eq!(
            *board.cells.row(0).iter().cloned().collect::<Vec<_>>(),
            [Undefined]
        );
        assert_eq!(board.desc_rows[0].vec, vec![BinaryBlock(1)]);
        assert_eq!(board.desc_rows[1].vec, vec![]);
        assert_eq!(board.desc_rows[2].vec, vec![BinaryBlock(1)]);
    }
}
