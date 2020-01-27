use std::iter::once;

use crate::block::{
    base::color::ColorPalette, binary::BinaryColor, multicolor::MultiColor, Block, Color,
    Description, Line,
};
use crate::utils::{self, rc::ReadRc};

type LineColor<T> = <<T as LineSolver>::BlockType as Block>::Color;

pub trait LineSolver {
    type BlockType: Block;

    fn new(desc: ReadRc<Description<Self::BlockType>>, line: ReadRc<Line<LineColor<Self>>>)
        -> Self;
    fn solve(&mut self) -> bool;
    fn get_solution(self) -> Line<LineColor<Self>>;
}

pub fn solve<L, B>(
    desc: ReadRc<Description<B>>,
    line: ReadRc<Line<B::Color>>,
) -> Result<Line<B::Color>, ()>
where
    L: LineSolver<BlockType = B>,
    B: Block,
{
    let mut solver = L::new(desc, line);
    if solver.solve() {
        Ok(solver.get_solution())
    } else {
        Err(())
    }
}

pub trait DynamicColor: Color
where
    Self: Sized,
{
    fn both_colors() -> Option<Self>;

    fn can_be_blank(&self) -> bool;
    fn can_be(&self, color: Self) -> bool;
    fn add_color(&self, color: Self) -> Self;
    fn solved_copy(&self) -> Self;
}

#[derive(Debug)]
pub struct DynamicSolver<B: Block, S = <B as Block>::Color> {
    desc: ReadRc<Description<B>>,
    line: ReadRc<Line<S>>,
    block_sums: Vec<usize>,
    job_size: usize,
    solution_matrix: Vec<Option<bool>>,
    solved_line: Line<S>,
}

impl<B> LineSolver for DynamicSolver<B>
where
    B: Block,
    B::Color: DynamicColor,
{
    type BlockType = B;

    fn new(desc: ReadRc<Description<B>>, line: ReadRc<Line<B::Color>>) -> Self {
        let block_sums = Self::calc_block_sum(&desc);

        let job_size = desc.vec.len() + 1;
        let solution_matrix = vec![None; job_size * line.len()];

        let solved_line = line.iter().map(DynamicColor::solved_copy).collect();

        Self {
            desc,
            line,
            block_sums,
            job_size,
            solution_matrix,
            solved_line,
        }
    }

    fn solve(&mut self) -> bool {
        if !self.try_solve() {
            return false;
        }

        let solved = &mut self.solved_line;

        let both = B::Color::both_colors();
        if let Some(both) = both {
            let init = B::Color::default();
            utils::replace(solved, &both, &init);
        }
        true
    }

    fn get_solution(self) -> Line<B::Color> {
        self.solved_line
    }
}

impl<B> DynamicSolver<B>
where
    B: Block,
    B::Color: DynamicColor,
{
    fn calc_block_sum(desc: &Description<B>) -> Vec<usize> {
        once(0)
            .chain(B::partial_sums(&desc.vec).into_iter().map(|size| size - 1))
            .collect()
    }

    fn try_solve(&mut self) -> bool {
        if self.line.is_empty() {
            return true;
        }

        let (position, block) = (self.line.len() - 1, self.desc.vec.len());
        self.get_sol(position as isize, block)
    }

    fn _get_sol(&self, position: usize, block: usize) -> Option<bool> {
        self.solution_matrix[position * self.job_size + block]
    }

    fn get_sol(&mut self, position: isize, block: usize) -> bool {
        if position < 0 {
            // finished placing the last block, exactly at the beginning of the line.
            return block == 0;
        }

        let position = position as usize;

        let can_be_solved = self._get_sol(position, block);
        can_be_solved.unwrap_or_else(|| {
            let can_be_solved = self.fill_matrix(position, block);
            self.set_sol(position, block, can_be_solved);
            can_be_solved
        })
    }

    fn set_sol(&mut self, position: usize, block: usize, can_be_solved: bool) {
        self.solution_matrix[position * self.job_size + block] = Some(can_be_solved)
    }

    fn color_at(&self, position: usize) -> B::Color {
        self.line[position]
    }

    fn block_at(&self, block_position: usize) -> B {
        self.desc.vec[block_position]
    }

    fn update_solved(&mut self, position: usize, color: B::Color) {
        let current = self.solved_line[position];
        self.solved_line[position] = current.add_color(color);
    }

    fn fill_matrix(&mut self, position: usize, block: usize) -> bool {
        // too many blocks left to fit this line segment
        if position < self.block_sums[block] {
            return false;
        }

        // do not short-circuit
        self.fill_matrix_blank(position, block) | self.fill_matrix_color(position, block)
    }

    fn fill_matrix_blank(&mut self, position: usize, block: usize) -> bool {
        if self.color_at(position).can_be_blank() {
            // current cell is either blank or unknown
            let has_blank = self.get_sol(position as isize - 1, block);
            if has_blank {
                let blank = B::Color::blank();
                // set cell blank and continue
                self.update_solved(position, blank);
                return true;
            }
        }

        false
    }

    fn fill_matrix_color(&mut self, position: usize, block: usize) -> bool {
        // block == 0 means we finished filling all the blocks (can still fill whitespace)
        if block == 0 {
            return false;
        }
        let current_block = self.block_at(block - 1);
        let mut block_size = current_block.size();
        let current_color = current_block.color();
        let should_have_trailing_space = self.trail_with_space(block);
        if should_have_trailing_space {
            block_size += 1;
        }

        let block_start = position as isize - block_size as isize + 1;

        // (position-block_size, position]
        if self.can_place_color(
            block_start,
            position,
            current_color,
            should_have_trailing_space,
        ) {
            let has_color = self.get_sol(block_start - 1, block - 1);
            if has_color {
                // set cell blank, place the current block and continue
                self.set_color_block(
                    block_start,
                    position,
                    current_color,
                    should_have_trailing_space,
                );
                return true;
            }
        }

        false
    }

    fn trail_with_space(&self, block: usize) -> bool {
        if block < self.desc.vec.len() {
            let current_color = self.block_at(block - 1).color();
            let next_color = self.block_at(block).color();

            if next_color == current_color {
                return true;
            }
        }

        false
    }

    fn can_place_color(
        &self,
        start: isize,
        mut end: usize,
        color: B::Color,
        trailing_space: bool,
    ) -> bool {
        if start < 0 {
            return false;
        }

        if trailing_space {
            if !self.color_at(end).can_be_blank() {
                return false;
            }
        } else {
            end += 1;
        }

        // the color can be placed in every cell
        self.line[start as usize..end]
            .iter()
            .all(|cell| cell.can_be(color))
    }

    fn set_color_block(
        &mut self,
        start: isize,
        mut end: usize,
        color: B::Color,
        trailing_space: bool,
    ) {
        if trailing_space {
            let blank = B::Color::blank();
            self.update_solved(end, blank);
        } else {
            end += 1
        }

        // set colored cells
        for i in start as usize..end {
            self.update_solved(i, color);
        }
    }
}

impl DynamicColor for BinaryColor {
    fn both_colors() -> Option<Self> {
        Some(Self::BlackOrWhite)
    }

    fn can_be_blank(&self) -> bool {
        self != &Self::Black
    }

    fn can_be(&self, _always_black: Self) -> bool {
        self != &Self::blank()
    }

    fn add_color(&self, color: Self) -> Self {
        match self {
            Self::Undefined => color,
            &value if value == color => value,
            _ => Self::BlackOrWhite,
        }
    }

    fn solved_copy(&self) -> Self {
        *self
    }
}

impl DynamicColor for MultiColor {
    fn both_colors() -> Option<Self> {
        None
    }

    fn can_be_blank(&self) -> bool {
        (self.0 & ColorPalette::WHITE_ID) == ColorPalette::WHITE_ID
    }

    fn can_be(&self, color: Self) -> bool {
        (self.0 & color.0) != 0
    }

    fn add_color(&self, color: Self) -> Self {
        Self(self.0 | color.0)
    }

    fn solved_copy(&self) -> Self {
        Self(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{
        binary::{
            BinaryBlock,
            BinaryColor::{self, Black, Undefined, White},
        },
        Description,
    };
    use crate::utils::rc::ReadRc;

    use super::{solve, DynamicSolver, LineSolver};

    fn simple_description() -> ReadRc<Description<BinaryBlock>> {
        ReadRc::new(Description::new(vec![BinaryBlock(3)]))
    }

    #[test]
    fn check_empty_line() {
        let l = Vec::<BinaryColor>::new();
        let ds = DynamicSolver::new(simple_description(), ReadRc::new(l));

        assert_eq!(*ds.line, vec![]);
    }

    #[test]
    fn check_no_additional_space() {
        let l = vec![White; 3];
        let ds = DynamicSolver::new(simple_description(), ReadRc::new(l));

        assert_eq!(*ds.line, vec![White, White, White]);
    }

    fn cases() -> Vec<(Vec<usize>, Vec<BinaryColor>, Vec<BinaryColor>)> {
        let (b, w, u) = (Black, White, Undefined);

        vec![
            (vec![], vec![u; 3], vec![w; 3]),
            (vec![1], vec![u], vec![b]),
            (vec![1], vec![u, u], vec![u, u]),
            (vec![2], vec![u, u, u], vec![u, b, u]),
            (vec![2], vec![w, u, u], vec![w, b, b]),
            (
                vec![4, 2],
                vec![u, b, u, u, u, w, u, u],
                vec![u, b, b, b, u, w, b, b],
            ),
            (
                vec![4, 2],
                vec![u, b, u, u, w, u, u, u],
                vec![b, b, b, b, w, u, b, u],
            ),
            // hard cases
            (
                vec![1, 1, 5],
                vec![
                    w, w, w, b, w, w, u, u, u, u, u, u, u, u, u, w, u, u, u, u, u, u, b, u,
                ],
                vec![
                    w, w, w, b, w, w, u, u, u, u, u, u, u, u, u, w, u, u, u, b, b, b, b, u,
                ],
            ),
            (
                vec![9, 1, 1, 1],
                vec![
                    u, u, u, w, w, b, b, b, b, b, b, b, b, b, w, w, w, w, w, w, w, u, u, u, b, w,
                    u, w, u,
                ],
                vec![
                    w, w, w, w, w, b, b, b, b, b, b, b, b, b, w, w, w, w, w, w, w, u, u, w, b, w,
                    u, w, u,
                ],
            ),
            (
                vec![5, 6, 3, 1, 1],
                vec![
                    u, u, u, u, u, u, u, u, u, u, u, u, u, u, u, b, w, u, w, w, w, w, w, u, u, u,
                    u, u, u, b, b, w, u, u, u, u, u, u, w, w, w, u, u, u, b, w,
                ],
                vec![
                    u, u, u, u, u, u, u, u, u, w, u, b, b, b, b, b, w, w, w, w, w, w, w, w, w, u,
                    u, u, b, b, b, w, u, u, u, u, u, u, w, w, w, u, u, w, b, w,
                ],
            ),
            (
                vec![1, 1, 2, 1, 1, 3, 1],
                vec![
                    b, w, w, u, u, w, u, b, u, w, w, b, u, u, u, u, u, b, u, u, u, u,
                ],
                vec![
                    b, w, w, u, u, w, u, b, u, w, w, b, w, u, u, u, u, b, u, u, u, u,
                ],
            ),
        ]
    }

    #[test]
    fn solve_basic() {
        let l = vec![Undefined; 3];
        assert_eq!(
            solve::<DynamicSolver<_>, _>(simple_description(), ReadRc::new(l)).unwrap(),
            vec![Black; 3]
        );
    }

    #[test]
    fn solve_cases() {
        for (desc, line, expected) in cases() {
            let as_blocks: Vec<_> = desc.iter().map(|b| BinaryBlock(*b)).collect();
            let desc = Description::new(as_blocks);

            let original_line = line.clone();

            let mut ds = DynamicSolver::new(ReadRc::new(desc), ReadRc::new(line));
            assert!(ds.solve());
            assert_eq!(*ds.line, original_line);
            assert_eq!(ds.get_solution(), expected);
        }
    }
}

#[cfg(test)]
mod tests_solve_color {
    use crate::block::{
        base::{
            color::{ColorId, ColorPalette},
            Description,
        },
        multicolor::{ColoredBlock, MultiColor},
    };
    use crate::utils::rc::ReadRc;

    use super::{solve, DynamicSolver, LineSolver};

    const fn w() -> ColorId {
        ColorPalette::WHITE_ID
    }

    fn unsolved_line(size: usize) -> Vec<MultiColor> {
        id_to_color_line(&vec![127; size])
    }

    fn id_to_color_line(line: &[ColorId]) -> Vec<MultiColor> {
        line.iter().cloned().map(MultiColor).collect()
    }

    fn desc_from_slice(desc: &[ColoredBlock]) -> ReadRc<Description<ColoredBlock>> {
        ReadRc::new(Description::new(desc.to_vec()))
    }

    fn check_solve(desc: &[ColoredBlock], initial: &[MultiColor], solved: &[ColorId]) {
        let desc = desc_from_slice(desc);
        assert_eq!(
            solve::<DynamicSolver<_>, _>(desc, ReadRc::new(initial.to_vec())).unwrap(),
            id_to_color_line(solved)
        );
    }

    #[test]
    fn empty_1_cell() {
        check_solve(&[], &unsolved_line(1), &[w()]);
    }

    #[test]
    fn empty_3_cells() {
        check_solve(&[], &unsolved_line(3), &[w(); 3])
    }

    #[test]
    fn simplest() {
        check_solve(
            &[ColoredBlock::from_size_and_color(1, 4)],
            &unsolved_line(1),
            &[4],
        );
    }

    #[test]
    fn two_different_cells() {
        check_solve(
            &[
                ColoredBlock::from_size_and_color(1, 4),
                ColoredBlock::from_size_and_color(1, 8),
            ],
            &unsolved_line(2),
            &[4, 8],
        );
    }

    #[test]
    fn undefined() {
        check_solve(
            &[ColoredBlock::from_size_and_color(1, 4)],
            &unsolved_line(2),
            &[4 + w(); 2],
        );
    }

    #[test]
    fn same_color() {
        check_solve(
            &[ColoredBlock::from_size_and_color(1, 4); 2],
            &unsolved_line(3),
            &[4, w(), 4],
        );
    }

    #[test]
    fn different_colors() {
        check_solve(
            &[
                ColoredBlock::from_size_and_color(1, 4),
                ColoredBlock::from_size_and_color(1, 8),
            ],
            &unsolved_line(3),
            &[4 + w(), 4 + 8 + w(), 8 + w()],
        );
    }

    #[test]
    fn long() {
        check_solve(
            &[
                ColoredBlock::from_size_and_color(2, 4),
                ColoredBlock::from_size_and_color(1, 4),
                ColoredBlock::from_size_and_color(1, 8),
            ],
            &unsolved_line(5),
            &[4, 4, w(), 4, 8],
        );
    }

    #[test]
    fn long_undefined() {
        check_solve(
            &[
                ColoredBlock::from_size_and_color(2, 4),
                ColoredBlock::from_size_and_color(1, 4),
                ColoredBlock::from_size_and_color(1, 8),
            ],
            &unsolved_line(6),
            &[4 + w(), 4, 4 + w(), 4 + w(), 4 + 8 + w(), 8 + w()],
        );
    }

    #[test]
    fn first_non_space() {
        let mut line = unsolved_line(3);
        line.insert(0, MultiColor(4));
        check_solve(
            &[
                ColoredBlock::from_size_and_color(2, 4),
                ColoredBlock::from_size_and_color(1, 8),
            ],
            &line,
            &[4, 4, 8 + w(), 8 + w()],
        );
    }

    #[test]
    fn bad() {
        let desc = desc_from_slice(&[
            ColoredBlock::from_size_and_color(2, 4),
            ColoredBlock::from_size_and_color(1, 4),
            ColoredBlock::from_size_and_color(1, 8),
        ]);

        let mut ds = DynamicSolver::new(desc, ReadRc::new(unsolved_line(4)));
        assert_eq!(ds.solve(), false);
    }
}
