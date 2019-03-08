pub mod backtracking;
pub mod line;
pub mod probing;
pub mod propagation;

use super::board::{Block, Board};
use backtracking::Solver;
use probing::ProbeSolver;

use std::cell::RefCell;
use std::rc::Rc;

pub fn run<B, S, P>(board: Rc<RefCell<Board<B>>>) -> Result<(), String>
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
        let solver = Solver::<_, P>::new(Rc::clone(&board));
        solver.run::<S>()?;
        solver.print_cache_info();
    }

    Ok(())
}
