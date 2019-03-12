pub mod backtracking;
pub mod line;
pub mod probing;
pub mod propagation;

use super::board::{Block, Board};
use super::solver::backtracking::Solver;
use super::solver::probing::ProbeSolver;

use std::cell::RefCell;
use std::rc::Rc;

pub fn run<B, S, P>(
    board: Rc<RefCell<Board<B>>>,
    max_solutions: Option<usize>,
    timeout: Option<u32>,
    max_depth: Option<usize>,
) -> Result<(), String>
where
    B: Block,
    S: line::LineSolver<BlockType = B>,
    P: ProbeSolver<BlockType = B>,
{
    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<S>()?;

    if !board.borrow().is_solved_full() {
        warn!("Trying to solve with backtracking");
        let mut solver =
            Solver::<_, P, S>::with_options(Rc::clone(&board), max_solutions, timeout, max_depth);
        solver.run()?;
        solver.print_cache_info();
    }

    Ok(())
}
