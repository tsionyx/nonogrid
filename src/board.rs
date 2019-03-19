use super::block::base::color::{ColorId, ColorPalette};
use super::block::base::{Block, Color, Description};
use super::utils::dedup;

use std::cell::{Ref, RefCell};
use std::rc::Rc;

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

pub struct Board<B>
where
    B: Block,
{
    cells: Vec<Rc<RefCell<Vec<B::Color>>>>,
    desc_rows: Vec<Rc<Description<B>>>,
    desc_cols: Vec<Rc<Description<B>>>,
    palette: Option<ColorPalette>,
    all_colors: Vec<ColorId>,
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Copy,
{
    pub fn with_descriptions(rows: Vec<Description<B>>, columns: Vec<Description<B>>) -> Board<B> {
        Self::with_descriptions_and_palette(rows, columns, None)
    }

    pub fn with_descriptions_and_palette(
        rows: Vec<Description<B>>,
        columns: Vec<Description<B>>,
        palette: Option<ColorPalette>,
    ) -> Board<B> {
        let height = rows.len();
        let width = columns.len();

        let all_colors = Self::all_colors(&rows);
        let init = B::Color::from_color_ids(&all_colors);

        let cells = (0..height)
            .map(|_| Rc::new(RefCell::new(vec![init; width])))
            .collect();

        let desc_rows = rows.into_iter().map(Rc::new).collect();
        let desc_cols = columns.into_iter().map(Rc::new).collect();
        Board {
            cells,
            desc_rows,
            desc_cols,
            palette,
            all_colors,
        }
    }

    fn all_colors(descriptions: &[Description<B>]) -> Vec<ColorId> {
        let mut colors: Vec<_> = descriptions
            .iter()
            .flat_map(|row| row.vec.iter().map(|block| block.color().as_color_id()))
            .collect();

        colors.push(ColorPalette::WHITE_ID);
        dedup(colors)
    }

    //fn fix_palette(&mut self) {
    //    if self.palette.is_none() {
    //        return;
    //    }
    //
    //    let desc_colors = dedup(self.desc_rows.iter().flat_map(|row| {
    //        row.vec.iter().map(|block| block.color())
    //    }).collect());
    //
    //    if let Some(palette) = &self.palette {
    //        for id in palette.ids() {
    //            if desc_colors.contains()
    //        }
    //    }
    //}

    pub fn cells(&self) -> Vec<Ref<Vec<B::Color>>> {
        self.cells.iter().map(|row| row.borrow()).collect()
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
    pub fn line_solution_rate(&self, line: &[B::Color]) -> f64 {
        let size = line.len();
        let colors = &self.all_colors;

        let solved: f64 = line.iter().map(|cell| cell.solution_rate(colors)).sum();

        solved / size as f64
    }

    /// How many cells in the row with given index are known to be of particular color
    pub fn row_solution_rate(&self, index: usize) -> f64 {
        self.line_solution_rate(&self.get_row(index))
    }

    /// How many cells in the column with given index are known to be of particular color
    pub fn column_solution_rate(&self, index: usize) -> f64 {
        self.line_solution_rate(&self.get_column(index))
    }

    /// How many cells in the whole grid are known to be of particular color
    pub fn solution_rate(&self) -> f64 {
        self.cells
            .iter()
            .map(|row| self.line_solution_rate(&row.borrow()))
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
    pub fn diff(&self, other: &[Rc<RefCell<Vec<B::Color>>>]) -> (Vec<Point>, Vec<Point>) {
        let mut removed = vec![];
        let mut added = vec![];

        for (y, (row, other_row)) in self.cells.iter().zip(other).enumerate() {
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

    pub fn make_snapshot(&self) -> Vec<Rc<RefCell<Vec<B::Color>>>> {
        self.cells
            .iter()
            .map(|row| {
                let row = row.borrow().to_vec();
                Rc::new(RefCell::new(row))
            })
            .collect()
    }

    pub fn restore(&mut self, cells: Vec<Rc<RefCell<Vec<B::Color>>>>) {
        self.cells = cells;
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
        Self {
            cells: self.make_snapshot(),
            desc_rows: self.desc_rows.clone(),
            desc_cols: self.desc_cols.clone(),
            palette: self.palette.clone(),
            all_colors: self.all_colors.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::block::binary::BinaryBlock;
    use super::super::block::binary::BinaryColor::Undefined;
    use super::super::block::{Block, Description};
    use super::Board;

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

        let board = Board::with_descriptions(rows, columns);
        assert_eq!(board.cells.len(), 5);
        assert_eq!(*board.cells[0].borrow(), [Undefined]);
        assert_eq!(board.desc_rows[0].vec, vec![BinaryBlock(1)]);
        assert_eq!(board.desc_rows[1].vec, vec![]);
        assert_eq!(board.desc_rows[2].vec, vec![BinaryBlock(1)]);
    }
}
