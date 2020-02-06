use hashbrown::HashSet;
use log::{debug, log_enabled, warn, Level};

use crate::block::{Block, Line};
use crate::board::{Board, Point};
use crate::cache::{cache_info, Cached, GrowableCache};
use crate::solver::line::{self, LineSolver};
use crate::utils::rc::{MutRc, ReadRc, ReadRef};

#[allow(missing_debug_implementations)]
pub struct Solver<B>
where
    B: Block,
{
    board: MutRc<Board<B>>,
    cache_rows: Option<LineSolverCache<B>>,
    cache_cols: Option<LineSolverCache<B>>,
}

type LinePosition = (bool, usize);

trait JobQueue {
    fn push(&mut self, job: LinePosition);
    fn pop(&mut self) -> Option<LinePosition>;
}

struct SmallJobQueue {
    vec: Vec<LinePosition>,
}

impl SmallJobQueue {
    fn with_point(point: Point) -> Self {
        Self {
            vec: vec![(true, point.x), (false, point.y)],
        }
    }
}

impl JobQueue for SmallJobQueue {
    fn push(&mut self, job: LinePosition) {
        self.vec.push(job)
    }

    fn pop(&mut self) -> Option<LinePosition> {
        let top_job = self.vec.pop()?;
        // remove all the previous occurrences of the new job
        self.vec.retain(|&x| x != top_job);

        if log_enabled!(Level::Debug) {
            let (is_column, index) = top_job;
            let line_description = if is_column { "column" } else { "row" };
            debug!("Solving {} {}", index, line_description);
        }
        Some(top_job)
    }
}

struct LongJobQueue {
    vec: Vec<LinePosition>,
    visited: HashSet<LinePosition>,
}

impl LongJobQueue {
    fn with_rows_and_columns(
        rows: impl Iterator<Item = usize>,
        columns: impl Iterator<Item = usize>,
    ) -> Self {
        let jobs = columns
            .map(|column_index| (true, column_index))
            .chain(rows.map(|row_index| (false, row_index)));

        Self {
            vec: jobs.collect(),
            visited: HashSet::new(),
        }
    }
}

impl JobQueue for LongJobQueue {
    fn push(&mut self, job: LinePosition) {
        let _ = self.visited.remove(&job);
        self.vec.push(job)
    }

    fn pop(&mut self) -> Option<LinePosition> {
        let top_job = loop {
            let top_job = self.vec.pop()?;
            if !self.visited.contains(&top_job) {
                break top_job;
            }
        };
        // mark the job as visited
        let _ = self.visited.insert(top_job);

        if log_enabled!(Level::Debug) {
            let (is_column, index) = top_job;
            let line_description = if is_column { "column" } else { "row" };
            debug!("Solving {} {}", index, line_description);
        }
        Some(top_job)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct CacheKey<B>
where
    B: Block,
{
    pub line_index: usize,
    pub source: ReadRc<Line<B::Color>>,
}

type CacheValue<B> = Result<ReadRc<Line<<B as Block>::Color>>, ()>;
type LineSolverCache<B> = GrowableCache<CacheKey<B>, CacheValue<B>>;

fn new_cache<B>(capacity: usize) -> LineSolverCache<B>
where
    B: Block,
{
    GrowableCache::with_capacity(capacity)
}

impl<B> Solver<B>
where
    B: Block,
{
    pub fn new(board: MutRc<Board<B>>) -> Self {
        Self {
            board,
            cache_rows: None,
            cache_cols: None,
        }
    }

    pub fn with_cache(board: MutRc<Board<B>>) -> Self {
        let mut self_ = Self::new(board);

        self_.init_cache();
        self_
    }

    fn board(&self) -> ReadRef<Board<B>> {
        self.board.read()
    }

    fn init_cache(&mut self) {
        let width = self.board().width();
        let height = self.board().height();

        self.cache_rows = Some(new_cache(2_000 * height));
        self.cache_cols = Some(new_cache(2_000 * width));
    }

    fn cached_solution(&mut self, is_column: bool, key: &CacheKey<B>) -> Option<CacheValue<B>> {
        let cache = if is_column {
            self.cache_cols.as_mut()
        } else {
            self.cache_rows.as_mut()
        };

        cache.and_then(|cache| cache.cache_get(key).cloned())
    }

    fn set_cached_solution(&mut self, is_column: bool, key: CacheKey<B>, solved: CacheValue<B>) {
        let cache = if is_column {
            self.cache_cols.as_mut()
        } else {
            self.cache_rows.as_mut()
        };

        if let Some(cache) = cache {
            cache.cache_set(key, solved)
        }
    }

    fn print_cache_info(&self) {
        if let Some(cache) = &self.cache_cols {
            let (s, h, r) = cache_info(cache);
            warn!("Cache columns: Size={}, hits={}, hit rate={}.", s, h, r);
        }
        if let Some(cache) = &self.cache_rows {
            let (s, h, r) = cache_info(cache);
            warn!("Cache rows: Size={}, hits={}, hit rate={}.", s, h, r);
        }
    }

    pub fn run<S>(&mut self, point: Option<Point>) -> Result<Vec<Point>, ()>
    where
        S: LineSolver<BlockType = B>,
    {
        if let Some(point) = point {
            debug!("Solving {:?}", point);
            let queue = SmallJobQueue::with_point(point);
            self.run_jobs::<S, _>(queue)
        } else {
            let queue = {
                let board = self.board();
                let rows = (0..board.height()).rev();
                let cols = (0..board.width()).rev();
                LongJobQueue::with_rows_and_columns(rows, cols)
            };
            self.run_jobs::<S, _>(queue)
        }
    }

    fn run_jobs<S, Q>(&mut self, mut queue: Q) -> Result<Vec<Point>, ()>
    where
        S: LineSolver<BlockType = B>,
        Q: JobQueue,
    {
        let mut lines_solved = 0_u32;
        let mut solved_cells = vec![];

        while let Some(job) = queue.pop() {
            let new_jobs = self.update_line::<S>(job)?;

            let (is_column, index) = job;
            let new_states = new_jobs.iter().map(|&another_index| {
                let (x, y) = if is_column {
                    (index, another_index)
                } else {
                    (another_index, index)
                };
                Point::new(x, y)
            });

            solved_cells.extend(new_states);

            new_jobs
                .into_iter()
                .rev()
                .map(|new_index| (!is_column, new_index))
                .for_each(|job| queue.push(job));

            lines_solved += 1;
        }

        if log_enabled!(Level::Debug) {
            debug!("Lines solved: {}", lines_solved);
        }

        Ok(solved_cells)
    }

    /// Solve a line with the solver S and update the board.
    /// If the line gets partially solved, put the crossed lines into queue.
    ///
    /// Return the list of indexes which was updated during this solution.
    fn update_line<S>(&mut self, position: LinePosition) -> Result<Vec<usize>, ()>
    where
        S: LineSolver<BlockType = B>,
    {
        let (is_column, index) = position;

        let (cache_key, line) = {
            let board = self.board();
            let line = ReadRc::new(if is_column {
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
                source: ReadRc::clone(&line),
            };
            (key, line)
        };

        let cached = self.cached_solution(is_column, &cache_key);

        let solution = cached.unwrap_or_else(|| {
            let line_desc = {
                let board = self.board();
                if is_column {
                    ReadRc::clone(&board.descriptions(false)[index])
                } else {
                    ReadRc::clone(&board.descriptions(true)[index])
                }
            };

            if log_enabled!(Level::Debug) {
                let name = if is_column { "column" } else { "row" };
                debug!(
                    "Solving {} {}: {:?}. Partial: {:?}",
                    index, name, line_desc, line
                );
            }

            let value = line::solve::<S, _>(line_desc, ReadRc::clone(&line)).map(ReadRc::new);

            self.set_cached_solution(is_column, cache_key, value.clone());
            value
        })?;

        let indexes = self.update_solved(position, &line, &solution);

        if log_enabled!(Level::Debug) && !indexes.is_empty() {
            let name = if is_column { "column" } else { "row" };
            debug!("New info on {} {}: {:?}", name, index, indexes);
        }

        Ok(indexes)
    }

    fn update_solved(
        &self,
        (is_column, index): LinePosition,
        old: &[B::Color],
        new: &[B::Color],
    ) -> Vec<usize> {
        if old == new {
            return vec![];
        }

        if is_column {
            Board::set_column_with_callback(MutRc::clone(&self.board), index, new);
        } else {
            Board::set_row_with_callback(MutRc::clone(&self.board), index, new);
        }

        debug!("Original: {:?}", old);
        debug!("Updated: {:?}", new);

        old.iter()
            .zip(new)
            .enumerate()
            .filter_map(|(i, (pre, post))| {
                if pre != post {
                    debug!(
                        "Diff on index={}: original={:?}, updated={:?}",
                        i, pre, post
                    );
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<B> Drop for Solver<B>
where
    B: Block,
{
    fn drop(&mut self) {
        self.print_cache_info()
    }
}
