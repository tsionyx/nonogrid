use super::super::board::{Block, Board};
use super::line::LineSolver;
use super::probing::ProbeSolver;

use std::cell::{Ref, RefCell};
use std::hash::Hash;
use std::rc::Rc;
use std::time::Instant;

use cached::Cached;

pub struct Solver<B, P>
where
    B: Block,
    P: ProbeSolver<BlockType = B>,
{
    board: Rc<RefCell<Board<B>>>,
    probe_solver: P,
}

impl<B, P> Solver<B, P>
where
    B: Block,
    P: ProbeSolver<BlockType = B>,
{
    pub fn new(board: Rc<RefCell<Board<B>>>) -> Self {
        let probe_solver = P::new(Rc::clone(&board));
        Self {
            board,
            probe_solver,
        }
    }

    pub fn run<S>(&self) -> Result<(), String>
    where
        S: LineSolver<BlockType = B>,
    {
        if self.is_solved() {
            return Ok(());
        }

        self.probe_solver.run::<S>()?;
        if self.is_solved() {
            return Ok(());
        }

        let start = Instant::now();

        // TODO: add backtracking logic here
        let depth_reached = 0u32;

        let total_time = start.elapsed();
        warn!(
            "Full solution: {}.{:06} sec",
            total_time.as_secs(),
            total_time.subsec_micros()
        );
        warn!("Depth reached: {}", depth_reached);

        Ok(())
    }

    pub fn print_cache_info(&self) {
        print_cache_info(self.probe_solver.cache());
    }

    fn board(&self) -> Ref<Board<B>> {
        self.board.borrow()
    }

    fn is_solved(&self) -> bool {
        self.board().is_solved_full()
    }
}

fn print_cache_info<K, V>(cache: Ref<Cached<K, V>>)
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
