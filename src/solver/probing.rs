use std::env;

use hashbrown::hash_map::DefaultHashBuilder;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue as PQ;

use crate::block::{Block, Color};
use crate::board::{Board, Point};
use crate::solver::{line::LineSolver, propagation};
use crate::utils::{
    iter::PartialEntry,
    rc::{MutRc, ReadRef},
};

//use std::time::Instant;

#[derive(Debug)]
pub struct ProbeImpact<C: Color> {
    point: Point,
    color: C,
    cells_solved: usize,
    probe_priority: Priority,
}

impl<C: Color> ProbeImpact<C> {
    pub fn as_tuple(&self) -> (Point, C, usize, Priority) {
        (
            self.point,
            self.color,
            self.cells_solved,
            self.probe_priority,
        )
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
//pub struct Priority(pub u32);
pub struct Priority(pub OrderedFloat<f64>);

impl Priority {
    //const MULTIPLIER: u32 = 10000;
    //const NEIGHBOUR_OF_NEWLY_SOLVED: Self = Self(10 * Self::MULTIPLIER);
    //const NEIGHBOUR_OF_CONTRADICTION: Self = Self(20 * Self::MULTIPLIER);
    const NEIGHBOUR_OF_NEWLY_SOLVED: Self = Self(OrderedFloat(10.0));
    const NEIGHBOUR_OF_CONTRADICTION: Self = Self(OrderedFloat(20.0));
}

impl From<f64> for Priority {
    fn from(val: f64) -> Self {
        //Self((val * Self::MULTIPLIER as f64) as u32)
        Self(OrderedFloat(val))
    }
}

pub type Impact<B> = Vec<ProbeImpact<<B as Block>::Color>>;
type OrderedPoints = PQ<Point, Priority, DefaultHashBuilder>;

pub trait ProbeSolver {
    type BlockType: Block;

    fn with_board(board: MutRc<Board<Self::BlockType>>) -> Self;

    fn unsolved_cells(&self) -> OrderedPoints;
    fn propagate_point<S>(&self, point: &Point) -> Result<Vec<(Point, Priority)>, ()>
    where
        S: LineSolver<BlockType = Self::BlockType>;

    fn run_unsolved<S>(&self) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = Self::BlockType>,
    {
        self.run::<S>(&mut self.unsolved_cells())
    }

    fn run<S>(&self, probes: &mut OrderedPoints) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = Self::BlockType>;
}

#[allow(missing_debug_implementations)]
pub struct FullProbe1<B>
where
    B: Block,
{
    board: MutRc<Board<B>>,
    low_threshold: Priority,
}

fn low_priority_threshold() -> Priority {
    env::var("LOW_PRIORITY")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(0.0)
        .into()
}

impl<B> ProbeSolver for FullProbe1<B>
where
    B: Block,
{
    type BlockType = B;

    fn with_board(board: MutRc<Board<B>>) -> Self {
        board.write().init_cache();
        Self {
            board,
            low_threshold: low_priority_threshold(),
        }
    }

    fn unsolved_cells(&self) -> OrderedPoints {
        let board = self.board();
        let unsolved = board.unsolved_cells();

        let mut row_rate_cache = Vec::with_none(board.height());
        let mut column_rate_cache = Vec::with_none(board.width());

        let mut queue = OrderedPoints::with_default_hasher();
        queue.extend(unsolved.map(|point| {
            let no_solved = 4 - board.unsolved_neighbours(&point).count();
            let row_rate = row_rate_cache
                .unwrap_or_insert_with(point.y(), || board.row_solution_rate(point.y()));
            let column_rate = column_rate_cache
                .unwrap_or_insert_with(point.x(), || board.column_solution_rate(point.x()));

            let priority = no_solved as f64 + row_rate + column_rate;
            (point, priority.into())
        }));

        queue
    }

    fn propagate_point<S>(&self, point: &Point) -> Result<Vec<(Point, Priority)>, ()>
    where
        S: LineSolver<BlockType = B>,
    {
        let fixed_points = self.run_propagation::<S>(point)?;
        let board = self.board();
        debug!("Solution rate: {:.2}", self.board().solution_rate());

        Ok(fixed_points
            .into_iter()
            .flat_map(|new_point| {
                board
                    .unsolved_neighbours(&new_point)
                    .map(|neighbour| (neighbour, Priority::NEIGHBOUR_OF_NEWLY_SOLVED))
            })
            .chain(
                self.board()
                    .unsolved_neighbours(&point)
                    .map(|neighbour| (neighbour, Priority::NEIGHBOUR_OF_CONTRADICTION)),
            )
            .collect())
    }

    fn run<S>(&self, probes: &mut OrderedPoints) -> Result<Impact<Self::BlockType>, String>
    where
        S: LineSolver<BlockType = B>,
    {
        //let start = Instant::now();
        let mut contradictions_number = 0;
        //let mut iteration_probes = HashSet::new();

        let impact =
            loop {
                let mut impact = Vec::new();

                if self.is_solved() {
                    break impact;
                }

                let mut false_probes = None;
                let mut probe_counter = 0_u32;

                while let Some((point, priority)) = probes.pop() {
                    probe_counter += 1;

                    debug!(
                        "Trying probe #{} {:?} with priority {:?}",
                        probe_counter, point, priority
                    );
                    if priority < self.low_threshold {
                        impact.extend(self.board().cell(&point).variants().into_iter().map(
                            |color| ProbeImpact {
                                point,
                                color,
                                probe_priority: priority,
                                cells_solved: 0,
                            },
                        ));
                        continue;
                    }

                    let probe_results = self.probe::<S>(point);
                    let (contradictions, non_contradictions): (Vec<_>, Vec<_>) = probe_results
                        .into_iter()
                        .partition(|(_color, res)| res.is_contradiction());

                    if !contradictions.is_empty() {
                        let bad_colors: Vec<_> = contradictions
                            .into_iter()
                            .map(|(color, _should_be_none)| color)
                            .collect();

                        false_probes = Some((point, bad_colors));
                        break;
                    }

                    impact.extend(non_contradictions.into_iter().map(|(color, updated)| {
                        ProbeImpact {
                            point,
                            color,
                            probe_priority: priority,
                            cells_solved: updated.unwrap(),
                        }
                    }));
                }

                if let Some((contradiction, colors)) = false_probes {
                    contradictions_number += 1;

                    for color in &colors {
                        Board::unset_color_with_callback(
                            MutRc::clone(&self.board),
                            &contradiction,
                            color,
                        )?;
                    }
                    let new_probes = self.propagate_point::<S>(&contradiction).map_err(|_| {
                        format!(
                            "Error while propagating contradicted values {:?} in {:?}",
                            &colors, &contradiction
                        )
                    })?;
                    probes.extend(new_probes);
                } else {
                    break impact;
                }
            };

        if contradictions_number > 0 {
            //let total_time = start.elapsed();
            //info!(
            //    "Full solution: {}.{:06} sec",
            //    total_time.as_secs(),
            //    total_time.subsec_micros()
            //);
            info!("Contradictions found: {}", contradictions_number);
        }
        Ok(impact)
    }
}

enum ProbeResult<PropagationResult> {
    Contradiction,
    NewInfo(PropagationResult),
}

impl<PropagationResult> ProbeResult<PropagationResult> {
    fn is_contradiction(&self) -> bool {
        if let ProbeResult::Contradiction = self {
            true
        } else {
            false
        }
    }

    fn unwrap(self) -> PropagationResult {
        if let ProbeResult::NewInfo(res) = self {
            return res;
        }

        panic!("The result is contradiction")
    }
}

impl<B> FullProbe1<B>
where
    B: Block,
{
    fn board(&self) -> ReadRef<Board<B>> {
        self.board.read()
    }

    fn run_propagation<S>(&self, point: &Point) -> Result<Vec<Point>, ()>
    where
        S: LineSolver<BlockType = B>,
    {
        let point_solver = propagation::Solver::with_point(MutRc::clone(&self.board), *point);
        point_solver.run::<S>()
    }

    fn is_solved(&self) -> bool {
        self.board().is_solved_full()
    }

    /// Try every color for given cell
    /// and return the number of solved cells (Some) or contradiction (None)
    fn probe<S>(&self, point: Point) -> Vec<(B::Color, ProbeResult<usize>)>
    where
        S: LineSolver<BlockType = B>,
    {
        if self.board().cell(&point).is_solved() {
            info!("Probing expired! {:?}", point);
        }

        let vars = self.board().cell(&point).variants();
        debug!("Probing {:?} for variants: {:?}", point, vars);

        vars.into_iter()
            .map(|assumption| {
                let save = self.board().make_snapshot();
                Board::set_color_with_callback(MutRc::clone(&self.board), &point, &assumption);

                let solved = self.run_propagation::<S>(&point);

                let impact = solved.ok().map_or_else(
                    || {
                        debug!("Contradiction found! {:?}: {:?}", point, assumption);
                        ProbeResult::Contradiction
                    },
                    |new_cells| {
                        if !new_cells.is_empty() {
                            debug!(
                                "Probing {:?}: {:?} brings some new info: {:?}",
                                point, assumption, new_cells
                            );
                        }
                        ProbeResult::NewInfo(new_cells.len())
                    },
                );
                Board::restore_with_callback(MutRc::clone(&self.board), save);

                (assumption, impact)
            })
            .collect()
    }
}
