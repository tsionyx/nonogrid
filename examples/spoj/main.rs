use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::marker::Sized;
use std::ops::{Add, Sub};
use std::rc::Rc;
use std::slice::Chunks;

use iter::StepByIter;
use line::LineSolver;
use probing::ProbeSolver;

pub trait Color
where
    Self: Debug
        + Eq
        + Hash
        + Default
        + Copy
        + Send
        + Sync
        + Ord
        + Add<Output = Self>
        + Sub<Output = Result<Self, String>>,
{
    fn blank() -> Self;
    fn is_solved(&self) -> bool;
    fn solution_rate(&self) -> f64;
    fn is_updated_with(&self, new: &Self) -> Result<bool, String>;
    fn variants(&self) -> Vec<Self>
    where
        Self: Sized;
}

pub trait Block
where
    Self: Debug + Eq + Hash + Default + Copy + Send + Sync,
{
    type Color: Color;

    fn from_size(size: usize) -> Self;
    fn partial_sums(desc: &[Self]) -> Vec<usize>
    where
        Self: Sized;

    fn size(&self) -> usize;
    fn color(&self) -> Self::Color;
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum BinaryColor {
    Undefined,
    White,
    Black,
    // especially for DynamicSolver
    BlackOrWhite,
}

impl Default for BinaryColor {
    fn default() -> Self {
        BinaryColor::Undefined
    }
}

impl Color for BinaryColor {
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

    fn variants(&self) -> Vec<Self> {
        if self.is_solved() {
            vec![*self]
        } else {
            vec![BinaryColor::White, BinaryColor::Black]
        }
    }
}

impl fmt::Display for BinaryColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BinaryColor::*;

        let symbol = match *self {
            White => '.',
            Black => '#',
            Undefined | BlackOrWhite => '?',
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

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct BinaryBlock(pub usize);

impl Block for BinaryBlock {
    type Color = BinaryColor;

    fn from_size(size: usize) -> Self {
        BinaryBlock(size)
    }

    fn partial_sums(desc: &[Self]) -> Vec<usize> {
        desc.iter()
            .scan(None, |prev, block| {
                let current = if let Some(prev_size) = *prev {
                    prev_size + block.0 + 1
                } else {
                    block.0
                };
                *prev = Some(current);
                *prev
            })
            .collect()
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
    pub fn new(mut vec: Vec<T>) -> Self {
        let zero = T::default();
        vec.retain(|x| *x != zero);
        Description::<T> { vec: vec }
    }
}

pub fn replace<T>(vec: &mut Vec<T>, what: &T, with_what: T)
where
    T: PartialEq + Clone,
{
    if what == &with_what {
        return;
    }

    if !vec.contains(what) {
        return;
    }

    let replaced_indexes: Vec<_> = vec
        .iter()
        .enumerate()
        .filter_map(|(index, val)| if val == what { Some(index) } else { None })
        .collect();

    vec.extend(vec![with_what; replaced_indexes.len()]);
    for index in replaced_indexes {
        vec.swap_remove(index);
    }
}

pub fn two_powers(mut num: u32) -> Vec<u32> {
    let mut res = vec![];
    while num > 0 {
        let rest = num & (num - 1);
        res.push(num - rest);
        num = rest
    }
    res
}

pub fn from_two_powers(numbers: &[u32]) -> u32 {
    numbers.iter().fold(0, |acc, &x| acc | x)
}

pub fn is_power_of_2(x: u32) -> bool {
    if x == 0 {
        return false;
    }

    x & (x - 1) == 0
}

pub fn dedup<T>(vec: &[T]) -> Vec<T>
where
    T: Eq + Hash + Clone,
{
    let set: HashSet<_> = vec.iter().cloned().collect();
    set.into_iter().collect()
}

pub fn product<T, U>(s1: &[T], s2: &[U]) -> Vec<(T, U)>
where
    T: Clone,
    U: Clone,
{
    s1.iter()
        .flat_map(|x| s2.iter().map(move |y| (x.clone(), y.clone())))
        .collect()
}

pub mod time {
    use std::time::Instant;

    #[cfg(feature = "std_time")]
    pub fn now() -> Option<Instant> {
        Some(Instant::now())
    }

    #[cfg(not(feature = "std_time"))]
    pub fn now() -> Option<Instant> {
        None
    }
}

pub struct GrowableCache<K, V>
where
    K: Eq + Hash,
{
    store: HashMap<K, V>,
    hits: u32,
    misses: u32,
}

impl<K: Hash + Eq, V> GrowableCache<K, V> {
    pub fn with_capacity(size: usize) -> Self {
        GrowableCache::<K, V> {
            store: HashMap::with_capacity(size),
            hits: 0,
            misses: 0,
        }
    }

    fn cache_get(&mut self, key: &K) -> Option<&V> {
        if let Some(v) = self.store.get(key) {
            self.hits += 1;
            Some(v)
        } else {
            self.misses += 1;
            None
        }
    }
    fn cache_set(&mut self, key: K, val: V) {
        self.store.insert(key, val);
    }
}

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
    GrowableCache::with_capacity(capacity)
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Point { x: x, y: y }
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
    cache_rows: Option<LineSolverCache<B>>,
    cache_cols: Option<LineSolverCache<B>>,
    rows_cache_indexes: Vec<usize>,
    cols_cache_indexes: Vec<usize>,
}

mod iter {
    pub struct StepBy<I> {
        iter: I,
        step: usize,
        first_take: bool,
    }

    impl<I> StepBy<I> {
        pub fn new(iter: I, step: usize) -> StepBy<I> {
            StepBy {
                iter: iter,
                step: step - 1,
                first_take: true,
            }
        }
    }

    impl<I> Iterator for StepBy<I>
    where
        I: Iterator,
    {
        type Item = I::Item;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.first_take {
                self.first_take = false;
                self.iter.next()
            } else {
                self.iter.nth(self.step)
            }
        }
    }

    pub trait StepByIter: Iterator {
        fn step_by(self, step: usize) -> StepBy<Self>
        where
            Self: Sized,
        {
            StepBy::new(self, step)
        }
    }

    impl<I: Iterator> StepByIter for I {}
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Copy,
{
    pub fn with_descriptions(rows: Vec<Description<B>>, columns: Vec<Description<B>>) -> Self {
        let height = rows.len();
        let width = columns.len();

        let init = B::Color::default();
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

        let desc_rows = rows.into_iter().map(Rc::new).collect();
        let desc_cols = columns.into_iter().map(Rc::new).collect();
        Board::<B> {
            cells: cells,
            desc_rows: desc_rows,
            desc_cols: desc_cols,
            cache_rows: None,
            cache_cols: None,
            rows_cache_indexes: rows_cache_indexes,
            cols_cache_indexes: cols_cache_indexes,
        }
    }

    pub fn iter_rows(&self) -> Chunks<B::Color> {
        self.cells.chunks(self.width())
    }

    pub fn descriptions(&self, rows: bool) -> &[Rc<Description<B>>] {
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
        self.iter_rows().nth(index).expect("Invalid row index")
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

    fn set_row(&mut self, index: usize, new: &[B::Color]) {
        let row_start = self.linear_index(index, 0);
        for (linear_index, &new_cell) in (row_start..).zip(new) {
            self.cells[linear_index] = new_cell;
        }
    }

    fn set_column(&mut self, index: usize, new: &[B::Color]) {
        let width = self.width();

        for (i, &new_cell) in new.iter().enumerate() {
            let linear_index = (i * width) + index;
            self.cells[linear_index] = new_cell;
        }
    }

    /// How many cells in a line are known to be of particular color
    pub fn line_solution_rate(&self, line: &[B::Color], size: usize) -> f64 {
        let solved: f64 = line.iter().map(|cell| cell.solution_rate()).sum();
        solved / size as f64
    }

    /// How many cells in the row with given index are known to be of particular color
    pub fn row_solution_rate(&self, index: usize) -> f64 {
        self.line_solution_rate(&self.get_row(index), self.width())
    }

    /// How many cells in the column with given index are known to be of particular color
    pub fn column_solution_rate(&self, index: usize) -> f64 {
        self.line_solution_rate(&self.get_column(index), self.height())
    }

    /// How many cells in the whole grid are known to be of particular color
    pub fn solution_rate(&self) -> f64 {
        self.line_solution_rate(&self.cells, self.height() * self.width())
    }

    pub fn unsolved_cells(&self) -> Vec<Point> {
        self.iter_rows()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, cell)| {
                    if cell.is_solved() {
                        None
                    } else {
                        Some(Point::new(x, y))
                    }
                })
            })
            .collect()
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
    pub fn unsolved_neighbours(&self, point: &Point) -> Vec<Point> {
        self.neighbours(&point)
            .into_iter()
            .filter(move |n| !self.cell(n).is_solved())
            .collect()
    }

    pub fn init_cache(&mut self) {
        let width = self.width();
        let height = self.height();

        self.cache_rows = Some(new_cache::<B>(2_000 * height));
        self.cache_cols = Some(new_cache::<B>(2_000 * width));
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
            for (x, (cell, other_cell)) in row.iter().zip(other_row).enumerate() {
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

    fn restore(&mut self, cells: Vec<B::Color>) {
        self.cells = cells;
    }
}

impl<B> Board<B>
where
    B: Block,
    B::Color: Copy,
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

mod line {
    use std::rc::Rc;

    use super::{replace, Block, Color, Description};

    pub trait LineSolver {
        type BlockType: Block;

        fn new(
            desc: Rc<Description<Self::BlockType>>,
            line: Rc<Vec<<Self::BlockType as Block>::Color>>,
        ) -> Self;
        fn solve(&mut self) -> Result<(), String>;
        fn get_solution(self) -> Vec<<Self::BlockType as Block>::Color>;
    }

    pub fn solve<L, B>(
        desc: Rc<Description<B>>,
        line: Rc<Vec<B::Color>>,
    ) -> Result<Vec<B::Color>, String>
    where
        L: LineSolver<BlockType = B>,
        B: Block,
    {
        let mut solver = L::new(desc, line);
        solver.solve()?;
        Ok(solver.get_solution())
    }

    pub trait DynamicColor: Color
    where
        Self: Sized,
    {
        // it can be implemented very simple with generics specialization
        // https://github.com/aturon/rfcs/blob/impl-specialization/text/0000-impl-specialization.md
        // https://github.com/rust-lang/rfcs/issues/1053
        fn set_additional_blank(line: Rc<Vec<Self>>) -> (Rc<Vec<Self>>, bool) {
            (line, false)
        }
        fn both_colors() -> Option<Self>;

        fn can_be_blank(&self) -> bool;
        fn can_be(&self, color: Self) -> bool;
        fn add_color(&self, color: Self) -> Self;
        fn solved_copy(&self) -> Self;
    }

    pub struct DynamicSolver<B: Block, S = <B as Block>::Color> {
        desc: Rc<Description<B>>,
        line: Rc<Vec<S>>,
        additional_space: bool,
        block_sums: Vec<usize>,
        job_size: usize,
        solution_matrix: Vec<Option<bool>>,
        solved_line: Vec<S>,
    }

    impl<B> LineSolver for DynamicSolver<B>
    where
        B: Block,
        B::Color: DynamicColor,
    {
        type BlockType = B;

        fn new(desc: Rc<Description<B>>, line: Rc<Vec<B::Color>>) -> Self {
            let (line, additional_space) = B::Color::set_additional_blank(line);

            let block_sums = Self::calc_block_sum(&*desc);

            let job_size = desc.vec.len() + 1;
            let solution_matrix = vec![None; job_size * line.len()];

            let solved_line = line.iter().map(DynamicColor::solved_copy).collect();

            DynamicSolver::<B> {
                desc: desc,
                line: line,
                additional_space: additional_space,
                block_sums: block_sums,
                job_size: job_size,
                solution_matrix: solution_matrix,
                solved_line: solved_line,
            }
        }

        fn solve(&mut self) -> Result<(), String> {
            if self.try_solve() {
                let mut solved = &mut self.solved_line;
                if self.additional_space {
                    let new_size = solved.len() - 1;
                    solved.truncate(new_size);
                }

                let both = B::Color::both_colors();
                if let Some(both) = both {
                    let init = B::Color::default();
                    replace(&mut solved, &both, init);
                }
                Ok(())
            } else {
                Err("Bad line".to_string())
            }
        }

        fn get_solution(self) -> Vec<B::Color> {
            self.solved_line
        }
    }

    impl<B> DynamicSolver<B>
    where
        B: Block,
        B::Color: DynamicColor,
    {
        fn calc_block_sum(desc: &Description<B>) -> Vec<usize> {
            let mut min_indexes: Vec<_> = B::partial_sums(&desc.vec)
                .iter()
                .map(|size| size - 1)
                .collect();
            min_indexes.insert(0, 0);
            min_indexes
        }

        fn try_solve(&mut self) -> bool {
            if self.line.is_empty() {
                return true;
            }

            let (position, block) = (self.line.len() - 1, self.desc.vec.len());
            self.get_sol(position as isize, block)
        }

        fn _get_sol(&self, position: usize, block: usize) -> Option<bool> {
            self.solution_matrix[position * self.job_size + block]
        }

        fn get_sol(&mut self, position: isize, block: usize) -> bool {
            if position < 0 {
                // finished placing the last block, exactly at the beginning of the line.
                return block == 0;
            }

            let position = position as usize;

            let can_be_solved = self._get_sol(position, block);
            can_be_solved.unwrap_or_else(|| {
                let can_be_solved = self.fill_matrix(position, block);
                self.set_sol(position, block, can_be_solved);
                can_be_solved
            })
        }

        fn set_sol(&mut self, position: usize, block: usize, can_be_solved: bool) {
            self.solution_matrix[position * self.job_size + block] = Some(can_be_solved)
        }

        fn color_at(&self, position: usize) -> B::Color {
            self.line[position]
        }

        fn block_at(&self, block_position: usize) -> B {
            self.desc.vec[block_position]
        }

        fn update_solved(&mut self, position: usize, color: B::Color) {
            let current = self.solved_line[position];
            self.solved_line[position] = current.add_color(color);
        }

        fn fill_matrix(&mut self, position: usize, block: usize) -> bool {
            // too many blocks left to fit this line segment
            if position < self.block_sums[block] {
                return false;
            }

            // do not short-circuit
            self.fill_matrix_blank(position, block) | self.fill_matrix_color(position, block)
        }

        fn fill_matrix_blank(&mut self, position: usize, block: usize) -> bool {
            if self.color_at(position).can_be_blank() {
                // current cell is either blank or unknown
                let has_blank = self.get_sol(position as isize - 1, block);
                if has_blank {
                    let blank = B::Color::blank();
                    // set cell blank and continue
                    self.update_solved(position, blank);
                    return true;
                }
            }

            false
        }

        fn fill_matrix_color(&mut self, position: usize, block: usize) -> bool {
            // block == 0 means we finished filling all the blocks (can still fill whitespace)
            if block == 0 {
                return false;
            }
            let current_block = self.block_at(block - 1);
            let mut block_size = current_block.size();
            let current_color = current_block.color();
            let should_have_trailing_space = self.trail_with_space(block);
            if should_have_trailing_space {
                block_size += 1;
            }

            let block_start = position as isize - block_size as isize + 1;

            // (position-block_size, position]
            if self.can_place_color(
                block_start,
                position,
                current_color,
                should_have_trailing_space,
            ) {
                let has_color = self.get_sol(block_start - 1, block - 1);
                if has_color {
                    // set cell blank, place the current block and continue
                    self.set_color_block(
                        block_start,
                        position,
                        current_color,
                        should_have_trailing_space,
                    );
                    return true;
                }
            }

            false
        }

        fn trail_with_space(&self, block: usize) -> bool {
            if block < self.desc.vec.len() {
                let current_color = self.block_at(block - 1).color();
                let next_color = self.block_at(block).color();

                if next_color == current_color {
                    return true;
                }
            }

            false
        }

        fn can_place_color(
            &self,
            start: isize,
            mut end: usize,
            color: B::Color,
            trailing_space: bool,
        ) -> bool {
            if start < 0 {
                return false;
            }

            if trailing_space {
                if !self.color_at(end).can_be_blank() {
                    return false;
                }
            } else {
                end += 1;
            }

            // the color can be placed in every cell
            self.line[start as usize..end]
                .iter()
                .all(|cell| cell.can_be(color))
        }

        fn set_color_block(
            &mut self,
            start: isize,
            mut end: usize,
            color: B::Color,
            trailing_space: bool,
        ) {
            if trailing_space {
                let blank = B::Color::blank();
                self.update_solved(end, blank);
            } else {
                end += 1
            }

            // set colored cells
            for i in start as usize..end {
                self.update_solved(i, color);
            }
        }
    }
}

impl line::DynamicColor for BinaryColor {
    fn both_colors() -> Option<Self> {
        Some(BinaryColor::BlackOrWhite)
    }

    fn can_be_blank(&self) -> bool {
        self != &BinaryColor::Black
    }

    fn can_be(&self, _always_black: Self) -> bool {
        self != &Self::blank()
    }

    fn add_color(&self, color: Self) -> Self {
        match *self {
            BinaryColor::Undefined => color,
            value => {
                if value == color {
                    value
                } else {
                    BinaryColor::BlackOrWhite
                }
            }
        }
    }

    fn solved_copy(&self) -> Self {
        *self
    }
}

mod propagation {
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::rc::Rc;

    use super::line;
    use super::line::LineSolver;
    use super::{Block, Board, Description, Point};

    pub struct Solver<B>
    where
        B: Block,
    {
        board: Rc<RefCell<Board<B>>>,
        point: Option<Point>,
    }

    type Job = (bool, usize);

    trait JobQueue {
        fn push(&mut self, job: Job);
        fn pop(&mut self) -> Option<Job>;
    }

    struct SmallJobQueue {
        vec: Vec<Job>,
    }

    impl SmallJobQueue {
        fn with_point(point: Point) -> Self {
            SmallJobQueue {
                vec: vec![(true, point.x()), (false, point.y())],
            }
        }
    }

    impl JobQueue for SmallJobQueue {
        fn push(&mut self, job: Job) {
            self.vec.push(job)
        }

        fn pop(&mut self) -> Option<Job> {
            let top_job = self.vec.pop();
            if top_job.is_none() {
                return None;
            }

            let top_job = top_job.unwrap();
            self.vec.retain(|&x| x != top_job);
            Some(top_job)
        }
    }

    struct LongJobQueue {
        vec: Vec<Job>,
        visited: HashSet<Job>,
    }

    impl LongJobQueue {
        fn with_rows_and_columns(rows: Vec<usize>, columns: Vec<usize>) -> Self {
            let mut jobs: Vec<_> = columns
                .into_iter()
                .map(|column_index| (true, column_index))
                .collect();
            jobs.extend(rows.into_iter().map(|row_index| (false, row_index)));

            LongJobQueue {
                vec: jobs,
                visited: HashSet::new(),
            }
        }
    }

    impl JobQueue for LongJobQueue {
        fn push(&mut self, job: Job) {
            self.visited.remove(&job);
            self.vec.push(job)
        }

        fn pop(&mut self) -> Option<Job> {
            let mut top_job;
            loop {
                let top = self.vec.pop();
                if top.is_none() {
                    return None;
                }

                top_job = top.unwrap();
                if !self.visited.contains(&top_job) {
                    break;
                }
            }
            // mark the job as visited
            self.visited.insert(top_job);
            Some(top_job)
        }
    }

    impl<B> Solver<B>
    where
        B: Block,
    {
        pub fn new(board: Rc<RefCell<Board<B>>>) -> Self {
            Solver::<B> {
                board: board,
                point: None,
            }
        }

        pub fn with_point(board: Rc<RefCell<Board<B>>>, point: Point) -> Self {
            Solver::<B> {
                board: board,
                point: Some(point),
            }
        }

        pub fn run<S>(&self) -> Result<Vec<Point>, String>
        where
            S: LineSolver<BlockType = B>,
        {
            if let Some(point) = self.point {
                let queue = SmallJobQueue::with_point(point);
                self.run_jobs::<S, _>(queue)
            } else {
                let queue = {
                    let board = self.board.borrow();
                    let rows: Vec<_> = (0..board.height()).rev().collect();
                    let cols: Vec<_> = (0..board.width()).rev().collect();

                    // `is_solved_full` is expensive, so minimize calls to it.
                    // Do not call if only a handful of lines has to be solved
                    if board.is_solved_full() {
                        //return 0, ()
                    }
                    LongJobQueue::with_rows_and_columns(rows, cols)
                };
                self.run_jobs::<S, _>(queue)
            }
        }

        fn run_jobs<S, Q>(&self, mut queue: Q) -> Result<Vec<Point>, String>
        where
            S: LineSolver<BlockType = B>,
            Q: JobQueue,
        {
            let mut solved_cells = vec![];

            while let Some((is_column, index)) = queue.pop() {
                let new_jobs = self.update_line::<S>(index, is_column)?;

                let new_states = new_jobs.iter().map(|another_index| {
                    let (x, y) = if is_column {
                        (&index, another_index)
                    } else {
                        (another_index, &index)
                    };
                    Point::new(*x, *y)
                });

                solved_cells.extend(new_states);

                for new_index in new_jobs.iter().rev() {
                    queue.push((!is_column, *new_index))
                }
            }

            Ok(solved_cells)
        }

        /// Solve a line with the solver S and update the board.
        /// If the line gets partially solved, put the crossed lines into queue.
        ///
        /// Return the list of indexes which was updated during this solution.
        pub fn update_line<S>(&self, index: usize, is_column: bool) -> Result<Vec<usize>, String>
        where
            S: LineSolver<BlockType = B>,
        {
            let (line_desc, line) = {
                let board = self.board.borrow();
                if is_column {
                    (
                        Rc::clone(&board.descriptions(false)[index]),
                        board.get_column(index),
                    )
                } else {
                    (
                        Rc::clone(&board.descriptions(true)[index]),
                        board.get_row(index),
                    )
                }
            };

            let line = Rc::new(line);
            let solution = self.solve::<S>(index, is_column, line_desc, Rc::clone(&line))?;
            let indexes = self.update_solved(index, is_column, &line, &solution);

            Ok(indexes)
        }

        fn update_solved(
            &self,
            index: usize,
            is_column: bool,
            old: &[B::Color],
            new: &[B::Color],
        ) -> Vec<usize> {
            if old == new {
                return vec![];
            }

            if is_column {
                self.board.borrow_mut().set_column(index, new);
            } else {
                self.board.borrow_mut().set_row(index, new);
            }

            old.iter()
                .zip(new)
                .enumerate()
                .filter_map(|(i, (pre, post))| if pre != post { Some(i) } else { None })
                .collect()
        }

        fn solve<S>(
            &self,
            index: usize,
            is_column: bool,
            line_desc: Rc<Description<B>>,
            line: Rc<Vec<B::Color>>,
        ) -> Result<Rc<Vec<B::Color>>, String>
        where
            S: LineSolver<BlockType = B>,
        {
            let cache_index = if is_column {
                self.board.borrow().column_cache_index(index)
            } else {
                self.board.borrow().row_cache_index(index)
            };
            let key = (cache_index, Rc::clone(&line));

            let cached = self.board.borrow_mut().cached_solution(is_column, &key);

            if let Some(cached) = cached {
                return cached;
            }

            let value = line::solve::<S, _>(line_desc, line);

            let rc_value = value.map(Rc::new);
            self.board
                .borrow_mut()
                .set_cached_solution(is_column, key, rc_value.clone());
            rc_value
        }
    }
}

mod probing {
    use std::cell::{Ref, RefCell};
    use std::collections::BinaryHeap;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::line::LineSolver;
    use super::{propagation, Block, Board, Color, Point};

    pub fn priority_ord(p: f64) -> u32 {
        (p * 1000.0) as u32
    }

    pub type Impact<B> = HashMap<(Point, <B as Block>::Color), (usize, u32)>;
    type FloatPriorityQueue<K> = BinaryHeap<K>;

    pub trait ProbeSolver {
        type BlockType: Block;

        fn with_board(board: Rc<RefCell<Board<Self::BlockType>>>) -> Self;

        fn unsolved_cells(&self) -> FloatPriorityQueue<(u32, Point)>;
        fn propagate_point<S>(&self, point: &Point) -> Result<Vec<(u32, Point)>, String>
        where
            S: LineSolver<BlockType = Self::BlockType>;

        fn run_unsolved<S>(&self) -> Result<Impact<Self::BlockType>, String>
        where
            S: LineSolver<BlockType = Self::BlockType>,
        {
            self.run::<S>(&mut self.unsolved_cells())
        }

        fn run<S>(
            &self,
            probes: &mut FloatPriorityQueue<(u32, Point)>,
        ) -> Result<Impact<Self::BlockType>, String>
        where
            S: LineSolver<BlockType = Self::BlockType>;
    }

    pub struct FullProbe1<B>
    where
        B: Block,
    {
        board: Rc<RefCell<Board<B>>>,
    }

    const PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED: f64 = 10.0;
    const PRIORITY_NEIGHBOURS_OF_CONTRADICTION: f64 = 20.0;

    impl<B> ProbeSolver for FullProbe1<B>
    where
        B: Block,
    {
        type BlockType = B;

        fn with_board(board: Rc<RefCell<Board<B>>>) -> Self {
            board.borrow_mut().init_cache();
            FullProbe1::<B> { board: board }
        }

        fn unsolved_cells(&self) -> FloatPriorityQueue<(u32, Point)> {
            let board = self.board();
            let unsolved = board.unsolved_cells();

            let mut queue = FloatPriorityQueue::new();
            queue.extend(unsolved.into_iter().map(|point| {
                let no_unsolved = board.unsolved_neighbours(&point).len() as f64;
                let row_rate = board.row_solution_rate(point.y());
                let column_rate = board.column_solution_rate(point.x());
                let priority = row_rate + column_rate - no_unsolved + 4.0;
                (priority_ord(priority), point)
            }));

            queue
        }

        fn propagate_point<S>(&self, point: &Point) -> Result<Vec<(u32, Point)>, String>
        where
            S: LineSolver<BlockType = B>,
        {
            let fixed_points = self.run_propagation::<S>(point)?;
            let mut new_jobs = vec![];

            for new_point in fixed_points {
                for neighbour in self.board().unsolved_neighbours(&new_point) {
                    new_jobs.push((priority_ord(PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED), neighbour));
                }
            }

            for neighbour in self.board().unsolved_neighbours(&point) {
                new_jobs.push((
                    priority_ord(PRIORITY_NEIGHBOURS_OF_CONTRADICTION),
                    neighbour,
                ));
            }

            Ok(new_jobs)
        }

        fn run<S>(
            &self,
            probes: &mut FloatPriorityQueue<(u32, Point)>,
        ) -> Result<Impact<Self::BlockType>, String>
        where
            S: LineSolver<BlockType = B>,
        {
            let mut impact;
            loop {
                impact = HashMap::new();

                if self.is_solved() {
                    break;
                }

                let mut false_probes = None;

                while let Some((priority, point)) = probes.pop() {
                    let probe_results = self.probe::<S>(point);

                    let (contradictions, non_contradictions): (Vec<_>, Vec<_>) = probe_results
                        .into_iter()
                        .partition(|&(_color, size)| size.is_none());

                    if !contradictions.is_empty() {
                        let bad_colors: Vec<_> = contradictions
                            .iter()
                            .map(|&(color, _should_be_none)| color)
                            .collect();

                        false_probes = Some((point, bad_colors));
                        break;
                    }

                    for (color, updated) in non_contradictions {
                        if let Some(updated_cells) = updated {
                            impact.insert((point, color), (updated_cells, priority));
                        }
                    }
                }

                if let Some((contradiction, colors)) = false_probes {
                    for color in colors {
                        self.board
                            .borrow_mut()
                            .unset_color(&contradiction, &color)?;
                    }
                    let new_probes = self.propagate_point::<S>(&contradiction)?;
                    for (priority, point) in new_probes {
                        probes.push((priority, point));
                    }
                } else {
                    break;
                }
            }

            Ok(impact)
        }
    }

    impl<B> FullProbe1<B>
    where
        B: Block,
    {
        fn board(&self) -> Ref<Board<B>> {
            self.board.borrow()
        }

        fn run_propagation<S>(&self, point: &Point) -> Result<Vec<Point>, String>
        where
            S: LineSolver<BlockType = B>,
        {
            let point_solver = propagation::Solver::with_point(Rc::clone(&self.board), *point);
            point_solver.run::<S>()
        }

        fn is_solved(&self) -> bool {
            self.board().is_solved_full()
        }

        fn probe<S>(&self, point: Point) -> HashMap<B::Color, Option<usize>>
        where
            S: LineSolver<BlockType = B>,
        {
            let mut changes = HashMap::new();

            let vars = self.board().cell(&point).variants();

            for assumption in vars {
                let save = self.board().make_snapshot();
                self.board.borrow_mut().set_color(&point, &assumption);

                let solved = self.run_propagation::<S>(&point);
                self.board.borrow_mut().restore(save);

                changes.insert(assumption, solved.ok().map(|new_cells| new_cells.len()));
            }

            changes
        }
    }
}

mod rev {
    use std::cmp::Ordering;

    #[derive(PartialEq, Eq)]
    pub struct Reverse<T>(pub T);

    impl<T: PartialOrd> PartialOrd for Reverse<T> {
        fn partial_cmp(&self, other: &Reverse<T>) -> Option<Ordering> {
            other.0.partial_cmp(&self.0)
        }

        fn lt(&self, other: &Self) -> bool {
            other.0 < self.0
        }
        fn le(&self, other: &Self) -> bool {
            other.0 <= self.0
        }
        fn gt(&self, other: &Self) -> bool {
            other.0 > self.0
        }
        fn ge(&self, other: &Self) -> bool {
            other.0 >= self.0
        }
    }

    impl<T: Ord> Ord for Reverse<T> {
        fn cmp(&self, other: &Reverse<T>) -> Ordering {
            other.0.cmp(&self.0)
        }
    }
}

mod backtracking {
    use std::cell::{Ref, RefCell};
    use std::collections::{HashMap, HashSet};
    use std::marker::PhantomData;
    use std::rc::Rc;

    use super::line::LineSolver;
    use super::probing::{priority_ord, Impact, ProbeSolver};
    use super::rev::Reverse;
    use super::{Block, Board, Color, Point};

    type Solution<B> = Vec<<B as Block>::Color>;

    pub struct Solver<B, P, S>
    where
        B: Block,
        P: ProbeSolver<BlockType = B>,
        S: LineSolver<BlockType = B>,
    {
        board: Rc<RefCell<Board<B>>>,
        probe_solver: P,

        // search options
        max_solutions: Option<usize>,

        // dynamic variables
        pub solutions: Vec<Solution<B>>,
        explored_paths: HashSet<Vec<(Point, B::Color)>>,

        _phantom: PhantomData<S>,
    }

    #[allow(dead_code)]
    enum ChoosePixel {
        Sum,
        Min,
        Max,
        Mul,
        Sqrt,
        MinLogm,
        MinLogd,
    }

    impl<B, P, S> Solver<B, P, S>
    where
        B: Block,
        P: ProbeSolver<BlockType = B>,
        S: LineSolver<BlockType = B>,
    {
        #[allow(dead_code)]
        pub fn new(board: Rc<RefCell<Board<B>>>) -> Self {
            Self::with_options(board, None)
        }

        pub fn with_options(board: Rc<RefCell<Board<B>>>, max_solutions: Option<usize>) -> Self {
            let probe_solver = P::with_board(Rc::clone(&board));
            Solver::<B, P, S> {
                board: board,
                probe_solver: probe_solver,
                max_solutions: max_solutions,
                solutions: vec![],
                explored_paths: HashSet::new(),
                _phantom: PhantomData,
            }
        }

        pub fn run(&mut self) -> Result<(), String> {
            if self.is_solved() {
                return Ok(());
            }

            let impact = self.probe_solver.run_unsolved::<S>()?;
            if self.is_solved() {
                return Ok(());
            }

            let directions = self.choose_directions(&impact);
            self.search(&directions, &[])?;
            Ok(())
        }

        fn board(&self) -> Ref<Board<B>> {
            self.board.borrow()
        }

        fn is_solved(&self) -> bool {
            self.board().is_solved_full()
        }

        fn set_explored(&mut self, path: &[(Point, B::Color)]) {
            let mut path = path.to_vec();
            path.sort();
            self.explored_paths.insert(path);
        }

        fn is_explored(&self, path: &[(Point, B::Color)]) -> bool {
            let mut path = path.to_vec();
            path.sort();
            self.explored_paths.contains(&path)
        }

        fn already_found(&self) -> bool {
            for (_i, solution) in self.solutions.iter().enumerate() {
                let (removed, added) = self.board().diff(solution);

                if removed.is_empty() && added.is_empty() {
                    return true;
                }
            }

            false
        }

        fn add_solution(&mut self) -> Result<(), String> {
            // force to check the board
            self.probe_solver.run_unsolved::<S>()?;

            if !self.already_found() {
                let cells = self.board().make_snapshot();
                self.solutions.push(cells);
            }

            Ok(())
        }

        /// The most promising (point+color) pair should go first
        fn choose_directions(&self, impact: &Impact<B>) -> Vec<(Point, B::Color)> {
            let mut point_wise = HashMap::new();

            for (&(point, color), &(new_points, priority)) in impact.iter() {
                if self.board().cell(&point).is_solved() {
                    continue;
                }
                let point_colors = point_wise.entry(point).or_insert_with(HashMap::new);
                point_colors.insert(color, (new_points, priority));
            }

            let mut points_rate: Vec<_> = point_wise
                .iter()
                .map(|(point, color_to_impact)| {
                    let values: Vec<_> = color_to_impact.values().into_iter().collect();
                    (point, priority_ord(Self::rate_by_impact(&values)))
                })
                .collect();
            points_rate.sort_by_key(|&(point, rate)| (Reverse(rate), point));

            points_rate
                .iter()
                .flat_map(|&(point, _rate)| {
                    let mut point_colors: Vec<_> =
                        point_wise[point].iter().map(|(k, v)| (*k, *v)).collect();
                    // the most impacting color goes first
                    point_colors
                        .sort_by_key(|&(_color, (new_points, _priority))| Reverse(new_points));
                    let point_order: Vec<_> = point_colors
                        .iter()
                        .map(|&(color, _impact)| (*point, color))
                        .collect();
                    point_order
                })
                .collect::<Vec<_>>()
        }

        fn choose_strategy() -> ChoosePixel {
            ChoosePixel::Sqrt
        }

        fn rate_by_impact(impact: &[&(usize, u32)]) -> f64 {
            let sizes_only: Vec<_> = impact
                .iter()
                .map(|&&(new_points, _priority)| new_points)
                .collect();

            let zero = 0;
            let min = sizes_only.iter().min().unwrap_or(&zero);
            let max = sizes_only.iter().max().unwrap_or(&zero);
            let sum = sizes_only.iter().sum::<usize>();

            let log = |f: f64| (1.0 + f).ln() + 1.0;

            // Max is the most trivial, but also most ineffective strategy.
            // For details, see https://ieeexplore.ieee.org/document/6476646
            match Self::choose_strategy() {
                ChoosePixel::Sum => sum as f64,
                ChoosePixel::Min => *min as f64,
                ChoosePixel::Max => *max as f64,
                ChoosePixel::Mul => sizes_only.iter().map(|x| (x + 1) as f64).product(),
                ChoosePixel::Sqrt => (*max as f64 / (min + 1) as f64).sqrt() + (*min as f64),
                ChoosePixel::MinLogm => {
                    let logm: f64 = sizes_only.iter().map(|&x| log(x as f64)).product();
                    logm + (*min as f64)
                }
                ChoosePixel::MinLogd => match sizes_only.as_slice() {
                    x if x.len() == 2 => {
                        let (first, second) = (x[0], x[1]);
                        let diff = log(first as f64) - log(second as f64);
                        *min as f64 + diff.abs()
                    }
                    _other => *min as f64,
                },
            }
        }

        /// Recursively search for solutions.
        /// Return False if the given path is a dead end (no solutions can be found)
        fn search(
            &mut self,
            directions: &[(Point, B::Color)],
            path: &[(Point, B::Color)],
        ) -> Result<bool, String> {
            if self.is_explored(path) {
                return Ok(true);
            }

            if self.limits_reached() {
                return Ok(true);
            }

            let save = self.board().make_snapshot();
            let result = self.search_mutable(directions, path);

            // do not restore the solved cells on a root path - they are really solved!
            if !path.is_empty() {
                self.board.borrow_mut().restore(save);
                self.set_explored(path);
            }

            result
        }

        fn search_mutable(
            &mut self,
            directions: &[(Point, B::Color)],
            path: &[(Point, B::Color)],
        ) -> Result<bool, String> {
            // this variable shows whether the board changed after the last probing
            // when the probing occurs it should immediately set to 'false'
            // to prevent succeeded useless probing on the same board
            let mut board_changed = true;
            //let mut search_counter = 0_u32;

            let mut directions = directions.to_vec();

            // push and pop from the end, so the most prioritized items are on the left
            directions.reverse();

            while let Some(direction) = directions.pop() {
                if self.limits_reached() {
                    return Ok(true);
                }

                if path.contains(&direction) {
                    continue;
                }

                let (point, color) = direction;
                let cell_colors: HashSet<B::Color> =
                    self.board().cell(&point).variants().into_iter().collect();

                if !cell_colors.contains(&color) {
                    continue;
                }

                if cell_colors.len() == 1 {
                    assert!(cell_colors.contains(&color));
                    if !board_changed {
                        continue;
                    }

                    let impact = self.probe_solver.run_unsolved::<S>();
                    board_changed = false;

                    if impact.is_err() {
                        // the whole `path` branch of a search tree is a dead end
                        return Ok(false);
                    }

                    if self.board().is_solved_full() {
                        self.add_solution()?;
                        return Ok(true);
                    }
                    continue;
                }

                let mut full_path = path.to_vec();
                full_path.push(direction);

                if self.is_explored(&full_path) {
                    continue;
                }

                let guess_save = self.board().make_snapshot();

                let state_result = self.try_direction(&full_path);
                self.board.borrow_mut().restore(guess_save);
                self.set_explored(&full_path);

                if state_result.is_err() {
                    return state_result;
                }

                let success = state_result.unwrap();

                if !success {
                    let err = self.board.borrow_mut().unset_color(&point, &color).err();
                    board_changed = true;
                    if err.is_some() {
                        return Ok(false);
                    }

                    if !board_changed {
                        continue;
                    }

                    let err = self.probe_solver.run_unsolved::<S>();
                    board_changed = false;
                    if err.is_err() {
                        return Ok(false);
                    }

                    if self.board().is_solved_full() {
                        self.add_solution()?;
                        return Ok(true);
                    }
                }

                if !success || self.board().is_solved_full() {
                    // immediately try the other colors as well
                    // if all of them goes to the dead end,
                    // then the parent path is a dead end

                    let states_to_try: Vec<_> = cell_colors
                        .iter()
                        .filter_map(|&other_color| {
                            if other_color == color {
                                None
                            } else {
                                Some((point, other_color))
                            }
                        })
                        .collect();

                    for direction in states_to_try {
                        if !directions.contains(&direction) {
                            directions.push(direction);
                        }
                    }
                }
            }
            Ok(true)
        }

        /// Trying to search for solutions in the given direction.
        /// At first it set the given state and get a list of the
        /// further jobs for finding the contradictions.
        /// Later that jobs will be used as candidates for a deeper search.
        fn try_direction(&mut self, path: &[(Point, B::Color)]) -> Result<bool, String> {
            let direction = *path.last().expect("Path should be non-empty");

            // add every cell to the jobs queue
            let mut probe_jobs = self.probe_solver.unsolved_cells();
            let new_jobs = self.set_guess(direction);
            match new_jobs {
                // update with more prioritized cells
                Ok(new_jobs) => {
                    probe_jobs.extend(new_jobs);
                }
                Err(_err) => {
                    return Ok(false);
                }
            }

            if self.limits_reached() {
                return Ok(true);
            }

            let impact = self.probe_solver.run::<S>(&mut probe_jobs);

            match impact {
                Ok(impact) => {
                    if self.limits_reached() || self.board().is_solved_full() {
                        return Ok(true);
                    }

                    let directions = self.choose_directions(&impact);
                    if directions.is_empty() {
                        Ok(true)
                    } else {
                        self.search(&directions, path)
                    }
                }
                Err(_err) => Ok(false),
            }
        }

        fn set_guess(&mut self, guess: (Point, B::Color)) -> Result<Vec<(u32, Point)>, String> {
            let (point, color) = guess;

            if !self.board().cell(&point).variants().contains(&color) {
                return Ok(vec![]);
            }

            let mut probes = vec![];
            self.board.borrow_mut().set_color(&point, &color);
            let new_probes = self.probe_solver.propagate_point::<S>(&point)?;
            for (priority, new_point) in new_probes {
                probes.push((priority, new_point));
            }
            if self.board().is_solved_full() {
                self.add_solution()?;
                return Ok(vec![]);
            }

            Ok(probes)
        }

        /// Whether we reached the defined limits:
        /// 1) number of solutions found
        /// 2) the maximum allowed run time
        /// 3) the maximum depth
        fn limits_reached(&self) -> bool {
            if let Some(max_solutions) = self.max_solutions {
                let solutions_number = self.solutions.len();
                if solutions_number >= max_solutions {
                    return true;
                }
            }

            false
        }
    }
}

pub fn run<B, S, P>(
    board: Rc<RefCell<Board<B>>>,
    max_solutions: Option<usize>,
) -> Result<Option<backtracking::Solver<B, P, S>>, String>
where
    B: Block,
    S: LineSolver<BlockType = B>,
    P: ProbeSolver<BlockType = B>,
{
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<S>()?;

    if !board.borrow().is_solved_full() {
        let mut solver =
            backtracking::Solver::<_, P, S>::with_options(Rc::clone(&board), max_solutions);
        solver.run()?;
        return Ok(Some(solver));
    }

    Ok(None)
}

fn read_next_line() -> Vec<usize> {
    let mut line = String::new();

    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");

    line.trim()
        .split_whitespace()
        .map(|c| c.parse::<usize>().expect("should be int"))
        .collect()
}

fn read_description() -> Description<BinaryBlock> {
    let mut row = read_next_line();
    let last = row.pop().unwrap();
    if last != 0 {
        row.push(last);
    }
    Description::new(row.into_iter().map(|x| BinaryBlock(x)).collect())
}

/// Read and parse lines from stdin to create a nonogram board
fn read() -> Vec<(Vec<Description<BinaryBlock>>, Vec<Description<BinaryBlock>>)> {
    let mut _n;
    loop {
        let first_line = read_next_line();
        if !first_line.is_empty() {
            _n = first_line[0];
            break;
        }
    }

    (0.._n)
        .map(|_i| {
            let dimensions = read_next_line();
            let (height, width) = (dimensions[0], dimensions[1]);

            let rows = (0..height).map(|_j| read_description()).collect();
            let columns = (0..width).map(|_j| read_description()).collect();

            (rows, columns)
        })
        .collect()
}

impl<B> fmt::Display for Board<B>
where
    B: Block,
    B::Color: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.iter_rows() {
            for cell in row.iter() {
                write!(f, "{}", cell)?
            }
            write!(f, "\n")?
        }
        Ok(())
    }
}

fn main() {
    use line::DynamicSolver;
    use probing::FullProbe1;
    for (rows, columns) in read() {
        let board = Board::with_descriptions(rows, columns);
        let board = Rc::new(RefCell::new(board));
        //println!("{:#?}\n{:#?}", board.borrow().desc_rows, board.borrow().desc_cols);
        let backtracking =
            run::<_, DynamicSolver<_>, FullProbe1<_>>(Rc::clone(&board), Some(1)).unwrap();

        if board.borrow().is_solved_full() {
            print!("{}", *board.borrow());
            continue;
        }

        if let Some(backtracking) = backtracking {
            let solutions = &backtracking.solutions;
            if !solutions.is_empty() {
                let solution = solutions[0].clone();
                board.borrow_mut().restore(solution);
                print!("{}", *board.borrow());
            }
        }
    }
}
