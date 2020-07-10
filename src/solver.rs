use log::warn;

use crate::block::Block;
use crate::board::Board;
use crate::solver::probing::ProbeSolver;
use crate::utils::rc::MutRc;

#[cfg(not(feature = "sat"))]
pub mod backtracking;
pub mod line;
pub mod probing;
pub mod propagation;
#[cfg(feature = "sat")]
pub mod sat;

#[cfg(not(feature = "sat"))]
pub fn run<B, S, P>(
    board: MutRc<Board<B>>,
    max_solutions: Option<usize>,
    timeout: Option<u32>,
    max_depth: Option<usize>,
) -> Result<Option<backtracking::Solver<B, P, S>>, String>
where
    B: Block,
    S: line::LineSolver<BlockType = B>,
    P: ProbeSolver<BlockType = B>,
{
    warn!("Solving with simple line propagation");
    let mut solver = propagation::Solver::new(MutRc::clone(&board));
    let solved_points = solver
        .run::<S>(None)
        .map_err(|_| "Bad puzzle for sure: simple propagation failed".to_string())?;

    warn!("Solved {} points", solved_points.len());

    if !board.read().is_solved_full() {
        warn!("Trying to solve with backtracking");
        let mut solver =
            backtracking::Solver::<_, P, S>::with_options(board, max_solutions, timeout, max_depth);
        solver.run()?;
        return Ok(Some(solver));
    }

    Ok(None)
}

#[cfg(feature = "sat")]
#[allow(clippy::needless_pass_by_value)]
pub fn run<B, S, P>(
    board: MutRc<Board<B>>,
    max_solutions: Option<usize>,
) -> Result<Option<impl Iterator<Item = Vec<B::Color>>>, String>
where
    B: Block,
    S: line::LineSolver<BlockType = B>,
    P: ProbeSolver<BlockType = B>,
{
    warn!("Solving with simple line propagation");
    let mut solver = propagation::Solver::new(MutRc::clone(&board));
    let solved_points = solver
        .run::<S>(None)
        .map_err(|_| "Bad puzzle for sure: simple propagation failed".to_string())?;
    warn!("Solved {} points", solved_points.len());

    if board.read().is_solved_full() {
        return Ok(None);
    }

    let impact = {
        warn!("Solving with probing");
        let mut probe_solver = P::with_board(MutRc::clone(&board));
        probe_solver.run_unsolved::<S>()?
    };

    if !board.read().is_solved_full() {
        warn!("Trying to solve with SAT");
        let solver = sat::ClauseGenerator::with_clues(
            board.read().descriptions(false),
            board.read().descriptions(true),
            board.read().make_snapshot(),
        );

        let solutions_iter = solver.run(impact, max_solutions);

        return Ok(Some(solutions_iter));
    }

    Ok(None)
}
