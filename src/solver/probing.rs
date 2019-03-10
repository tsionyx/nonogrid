use super::super::board::{Block, Board, Color, Point};
use super::line::LineSolver;
use super::propagation;
use super::propagation::{CacheKey, CacheValue};

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use cached::Cached;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

pub type Impact<B> = HashMap<(Point, <B as Block>::Color), (usize, f64)>;

pub trait ProbeSolver {
    type BlockType: Block;

    fn new(board: Rc<RefCell<Board<Self::BlockType>>>) -> Self;

    fn run_unsolved<S>(&self) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = Self::BlockType>;
    fn run<S>(
        &self,
        probes: &mut PriorityQueue<Point, OrderedFloat<f64>>,
    ) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = Self::BlockType>;

    fn cache(&self) -> Ref<Cached<CacheKey<Self::BlockType>, CacheValue<Self::BlockType>>>;
}

pub struct FullProbe1<B>
where
    B: Block,
{
    board: Rc<RefCell<Board<B>>>,
    cache: propagation::ExternalCache<B>,
}

const PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED: f64 = 10.0;
const PRIORITY_NEIGHBOURS_OF_CONTRADICTION: f64 = 20.0;

impl<B> ProbeSolver for FullProbe1<B>
where
    B: Block,
{
    type BlockType = B;

    fn new(board: Rc<RefCell<Board<B>>>) -> Self {
        Self::with_cache(board, 10_000)
    }

    fn run_unsolved<S>(&self) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        self.run::<S>(&mut self.unsolved_cells())
    }

    fn run<S>(
        &self,
        probes: &mut PriorityQueue<Point, OrderedFloat<f64>>,
    ) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let mut impact = HashMap::new();

        if self.is_solved() {
            return Ok(impact);
        }

        let start = Instant::now();
        let mut contradictions_number = 0;
        //let mut iteration_probes = HashSet::new();

        loop {
            if self.is_solved() {
                break;
            }

            let mut contradiction = None;
            let mut probe_counter = 0u32;

            'outer: while let Some((point, priority)) = probes.pop() {
                probe_counter += 1;
                //if iteration_probes.contains(&point) {
                //    warn!("The probe {:?} with priority {} has been already tried before last contradiction", &point, priority.0);
                //    continue;
                //}

                warn!(
                    "Trying probe #{} {:?} with priority {}",
                    probe_counter, &point, priority.0
                );
                let probe_results = self.probe::<S>(point);

                for (color, updated) in probe_results {
                    if let Some(updated_cells) = updated {
                        impact.insert((point, color), (updated_cells, priority.0));
                    } else {
                        self.board().unset_color(&point, &color)?;
                        contradiction = Some(point);
                        break 'outer;
                    }
                }
                //iteration_probes.insert(point);
            }

            if let Some(contradiction) = contradiction {
                contradictions_number += 1;
                //iteration_probes.clear();

                for (point, priority) in self.propagate_point::<S>(&contradiction)? {
                    probes.push(point, priority);
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

        Ok(impact)
    }

    fn cache(&self) -> Ref<Cached<CacheKey<Self::BlockType>, CacheValue<Self::BlockType>>> {
        self.cache.borrow()
    }
}

impl<B> FullProbe1<B>
where
    B: Block,
{
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

    /// Try every color for given cell
    /// and return the number of solved cells (Some) or contradiction (None)
    fn probe<S>(&self, point: Point) -> HashMap<B::Color, Option<usize>>
    where
        S: LineSolver<BlockType = B>,
    {
        let mut changes = HashMap::new();

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
                changes.insert(assumption, Some(new_cells.len()));
            } else {
                warn!("Contradiction found! {:?}: {:?}", &point, &assumption);
                changes.insert(assumption, None);
            }
        }

        changes
    }
}
