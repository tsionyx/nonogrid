pub mod backtracking;
pub mod line;
pub mod probing;
pub mod propagation;

use super::block::Block;
use super::board::Board;
use super::solver::backtracking::Solver;
use super::solver::probing::ProbeSolver;
use super::utils::rc::MutRc;

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
    let solver = propagation::Solver::new(MutRc::clone(&board));
    solver.run::<S>()?;

    if !board.read().is_solved_full() {
        warn!("Trying to solve with backtracking");
        let mut solver = Solver::<_, P, S>::with_options(
            MutRc::clone(&board),
            max_solutions,
            timeout,
            max_depth,
        );
        solver.run()?;
        return Ok(Some(solver));
    }

    Ok(None)
}
