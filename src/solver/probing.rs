use super::super::board::{Block, Board, Color, Point};
use super::line::LineSolver;
use super::propagation;

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Sub};
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
    B::Color:
        Clone + Debug + Eq + Hash + Add<Output = B::Color> + Sub<Output = Result<B::Color, String>>,
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

    fn propagate<S>(&self) -> Result<HashMap<Point, B::Color>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        self.propagate_board::<S>(Rc::clone(&self.board), None, None)
    }

    fn propagate_board<S>(
        &self,
        board: Rc<RefCell<Board<B>>>,
        rows: Option<Vec<usize>>,
        columns: Option<Vec<usize>>,
    ) -> Result<HashMap<Point, B::Color>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let cached_solver = propagation::Solver::with_options(
            board,
            rows,
            columns,
            false,
            Some(Rc::clone(&self.cache)),
        );
        cached_solver.run::<S>()
    }

    pub fn run<S>(&self) -> Result<(), String>
    where
        S: LineSolver<BlockType = B>,
    {
        if self.board().is_solved_full() {
            return Ok(());
        }

        'outer: loop {
            if self.board().is_solved_full() {
                break;
            }

            self.update_all();

            let mut found_update = false;
            for point in self.board().unsolved_cells() {
                let updated = self.probe::<S>(point)?;
                if updated > 0 {
                    found_update = true;
                    break;
                }
            }

            if !found_update || self.board().is_solved_full() {
                break;
            } else {
                self.propagate::<S>()?;
            }
        }

        Ok(())
    }

    fn update_all(&self) {}

    fn probe<S>(&self, point: Point) -> Result<u32, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let probes: HashMap<B::Color, Board<B>> = HashMap::new();

        for assumption in self.board().cell(&point).variants() {
            let board_temp = self.board().clone();
            board_temp.set_color(&point, &assumption);

            //let diff = self.board().diff(&board_temp);

            let solved = self.propagate_board::<S>(
                Rc::new(RefCell::new(board_temp)),
                Some(vec![point.x()]),
                Some(vec![point.y()]),
            );


            if let Ok(new_cells) = solved {
                if new_cells.len() > 0 {
                    info!("Probing {:?}: {:?}", point, assumption);
                    debug!("New info: {:?}", new_cells);
                }
            } else {
                warn!("Probing failed! {:?}: {:?}", point, assumption);
            }
        }

        Ok(0)
    }
}
