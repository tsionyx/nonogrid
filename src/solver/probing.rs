use super::super::board::{Block, Board, Color, Point};
use super::line::LineSolver;
use super::propagation;

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

pub struct FullProbe1<B>
where
    B: Block,
{
    board: Rc<RefCell<Board<B>>>,
    pub cache: propagation::ExternalCache<B>,
}

const PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED: f64 = 10.0;
const PRIORITY_NEIGHBOURS_OF_CONTRADICTION: f64 = 20.0;

impl<B> FullProbe1<B>
where
    B: Block,
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

    fn propagate_point<S>(&self, point: &Point) -> Result<Vec<(Point, OrderedFloat<f64>)>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let fixed_points = self.propagate_board::<S>(
            Rc::clone(&self.board),
            Some(vec![point.y()]),
            Some(vec![point.x()]),
        )?;
        let mut new_jobs = vec![];

        for new_point in fixed_points.keys() {
            for neighbour in self.board().unsolved_neighbours(new_point) {
                new_jobs.push((neighbour, OrderedFloat(PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED)));
            }
        }

        for neighbour in self.board().unsolved_neighbours(&point) {
            new_jobs.push((
                neighbour,
                OrderedFloat(PRIORITY_NEIGHBOURS_OF_CONTRADICTION),
            ));
        }

        info!("Solution rate: {}", self.board().solution_rate());
        Ok(new_jobs)
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

    fn is_solved(&self) -> bool {
        self.board().is_solved_full()
    }

    fn unsolved_cells(&self) -> PriorityQueue<Point, OrderedFloat<f64>> {
        let mut queue = PriorityQueue::new();
        let board = self.board();
        let unsolved = board.unsolved_cells();
        unsolved.iter().for_each(|p| {
            let no_unsolved = board.unsolved_neighbours(p).len() as f64;
            let row_rate = board.row_solution_rate(p.y());
            let column_rate = board.column_solution_rate(p.x());
            let priority = row_rate + column_rate - no_unsolved + 4.0;
            queue.push(*p, OrderedFloat(priority));
        });

        queue
    }

    pub fn run<S>(&self) -> Result<(), String>
    where
        S: LineSolver<BlockType = B>,
    {
        if self.is_solved() {
            return Ok(());
        }

        let start = Instant::now();
        let mut contradictions_number = 0;

        let unsolved_probes = &mut self.unsolved_cells();

        loop {
            if self.is_solved() {
                break;
            }

            let mut contradiction = None;

            while let Some((point, priority)) = unsolved_probes.pop() {
                warn!("Trying probe {:?} with priority {}", &point, priority.0);
                let found_update = self.probe::<S>(point)?;
                if found_update {
                    contradiction = Some(point);
                    contradictions_number += 1;
                    break;
                }
            }

            if let Some(contradiction) = contradiction {
                for (point, priority) in self.propagate_point::<S>(&contradiction)? {
                    unsolved_probes.push(point, priority);
                }
            } else {
                break;
            }
        }

        let total_time = start.elapsed();
        warn!(
            "Full solution: {}.{:06} sec",
            total_time.as_secs(),
            total_time.subsec_micros()
        );
        warn!("Contradictions found: {}", contradictions_number);

        Ok(())
    }

    fn probe<S>(&self, point: Point) -> Result<bool, String>
    where
        S: LineSolver<BlockType = B>,
    {
        //let probes: HashMap<B::Color, Board<B>> = HashMap::new();

        if self.board().cell(&point).is_solved() {
            info!("Probing expired! {:?}", &point);
        }

        let vars = self.board().cell(&point).variants();

        for assumption in vars {
            let board_temp = self.board().clone();
            board_temp.set_color(&point, &assumption);

            //let diff = self.board().diff(&board_temp);

            let solved = self.propagate_board::<S>(
                Rc::new(RefCell::new(board_temp)),
                Some(vec![point.y()]),
                Some(vec![point.x()]),
            );

            if let Ok(new_cells) = solved {
                if !new_cells.is_empty() {
                    info!("Probing {:?}: {:?}", point, assumption);
                    debug!("New info: {:?}", new_cells);
                }
            } else {
                warn!("Probing failed! {:?}: {:?}", &point, &assumption);
                self.board().unset_color(&point, &assumption)?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}
