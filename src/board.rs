use super::block::base::color::{ColorDesc, ColorId, ColorPalette};
use super::block::base::{Block, Color, Description};
use super::cache::{cache_info, GrowableCache};
use super::utils::dedup;

use std::rc::Rc;

use cached::Cached;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Point {
    x: usize,
    y: usize,
}

pub type CacheKey<B> = (usize, Rc<Vec<<B as Block>::Color>>);
pub type CacheValue<B> = Result<Rc<Vec<<B as Block>::Color>>, String>;
pub type LineSolverCache<B> = GrowableCache<CacheKey<B>, CacheValue<B>>;

pub fn new_cache<B>(capacity: usize) -> LineSolverCache<B>
where
    B: Block,
{
    GrowableCache::with_capacity_and_increase(capacity, 2)
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
    cells: Vec<B::Color>,
    desc_rows: Vec<Rc<Description<B>>>,
    desc_cols: Vec<Rc<Description<B>>>,
    palette: Option<ColorPalette>,
    all_colors: Vec<ColorId>,
    cache_rows: Option<LineSolverCache<B>>,
    cache_cols: Option<LineSolverCache<B>>,
    // use with caching duplicated clues
    // https://webpbn.com/survey/caching.html
    rows_cache_indexes: Vec<usize>,
    cols_cache_indexes: Vec<usize>,
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Copy,
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
        let cells = vec![init; width * height];

        let uniq_rows = dedup(&rows.iter().map(|desc| desc.vec.clone()).collect::<Vec<_>>());
        let uniq_cols = dedup(
            &columns
                .iter()
                .map(|desc| desc.vec.clone())
                .collect::<Vec<_>>(),
        );
        let rows_cache_indexes = rows
            .iter()
            .map(|desc| {
                uniq_rows
                    .iter()
                    .position(|uniq_row| uniq_row == &desc.vec)
                    .unwrap()
            })
            .collect();
        let cols_cache_indexes = columns
            .iter()
            .map(|desc| {
                uniq_cols
                    .iter()
                    .position(|unqi_col| unqi_col == &desc.vec)
                    .unwrap()
            })
            .collect();

        let desc_rows = rows.into_iter().map(Rc::new).collect();
        let desc_cols = columns.into_iter().map(Rc::new).collect();
        Self {
            cells,
            desc_rows,
            desc_cols,
            palette,
            all_colors,
            cache_rows: None,
            cache_cols: None,
            rows_cache_indexes,
            cols_cache_indexes,
        }
    }

    fn all_colors(descriptions: &[Description<B>]) -> Vec<ColorId> {
        let mut colors: Vec<_> = descriptions
            .iter()
            .flat_map(|row| row.vec.iter().map(|block| block.color().as_color_id()))
            .collect();

        colors.push(ColorPalette::WHITE_ID);
        dedup(&colors)
    }

    pub fn desc_by_id(&self, id: ColorId) -> Option<ColorDesc> {
        if let Some(palette) = &self.palette {
            palette.desc_by_id(id)
        } else {
            None
        }
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[B::Color]> {
        self.cells.chunks(self.width())
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
        self.cells.iter().all(|cell| cell.is_solved())
    }

    fn get_row_slice(&self, index: usize) -> &[B::Color] {
        //let width = self.width();
        //let start_index = width * index;
        //self.cells.iter().skip(start_index).take(width)
        self.iter_rows().nth(index).unwrap()
    }

    pub fn get_row(&self, index: usize) -> Vec<B::Color> {
        self.get_row_slice(index).to_vec()
    }

    pub fn get_column(&self, index: usize) -> Vec<B::Color> {
        self.cells
            .iter()
            .skip(index)
            .step_by(self.width())
            .cloned()
            .collect()
    }

    fn linear_index(&self, row_index: usize, column_index: usize) -> usize {
        let width = self.width();
        row_index * width + column_index
    }

    pub fn set_row(&mut self, index: usize, new: &[B::Color]) {
        let row_start = self.linear_index(index, 0);
        (row_start..)
            .zip(new)
            .for_each(|(linear_index, &new_cell)| {
                self.cells[linear_index] = new_cell;
            });
    }

    pub fn set_column(&mut self, index: usize, new: &[B::Color]) {
        let width = self.width();
        let column_indexes = (index..).step_by(width);

        column_indexes
            .zip(new)
            .for_each(|(linear_index, &new_cell)| {
                self.cells[linear_index] = new_cell;
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
        self.line_solution_rate(self.get_row_slice(index))
    }

    /// How many cells in the column with given index are known to be of particular color
    pub fn column_solution_rate(&self, index: usize) -> f64 {
        self.line_solution_rate(&self.get_column(index))
    }

    /// How many cells in the whole grid are known to be of particular color
    pub fn solution_rate(&self) -> f64 {
        self.line_solution_rate(&self.cells)
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
    fn neighbours(&self, point: &Point) -> Vec<Point> {
        let Point { x, y } = *point;
        let mut res = Vec::with_capacity(4);
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
        #[allow(clippy::unnecessary_filter_map)]
        self.neighbours(&point).into_iter().filter_map(move |n| {
            if self.cell(&n).is_solved() {
                None
            } else {
                Some(n)
            }
        })
    }

    pub fn init_cache(&mut self) {
        let width = self.width();
        let height = self.height();

        self.cache_rows = Some(new_cache::<B>(1_000 * height));
        self.cache_cols = Some(new_cache::<B>(1_000 * width));
    }

    pub fn cached_solution(&mut self, is_column: bool, key: &CacheKey<B>) -> Option<CacheValue<B>> {
        let cache = if is_column {
            self.cache_cols.as_mut()
        } else {
            self.cache_rows.as_mut()
        };

        cache.and_then(|cache| cache.cache_get(key).cloned())
    }

    pub fn set_cached_solution(
        &mut self,
        is_column: bool,
        key: CacheKey<B>,
        solved: CacheValue<B>,
    ) {
        let cache = if is_column {
            self.cache_cols.as_mut()
        } else {
            self.cache_rows.as_mut()
        };

        if let Some(cache) = cache {
            cache.cache_set(key, solved)
        }
    }

    pub fn print_cache_info(&self) {
        if let Some(cache) = &self.cache_cols {
            let (s, h, r) = cache_info(cache);
            warn!("Cache columns: Size={}, hits={}, hit rate={}.", s, h, r);
        }
        if let Some(cache) = &self.cache_rows {
            let (s, h, r) = cache_info(cache);
            warn!("Cache rows: Size={}, hits={}, hit rate={}.", s, h, r);
        }
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
    /// Difference between two boards as coordinates of changed cells.
    /// Standard diff semantic as result:
    /// - first returned points which set in current board and unset in the other
    /// - second returned points which unset in current board and set in the other
    pub fn diff(&self, other: &[B::Color]) -> (Vec<Point>, Vec<Point>) {
        let mut removed = vec![];
        let mut added = vec![];

        let other = other.chunks(self.width());
        for (y, (row, other_row)) in self.iter_rows().zip(other).enumerate() {
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

    pub fn make_snapshot(&self) -> Vec<B::Color> {
        self.cells.clone()
    }

    pub fn restore(&mut self, cells: Vec<B::Color>) {
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
        let index = self.linear_index(y, x);
        self.cells[index] = old_value + *color;
    }

    pub fn unset_color(&mut self, point: &Point, color: &B::Color) -> Result<(), String> {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        let index = self.linear_index(y, x);
        self.cells[index] = (old_value - *color)?;
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
            cache_rows: None,
            cache_cols: None,
            rows_cache_indexes: self.rows_cache_indexes.clone(),
            cols_cache_indexes: self.cols_cache_indexes.clone(),
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
        assert_eq!(board.cells.len(), 9);
        assert_eq!(board.get_row(0), &[Undefined, Undefined, Undefined]);
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
        assert_eq!(board.get_row(0), &[Undefined]);
        assert_eq!(board.desc_rows[0].vec, vec![BinaryBlock(1)]);
        assert_eq!(board.desc_rows[1].vec, vec![]);
        assert_eq!(board.desc_rows[2].vec, vec![BinaryBlock(1)]);
    }
}
