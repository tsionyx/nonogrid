use super::super::board::{Block, Board};
use super::line::LineSolver;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Instant;

use log::Level;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::fmt::Debug;

pub fn solve<B, S>(
    board: Rc<RefCell<Board<B>>>,
    rows: Option<Vec<usize>>,
    columns: Option<Vec<usize>>,
    contradiction_mode: bool,
) where
    B: Block + Debug,
    <B as Block>::Color: Clone + Debug,
    S: LineSolver<BlockType = B>,
{
    let (rows, columns) = {
        // for safe borrowing
        let board = board.borrow();
        (
            rows.unwrap_or_else(|| (0..board.height()).collect()),
            columns.unwrap_or_else(|| (0..board.width()).collect()),
        )
    };

    // `is_solved_full` is expensive, so minimize calls to it.
    // Do not call if only a handful of lines has to be solved
    if rows.len() > 2 || columns.len() > 2 {
        // do not shortcut in contradiction_mode
        if !contradiction_mode && board.borrow().is_solved_full() {
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

    while let Some((is_column, index, priority)) = get_top_job(&mut line_jobs) {
        let new_jobs = solve_row::<B, S>(Rc::clone(&board), index, is_column);

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
    if !contradiction_mode {
        //let board = board.borrow();
        //board.solution_round_completed()
        //let rate = board.solution_rate();
        //if rate != 1 {
        //    warn!("The nonogram is not solved full: {:.4}", rate)
        //}
        let total_time = start.elapsed();
        info!(
            "Full solution: {}.{} sec",
            total_time.as_secs(),
            total_time.subsec_millis()
        );
        info!("Lines solved: {}", lines_solved);
    }
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
pub fn solve_row<B, S>(
    board: Rc<RefCell<Board<B>>>,
    index: usize,
    is_column: bool,
) -> Vec<(bool, usize)>
where
    B: Block + Debug,
    <B as Block>::Color: Clone + Debug,
    S: LineSolver<BlockType = B>,
{
    let start = Instant::now();

    {
        let board = board.borrow();
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
        let mut solver = S::new(line_desc, line);
        let updated = solver.solve();
    }

    vec![]
}
//    updated = solve_line(row_desc, row, method=method, normalized=True)
//
//    new_jobs = []
//
//    # if board.line_solution_rate(updated) > pre_solution_rate:
//    if row != updated:
//        # LOG.debug('Queue: %s', jobs_queue)
//        # LOG.debug(row)
//        # LOG.debug(updated)
//        for i, (pre, post) in enumerate(zip(row, updated)):
//            if _is_pixel_updated(pre, post):
//                new_jobs.append((not is_column, i))
//        # LOG.debug('Queue: %s', jobs_queue)
//        # LOG.debug('New info on %s %s: %s', desc, index, [job_index for _, job_index in new_jobs])
//
//        if is_column:
//            board.set_column(index, updated)
//        else:
//            board.set_row(index, updated)
//
//    # LOG.debug('%ss solution: %.6f sec', desc, time.time() - start)
//    return new_jobs
