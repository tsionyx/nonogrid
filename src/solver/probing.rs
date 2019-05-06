use super::super::block::{Block, Color};
use super::super::board::{Board, Point};
use super::line::LineSolver;
use super::propagation;

use std::cell::{Ref, RefCell};
use std::rc::Rc;
//use std::time::Instant;

use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::HashMap;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue as PQ;

pub type Impact<B> = HashMap<(Point, <B as Block>::Color), (usize, f64)>;
type FloatPriorityQueue<K> = PQ<K, OrderedFloat<f64>, DefaultHashBuilder>;

pub trait ProbeSolver {
    type BlockType: Block;

    fn with_board(board: Rc<RefCell<Board<Self::BlockType>>>) -> Self;

    fn unsolved_cells(&self) -> FloatPriorityQueue<Point>;
    fn propagate_point<S>(&self, point: &Point) -> Result<Vec<(Point, OrderedFloat<f64>)>, String>
    where
        S: LineSolver<BlockType = Self::BlockType>;

    fn run_unsolved<S>(&self) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = Self::BlockType>,
    {
        self.run::<S>(&mut self.unsolved_cells())
    }

    fn run<S>(
        &self,
        probes: &mut FloatPriorityQueue<Point>,
    ) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = Self::BlockType>;
}

pub struct FullProbe1<B>
where
    B: Block,
{
    board: Rc<RefCell<Board<B>>>,
}

const PRIORITY_NEIGHBOURS_OF_NEWLY_SOLVED: f64 = 10.0;
const PRIORITY_NEIGHBOURS_OF_CONTRADICTION: f64 = 20.0;

impl<B> ProbeSolver for FullProbe1<B>
where
    B: Block,
{
    type BlockType = B;

    fn with_board(board: Rc<RefCell<Board<B>>>) -> Self {
        board.borrow_mut().init_cache();
        Self { board }
    }

    fn unsolved_cells(&self) -> FloatPriorityQueue<Point> {
        let board = self.board();
        let unsolved = board.unsolved_cells();

        let mut queue = FloatPriorityQueue::with_default_hasher();
        queue.extend(unsolved.map(|point| {
            let no_unsolved = board.unsolved_neighbours(&point).count() as f64;
            let row_rate = board.row_solution_rate(point.y());
            let column_rate = board.column_solution_rate(point.x());
            let priority = row_rate + column_rate - no_unsolved + 4.0;
            (point, OrderedFloat(priority))
        }));

        queue
    }

    fn propagate_point<S>(&self, point: &Point) -> Result<Vec<(Point, OrderedFloat<f64>)>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let fixed_points = self.run_propagation::<S>(point)?;
        let mut new_jobs = vec![];

        for new_point in fixed_points {
            for neighbour in self.board().unsolved_neighbours(&new_point) {
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

    fn run<S>(
        &self,
        probes: &mut FloatPriorityQueue<Point>,
    ) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        //let start = Instant::now();
        let mut contradictions_number = 0;
        //let mut iteration_probes = HashSet::new();

        let impact = loop {
            let mut impact = HashMap::new();

            if self.is_solved() {
                break impact;
            }

            let mut false_probes = None;
            let mut probe_counter = 0_u32;

            while let Some((point, priority)) = probes.pop() {
                probe_counter += 1;
                //if iteration_probes.contains(&point) {
                //    warn!("The probe {:?} with priority {} has been already tried before last contradiction", &point, priority.0);
                //    continue;
                //}

                info!(
                    "Trying probe #{} {:?} with priority {}",
                    probe_counter, &point, priority.0
                );
                let probe_results = self.probe::<S>(point);

                let (contradictions, non_contradictions): (Vec<_>, Vec<_>) = probe_results
                    .into_iter()
                    .partition(|(_color, size)| size.is_none());

                if !contradictions.is_empty() {
                    let bad_colors: Vec<_> = contradictions
                        .iter()
                        .map(|(color, _should_be_none)| *color)
                        .collect();

                    false_probes = Some((point, bad_colors));
                    break;
                }

                for (color, updated) in non_contradictions {
                    if let Some(updated_cells) = updated {
                        impact.insert((point, color), (updated_cells, priority.0));
                    }
                }
                //iteration_probes.insert(point);
            }

            if let Some((contradiction, colors)) = false_probes {
                contradictions_number += 1;
                //iteration_probes.clear();

                for color in colors {
                    Board::unset_color_with_callback(
                        Rc::clone(&self.board),
                        &contradiction,
                        &color,
                    )?;
                }
                for (point, priority) in self.propagate_point::<S>(&contradiction)? {
                    probes.push(point, priority);
                }
            } else {
                break impact;
            }
        };

        if contradictions_number > 0 {
            //let total_time = start.elapsed();
            //warn!(
            //    "Full solution: {}.{:06} sec",
            //    total_time.as_secs(),
            //    total_time.subsec_micros()
            //);
            warn!("Contradictions found: {}", contradictions_number);
        }
        Ok(impact)
    }
}

impl<B> FullProbe1<B>
where
    B: Block,
{
    fn board(&self) -> Ref<Board<B>> {
        self.board.borrow()
    }

    fn run_propagation<S>(&self, point: &Point) -> Result<Vec<Point>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        let point_solver = propagation::Solver::with_point(Rc::clone(&self.board), *point);
        point_solver.run::<S>()
    }

    fn is_solved(&self) -> bool {
        self.board().is_solved_full()
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
        debug!("Probing {:?} for variants: {:?}", point, &vars);

        for assumption in vars {
            let save = self.board().make_snapshot();
            Board::set_color_with_callback(Rc::clone(&self.board), &point, &assumption);

            let solved = self.run_propagation::<S>(&point);
            Board::restore_with_callback(Rc::clone(&self.board), save);

            if let Ok(new_cells) = solved {
                if !new_cells.is_empty() {
                    info!("Probing {:?}: {:?}", point, assumption);
                    debug!("New info: {:?}", new_cells);
                }
                changes.insert(assumption, Some(new_cells.len()));
            } else {
                info!("Contradiction found! {:?}: {:?}", &point, &assumption);
                changes.insert(assumption, None);
            }
        }

        debug!("Found impact: {:?}", changes);
        changes
    }
}
