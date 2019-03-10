use super::super::board::{Block, Board, Color, Point};
use super::line::LineSolver;
use super::probing::{Impact, ProbeSolver};

use std::cell::{Ref, RefCell};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use std::time::Instant;

use cached::Cached;
use ordered_float::OrderedFloat;

type Solution<B> = Vec<Rc<RefCell<Vec<<B as Block>::Color>>>>;

pub struct Solver<B, P>
where
    B: Block,
    P: ProbeSolver<BlockType = B>,
{
    board: Rc<RefCell<Board<B>>>,
    probe_solver: P,

    // search options
    max_solutions: Option<usize>,
    timeout: Option<u32>,
    max_depth: Option<usize>,

    // dynamic variables
    solutions: Vec<Solution<B>>,
    depth_reached: u32,
    start_time: Option<Instant>,
    explored_paths: HashSet<Vec<Point>>,
}

impl<B, P> Solver<B, P>
where
    B: Block,
    P: ProbeSolver<BlockType = B>,
{
    pub fn new(board: Rc<RefCell<Board<B>>>) -> Self {
        Self::with_options(board, None, None, None)
    }

    pub fn with_options(
        board: Rc<RefCell<Board<B>>>,
        max_solutions: Option<usize>,
        timeout: Option<u32>,
        max_depth: Option<usize>,
    ) -> Self {
        let probe_solver = P::new(Rc::clone(&board));
        Self {
            board,
            probe_solver,
            max_solutions,
            timeout,
            max_depth,
            solutions: vec![],
            depth_reached: 0,
            start_time: None,
            explored_paths: HashSet::new(),
        }
    }

    pub fn run<S>(&mut self) -> Result<(), String>
    where
        S: LineSolver<BlockType = B>,
    {
        if self.is_solved() {
            return Ok(());
        }

        let impact = self.probe_solver.run_unsolved::<S>()?;
        if self.is_solved() {
            return Ok(());
        }

        self.start_time = Some(Instant::now());

        let directions = self.choose_directions(&impact);
        warn!(
            "Starting depth-first search (initial rate is {:.4})",
            self.board().solution_rate()
        );
        let mut path = vec![];
        self.search(&directions, &mut path);

        let total_time = self.start_time.unwrap().elapsed();
        warn!(
            "Search completed (depth reached: {}, solutions found: {})",
            self.depth_reached,
            self.solutions.len()
        );

        warn!(
            "Full solution: {}.{:06} sec. The rate is {:.4}",
            total_time.as_secs(),
            total_time.subsec_micros(),
            self.board().solution_rate(),
        );
        warn!("Depth reached: {}", self.depth_reached);

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

    fn set_explored(&mut self, path: &[Point]) {
        let mut path = path.to_vec();
        path.sort();
        self.explored_paths.insert(path);
    }

    fn is_explored(&self, path: &[Point]) -> bool {
        let mut path = path.to_vec();
        path.sort();
        self.explored_paths.contains(&path)
    }

    fn already_found(&self) -> bool {
        for (i, solution) in self.solutions.iter().enumerate() {
            let (removed, added) = self.board().diff(solution);

            if removed.is_empty() && added.is_empty() {
                info!("The solution is the same as {}-th", i);
                return true;
            }
            info!("The current solution differs from {}-th saved one: (added in current: {:?}, removed in current: {:?})", i, removed, added);
        }

        false
    }

    fn add_solution<S>(&mut self) -> Result<(), String>
    where
        S: LineSolver<BlockType = B>,
    {
        // force to check the board
        self.probe_solver.run_unsolved::<S>()?;

        info!("Found one of solutions");
        if self.already_found() {
            info!("Solution already exists.");
        } else {
            let cells = self.board().make_snapshot();
            self.solutions.push(cells);
        }

        Ok(())
    }

    /// The most promising (point+color) pair should go first
    fn choose_directions(&self, impact: &Impact<B>) -> Vec<(Point, B::Color)> {
        let mut point_wise = HashMap::new();

        for ((point, color), (new_points, priority)) in impact.iter() {
            if self.board().cell(point).is_solved() {
                continue;
            }
            let point_colors = point_wise.entry(*point).or_insert_with(HashMap::new);
            point_colors.insert(color, (*new_points, *priority));
        }

        let mut points_rate: Vec<_> = point_wise
            .iter()
            .map(|(point, color_to_impact)| {
                let values: Vec<_> = color_to_impact.values().collect();
                (point, OrderedFloat(Self::rate_by_impact(values)))
            })
            .collect();
        points_rate.sort_by_key(|&(_point, rate)| Reverse(rate));
        dbg!(&points_rate[..10]);

        points_rate
            .iter()
            .map(|&(point, _rate)| {
                let mut point_colors: Vec<_> =
                    point_wise[point].iter().map(|(k, v)| (**k, *v)).collect();
                // the most impacting color goes first
                point_colors.sort_by_key(|(_color, (new_points, _priority))| Reverse(*new_points));
                let point_order: Vec<_> = point_colors
                    .iter()
                    .map(|(color, _impact)| (*point, *color))
                    .collect();
                point_order
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    fn rate_by_impact(impact: Vec<&(usize, f64)>) -> f64 {
        let only_new_points: Vec<_> = impact
            .iter()
            .map(|(new_points, _priority)| *new_points)
            .collect();

        // MAX is the most trivial, but also most ineffective solution.
        // For details, see https://ieeexplore.ieee.org/document/6476646
        //
        //let max = only_new_points.iter().max().unwrap_or(&&0);
        //*max.clone() as f64

        match only_new_points.as_slice() {
            [] => 0.0,
            [single] => *single as f64,
            [first, second] => {
                let min = only_new_points.iter().min().unwrap();
                let diff = ((first + 1) as f64).ln() - ((second + 1) as f64).ln();
                *min as f64 + diff.abs()
            }
            _more_than_two => only_new_points.iter().map(|x| (x + 1) as f64).product(),
        }
    }

    fn search(&mut self, candidates: &[(Point, B::Color)], path: &mut Vec<Point>) -> bool {
        if self.is_explored(path) {
            return true;
        }

        //TODO: implement
        true
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
