use super::super::board::BinaryColor;
use super::super::board::{Block, Color, Description};
use super::super::utils::replace;
use std::fmt::Debug;
use std::rc::Rc;

pub trait LineSolver {
    type BlockType: Block;

    fn new(
        desc: Rc<Description<Self::BlockType>>,
        line: Rc<Vec<<<Self as LineSolver>::BlockType as Block>::Color>>,
    ) -> Self;
    fn solve(&mut self) -> Result<&Vec<<<Self as LineSolver>::BlockType as Block>::Color>, String>;
}

pub trait DynamicColor: Color
where
    Self: Sized,
{
    // it can be implemented very simple with generics specialization
    // https://github.com/aturon/rfcs/blob/impl-specialization/text/0000-impl-specialization.md
    // https://github.com/rust-lang/rfcs/issues/1053
    fn set_additional_blank(line: Rc<Vec<Self>>) -> (Rc<Vec<Self>>, bool);
    fn both_colors() -> Option<Self>;

    fn can_be_blank(&self) -> bool;
    fn can_be(&self, color: &Self) -> bool;
    fn add_color(&self, color: Self) -> Self;
}

pub struct DynamicSolver<B: Block, S = <B as Block>::Color> {
    pub desc: Rc<Description<B>>,
    pub line: Rc<Vec<S>>,
    additional_space: bool,
    block_sums: Vec<usize>,
    solution_matrix: Vec<Vec<Option<bool>>>,
    solved_line: Vec<S>,
}

impl<B> LineSolver for DynamicSolver<B>
where
    B: Block,
    B::Color: Clone + PartialEq + DynamicColor + Debug,
{
    type BlockType = B;

    fn new(desc: Rc<Description<B>>, line: Rc<Vec<B::Color>>) -> Self {
        let (line, additional_space) = B::Color::set_additional_blank(line);

        let block_sums = Self::calc_block_sum(&*desc);
        let solution_matrix = Self::build_solution_matrix(&*desc, &*line);
        let solved_line = line.to_vec();

        Self {
            desc,
            line,
            additional_space,
            block_sums,
            solution_matrix,
            solved_line,
        }
    }

    fn solve(&mut self) -> Result<&Vec<B::Color>, String> {
        if self.try_solve() {
            let mut solved = &mut self.solved_line;
            if self.additional_space {
                solved.truncate(solved.len() - 1);
            }

            let both = B::Color::both_colors();
            if both.is_some() {
                let both = both.unwrap();
                let init = B::Color::initial();

                replace(&mut solved, both, init);
            }
            Ok(solved)
        } else {
            Err("Bad line".to_string())
        }
    }
}

impl<B> DynamicSolver<B>
where
    B: Block,
    B::Color: DynamicColor + PartialEq + Clone,
{
    fn calc_block_sum(desc: &Description<B>) -> Vec<usize> {
        let mut min_indexes: Vec<usize> = B::partial_sums(&desc.vec)
            .iter()
            .map(|size| size - 1)
            .collect();
        min_indexes.insert(0, 0);
        min_indexes
    }

    fn build_solution_matrix(desc: &Description<B>, line: &[B::Color]) -> Vec<Vec<Option<bool>>> {
        let positions = line.len();
        let job_size = desc.vec.len() + 1;
        vec![vec![None; positions]; job_size]
    }

    fn try_solve(&mut self) -> bool {
        if self.line.is_empty() {
            return true;
        }

        let (position, block) = (self.line.len() - 1, self.desc.vec.len());
        self.get_sol(position as isize, block)
    }

    fn _get_sol(&self, position: usize, block: usize) -> Option<bool> {
        self.solution_matrix[block][position]
    }

    fn get_sol(&mut self, position: isize, block: usize) -> bool {
        if position < 0 {
            // finished placing the last block, exactly at the beginning of the line.
            return block == 0;
        }

        let position = position as usize;

        let can_be_solved = self._get_sol(position, block);
        if can_be_solved.is_none() {
            let can_be_solved = self.fill_matrix(position, block);
            self.set_sol(position, block, can_be_solved);
            can_be_solved
        } else {
            can_be_solved.unwrap()
        }
    }

    fn set_sol(&mut self, position: usize, block: usize, can_be_solved: bool) {
        self.solution_matrix[block][position] = Some(can_be_solved)
    }

    fn color_at(&self, position: usize) -> &B::Color {
        &self.line[position]
    }

    fn block_at(&self, block_position: usize) -> &B {
        &self.desc.vec[block_position]
    }

    fn set_solved(&mut self, position: usize, color: B::Color) {
        self.solved_line[position] = color.clone();
    }

    fn update_solved(&mut self, position: usize, color: &B::Color) {
        let current = &self.solved_line[position];
        self.set_solved(position, current.add_color(color.clone()))
    }

    fn fill_matrix(&mut self, position: usize, block: usize) -> bool {
        // too many blocks left to fit this line segment
        if position < self.block_sums[block] {
            return false;
        }

        let blank = B::Color::blank();

        let mut has_blank = false;
        if self.color_at(position).can_be_blank() {
            // current cell is either blank or unknown
            has_blank = self.get_sol(position as isize - 1, block);
            if has_blank {
                // set cell blank and continue
                self.update_solved(position, &blank);
            }
        }

        let mut has_color = false;
        // block == 0 means we finished filling all the blocks (can still fill whitespace)
        if block > 0 {
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
                &current_color,
                should_have_trailing_space,
            ) {
                has_color = self.get_sol(block_start - 1, block - 1);
                if has_color {
                    // set cell blank, place the current block and continue
                    self.set_color_block(
                        block_start,
                        position,
                        &current_color,
                        should_have_trailing_space,
                    );
                }
            }
        }

        has_blank || has_color
    }

    fn trail_with_space(&self, block: usize) -> bool {
        if block < self.desc.vec.len() {
            let current_color = self.block_at(block - 1).color();
            let next_color = self.block_at(block - 1).color();

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
        color: &B::Color,
        trailing_space: bool,
    ) -> bool {
        if start < 0 {
            return false;
        }

        let start = start as usize;

        if trailing_space {
            if !self.color_at(end).can_be_blank() {
                return false;
            }
        } else {
            end += 1;
        }

        // the color can be placed in every cell
        self.line[start..end].iter().all(|cell| cell.can_be(&color))
    }

    fn set_color_block(
        &mut self,
        start: isize,
        mut end: usize,
        color: &B::Color,
        trailing_space: bool,
    ) {
        if trailing_space {
            let blank = B::Color::blank();
            self.update_solved(end, &blank);
        } else {
            end += 1
        }

        // set colored cells
        for i in start as usize..end {
            self.update_solved(i, &color);
        }
    }
}

impl DynamicColor for BinaryColor {
    fn set_additional_blank(line: Rc<Vec<Self>>) -> (Rc<Vec<Self>>, bool) {
        //let space = BinaryColor::White;
        //
        //if line.last().unwrap_or(&space) != &space {
        //    let mut with_space = line.to_vec();
        //    with_space.push(space);
        //    return (Rc::new(with_space), true);
        (line, false)
    }

    fn both_colors() -> Option<Self> {
        Some(BinaryColor::BlackOrWhite)
    }

    fn can_be_blank(&self) -> bool {
        self != &BinaryColor::Black
    }

    fn can_be(&self, _always_black: &Self) -> bool {
        self != &Self::blank()
    }

    fn add_color(&self, color: Self) -> Self {
        match self {
            BinaryColor::Undefined => color,
            &value => {
                if value == color {
                    value
                } else {
                    BinaryColor::BlackOrWhite
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::board::BinaryColor::{Black, Undefined, White};
    use super::super::super::board::{BinaryBlock, BinaryColor, Description};
    use super::{DynamicSolver, LineSolver};
    use std::rc::Rc;

    fn simple_description() -> Rc<Description<BinaryBlock>> {
        Rc::new(Description::new(vec![BinaryBlock(3)]))
    }

    #[test]
    fn check_empty_line() {
        let l = Vec::<BinaryColor>::new();
        let ds = DynamicSolver::new(simple_description(), Rc::new(l));

        assert_eq!(*ds.line, vec![]);
    }

    #[test]
    fn check_no_additional_space() {
        let l = vec![White; 3];
        let ds = DynamicSolver::new(simple_description(), Rc::new(l));

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
        let mut ds = DynamicSolver::new(simple_description(), Rc::new(l));
        assert_eq!(ds.solve().unwrap(), &vec![Black; 3]);
    }

    #[test]
    fn solve_cases() {
        for (desc, line, expected) in cases() {
            let as_blocks: Vec<_> = desc.iter().map(|b| BinaryBlock(*b)).collect();
            let desc = Description::new(as_blocks);

            let original_line = line.clone();

            let mut ds = DynamicSolver::new(Rc::new(desc), Rc::new(line));
            assert_eq!(ds.solve().unwrap(), &expected);
            assert_eq!(*ds.line, original_line);
        }
    }
}
