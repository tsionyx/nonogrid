use super::super::board::{Block, Board, Color, Description};
use super::line::LineSolver;

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Instant;

use log::Level;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

pub struct Solver<B>
where
    B: Block,
{
    board: Rc<RefCell<Board<B>>>,
    rows: Option<Vec<usize>>,
    columns: Option<Vec<usize>>,
    contradiction_mode: bool,
}

impl<B> Solver<B>
where
    B: Block + Debug,
    <B as Block>::Color: Clone + Debug + PartialEq,
{
    pub fn new(board: Rc<RefCell<Board<B>>>) -> Self {
        Self::with_options(board, None, None, false)
    }

    pub fn with_options(
        board: Rc<RefCell<Board<B>>>,
        rows: Option<Vec<usize>>,
        columns: Option<Vec<usize>>,
        contradiction_mode: bool,
    ) -> Self {
        Self {
            board,
            rows,
            columns,
            contradiction_mode,
        }
    }

    pub fn solve<S>(&self) -> Result<(), String>
    where
        S: LineSolver<BlockType = B>,
    {
        let (rows, columns) = {
            // for safe borrowing
            let board = self.board.borrow();
            (
                self.rows
                    .clone()
                    .unwrap_or_else(|| (0..board.height()).collect()),
                self.columns
                    .clone()
                    .unwrap_or_else(|| (0..board.width()).collect()),
            )
        };

        // `is_solved_full` is expensive, so minimize calls to it.
        // Do not call if only a handful of lines has to be solved
        if rows.len() > 2 || columns.len() > 2 {
            // do not shortcut in contradiction_mode
            if !self.contradiction_mode && self.board.borrow().is_solved_full() {
                //return 0, ()
            }
        }
        // has_blots = board.has_blots

        let start = Instant::now();
        let mut lines_solved = 0u32;

        // every job is a tuple (is_column, index)
        //
        // Why `is_column`, not `is_row`?
        // To assign more priority to the rows:
        // when adding row, `is_column = False = 0`
        // when adding column, `is_column = True = 1`
        // heap always pops the lowest item, so the rows will go first

        debug!(
            "Solving {:?} rows and {:?} columns with {} method",
            &rows, &columns, "standard"
        );

        let mut line_jobs = PriorityQueue::new();
        let mut all_jobs = HashSet::new();

        let mut add_job = |job: (bool, usize), priority: f64| {
            let priority = OrderedFloat(priority);
            line_jobs.push(job, priority);
            all_jobs.insert(job);
        };

        for row_index in rows {
            // the more this line solved
            // priority = 1 - board.row_solution_rate(row_index)

            // the closer to edge
            // priority = 1 - abs(2.0 * row_index / board.height - 1)

            // the more 'dense' this line
            // priority = 1 - board.densities[False][row_index]

            let new_job = (false, row_index);

            let priority = 0.0;
            // if has_blots:
            //    // the more attempts the less priority
            //    priority = board.attempts_to_try(*new_job)

            add_job(new_job, priority);
        }

        for column_index in columns {
            // the more this line solved
            // priority = 1 - board.column_solution_rate(column_index)

            // the closer to edge
            // priority = 1 - abs(2.0 * column_index / board.width - 1)

            // the more 'dense' this line
            // priority = 1 - board.densities[True][column_index]

            let new_job = (true, column_index);

            let priority = 0.0;
            // if has_blots:
            //   // the more attempts the less priority
            //   priority = board.attempts_to_try(*new_job)

            add_job(new_job, priority);
        }

        let mut _total_cells_solved = 0usize;

        while let Some((is_column, index, priority)) = Self::get_top_job(&mut line_jobs) {
            let new_jobs = self.solve_row::<S>(index, is_column)?;

            _total_cells_solved += new_jobs.len();
            for new_job in new_jobs {
                let new_priority = priority + 1.0;
                // if board.has_blots:
                //    // the more attempts the less priority
                //    new_priority = board.attempts_to_try(*new_job)

                // higher priority = more priority
                //add_job(new_job, new_priority);
                line_jobs.push(new_job, OrderedFloat(new_priority));
            }
            lines_solved += 1;
        }

        // all the following actions applied only to verified solving
        if !self.contradiction_mode {
            //let board = board.borrow();
            //board.solution_round_completed()
            //let rate = board.solution_rate();
            //if rate != 1 {
            //    warn!("The nonogram is not solved full: {:.4}", rate)
            //}
            let total_time = start.elapsed();
            warn!(
                "Full solution: {}.{:06} sec",
                total_time.as_secs(),
                total_time.subsec_micros()
            );
            warn!("Lines solved: {}", lines_solved);
        }

        Ok(())
    }

    fn get_top_job(
        pq: &mut PriorityQueue<(bool, usize), OrderedFloat<f64>>,
    ) -> Option<(bool, usize, f64)> {
        let ((is_column, index), priority) = pq.pop()?;

        if log_enabled!(Level::Info) {
            let line_description = if is_column { "column" } else { "row" };
            info!(
                "Solving {} {} with priority {}",
                index, line_description, priority
            );
        }
        Some((is_column, index, priority.0))
    }

    /// Solve a line with the solver S.
    /// If the line gets partially solved, put the crossed lines into queue.
    ///
    /// Return the list of new jobs that should be solved next (one job for each solved cell).
    pub fn solve_row<S>(&self, index: usize, is_column: bool) -> Result<Vec<(bool, usize)>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let start = Instant::now();

        let (line, updated) = {
            let board = self.board.borrow();
            let (line_desc, line, name) = if is_column {
                (
                    Rc::clone(&board.desc_cols[index]),
                    board.get_column(index),
                    "column",
                )
            } else {
                (
                    Rc::clone(&board.desc_rows[index]),
                    board.get_row(index),
                    "row",
                )
            };

            //let pre_solution_rate = Board::<B>::line_solution_rate(&line);
            //if pre_solution_rate == 1 {
            //    // do not check solved lines in trusted mode
            //    if ! contradiction_mode {
            //        return vec![];
            //     }
            //}

            debug!(
                "Solving {} {}: {:?}. Partial: {:?}",
                index, name, line_desc, line
            );

            let solution = Self::solve_with_cache::<S>(line_desc, Rc::clone(&line))?;
            (Rc::clone(&line), solution)
        };

        // let new_solution_rate = Board::<B>::line_solution_rate(&updated);

        let mut new_jobs = vec![];
        // if new_solution_rate > pre_solution_rate

        let line = line.borrow();
        if *line != updated {
            debug!("Original: {:?}", line);
            debug!("Updated: {:?}", &updated);

            new_jobs = line
                .iter()
                .zip(&updated)
                .enumerate()
                .filter_map(|(i, (pre, post))| {
                    if pre.is_updated_with(post) {
                        debug!(
                            "Diff on index={}: original={:?}, updated={:?}",
                            i, pre, &post
                        );
                        Some((!is_column, i))
                    } else {
                        None
                    }
                })
                .collect();

            let mut board = self.board.borrow_mut();
            let updated = updated.to_owned();
            if is_column {
                board.set_column(index, updated);
            } else {
                board.set_row(index, updated);
            }
        }

        if log_enabled!(Level::Debug) {
            let name = if is_column { "column" } else { "row" };
            let total_time = start.elapsed();
            debug!(
                "{}s solution: {}.{:06} sec",
                name,
                total_time.as_secs(),
                total_time.subsec_micros()
            );
            if !new_jobs.is_empty() {
                debug!("New info on {} {}: {:?}", name, index, new_jobs);
            }
        }
        Ok(new_jobs)
    }

    fn solve_with_cache<S>(
        line_desc: Rc<Description<B>>,
        line: Rc<RefCell<Vec<<B as Block>::Color>>>,
    ) -> Result<Vec<<B as Block>::Color>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let mut solver = S::new(line_desc, Rc::clone(&line));
        Ok(solver.solve()?.to_owned())
    }
}
