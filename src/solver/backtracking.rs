use super::super::board::{Block, Board, Color, Point};
use super::line::LineSolver;
use super::probing::{Impact, ProbeSolver};

use std::cell::{Ref, RefCell};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Instant;

use ordered_float::OrderedFloat;

use rulinalg::matrix::Matrix;

type Solution<B> = Matrix<<B as Block>::Color>;

pub struct Solver<B, P, S>
where
    B: Block,
    P: ProbeSolver<BlockType = B>,
    S: LineSolver<BlockType = B>,
{
    board: Rc<RefCell<Board<B>>>,
    probe_solver: P,

    // search options
    max_solutions: Option<usize>,
    timeout: Option<u32>,
    max_depth: Option<usize>,

    // dynamic variables
    pub solutions: Vec<Solution<B>>,
    depth_reached: usize,
    start_time: Option<Instant>,
    explored_paths: HashSet<Vec<(Point, B::Color)>>,
    pub search_tree: SearchTreeRef<(Point, B::Color), f64>,

    _phantom: PhantomData<S>,
}

#[allow(dead_code)]
enum ChoosePixel {
    Sum,
    Min,
    Max,
    Mul,
    Sqrt,
    MinLogm,
    MinLogd,
}

type SearchTreeRef<K, V> = Rc<RefCell<SearchTree<K, V>>>;

pub struct SearchTree<K, V> {
    value: Option<V>,
    children: Vec<(K, SearchTreeRef<K, V>)>,
}

impl<K, V> SearchTree<K, V>
where
    K: PartialEq + Clone,
    V: Clone,
{
    fn new() -> Self {
        Self::with_option(None)
    }

    #[allow(dead_code)]
    fn with_value(value: V) -> Self {
        Self::with_option(Some(value))
    }

    fn with_option(value: Option<V>) -> Self {
        Self {
            value,
            children: vec![],
        }
    }

    fn new_children(&mut self, key: K, value: Option<V>) {
        self.children
            .push((key, Rc::new(RefCell::new(Self::with_option(value)))));
    }

    fn get(&self, key: &K) -> Option<Rc<RefCell<Self>>> {
        self.children.iter().find_map(|(child_key, child)| {
            if child_key == key {
                Some(Rc::clone(child))
            } else {
                None
            }
        })
    }

    fn add(this: Rc<RefCell<Self>>, path: &[K], value: Option<V>) {
        if path.is_empty() && this.borrow_mut().value.is_none() {
            this.borrow_mut().value = value;
            return;
        }

        let mut current = this;
        for (i, node) in path.iter().enumerate() {
            let child = current.borrow().get(node);
            if child.is_none() {
                let child_value = if i == path.len() - 1 {
                    value.clone()
                } else {
                    None
                };
                current.borrow_mut().new_children(node.clone(), child_value);
            }

            let child = current.borrow().get(node).unwrap();
            current = child;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.children.is_empty()
    }
}

impl<K, V> SearchTree<K, V>
where
    K: fmt::Debug,
    V: Clone + fmt::Display,
{
    fn format(&self, spaces: usize, indent_size: usize) -> String {
        if self.children.is_empty() {
            self.format_value()
        } else {
            self.format_with_children(spaces, indent_size)
        }
    }

    fn format_value(&self) -> String {
        if let Some(value) = self.value.clone() {
            format!("{:.6}", value)
        } else {
            "None".to_string()
        }
    }

    fn format_with_children(&self, spaces: usize, indent_size: usize) -> String {
        let mut res = vec![
            format!("{}{}", indent_space(spaces * indent_size), '{'),
            format!(
                "{}{:?}: {},",
                indent_space((spaces + 1) * indent_size),
                "value",
                self.format_value()
            ),
            format!(
                "{}{:?}: {}",
                indent_space((spaces + 1) * indent_size),
                "children",
                '{'
            ),
        ];

        let last_index = self.children.len() - 1;

        for (i, (child_key, child)) in self.children.iter().enumerate() {
            let child_format = child.borrow().format(spaces + 2, indent_size);
            let trail_comma = if i == last_index { "" } else { "," };
            res.push(format!(
                "{}{:?}: {}{}",
                indent_space((spaces + 2) * indent_size),
                child_key,
                child_format.trim_start(),
                trail_comma,
            ));
        }

        res.extend_from_slice(&[
            format!("{}{}", indent_space((spaces + 1) * indent_size), '}'),
            format!("{}{}", indent_space(spaces * indent_size), '}'),
        ]);
        res.join("\n")
    }
}

fn indent_space(size: usize) -> String {
    //String::from_utf8(vec![b' '; size]).unwrap()
    (0..size).map(|_| " ").collect()
}

impl<K, V> fmt::Display for SearchTree<K, V>
where
    K: fmt::Debug,
    V: Clone + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format(0, 2))
    }
}

impl<B, P, S> Solver<B, P, S>
where
    B: Block,
    P: ProbeSolver<BlockType = B>,
    S: LineSolver<BlockType = B>,
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
        let probe_solver = P::with_board(Rc::clone(&board));
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
            search_tree: Rc::new(RefCell::new(SearchTree::new())),
            _phantom: PhantomData,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
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
        self.search(&directions, &[])?;

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

        Ok(())
    }

    fn board(&self) -> Ref<Board<B>> {
        self.board.borrow()
    }

    fn is_solved(&self) -> bool {
        self.board().is_solved_full()
    }

    fn set_explored(&mut self, path: &[(Point, B::Color)]) {
        let mut path = path.to_vec();
        path.sort();
        self.explored_paths.insert(path);
    }

    fn is_explored(&self, path: &[(Point, B::Color)]) -> bool {
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

    fn add_solution(&mut self) -> Result<(), String> {
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
        points_rate.sort_by_key(|&(point, rate)| (Reverse(rate), point));
        //dbg!(&points_rate[..10]);

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

    const CHOOSE_STRATEGY: ChoosePixel = ChoosePixel::Sqrt;

    fn rate_by_impact(impact: Vec<&(usize, f64)>) -> f64 {
        let sizes_only: Vec<_> = impact
            .iter()
            .map(|(new_points, _priority)| *new_points)
            .collect();

        let min = sizes_only.iter().min().unwrap_or(&0);
        let max = sizes_only.iter().max().unwrap_or(&0);
        let sum = sizes_only.iter().sum::<usize>();

        let log = |f: f64| (1.0 + f).ln() + 1.0;

        // Max is the most trivial, but also most ineffective strategy.
        // For details, see https://ieeexplore.ieee.org/document/6476646
        match Self::CHOOSE_STRATEGY {
            ChoosePixel::Sum => sum as f64,
            ChoosePixel::Min => *min as f64,
            ChoosePixel::Max => *max as f64,
            ChoosePixel::Mul => sizes_only.iter().map(|x| (x + 1) as f64).product(),
            ChoosePixel::Sqrt => (*max as f64 / (min + 1) as f64).sqrt() + (*min as f64),
            ChoosePixel::MinLogm => {
                let logm: f64 = sizes_only.iter().map(|&x| log(x as f64)).product();
                logm + (*min as f64)
            }
            ChoosePixel::MinLogd => match sizes_only.as_slice() {
                [first, second] => {
                    let diff = log(*first as f64) - log(*second as f64);
                    *min as f64 + diff.abs()
                }
                _other => *min as f64,
            },
        }
    }

    /// Recursively search for solutions.
    /// Return False if the given path is a dead end (no solutions can be found)
    fn search(
        &mut self,
        directions: &[(Point, B::Color)],
        path: &[(Point, B::Color)],
    ) -> Result<bool, String> {
        if self.is_explored(path) {
            return Ok(true);
        }

        let depth = path.len();
        if self.limits_reached(depth) {
            return Ok(true);
        }

        let save = self.board().make_snapshot();
        let result = self.search_mutable(directions, path);

        // do not restore the solved cells on a root path - they are really solved!
        if !path.is_empty() {
            self.board.borrow_mut().restore(save);
            self.set_explored(path);
        }

        result
    }

    fn search_mutable(
        &mut self,
        directions: &[(Point, B::Color)],
        path: &[(Point, B::Color)],
    ) -> Result<bool, String> {
        let depth = path.len();
        // going to dive deeper, so increment it (full_path's length)
        self.depth_reached = self.depth_reached.max(depth + 1);

        // this variable shows whether the board changed after the last probing
        // when the probing occurs it should immediately set to 'false'
        // to prevent succeeded useless probing on the same board
        let mut board_changed = true;
        let mut search_counter = 0u32;

        let mut directions = directions.to_vec();

        // push and pop from the end, so the most prioritized items are on the left
        directions.reverse();

        while let Some(direction) = directions.pop() {
            let total_number_of_directions = directions.len() + 1;
            search_counter += 1;

            if self.limits_reached(depth) {
                return Ok(true);
            }

            if path.contains(&direction) {
                continue;
            }

            let (point, color) = direction;
            let cell_colors: HashSet<B::Color> = self.board().cell(&point).variants();

            if !cell_colors.contains(&color) {
                warn!(
                    "The color {:?} is already expired. Possible colors for {:?} are {:?}",
                    color, point, cell_colors
                );
                continue;
            }

            if cell_colors.len() == 1 {
                warn!(
                    "Only one color for {:?} left: {:?}. Solve it unconditionally",
                    point, color
                );
                assert!(cell_colors.contains(&color));
                if !board_changed {
                    warn!("The board does not change since the last unconditional solving, skip.");
                    continue;
                }

                let impact = self.probe_solver.run_unsolved::<S>();
                board_changed = false;

                if impact.is_err() {
                    // the whole `path` branch of a search tree is a dead end
                    warn!("The last possible color {:?} for the {:?} lead to the contradiction. The path {:?} is invalid", color, point, path);
                    // self._add_search_result(path, False)
                    return Ok(false);
                }

                // rate = board.solution_rate
                // self._add_search_result(path, rate)
                if self.board().is_solved_full() {
                    self.add_solution()?;
                    warn!("The only color {:?} for the {:?} lead to full solution. No need to traverse the path {:?} anymore", color, point, path);
                    return Ok(true);
                }
                continue;
            }

            let mut full_path = path.to_vec();
            full_path.push(direction);

            if self.is_explored(&full_path) {
                info!("The path {:?} already explored", &full_path);
                continue;
            }

            let rate = self.board().solution_rate();
            let guess_save = self.board().make_snapshot();

            warn!(
                "Trying direction ({}/{}): {:?} (depth={}, rate={:.4}, previous={:?})",
                search_counter, total_number_of_directions, &direction, depth, rate, path
            );
            self.add_search_score(path, rate);

            let state_result = self.try_direction(&full_path);
            //let is_solved = board.is_solved_full();
            self.board.borrow_mut().restore(guess_save);
            self.set_explored(&full_path);

            if state_result.is_err() {
                return state_result;
            }

            let success = state_result.unwrap();

            if !success {
                // TODO: add backjumping here
                warn!(
                    "Unset the color {:?} for {:?}. Solve it unconditionally",
                    color, point
                );

                let err = self.board.borrow_mut().unset_color(&point, &color);
                board_changed = true;
                if err.is_err() {
                    // the whole `path` branch of a search tree is a dead end
                    warn!(
                        "The last possible color {:?} for the {:?} cannot be unset. The whole branch (depth={}) is invalid.",
                        color, point, depth);
                    // self._add_search_result(path, False)
                    return Ok(false);
                }

                if !board_changed {
                    warn!("The board does not change since the last unconditional solving, skip.");
                    continue;
                }

                let err = self.probe_solver.run_unsolved::<S>();
                board_changed = false;
                if err.is_err() {
                    // the whole `path` branch of a search tree is a dead end
                    warn!(
                        "The last possible color {:?} for the {:?} lead to the contradiction. The whole branch (depth={}) is invalid.",
                        color, point, depth);
                    // self._add_search_result(path, False)
                    return Ok(false);
                }

                // rate = board.solution_rate
                // self._add_search_result(path, rate)
                if self.board().is_solved_full() {
                    self.add_solution()?;
                    warn!(
                        "The negation of color {:?} for the {:?} lead to full solution. No need to traverse the path {:?} anymore.",
                          color, point, path);
                    return Ok(true);
                }
            }

            if !success || self.board().is_solved_full() {
                // immediately try the other colors as well
                // if all of them goes to the dead end,
                // then the parent path is a dead end

                let states_to_try: Vec<_> = cell_colors
                    .iter()
                    .filter_map(|&other_color| {
                        if other_color == color {
                            None
                        } else {
                            Some((point, other_color))
                        }
                    })
                    .collect();

                // if all(self.is_explored(path + (direction,)) for direction in states_to_try) {
                //     warn!("All other colors ({:?}) of {:?} already explored",
                //           states_to_try, cell)
                //     return true;
                // }

                for direction in states_to_try {
                    if !directions.contains(&direction) {
                        directions.push(direction);
                    }
                }
            }
        }
        Ok(true)
    }

    fn add_search_score(&mut self, path: &[(Point, B::Color)], score: f64) {
        SearchTree::add(Rc::clone(&self.search_tree), path, Some(score));
    }

    fn add_search_deadend(&mut self, path: &[(Point, B::Color)]) {
        SearchTree::add(Rc::clone(&self.search_tree), path, None);
    }

    /// Trying to search for solutions in the given direction.
    /// At first it set the given state and get a list of the
    /// further jobs for finding the contradictions.
    /// Later that jobs will be used as candidates for a deeper search.
    fn try_direction(&mut self, path: &[(Point, B::Color)]) -> Result<bool, String> {
        let depth = path.len();
        let direction = *path.last().unwrap();

        // add every cell to the jobs queue
        let mut probe_jobs = self.probe_solver.unsolved_cells();
        let new_jobs = self.set_guess(direction);
        match new_jobs {
            // update with more prioritized cells
            Ok(new_jobs) => {
                for (new_job, priority) in new_jobs {
                    probe_jobs.push(new_job, priority);
                }
            }
            Err(err) => {
                warn!("Guess {:?} failed: {}", direction, err);
                self.add_search_deadend(path);
                return Ok(false);
            }
        }

        if self.limits_reached(depth) {
            return Ok(true);
        }

        let impact = self.probe_solver.run::<S>(&mut probe_jobs);

        match impact {
            Ok(impact) => {
                let rate = self.board().solution_rate();
                info!("Reached rate {:.4} on {:?} path", rate, path);
                self.add_search_score(path, rate);

                if self.limits_reached(depth) || self.board().is_solved_full() {
                    return Ok(true);
                }

                //let cells_left = round((1 - rate) * board.width * board.height);
                //LOG.info('Unsolved cells left: %d', cells_left)

                let directions = self.choose_directions(&impact);
                if directions.is_empty() {
                    Ok(true)
                } else {
                    self.search(&directions, path)
                }
            }
            Err(err) => {
                warn!("Guess {:?} failed on probing stage: {}", direction, err);
                self.add_search_deadend(path);
                Ok(false)
            }
        }
    }

    fn set_guess(
        &mut self,
        guess: (Point, B::Color),
    ) -> Result<Vec<(Point, OrderedFloat<f64>)>, String> {
        let (point, color) = guess;

        if !self.board().cell(&point).variants().contains(&color) {
            warn!("The probe is useless: color {:?} already unset", color);
            return Ok(vec![]);
        }

        //let save = self.board().make_snapshot();

        let mut probes = vec![];
        self.board.borrow_mut().set_color(&point, &color);
        for (new_point, priority) in self.probe_solver.propagate_point::<S>(&point)? {
            probes.push((new_point, priority));
        }
        if self.board().is_solved_full() {
            self.add_solution()?;
            return Ok(vec![]);
        }

        Ok(probes)
    }

    /// Whether we reached the defined limits:
    /// 1) number of solutions found
    /// 2) the maximum allowed run time
    /// 3) the maximum depth
    fn limits_reached(&self, depth: usize) -> bool {
        if let Some(max_solutions) = self.max_solutions {
            let solutions_number = self.solutions.len();
            if solutions_number >= max_solutions {
                if depth == 0 {
                    // only show log on the most top level
                    warn!("{} solutions is enough", solutions_number);
                }
                return true;
            }
        }

        if let Some(timeout) = self.timeout {
            if let Some(start_time) = self.start_time {
                let run_time = start_time.elapsed();
                if run_time.as_secs() >= timeout.into() {
                    if depth == 0 {
                        // only show log on the most top level
                        warn!("Searched too long: {:.4}s", run_time.as_secs());
                    }
                    return true;
                }
            }
        }

        if let Some(max_depth) = self.max_depth {
            if depth > max_depth {
                if depth == 0 {
                    warn!(
                        "Next step on the depth {} is deeper than the max ({})",
                        depth, max_depth
                    );
                }
                return true;
            }
        }

        false
    }
}
