//! The algorithm based on the ideas
//! from the [article](https://habr.com/ru/post/433330/)
use std::{
    iter::{from_fn, once},
    marker::PhantomData,
    ops::Range,
};

use varisat::{solver::Solver, CnfFormula, ExtendFormula, Lit, Var};

use crate::{
    block::{Block, Color, Description},
    board::Point,
    solver::probing::Impact,
    utils::{is_touching, pair_combinations, product, rc::ReadRc},
};

trait BlockPosition {
    fn block_starts(&self) -> Vec<usize>;
    fn min_space(&self) -> usize;
    fn positions_number(&self, line_length: usize) -> usize {
        let min_space = self.min_space();
        assert!(line_length >= min_space);
        line_length - min_space + 1
    }
}

impl<B> BlockPosition for Description<B>
where
    B: Block,
{
    fn block_starts(&self) -> Vec<usize> {
        self.vec
            .iter()
            .zip(Block::partial_sums(&self.vec))
            .map(|(block, end)| end - block.size())
            .collect()
    }

    fn min_space(&self) -> usize {
        if self.vec.is_empty() {
            return 0;
        }
        *Block::partial_sums(&self.vec)
            .last()
            .expect("Partial sums should be non-empty")
    }
}

#[derive(Debug, Clone)]
struct Position {
    var: Var,
    range: Range<usize>,
}

#[derive(Debug, Clone)]
struct BlockPositions {
    index: usize,
    vec: Vec<Position>,
}

#[derive(Debug)]
struct LinePositions(Vec<BlockPositions>);

#[derive(Debug)]
pub struct ClauseGenerator<B>
where
    B: Block,
{
    columns_vars: Vec<LinePositions>,
    rows_vars: Vec<LinePositions>,
    cell_vars: Vec<Vec<Var>>,
    cells: Vec<B::Color>,
    width: usize,
    height: usize,
    formula: CnfFormula,
    _phantom: PhantomData<B>,
}

impl<B> ClauseGenerator<B>
where
    B: Block,
{
    pub fn with_clues(
        columns: &[ReadRc<Description<B>>],
        rows: &[ReadRc<Description<B>>],
        cells: Vec<B::Color>,
    ) -> Self {
        let width = columns.len();
        let height = rows.len();
        let mut formula = CnfFormula::new();

        let columns_vars = Self::clues_vars(columns, height, &mut formula);
        let rows_vars = Self::clues_vars(rows, width, &mut formula);
        let clues_vars = formula.var_count();

        let cell_vars = (0..height)
            .map(|_| formula.new_var_iter(width).collect())
            .collect();

        let vars_total = formula.var_count();
        warn!(
            "Vars: {} (clues: {}, cells: {})",
            vars_total,
            clues_vars,
            vars_total - clues_vars
        );
        Self {
            columns_vars,
            rows_vars,
            cell_vars,
            cells,
            width,
            height,
            formula,
            _phantom: PhantomData,
        }
    }

    fn clues_vars(
        clues: &[ReadRc<Description<B>>],
        line_length: usize,
        formula: &mut CnfFormula,
    ) -> Vec<LinePositions> {
        clues
            .iter()
            .map(|clue| {
                let positions = clue.positions_number(line_length);
                LinePositions(
                    clue.block_starts()
                        .iter()
                        .zip(&clue.vec)
                        .enumerate()
                        .map(|(index, (&start, block))| BlockPositions {
                            index,
                            vec: formula
                                .new_var_iter(positions)
                                .zip(start..)
                                .map(|(var, start)| Position {
                                    var,
                                    range: start..start + block.size(),
                                })
                                .collect(),
                        })
                        .collect(),
                )
            })
            .collect()
    }

    fn block_positions_clause(positions: &BlockPositions) -> Vec<Lit> {
        // 1. Каждый блок, объявленный в строке или столбце обязан появиться хотя-бы в одной позиции.
        // Этому соответствует клоз вида (X1 V X2 V… XN),
        // где X1, X2… XN — все возможные позиции данного блока в строке или столбце.
        positions.vec.iter().map(|pos| pos.var.positive()).collect()
    }

    fn block_once_clauses(positions: &BlockPositions) -> impl Iterator<Item = Vec<Lit>> {
        // 2. Каждый блок в строке или столбце должен появиться не более одного раза.
        // Этому соответствует множество клозов вида (not Xi) V (not Xj),
        // где Xi, Xj (i != j) — все возможные позиции данного блока в строке или столбце.
        let pairs = pair_combinations(&positions.vec);
        pairs
            .into_iter()
            .map(|(f, s)| vec![f.var.negative(), s.var.negative()])
    }

    fn non_overlap_clauses(positions: &LinePositions) -> impl Iterator<Item = Vec<Lit>> {
        // 3. Правильный порядок блоков. Поскольку необходимо поддерживать правильный порядок расположения блоков,
        // а также исключить их пересечение, необходимо добавить клозы, вида (not Xi) V (not Xj),
        // где Xi, Xj — переменные, соответствующие позициям разных блоков, которые имеют неправильный порядок или пересекаются.
        let pairs = pair_combinations(&positions.0);
        pairs.into_iter().flat_map(|(block1, block2)| {
            let pairs = product(&block1.vec, &block2.vec);
            pairs.into_iter().filter_map(
                move |(
                    Position {
                        range: range1,
                        var: var1,
                    },
                    Position {
                        range: range2,
                        var: var2,
                    },
                )| {
                    if is_touching(range1.clone(), range2.clone())
                        || ((block1.index < block2.index) && (range1.start > range2.start))
                    {
                        Some(vec![var1.negative(), var2.negative()])
                    } else {
                        None
                    }
                },
            )
        })
    }

    fn covering_positions(&self, cell_point: Point) -> (Vec<Var>, Vec<Var>) {
        let (x, y) = (cell_point.x(), cell_point.y());
        let column = &self.columns_vars[x];
        let row = &self.rows_vars[y];

        let column_vars = column
            .0
            .iter()
            .flat_map(move |block_pos| {
                block_pos.vec.iter().filter_map(move |pos| {
                    if pos.range.contains(&y) {
                        Some(pos.var)
                    } else {
                        None
                    }
                })
            })
            .collect();
        let row_vars = row
            .0
            .iter()
            .flat_map(move |block_pos| {
                block_pos.vec.iter().filter_map(move |pos| {
                    if pos.range.contains(&x) {
                        Some(pos.var)
                    } else {
                        None
                    }
                })
            })
            .collect();

        (column_vars, row_vars)
    }

    fn cell_box_clauses(&self, cell_point: Point) -> (Vec<Lit>, Vec<Lit>) {
        // 4. Окрашенная клетка должна содержаться внутри хотя бы одного блока, позиция которого включает данную клетку.
        // Этому соответствует клоз вида ((not Yk) V X1 V X2… XN), где Yk — переменная, соответствующая клетке,
        // а X1, X2… XN — переменные, соответствующие позициям блоков, содержащих данную клетку.
        let (column_vars, row_vars) = self.covering_positions(cell_point);
        let point_var = self.cell_vars[cell_point.y()][cell_point.x()];
        let column_clause = column_vars
            .into_iter()
            .map(|var| var.positive())
            .chain(once(point_var.negative()))
            .collect();

        let row_clause = row_vars
            .into_iter()
            .map(|var| var.positive())
            .chain(once(point_var.negative()))
            .collect();

        (column_clause, row_clause)
    }

    fn cell_space_clauses(&self, cell_point: Point) -> impl Iterator<Item = Vec<Lit>> + '_ {
        // 5. Каждая пустая клетка не должна содержаться ни в одной возможной позиции ни одного блока.
        // Этому соответствует множество клозов вида Yi V (not Xj), где Yi — переменная, соответствующая клетке,
        // а Xj — переменная, соответствующая одной позиции какого-либо блока, содержащая данную клетку.
        let (column_vars, row_vars) = self.covering_positions(cell_point);
        let point_var = self.cell_vars[cell_point.y()][cell_point.x()];
        column_vars
            .into_iter()
            .chain(row_vars)
            .map(move |var| vec![var.negative(), point_var.positive()])
    }

    fn precomputed_cells_clauses(&self) -> Vec<Lit> {
        if self.cells.is_empty() {
            return Vec::new();
        }

        self.cell_vars
            .concat()
            .iter()
            .zip(&self.cells)
            .filter_map(|(var, &cell)| {
                if cell.is_solved() {
                    let is_color = cell != B::Color::blank();
                    Some(var.lit(is_color))
                } else {
                    None
                }
            })
            .collect()
    }

    fn clauses(&self) -> impl Iterator<Item = Vec<Lit>> + '_ {
        let columns_positions = self
            .columns_vars
            .iter()
            .flat_map(|line_positions| line_positions.0.iter().map(Self::block_positions_clause));
        let rows_positions = self
            .rows_vars
            .iter()
            .flat_map(|line_positions| line_positions.0.iter().map(Self::block_positions_clause));

        let columns_once_positions = self
            .columns_vars
            .iter()
            .flat_map(|line_positions| line_positions.0.iter().flat_map(Self::block_once_clauses));
        let rows_once_positions = self
            .rows_vars
            .iter()
            .flat_map(|line_positions| line_positions.0.iter().flat_map(Self::block_once_clauses));

        let non_overlap_columns = self.columns_vars.iter().flat_map(Self::non_overlap_clauses);
        let non_overlap_rows = self.rows_vars.iter().flat_map(Self::non_overlap_clauses);

        let all_points = product(
            &(0..self.width).collect::<Vec<_>>(),
            &(0..self.height).collect::<Vec<_>>(),
        );
        let box_clauses = all_points.clone().into_iter().flat_map(move |(x, y)| {
            let box_clauses = self.cell_box_clauses(Point::new(x, y));
            vec![box_clauses.0, box_clauses.1]
        });
        let space_clauses = all_points
            .into_iter()
            .flat_map(move |(x, y)| self.cell_space_clauses(Point::new(x, y)));

        let fixed_points = self
            .precomputed_cells_clauses()
            .into_iter()
            .map(|lit| vec![lit]);

        columns_positions
            .chain(rows_positions)
            .chain(columns_once_positions)
            .chain(rows_once_positions)
            .chain(non_overlap_columns)
            .chain(non_overlap_rows)
            .chain(box_clauses)
            .chain(space_clauses)
            .chain(fixed_points)
    }

    fn get_formula(&self) -> CnfFormula {
        debug!("{:#?}", self);

        let mut formula = CnfFormula::new();
        for clause in self.clauses() {
            formula.add_clause(&clause);
        }
        formula
    }

    fn generate_implications(
        &self,
        probe: Point,
        probe_color: B::Color,
        impact: Vec<(Point, B::Color)>,
    ) -> impl Iterator<Item = (Vec<Lit>)> + '_ {
        // (X -> Y) == (-X \/ Y)
        let probe_var = self.cell_vars[probe.y()][probe.x()];
        impact.into_iter().map(move |(point, color)| {
            let var = self.cell_vars[point.y()][point.x()];
            let is_white_probe = probe_color == B::Color::blank();
            let is_white_impact = color == B::Color::blank();
            vec![probe_var.lit(is_white_probe), var.lit(!is_white_impact)]
        })
    }

    pub fn run(
        &self,
        probing_impact: Impact<B>,
        solutions_number: Option<usize>,
    ) -> impl Iterator<Item = Vec<bool>> {
        let mut formula = self.get_formula();
        let total_impact_clauses: usize = probing_impact
            .into_iter()
            .map(|impact| {
                let (probe, probe_color, impact, _) = impact.into_tuple();
                self.generate_implications(probe, probe_color, impact)
                    .map(|clause| {
                        formula.add_clause(&clause);
                    })
                    .count()
            })
            .sum();

        warn!("Add {} impact clauses", total_impact_clauses);
        warn!("Total clauses: {}", formula.len());
        for (i, clause) in formula.iter().enumerate() {
            info!("{}. {:?}", i, clause);
        }

        let area = self.width * self.height;

        let mut solver = Solver::new();
        solver.add_formula(&formula);

        let mut found = 0;
        from_fn(move || {
            if let Some(solutions_number) = solutions_number {
                if found >= solutions_number {
                    return None;
                }
            }

            let _ = solver.solve().unwrap();
            //assert_eq!(solution, true); // satisfiable
            solver.model().map(|model| {
                found += 1;
                let block_vars = model.len() - area;

                let cells = &model[block_vars..];
                let res = cells.iter().map(|cell| cell.is_positive()).collect();

                let fixed_clause: Vec<_> = cells.iter().map(|&lit| !lit).collect();
                solver.add_clause(&fixed_clause);
                res
            })
        })
    }
}
