use super::super::board::{Block, Board};
use super::line::LineSolver;
use super::propagation;

use std::cell::{Ref, RefCell};
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

pub struct FullProbe1<B>
where
    B: Block,
{
    board: Rc<RefCell<Board<B>>>,
    pub cache: propagation::ExternalCache<B>,
}

impl<B> FullProbe1<B>
where
    B: Block + Debug + Eq + Hash,
    <B as Block>::Color: Clone + Debug + Eq + Hash,
{
    pub fn new(board: Rc<RefCell<Board<B>>>) -> Self {
        Self::with_cache(board, 10_000)
    }

    pub fn with_cache(board: Rc<RefCell<Board<B>>>, cache_capacity: usize) -> Self {
        let cache = propagation::new_cache(cache_capacity);
        Self { board, cache }
    }

    fn board(&self) -> Ref<Board<B>> {
        self.board.borrow()
    }

    pub fn run<S>(&self) -> Result<(), String>
    where
        S: LineSolver<BlockType = B>,
    {
        if self.board().is_solved_full() {
            return Ok(());
        }

        loop {
            let cached_solver = propagation::Solver::with_options(
                Rc::clone(&self.board),
                None,
                None,
                false,
                Some(Rc::clone(&self.cache)),
            );
            cached_solver.run::<S>()?;

            if self.board().is_solved_full() {
                break;
            }
        }

        Ok(())
    }
}
