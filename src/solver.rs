use crate::block::Block;
use crate::board::Board;
use crate::solver::{backtracking::Solver, probing::ProbeSolver};
use crate::utils::rc::MutRc;

pub mod backtracking;
pub mod line;
pub mod probing;
pub mod propagation;
#[cfg(feature = "sat")]
pub mod sat;

pub fn run<B, S, P>(
    board: MutRc<Board<B>>,
    max_solutions: Option<usize>,
    timeout: Option<u32>,
    max_depth: Option<usize>,
) -> Result<Option<Solver<B, P, S>>, String>
where
    B: Block,
    S: line::LineSolver<BlockType = B>,
    P: ProbeSolver<BlockType = B>,
{
    warn!("Solving with simple line propagation");
    let mut solver = propagation::Solver::new(MutRc::clone(&board));
    let solved = solver
        .run::<S>(None)
        .map_err(|_| "Bad puzzle for sure: simple propagation failed".to_string())?;

    warn!("Solved {} points", solved.len());

    if !board.read().is_solved_full() {
        warn!("Trying to solve with backtracking");
        let mut solver = Solver::<_, P, S>::with_options(board, max_solutions, timeout, max_depth);
        solver.run()?;
        return Ok(Some(solver));
    }

    Ok(None)
}
