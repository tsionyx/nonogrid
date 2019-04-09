use super::super::block::{Block, Description};
use super::super::board::{Board, Point};
use super::line::LineSolver;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use hashbrown::HashSet;
use log::Level;

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
        //let before_retain_size = pq.len();
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

        if log_enabled!(Level::Debug) {
            let (is_column, index) = top_job;
            let line_description = if is_column { "column" } else { "row" };
            debug!("Solving {} {}", index, line_description);
        }
        Some(top_job)
    }
}

impl<B> Solver<B>
where
    B: Block,
{
    pub fn new(board: Rc<RefCell<Board<B>>>) -> Self {
        Self { board, point: None }
    }

    pub fn with_point(board: Rc<RefCell<Board<B>>>, point: Point) -> Self {
        Self {
            board,
            point: Some(point),
        }
    }

    pub fn run<S>(&self) -> Result<Vec<Point>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        if let Some(point) = self.point {
            debug!("Solving {:?}", &point);
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
        let start = Instant::now();
        let mut lines_solved = 0_u32;
        let mut solved_cells = vec![];

        while let Some((is_column, index)) = queue.pop() {
            let new_jobs = self.update_line::<S>(index, is_column)?;

            let new_states = new_jobs.iter().map(|(another_index, _color)| {
                let (x, y) = if is_column {
                    (&index, another_index)
                } else {
                    (another_index, &index)
                };
                Point::new(*x, *y)
            });

            solved_cells.extend(new_states);

            new_jobs
                .into_iter()
                .rev()
                .map(|(new_index, _color)| (!is_column, new_index))
                .for_each(|job| queue.push(job));

            lines_solved += 1;
        }

        // all the following actions applied only to verified solving
        //if !self.contradiction_mode
        {
            //let board = board.borrow();
            //board.solution_round_completed()
            //let rate = board.solution_rate();
            //if rate != 1 {
            //    warn!("The nonogram is not solved full: {:.4}", rate)
            //}

            if log_enabled!(Level::Info) {
                let total_time = start.elapsed();
                info!(
                    "Full solution: {}.{:06} sec",
                    total_time.as_secs(),
                    total_time.subsec_micros()
                );
                info!("Lines solved: {}", lines_solved);
            }
        }

        Ok(solved_cells)
    }

    /// Solve a line with the solver S and update the board.
    /// If the line gets partially solved, put the crossed lines into queue.
    ///
    /// Return the list of indexes which was updated during this solution.
    pub fn update_line<S>(
        &self,
        index: usize,
        is_column: bool,
    ) -> Result<Vec<(usize, B::Color)>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let start = Instant::now();

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

        //let pre_solution_rate = Board::<B>::line_solution_rate(&line);
        //if pre_solution_rate == 1 {
        //    // do not check solved lines in trusted mode
        //    if ! contradiction_mode {
        //        return vec![];
        //     }
        //}

        if log_enabled!(Level::Debug) {
            let name = if is_column { "column" } else { "row" };
            debug!(
                "Solving {} {}: {:?}. Partial: {:?}",
                index, name, line_desc, line
            );
        }

        let line = Rc::new(line);
        let solution = self.solve::<S>(index, is_column, line_desc, Rc::clone(&line))?;
        let indexes = self.update_solved(index, is_column, &line, &solution);

        if log_enabled!(Level::Debug) {
            let name = if is_column { "column" } else { "row" };
            let total_time = start.elapsed();
            debug!(
                "{}s solution: {}.{:06} sec",
                name,
                total_time.as_secs(),
                total_time.subsec_micros()
            );
            if !indexes.is_empty() {
                debug!("New info on {} {}: {:?}", name, index, indexes);
            }
        }

        Ok(indexes)
    }

    fn update_solved(
        &self,
        index: usize,
        is_column: bool,
        old: &[B::Color],
        new: &[B::Color],
    ) -> Vec<(usize, B::Color)> {
        // let new_solution_rate = Board::<B>::line_solution_rate(&updated);
        // if new_solution_rate > pre_solution_rate

        if old == new {
            return vec![];
        }
        let mut board = self.board.borrow_mut();

        if is_column {
            board.set_column(index, new);
        } else {
            board.set_row(index, new);
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
                        i, pre, &post
                    );
                    Some((i, *post))
                } else {
                    None
                }
            })
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

        let mut line_solver = S::new(line_desc, line);
        let value = line_solver.solve();

        let rc_value = value.map(Rc::new);
        self.board
            .borrow_mut()
            .set_cached_solution(is_column, key, rc_value.clone());
        rc_value
    }
}
