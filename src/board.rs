use std::fmt;
use std::iter::once;

use hashbrown::HashMap;
use smallvec::SmallVec;

pub use callbacks::{ChangeColorCallback, RestoreCallback, SetLineCallback};

use crate::block::{
    base::{
        clues_from_solution,
        color::{ColorDesc, ColorId, ColorPalette},
    },
    Block, Color, Description, Line,
};
use crate::utils::{
    dedup,
    rc::{mutate_ref, InteriorMutableRef, MutRc, ReadRc},
};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub const fn x(&self) -> usize {
        self.x
    }

    pub const fn y(&self) -> usize {
        self.y
    }
}

#[cfg(not(feature = "threaded"))]
mod callbacks {
    use super::Point;

    pub trait SetLineCallback: Fn(bool, usize) {}

    impl<F> SetLineCallback for F where F: Fn(bool, usize) {}

    pub trait RestoreCallback: Fn() {}

    impl<F> RestoreCallback for F where F: Fn() {}

    pub trait ChangeColorCallback: Fn(Point) {}

    impl<F> ChangeColorCallback for F where F: Fn(Point) {}
}

#[cfg(feature = "threaded")]
mod callbacks {
    use super::Point;

    pub trait SetLineCallback: Fn(bool, usize) + Send + Sync {}

    impl<F> SetLineCallback for F where F: Fn(bool, usize) + Send + Sync {}

    pub trait RestoreCallback: Fn() + Send + Sync {}

    impl<F> RestoreCallback for F where F: Fn() + Send + Sync {}

    pub trait ChangeColorCallback: Fn(Point) + Send + Sync {}

    impl<F> ChangeColorCallback for F where F: Fn(Point) + Send + Sync {}
}

pub struct Board<B>
where
    B: Block,
{
    cells: Vec<B::Color>,
    desc_rows: Vec<ReadRc<Description<B>>>,
    desc_cols: Vec<ReadRc<Description<B>>>,
    palette: Option<ColorPalette>,
    all_colors: Vec<ColorId>,
    // use with caching duplicated clues
    // https://webpbn.com/survey/caching.html
    rows_cache_indexes: Vec<usize>,
    cols_cache_indexes: Vec<usize>,
    cell_rate_memo: InteriorMutableRef<HashMap<B::Color, f64>>,
    // callbacks
    on_set_line: Option<Box<dyn SetLineCallback>>,
    on_restore: Option<Box<dyn RestoreCallback>>,
    on_change_color: Option<Box<dyn ChangeColorCallback>>,
}

impl<B> fmt::Debug for Board<B>
where
    B: Block,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Board(height={}, width={}, colors={:?})",
            self.height(),
            self.width(),
            self.all_colors
        )
    }
}

impl<B> Board<B>
where
    B: Block,
{
    #[allow(dead_code)]
    pub fn with_descriptions(rows: Vec<Description<B>>, columns: Vec<Description<B>>) -> Self {
        Self::with_descriptions_and_palette(rows, columns, None)
    }

    pub fn with_descriptions_and_palette(
        rows: Vec<Description<B>>,
        columns: Vec<Description<B>>,
        palette: Option<ColorPalette>,
    ) -> Self {
        let height = rows.len();
        let width = columns.len();

        let all_colors = Self::all_colors(&rows);
        let init = B::Color::from_color_ids(&all_colors);
        warn!("Initializing board: height={}, width={}", height, width);
        let cells = vec![init; width * height];

        let uniq_rows = dedup(rows.iter().map(|desc| desc.vec.clone()));
        let uniq_cols = dedup(columns.iter().map(|desc| desc.vec.clone()));

        if uniq_rows.len() < height {
            warn!(
                "Reducing number of rows clues: {} --> {}",
                height,
                uniq_rows.len()
            );
        }
        if uniq_cols.len() < width {
            warn!(
                "Reducing number of columns clues: {} --> {}",
                width,
                uniq_cols.len()
            );
        }

        let rows_cache_indexes = rows
            .iter()
            .map(|desc| {
                uniq_rows
                    .iter()
                    .position(|uniq_row| uniq_row == &desc.vec)
                    .expect("Every row should be present in unique rows")
            })
            .collect();
        let cols_cache_indexes = columns
            .iter()
            .map(|desc| {
                uniq_cols
                    .iter()
                    .position(|uniq_col| uniq_col == &desc.vec)
                    .expect("Every column should be present in unique columns")
            })
            .collect();

        let desc_rows = rows.into_iter().map(ReadRc::new).collect();
        let desc_cols = columns.into_iter().map(ReadRc::new).collect();
        Self {
            cells,
            desc_rows,
            desc_cols,
            palette,
            all_colors,
            rows_cache_indexes,
            cols_cache_indexes,
            cell_rate_memo: InteriorMutableRef::new(HashMap::new()),
            on_set_line: None,
            on_restore: None,
            on_change_color: None,
        }
    }

    fn all_colors(descriptions: &[Description<B>]) -> Vec<ColorId> {
        let colors = descriptions
            .iter()
            .flat_map(Description::colors)
            .chain(once(ColorPalette::WHITE_ID));

        dedup(colors)
    }

    pub fn desc_by_id(&self, id: ColorId) -> Option<ColorDesc> {
        self.palette
            .as_ref()
            .and_then(|palette| palette.desc_by_id(id))
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[B::Color]> {
        self.cells.chunks(self.width())
    }

    pub fn descriptions(&self, rows: bool) -> &[ReadRc<Description<B>>] {
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
        self.cells.iter().all(Color::is_solved)
    }

    fn get_row_slice(&self, index: usize) -> &[B::Color] {
        //let width = self.width();
        //let start_index = width * index;
        //self.cells.iter().skip(start_index).take(width)
        self.iter_rows().nth(index).expect("Invalid row index")
    }

    pub fn get_row(&self, index: usize) -> Line<B::Color> {
        self.get_row_slice(index).into()
    }

    fn get_column_iter(&self, index: usize) -> impl Iterator<Item = &B::Color> {
        self.cells.iter().skip(index).step_by(self.width())
    }

    pub fn get_column(&self, index: usize) -> Line<B::Color> {
        self.get_column_iter(index).cloned().collect()
    }

    fn linear_index(&self, row_index: usize, column_index: usize) -> usize {
        let width = self.width();
        row_index * width + column_index
    }

    fn set_row(&mut self, index: usize, new: &[B::Color]) {
        let row_start = self.linear_index(index, 0);
        (row_start..)
            .zip(new)
            .for_each(|(linear_index, &new_cell)| {
                self.cells[linear_index] = new_cell;
            });
    }

    fn set_column(&mut self, index: usize, new: &[B::Color]) {
        let width = self.width();
        let column_indexes = (index..).step_by(width);

        column_indexes
            .zip(new)
            .for_each(|(linear_index, &new_cell)| {
                self.cells[linear_index] = new_cell;
            });
    }

    /// How many cells in a line are known to be of particular color
    fn line_solution_rate<'a>(&self, line: impl Iterator<Item = &'a B::Color>, size: usize) -> f64
    where
        B::Color: 'a,
    {
        let solved: f64 = line.map(|cell| self.cell_solution_rate(cell)).sum();
        solved / size as f64
    }

    ///How the cell's color set is close
    ///to the full solution (one color).
    fn cell_solution_rate(&self, cell: &B::Color) -> f64 {
        let colors = &self.all_colors;
        if !B::Color::memoize_rate() {
            return cell.solution_rate(colors);
        }

        *mutate_ref(&self.cell_rate_memo)
            .entry(*cell)
            .or_insert_with(|| cell.solution_rate(colors))
    }

    /// How many cells in the row with given index are known to be of particular color
    pub fn row_solution_rate(&self, index: usize) -> f64 {
        self.line_solution_rate(self.get_row_slice(index).iter(), self.width())
    }

    /// How many cells in the column with given index are known to be of particular color
    pub fn column_solution_rate(&self, index: usize) -> f64 {
        self.line_solution_rate(self.get_column_iter(index), self.height())
    }

    /// How many cells in the whole grid are known to be of particular color
    pub fn solution_rate(&self) -> f64 {
        self.line_solution_rate(self.cells.iter(), self.height() * self.width())
    }

    pub fn unsolved_cells(&self) -> impl Iterator<Item = Point> + '_ {
        self.iter_rows().enumerate().flat_map(|(y, row)| {
            row.iter().enumerate().filter_map(move |(x, cell)| {
                if cell.is_solved() {
                    None
                } else {
                    Some(Point::new(x, y))
                }
            })
        })
    }

    pub fn cell(&self, point: &Point) -> B::Color {
        let Point { x, y } = *point;
        self.cells[self.linear_index(y, x)]
    }

    /// For the given cell yield
    /// the four possible neighbour cells.
    /// When the given cell is on a border,
    /// that number can reduce to three or two.
    fn neighbours(&self, point: &Point) -> SmallVec<[Point; 4]> {
        let Point { x, y } = *point;
        let mut res = SmallVec::with_capacity(4);
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
    pub fn unsolved_neighbours(&self, point: &Point) -> impl Iterator<Item = Point> + '_ {
        self.neighbours(point)
            .into_iter()
            .filter(move |n| !self.cell(n).is_solved())
    }

    pub fn row_cache_index(&self, row_index: usize) -> usize {
        self.rows_cache_indexes[row_index]
    }

    pub fn column_cache_index(&self, column_index: usize) -> usize {
        self.cols_cache_indexes[column_index]
    }
}

impl<B> Board<B>
where
    B: Block,
{
    pub fn differs(&self, other: &[B::Color]) -> bool {
        self.cells.as_slice() != other
    }

    pub fn make_snapshot(&self) -> Vec<B::Color> {
        self.cells.clone()
    }

    fn restore(&mut self, cells: Vec<B::Color>) {
        self.cells = cells;

        if self.is_solved_full() {
            // validate
            let white = 0;
            let black = 1;
            let solution_matrix: Vec<_> = self
                .iter_rows()
                .map(|row| {
                    row.iter()
                        .map(|&cell| {
                            if cell == B::Color::blank() {
                                white
                            } else {
                                cell.as_color_id().unwrap_or(black)
                            }
                        })
                        .collect()
                })
                .collect();

            let (columns, rows) = clues_from_solution::<B>(&solution_matrix, white);
            let columns: Vec<_> = columns.into_iter().map(ReadRc::new).collect();
            let rows: Vec<_> = rows.into_iter().map(ReadRc::new).collect();
            assert_eq!(self.desc_cols, columns);
            assert_eq!(self.desc_rows, rows);
        }
    }

    pub fn diff(&self, other: &[B::Color]) -> Vec<Point> {
        let width = self.width();
        self.cells
            .iter()
            .zip(other)
            .enumerate()
            .filter_map(|(i, (current, other))| {
                if current == other {
                    None
                } else {
                    let x = i % width;
                    let y = i / width;
                    Some(Point::new(x, y))
                }
            })
            .collect()
    }
}

impl<B> Board<B>
where
    B: Block,
{
    fn set_color(&mut self, point: &Point, color: &B::Color) {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        let index = self.linear_index(y, x);
        self.cells[index] = old_value + *color;
    }

    fn unset_color(&mut self, point: &Point, color: &B::Color) -> Result<(), String> {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        let index = self.linear_index(y, x);
        self.cells[index] = (old_value - *color)?;

        Ok(())
    }
}

impl<B> Board<B>
where
    B: Block,
{
    pub fn set_callback_on_set_line<CB: SetLineCallback + 'static>(&mut self, f: CB) {
        self.on_set_line = Some(Box::new(f));
    }

    pub fn set_callback_on_restore<CB: RestoreCallback + 'static>(&mut self, f: CB) {
        self.on_restore = Some(Box::new(f));
    }

    pub fn set_callback_on_change_color<CB: ChangeColorCallback + 'static>(&mut self, f: CB) {
        self.on_change_color = Some(Box::new(f));
    }

    pub fn set_row_with_callback(board_ref: MutRc<Self>, index: usize, new: &[B::Color]) {
        board_ref.write().set_row(index, new);
        if let Some(f) = &board_ref.read().on_set_line {
            f(false, index);
        }
    }

    pub fn set_column_with_callback(board_ref: MutRc<Self>, index: usize, new: &[B::Color]) {
        board_ref.write().set_column(index, new);
        if let Some(f) = &board_ref.read().on_set_line {
            f(true, index);
        }
    }

    pub fn restore_with_callback(board_ref: MutRc<Self>, cells: Vec<B::Color>) {
        board_ref.write().restore(cells);
        if let Some(f) = &board_ref.read().on_restore {
            f();
        }
    }

    pub fn set_color_with_callback(board_ref: MutRc<Self>, point: &Point, color: &B::Color) {
        board_ref.write().set_color(point, color);
        if let Some(f) = &board_ref.read().on_change_color {
            f(*point);
        }
    }

    pub fn unset_color_with_callback(
        board_ref: MutRc<Self>,
        point: &Point,
        color: &B::Color,
    ) -> Result<(), String> {
        board_ref.write().unset_color(point, color)?;
        if let Some(f) = &board_ref.read().on_change_color {
            f(*point);
        }
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
            // ignore caches while cloning
            rows_cache_indexes: self.rows_cache_indexes.clone(),
            cols_cache_indexes: self.cols_cache_indexes.clone(),
            cell_rate_memo: InteriorMutableRef::new(HashMap::new()),
            on_set_line: None,
            on_restore: None,
            on_change_color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{
        binary::{BinaryBlock, BinaryColor::Undefined},
        Description,
    };

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
        assert_eq!(board.cells.len(), 9);
        assert_eq!(board.get_row(0), &[Undefined, Undefined, Undefined]);
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
        assert_eq!(board.get_row(0), &[Undefined]);
        assert_eq!(board.desc_rows[0].vec, vec![BinaryBlock(1)]);
        assert_eq!(board.desc_rows[1].vec, vec![]);
        assert_eq!(board.desc_rows[2].vec, vec![BinaryBlock(1)]);
    }
}
