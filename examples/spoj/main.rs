#![allow(clippy::missing_const_for_fn)]

use std::cell::RefCell;
use std::fmt;
use std::io;
use std::ops::Sub;
use std::rc::Rc;
use std::slice::Chunks;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum BW {
    Undefined,
    White,
    Black,
    BlackOrWhite,
}

impl Default for BW {
    fn default() -> Self {
        BW::Undefined
    }
}

impl BW {
    fn blank() -> Self {
        BW::White
    }

    fn is_solved(self) -> bool {
        self == BW::Black || self == BW::White
    }

    fn solution_rate(self) -> f64 {
        if self.is_solved() {
            1.0
        } else {
            0.0
        }
    }

    fn variants(self) -> Vec<Self> {
        if self.is_solved() {
            vec![self]
        } else {
            vec![BW::White, BW::Black]
        }
    }

    fn both_colors() -> Option<Self> {
        Some(BW::BlackOrWhite)
    }

    fn can_be_blank(self) -> bool {
        self != BW::Black
    }

    fn can_be(self) -> bool {
        self != Self::blank()
    }

    fn add_color(self, color: Self) -> Self {
        match self {
            BW::Undefined => color,
            value => {
                if value == color {
                    value
                } else {
                    BW::BlackOrWhite
                }
            }
        }
    }
}

impl fmt::Display for BW {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BW::*;

        let symbol = match *self {
            White => '.',
            Black => '#',
            Undefined | BlackOrWhite => '#',
        };
        write!(f, "{}", symbol)
    }
}

impl Sub for BW {
    type Output = Result<Self, String>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_solved() {
            return Err(format!("Cannot unset already set cell {:?}", &self));
        }

        Ok(match rhs {
            BW::Black => BW::White,
            BW::White => BW::Black,
            _ => self,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
struct BB(usize);

impl BB {
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

    fn size(self) -> usize {
        self.0
    }

    fn color(self) -> BW {
        BW::Black
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Clues {
    vec: Vec<BB>,
}

impl Clues {
    fn new(mut vec: Vec<BB>) -> Self {
        let zero = BB::default();
        vec.retain(|x| *x != zero);
        Self { vec }
    }
}

mod utils {
    use std::collections::{HashMap, HashSet};
    use std::hash::Hash;

    pub fn replace<T>(vec: &mut [T], what: &T, with_what: &T)
    where
        T: PartialEq + Clone,
    {
        if what == with_what {
            return;
        }

        for x in vec {
            if *x == *what {
                *x = with_what.clone();
            }
        }
    }

    pub fn dedup<T>(vec: Vec<T>) -> Vec<T>
    where
        T: Eq + Hash + Clone,
    {
        let set: HashSet<_> = vec.into_iter().collect();
        set.into_iter().collect()
    }

    /// The copy of `hashbrown::fx`
    mod fx {
        use std::default::Default;
        use std::hash::{BuildHasherDefault, Hasher};
        use std::intrinsics::copy_nonoverlapping;
        use std::mem::size_of;
        use std::ops::BitXor;

        pub type FxHashBuilder = BuildHasherDefault<FxHasher>;

        pub struct FxHasher {
            hash: usize,
        }

        #[cfg(target_pointer_width = "32")]
        const K: usize = 0x9e37_79b9;
        #[cfg(target_pointer_width = "64")]
        const K: usize = 0x517c_c1b7_2722_0a95;

        impl Default for FxHasher {
            #[inline]
            fn default() -> Self {
                Self { hash: 0 }
            }
        }

        impl FxHasher {
            #[inline]
            fn add_to_hash(&mut self, i: usize) {
                self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
            }
        }

        impl Hasher for FxHasher {
            #[inline]
            fn finish(&self) -> u64 {
                self.hash as u64
            }

            #[inline]
            fn write(&mut self, mut bytes: &[u8]) {
                macro_rules! read_bytes {
                    ($ty:ty, $src:expr) => {{
                        assert!(size_of::<$ty>() <= $src.len());
                        let mut data: $ty = 0;
                        unsafe {
                            copy_nonoverlapping(
                                $src.as_ptr(),
                                &mut data as *mut $ty as *mut u8,
                                size_of::<$ty>(),
                            );
                        }
                        data
                    }};
                }

                let mut hash = Self { hash: self.hash };
                assert!(size_of::<usize>() <= 8);
                while bytes.len() >= size_of::<usize>() {
                    hash.add_to_hash(read_bytes!(usize, bytes) as usize);
                    bytes = &bytes[size_of::<usize>()..];
                }
                if (size_of::<usize>() > 4) && (bytes.len() >= 4) {
                    hash.add_to_hash(read_bytes!(u32, bytes) as usize);
                    bytes = &bytes[4..];
                }
                if (size_of::<usize>() > 2) && bytes.len() >= 2 {
                    hash.add_to_hash(read_bytes!(u16, bytes) as usize);
                    bytes = &bytes[2..];
                }
                if (size_of::<usize>() > 1) && !bytes.is_empty() {
                    hash.add_to_hash(bytes[0] as usize);
                }
                self.hash = hash.hash;
            }

            #[inline]
            fn write_u8(&mut self, i: u8) {
                self.add_to_hash(i as usize);
            }

            #[inline]
            fn write_u16(&mut self, i: u16) {
                self.add_to_hash(i as usize);
            }

            #[inline]
            fn write_u32(&mut self, i: u32) {
                self.add_to_hash(i as usize);
            }

            #[cfg(target_pointer_width = "32")]
            #[inline]
            fn write_u64(&mut self, i: u64) {
                self.add_to_hash(i as usize);
                self.add_to_hash((i >> 32) as usize);
            }

            #[cfg(target_pointer_width = "64")]
            #[inline]
            fn write_u64(&mut self, i: u64) {
                self.add_to_hash(i as usize);
            }

            #[inline]
            fn write_usize(&mut self, i: usize) {
                self.add_to_hash(i);
            }
        }
    }

    pub struct GrowableCache<K, V>
    where
        K: Eq + Hash,
    {
        store: HashMap<K, V, fx::FxHashBuilder>,
    }

    impl<K: Hash + Eq, V> GrowableCache<K, V> {
        pub fn with_capacity(size: usize) -> Self {
            Self {
                store: HashMap::with_capacity_and_hasher(size, <_>::default()),
            }
        }

        pub fn cache_get(&mut self, key: &K) -> Option<&V> {
            self.store.get(key)
        }

        pub fn cache_set(&mut self, key: K, val: V) {
            self.store.insert(key, val);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn x(&self) -> usize {
        self.x
    }

    fn y(&self) -> usize {
        self.y
    }
}

pub struct Board {
    cells: Vec<BW>,
    desc_rows: Vec<Rc<Clues>>,
    desc_cols: Vec<Rc<Clues>>,
    rows_cache_indexes: Vec<usize>,
    cols_cache_indexes: Vec<usize>,
}

impl Board {
    fn with_descriptions(rows: Vec<Clues>, columns: Vec<Clues>) -> Self {
        let height = rows.len();
        let width = columns.len();

        let init = BW::default();
        let cells = vec![init; width * height];

        let uniq_rows = utils::dedup(rows.iter().map(|desc| desc.vec.clone()).collect());
        let uniq_cols = utils::dedup(columns.iter().map(|desc| desc.vec.clone()).collect());

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
        Self {
            cells,
            desc_rows,
            desc_cols,
            rows_cache_indexes,
            cols_cache_indexes,
        }
    }

    fn iter_rows(&self) -> Chunks<BW> {
        self.cells.chunks(self.width())
    }

    fn descriptions(&self, rows: bool) -> &[Rc<Clues>] {
        if rows {
            &self.desc_rows
        } else {
            &self.desc_cols
        }
    }

    fn height(&self) -> usize {
        self.desc_rows.len()
    }

    fn width(&self) -> usize {
        self.desc_cols.len()
    }

    fn is_solved_full(&self) -> bool {
        self.cells.iter().all(|cell| cell.is_solved())
    }

    fn get_row_slice(&self, index: usize) -> &[BW] {
        self.iter_rows().nth(index).expect("Invalid row index")
    }

    fn get_row(&self, index: usize) -> Vec<BW> {
        self.get_row_slice(index).to_vec()
    }

    fn get_column(&self, index: usize) -> Vec<BW> {
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

    fn set_row(&mut self, index: usize, new: &[BW]) {
        let row_start = self.linear_index(index, 0);
        for (linear_index, &new_cell) in (row_start..).zip(new) {
            self.cells[linear_index] = new_cell;
        }
    }

    fn set_column(&mut self, index: usize, new: &[BW]) {
        let width = self.width();

        for (i, &new_cell) in new.iter().enumerate() {
            let linear_index = (i * width) + index;
            self.cells[linear_index] = new_cell;
        }
    }

    fn row_solution_rate(&self, index: usize) -> f64 {
        let solved: f64 = self
            .get_row_slice(index)
            .iter()
            .map(|cell| cell.solution_rate())
            .sum();
        solved / self.width() as f64
    }

    fn column_solution_rate(&self, index: usize) -> f64 {
        let column = self.cells.iter().skip(index).step_by(self.width());

        let solved: f64 = column.map(|cell| cell.solution_rate()).sum();
        solved / self.height() as f64
    }

    fn unsolved_cells(&self) -> impl Iterator<Item = Point> + '_ {
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

    fn cell(&self, point: &Point) -> BW {
        let Point { x, y } = *point;
        self.cells[self.linear_index(y, x)]
    }

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

    fn unsolved_neighbours(&self, point: &Point) -> impl Iterator<Item = Point> + '_ {
        self.neighbours(point)
            .into_iter()
            .filter(move |n| !self.cell(n).is_solved())
    }

    fn row_cache_index(&self, row_index: usize) -> usize {
        self.rows_cache_indexes[row_index]
    }

    fn column_cache_index(&self, column_index: usize) -> usize {
        self.cols_cache_indexes[column_index]
    }
}

impl Board {
    fn make_snapshot(&self) -> Vec<BW> {
        self.cells.clone()
    }

    fn restore(&mut self, cells: Vec<BW>) {
        self.cells = cells;
    }
}

impl Board {
    fn set_color(&mut self, point: &Point, color: BW) {
        let Point { x, y } = *point;
        let index = self.linear_index(y, x);
        self.cells[index] = color;
    }

    fn unset_color(&mut self, point: &Point, color: BW) -> Result<(), String> {
        let old_value = self.cell(point);
        let Point { x, y } = *point;
        let index = self.linear_index(y, x);
        self.cells[index] = (old_value - color)?;

        Ok(())
    }
}

mod line {
    use std::iter::once;
    use std::rc::Rc;

    use super::utils::replace;
    use super::{Clues, BB, BW};

    pub fn solve(desc: Rc<Clues>, line: Rc<Vec<BW>>) -> Result<Vec<BW>, ()> {
        let mut solver = DynamicSolver::new(desc, line);
        if solver.solve() {
            Ok(solver.get_solution())
        } else {
            Err(())
        }
    }

    struct DynamicSolver {
        desc: Rc<Clues>,
        line: Rc<Vec<BW>>,
        block_sums: Vec<usize>,
        job_size: usize,
        solution_matrix: Vec<Option<bool>>,
        solved_line: Vec<BW>,
    }

    impl DynamicSolver {
        fn new(desc: Rc<Clues>, line: Rc<Vec<BW>>) -> Self {
            let block_sums = Self::calc_block_sum(&*desc);

            let job_size = desc.vec.len() + 1;
            let solution_matrix = vec![None; job_size * line.len()];

            let solved_line = line.iter().cloned().collect();

            Self {
                desc,
                line,
                block_sums,
                job_size,
                solution_matrix,
                solved_line,
            }
        }

        fn solve(&mut self) -> bool {
            if !self.try_solve() {
                return false;
            }
            let solved = &mut self.solved_line;

            let both = BW::both_colors();
            if let Some(both) = both {
                let init = BW::default();
                replace(solved, &both, &init);
            }
            true
        }

        fn get_solution(self) -> Vec<BW> {
            self.solved_line
        }

        fn calc_block_sum(desc: &Clues) -> Vec<usize> {
            once(0)
                .chain(BB::partial_sums(&desc.vec).into_iter().map(|size| size - 1))
                .collect()
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

        fn color_at(&self, position: usize) -> BW {
            self.line[position]
        }

        fn block_at(&self, block_position: usize) -> BB {
            self.desc.vec[block_position]
        }

        fn update_solved(&mut self, position: usize, color: BW) {
            let current = self.solved_line[position];
            self.solved_line[position] = current.add_color(color);
        }

        fn fill_matrix(&mut self, position: usize, block: usize) -> bool {
            if position < self.block_sums[block] {
                return false;
            }

            self.fill_matrix_blank(position, block) | self.fill_matrix_color(position, block)
        }

        fn fill_matrix_blank(&mut self, position: usize, block: usize) -> bool {
            if self.color_at(position).can_be_blank() {
                let has_blank = self.get_sol(position as isize - 1, block);
                if has_blank {
                    let blank = BW::blank();
                    self.update_solved(position, blank);
                    return true;
                }
            }

            false
        }

        fn fill_matrix_color(&mut self, position: usize, block: usize) -> bool {
            if block == 0 {
                return false;
            }
            let current_block = self.block_at(block - 1);
            let mut block_size = current_block.size();
            let should_have_trailing_space = self.trail_with_space(block);
            if should_have_trailing_space {
                block_size += 1;
            }

            let block_start = position as isize - block_size as isize + 1;

            if self.can_place_color(block_start, position, should_have_trailing_space) {
                let has_color = self.get_sol(block_start - 1, block - 1);
                if has_color {
                    self.set_color_block(
                        block_start,
                        position,
                        current_block.color(),
                        should_have_trailing_space,
                    );
                    return true;
                }
            }

            false
        }

        fn trail_with_space(&self, block: usize) -> bool {
            block < self.desc.vec.len()
        }

        fn can_place_color(&self, start: isize, mut end: usize, trailing_space: bool) -> bool {
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

            self.line[start as usize..end]
                .iter()
                .all(|cell| cell.can_be())
        }

        fn set_color_block(
            &mut self,
            start: isize,
            mut end: usize,
            color: BW,
            trailing_space: bool,
        ) {
            if trailing_space {
                let blank = BW::blank();
                self.update_solved(end, blank);
            } else {
                end += 1
            }

            for i in start as usize..end {
                self.update_solved(i, color);
            }
        }
    }
}

mod propagation {
    use std::cell::{Ref, RefCell};
    use std::collections::HashSet;
    use std::rc::Rc;

    use super::line;
    use super::utils::GrowableCache;
    use super::{Board, Point, BW};

    #[derive(Debug, PartialEq, Eq, Hash)]
    struct CacheKey {
        pub line_index: usize,
        pub source: Rc<Vec<BW>>,
    }

    type CacheValue = Result<Rc<Vec<BW>>, ()>;
    type LineSolverCache = GrowableCache<CacheKey, CacheValue>;

    fn new_cache(capacity: usize) -> LineSolverCache {
        GrowableCache::with_capacity(capacity)
    }

    pub struct Solver {
        board: Rc<RefCell<Board>>,
        cache_rows: Option<LineSolverCache>,
        cache_cols: Option<LineSolverCache>,
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
            Self {
                vec: vec![(true, point.x()), (false, point.y())],
            }
        }
    }

    impl JobQueue for SmallJobQueue {
        fn push(&mut self, job: Job) {
            self.vec.push(job)
        }

        fn pop(&mut self) -> Option<Job> {
            let top_job = self.vec.pop()?;

            self.vec.retain(|&x| x != top_job);
            Some(top_job)
        }
    }

    struct LongJobQueue {
        vec: Vec<Job>,
        visited: HashSet<Job>,
    }

    impl LongJobQueue {
        fn with_rows_and_columns(
            rows: impl Iterator<Item = usize>,
            columns: impl Iterator<Item = usize>,
        ) -> Self {
            let jobs = columns
                .map(|column_index| (true, column_index))
                .chain(rows.map(|row_index| (false, row_index)))
                .collect();

            Self {
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
            let top_job = loop {
                let top_job = self.vec.pop()?;
                if !self.visited.contains(&top_job) {
                    break top_job;
                }
            };
            // mark the job as visited
            self.visited.insert(top_job);
            Some(top_job)
        }
    }

    impl Solver {
        pub fn new(board: Rc<RefCell<Board>>) -> Self {
            Self {
                board,
                cache_rows: None,
                cache_cols: None,
            }
        }

        pub fn with_cache(board: Rc<RefCell<Board>>) -> Self {
            let mut self_ = Self::new(board);

            self_.init_cache();
            self_
        }

        fn board(&self) -> Ref<Board> {
            self.board.borrow()
        }

        fn init_cache(&mut self) {
            let width = self.board().width();
            let height = self.board().height();

            self.cache_rows = Some(new_cache(2_000 * height));
            self.cache_cols = Some(new_cache(2_000 * width));
        }

        fn cached_solution(&mut self, is_column: bool, key: &CacheKey) -> Option<CacheValue> {
            let cache = if is_column {
                self.cache_cols.as_mut()
            } else {
                self.cache_rows.as_mut()
            };

            cache.and_then(|cache| cache.cache_get(key).cloned())
        }

        fn set_cached_solution(&mut self, is_column: bool, key: CacheKey, solved: CacheValue) {
            let cache = if is_column {
                self.cache_cols.as_mut()
            } else {
                self.cache_rows.as_mut()
            };

            if let Some(cache) = cache {
                cache.cache_set(key, solved)
            }
        }

        pub fn run(&mut self, point: Option<Point>) -> Result<Vec<Point>, ()> {
            if let Some(point) = point {
                let queue = SmallJobQueue::with_point(point);
                self.run_jobs(queue)
            } else {
                let queue = {
                    let board = self.board.borrow();
                    let rows = (0..board.height()).rev();
                    let cols = (0..board.width()).rev();
                    LongJobQueue::with_rows_and_columns(rows, cols)
                };
                self.run_jobs(queue)
            }
        }

        fn run_jobs<Q>(&mut self, mut queue: Q) -> Result<Vec<Point>, ()>
        where
            Q: JobQueue,
        {
            let mut solved_cells = vec![];

            while let Some((is_column, index)) = queue.pop() {
                let new_jobs = self.update_line(index, is_column)?;

                let new_states = new_jobs.iter().map(|&another_index| {
                    let (x, y) = if is_column {
                        (index, another_index)
                    } else {
                        (another_index, index)
                    };
                    Point::new(x, y)
                });

                solved_cells.extend(new_states);

                for new_index in new_jobs.iter().rev() {
                    queue.push((!is_column, *new_index))
                }
            }

            Ok(solved_cells)
        }

        fn update_line(&mut self, index: usize, is_column: bool) -> Result<Vec<usize>, ()> {
            let (cache_key, line) = {
                let board = self.board();
                let line = Rc::new(if is_column {
                    board.get_column(index)
                } else {
                    board.get_row(index)
                });

                let cache_index = if is_column {
                    board.column_cache_index(index)
                } else {
                    board.row_cache_index(index)
                };

                let key = CacheKey {
                    line_index: cache_index,
                    source: Rc::clone(&line),
                };
                (key, line)
            };

            let cached = self.cached_solution(is_column, &cache_key);

            let solution = cached.unwrap_or_else(|| {
                let line_desc = {
                    let board = self.board();
                    if is_column {
                        Rc::clone(&board.descriptions(false)[index])
                    } else {
                        Rc::clone(&board.descriptions(true)[index])
                    }
                };

                let value = line::solve(line_desc, Rc::clone(&line)).map(Rc::new);

                self.set_cached_solution(is_column, cache_key, value.clone());
                value
            })?;

            let indexes = self.update_solved(index, is_column, &line, &solution);

            Ok(indexes)
        }

        fn update_solved(
            &self,
            index: usize,
            is_column: bool,
            old: &[BW],
            new: &[BW],
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
                .filter_map(|(i, (pre, post))| if pre == post { None } else { Some(i) })
                .collect()
        }
    }
}

mod probing {
    use std::cell::{Ref, RefCell};
    use std::collections::BinaryHeap;
    use std::rc::Rc;

    use super::{propagation, Board, PartialEntry, Point, BW};

    #[derive(Debug)]
    pub struct ProbeImpact {
        point: Point,
        color: BW,
        cells_solved: usize,
        probe_priority: Priority,
    }

    impl ProbeImpact {
        pub fn as_tuple(&self) -> (Point, BW, usize, Priority) {
            (
                self.point,
                self.color,
                self.cells_solved,
                self.probe_priority,
            )
        }
    }

    pub type Impact = Vec<ProbeImpact>;
    type OrderedPoints = BinaryHeap<(Priority, Point)>;

    pub struct FullProbe1 {
        board: Rc<RefCell<Board>>,
        propagation_solver: propagation::Solver,
    }

    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
    pub struct Priority(pub u32);

    const MULTIPLIER: u32 = 10000;
    pub const PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED: Priority = Priority(10 * MULTIPLIER);
    pub const PRIORITY_NEIGHBOURS_OF_CONTRADICTION: Priority = Priority(20 * MULTIPLIER);

    impl From<f64> for Priority {
        fn from(val: f64) -> Self {
            Self((val * f64::from(MULTIPLIER)) as u32)
        }
    }

    impl FullProbe1 {
        pub fn with_board(board: Rc<RefCell<Board>>) -> Self {
            let propagation_solver = propagation::Solver::with_cache(Rc::clone(&board));
            Self {
                board,
                propagation_solver,
            }
        }

        pub fn unsolved_cells(&self) -> OrderedPoints {
            let board = self.board();
            let unsolved = board.unsolved_cells();

            let mut row_rate_cache = Vec::with_none(board.height());
            let mut column_rate_cache = Vec::with_none(board.width());

            let mut queue = OrderedPoints::new();
            queue.extend(unsolved.map(|point| {
                let no_solved = 4 - board.unsolved_neighbours(&point).count();
                let row_rate = row_rate_cache
                    .unwrap_or_insert_with(point.y(), || board.row_solution_rate(point.y()));
                let column_rate = column_rate_cache
                    .unwrap_or_insert_with(point.x(), || board.column_solution_rate(point.x()));
                let priority = no_solved as f64 + row_rate + column_rate;
                (priority.into(), point)
            }));

            queue
        }

        pub fn propagate_point(&mut self, point: &Point) -> Result<Vec<(Priority, Point)>, ()> {
            let fixed_points = self.run_propagation(point)?;

            let board = self.board();

            let res = fixed_points
                .iter()
                .flat_map(|new_point| {
                    board
                        .unsolved_neighbours(new_point)
                        .map(|neighbour| (PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED, neighbour))
                })
                .chain(
                    board
                        .unsolved_neighbours(point)
                        .map(|neighbour| (PRIORITY_NEIGHBOURS_OF_CONTRADICTION, neighbour)),
                )
                .collect();
            Ok(res)
        }

        pub fn run_unsolved(&mut self) -> Result<Impact, String> {
            let unsolved = &mut self.unsolved_cells();
            self.run(unsolved)
        }

        pub fn run(&mut self, probes: &mut OrderedPoints) -> Result<Impact, String> {
            let impact = loop {
                let mut impact = Impact::new();

                if self.is_solved() {
                    break impact;
                }

                let mut false_probes = None;

                while let Some((priority, point)) = probes.pop() {
                    let probe_results = self.probe(point);

                    let (contradictions, non_contradictions): (Vec<_>, Vec<_>) = probe_results
                        .into_iter()
                        .partition(|&(_color, size)| size.is_none());

                    if !contradictions.is_empty() {
                        let bad_colors = contradictions
                            .into_iter()
                            .map(|(color, _should_be_none)| color);

                        false_probes = Some((point, bad_colors));
                        break;
                    }

                    impact.extend(non_contradictions.into_iter().map(|(color, updated)| {
                        ProbeImpact {
                            point,
                            color,
                            probe_priority: priority,
                            cells_solved: updated.expect("Number of cells"),
                        }
                    }));
                }

                if let Some((contradiction, colors)) = false_probes {
                    for color in colors {
                        self.board.borrow_mut().unset_color(&contradiction, color)?;
                    }
                    let new_probes = self.propagate_point(&contradiction).map_err(|_| {
                        format!(
                            "Error while propagating contradicted values in {:?}",
                            &contradiction
                        )
                    })?;

                    probes.extend(new_probes);
                } else {
                    break impact;
                }
            };

            Ok(impact)
        }
    }

    impl FullProbe1 {
        fn board(&self) -> Ref<Board> {
            self.board.borrow()
        }

        fn run_propagation(&mut self, point: &Point) -> Result<Vec<Point>, ()> {
            self.propagation_solver.run(Some(*point))
        }

        fn is_solved(&self) -> bool {
            self.board().is_solved_full()
        }

        fn probe(&mut self, point: Point) -> Vec<(BW, Option<usize>)> {
            let vars = self.board().cell(&point).variants();

            vars.into_iter()
                .map(|assumption| {
                    let save = self.board().make_snapshot();
                    self.board.borrow_mut().set_color(&point, assumption);

                    let solved = self.run_propagation(&point);
                    self.board.borrow_mut().restore(save);

                    (assumption, solved.ok().map(|new_cells| new_cells.len()))
                })
                .collect()
        }
    }
}

mod backtracking {
    use std::cell::{Ref, RefCell};
    use std::cmp::Reverse;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::probing::{FullProbe1, Impact, Priority};
    use super::{Board, Point, BW};

    type Solution = Vec<BW>;

    pub struct Solver {
        board: Rc<RefCell<Board>>,
        probe_solver: FullProbe1,
        max_solutions: Option<usize>,
        pub solutions: Vec<Solution>,
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

    impl Solver {
        pub fn with_options(board: Rc<RefCell<Board>>, max_solutions: Option<usize>) -> Self {
            let probe_solver = FullProbe1::with_board(Rc::clone(&board));
            Self {
                board,
                probe_solver,
                max_solutions,
                solutions: vec![],
            }
        }

        pub fn run(&mut self) -> Result<(), String> {
            if self.is_solved() {
                return Ok(());
            }

            let impact = self.probe_solver.run_unsolved()?;
            if self.is_solved() {
                return Ok(());
            }

            let directions = self.choose_directions(impact);
            let success = self.search(directions, &[])?;
            if !success {
                return Err("Backtracking failed".to_string());
            }
            Ok(())
        }

        fn board(&self) -> Ref<Board> {
            self.board.borrow()
        }

        fn is_solved(&self) -> bool {
            self.board().is_solved_full()
        }

        fn already_found(&self) -> bool {
            for (_i, solution) in self.solutions.iter().enumerate() {
                if &self.board().cells == solution {
                    return true;
                }
            }

            false
        }

        fn add_solution(&mut self) -> Result<(), String> {
            if !self.already_found() {
                let cells = self.board().make_snapshot();
                self.solutions.push(cells);
            }

            Ok(())
        }

        fn choose_directions(&self, impact: Impact) -> Vec<(Point, BW)> {
            let mut point_wise = HashMap::new();

            for (point, color, new_points, priority) in impact.into_iter().map(|x| x.as_tuple()) {
                if self.board().cell(&point).is_solved() {
                    continue;
                }
                let point_colors = point_wise.entry(point).or_insert_with(HashMap::new);
                point_colors.insert(color, (new_points, priority));
            }

            let mut points_rate: Vec<_> = point_wise
                .iter()
                .map(|(point, color_to_impact)| {
                    let values: Vec<_> = color_to_impact.values().collect();
                    let priority: Priority = Self::rate_by_impact(&values).into();
                    (point, priority)
                })
                .collect();
            points_rate.sort_by_key(|&(point, rate)| (Reverse(rate), point));

            points_rate
                .iter()
                .flat_map(|&(point, _rate)| {
                    let mut point_colors: Vec<_> =
                        point_wise[point].iter().map(|(&k, &v)| (k, v)).collect();
                    point_colors
                        .sort_by_key(|&(_color, (new_points, _priority))| Reverse(new_points));
                    point_colors
                        .into_iter()
                        .map(move |(color, _impact)| (*point, color))
                })
                .collect()
        }

        fn choose_strategy() -> ChoosePixel {
            ChoosePixel::Min
        }

        fn rate_by_impact(impact: &[&(usize, Priority)]) -> f64 {
            let sizes_only: Vec<_> = impact
                .iter()
                .map(|&&(new_points, _priority)| new_points)
                .collect();

            let zero = 0;
            let min = sizes_only.iter().min().unwrap_or(&zero);
            let max = sizes_only.iter().max().unwrap_or(&zero);
            let sum = sizes_only.iter().sum::<usize>();

            let log = |f: f64| (1.0 + f).ln() + 1.0;

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

        fn search(
            &mut self,
            directions: Vec<(Point, BW)>,
            path: &[(Point, BW)],
        ) -> Result<bool, String> {
            if self.limits_reached() {
                return Ok(true);
            }

            let save = self.board().make_snapshot();
            let result = self.search_mutable(directions, path);

            if !path.is_empty() {
                self.board.borrow_mut().restore(save);
            }

            result
        }

        fn search_mutable(
            &mut self,
            mut directions: Vec<(Point, BW)>,
            path: &[(Point, BW)],
        ) -> Result<bool, String> {
            let mut board_changed = true;

            directions.reverse();

            while let Some(direction) = directions.pop() {
                if self.limits_reached() {
                    return Ok(true);
                }

                if path.contains(&direction) {
                    continue;
                }

                let (point, color) = direction;
                let cell_colors: Vec<BW> =
                    self.board().cell(&point).variants().into_iter().collect();

                if !cell_colors.contains(&color) {
                    continue;
                }

                if cell_colors.len() == 1 {
                    assert!(cell_colors.contains(&color));
                    if !board_changed {
                        continue;
                    }

                    let impact = self.probe_solver.run_unsolved();
                    board_changed = false;

                    if impact.is_err() {
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

                let guess_save = self.board().make_snapshot();
                let state_result = self.try_direction(&full_path);
                self.board.borrow_mut().restore(guess_save);

                let success = state_result?;

                if !success {
                    let unset_result = self.board.borrow_mut().unset_color(&point, color);
                    if unset_result.is_err() {
                        return Ok(false);
                    }

                    let run_with_new_info = self.probe_solver.run_unsolved();
                    board_changed = false;
                    if run_with_new_info.is_err() {
                        return Ok(false);
                    }

                    if self.board().is_solved_full() {
                        self.add_solution()?;
                        return Ok(true);
                    }
                }

                if !success || self.board().is_solved_full() {
                    let states_to_try = cell_colors.into_iter().filter_map(|other_color| {
                        if other_color == color {
                            None
                        } else {
                            Some((point, other_color))
                        }
                    });

                    for direction in states_to_try {
                        if !directions.contains(&direction) {
                            directions.push(direction);
                        }
                    }
                }
            }
            Ok(true)
        }

        fn try_direction(&mut self, path: &[(Point, BW)]) -> Result<bool, String> {
            let direction = *path.last().expect("Path should be non-empty");

            let mut probe_jobs = self.probe_solver.unsolved_cells();
            let new_jobs = self.set_guess(direction);
            match new_jobs {
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

            let impact = self.probe_solver.run(&mut probe_jobs);

            match impact {
                Ok(impact) => {
                    if self.limits_reached() || self.board().is_solved_full() {
                        return Ok(true);
                    }

                    let directions = self.choose_directions(impact);
                    if directions.is_empty() {
                        Ok(true)
                    } else {
                        self.search(directions, path)
                    }
                }
                Err(_err) => Ok(false),
            }
        }

        fn set_guess(&mut self, guess: (Point, BW)) -> Result<Vec<(Priority, Point)>, String> {
            let (point, color) = guess;

            if !self.board().cell(&point).variants().contains(&color) {
                return Ok(vec![]);
            }

            self.board.borrow_mut().set_color(&point, color);
            let new_probes = self.probe_solver.propagate_point(&point).map_err(|_| {
                format!("Error while propagating value {:?} in {:?}", &color, &point)
            })?;

            if self.board().is_solved_full() {
                self.add_solution()?;
                return Ok(vec![]);
            }

            Ok(new_probes)
        }

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

fn run(
    board: Rc<RefCell<Board>>,
    max_solutions: Option<usize>,
) -> Result<Option<backtracking::Solver>, String> {
    let mut solver = propagation::Solver::new(Rc::clone(&board));
    solver
        .run(None)
        .map_err(|_| "Bad puzzle for sure: simple propagation failed".to_string())?;

    if !board.borrow().is_solved_full() {
        let mut solver = backtracking::Solver::with_options(board, max_solutions);
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

fn read_description() -> Clues {
    let mut row = read_next_line();
    let last = row.pop().unwrap();
    if last != 0 {
        row.push(last);
    }
    Clues::new(row.into_iter().map(BB).collect())
}

fn read() -> Vec<(Vec<Clues>, Vec<Clues>)> {
    let n = loop {
        let first_line = read_next_line();
        if !first_line.is_empty() {
            break first_line[0];
        }
    };

    (0..n)
        .map(|_i| {
            let dimensions = read_next_line();
            let (height, width) = (dimensions[0], dimensions[1]);

            let rows = (0..height).map(|_j| read_description()).collect();
            let columns = (0..width).map(|_j| read_description()).collect();

            (rows, columns)
        })
        .collect()
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.iter_rows() {
            for cell in row.iter() {
                write!(f, "{}", cell)?
            }
            writeln!(f)?
        }
        Ok(())
    }
}

fn main() {
    for (rows, columns) in read() {
        let board = Board::with_descriptions(rows, columns);
        let board = Rc::new(RefCell::new(board));
        let backtracking = run(Rc::clone(&board), Some(1)).unwrap();

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

pub trait PartialEntry {
    type Output: Copy;

    fn unwrap_or_insert_with<F>(&mut self, index: usize, default: F) -> Self::Output
    where
        F: FnOnce() -> Self::Output;

    fn with_none(capacity: usize) -> Self;
}

impl<T> PartialEntry for Vec<Option<T>>
where
    T: Copy,
{
    type Output = T;

    fn unwrap_or_insert_with<F: FnOnce() -> T>(&mut self, index: usize, default: F) -> T {
        if let Some(elem) = self.get(index) {
            if let Some(y) = *elem {
                return y;
            }
        }

        let new = default();
        self[index] = Some(new);
        new
    }

    fn with_none(capacity: usize) -> Self {
        vec![None; capacity]
    }
}
