pub mod line;
pub mod probing;
pub mod propagation;

use super::board::{Block, Board};

use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use cached::{Cached, UnboundCache};

pub fn run<B, S>(board: Rc<RefCell<Board<B>>>) -> Result<(), String>
where
    B: Block,
    S: line::LineSolver<BlockType = B>,
{
    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<S>()?;

    if !board.borrow().is_solved_full() {
        warn!("Trying to solve with probing");
        let solver = probing::FullProbe1::new(Rc::clone(&board));
        solver.run::<S>()?;
        print_cache_info(&solver.cache.borrow());
    }

    Ok(())
}

fn print_cache_info<K, V>(cache: &UnboundCache<K, V>)
where
    K: Hash + Eq,
{
    if cache.cache_size() > 0 {
        let hits = cache.cache_hits().unwrap_or(0);
        let misses = cache.cache_misses().unwrap_or(0);
        let hit_rate = if hits == 0 {
            0.0
        } else {
            hits as f32 / (hits + misses) as f32
        };

        warn!(
            "Cache size: {}, hits: {}, hit rate: {}",
            cache.cache_size(),
            hits,
            hit_rate,
        );
    }
}
