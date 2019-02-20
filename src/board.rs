use std::fmt;

#[derive(Debug)]
struct Point {
    x: usize,
    y: usize,
}

pub trait State {
    fn initial() -> Self;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlackState {
    Undefined,
    Space,
    Box,
    //Color(u32),
}

impl State for BlackState {
    fn initial() -> Self {
        BlackState::Undefined
    }
}

impl fmt::Display for BlackState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BlackState::*;

        let symbol = match self {
            Undefined => '?',
            Space => '.',
            Box => '\u{2b1b}',
        };
        write!(f, "{}", symbol)
    }
}

struct Cell {
    coord: Point,
    // state: State,
}

pub trait Block {
    type State: State;

    fn from_str(s: &str) -> Self;
}

#[derive(Debug, PartialEq)]
pub struct BlackBlock(pub usize);

impl Block for BlackBlock {
    type State = BlackState;

    fn from_str(s: &str) -> Self {
        Self(s.parse::<usize>().unwrap())
    }
}

impl fmt::Display for BlackBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Description<T: Block>
where
    T: Block,
{
    pub vec: Vec<T>,
}

impl<T> Description<T>
where
    T: Block,
{
    pub fn new(vec: Vec<T>) -> Description<T> {
        Description { vec }
    }
}

#[derive(Debug)]
pub struct Board<B>
where
    B: Block,
{
    pub cells: Vec<Vec<B::State>>,
    pub desc_rows: Vec<Description<B>>,
    pub desc_cols: Vec<Description<B>>,
}

impl<B> Board<B>
where
    B: Block,
    B::State: Clone,
{
    pub fn with_descriptions(rows: Vec<Description<B>>, columns: Vec<Description<B>>) -> Board<B> {
        let height = rows.len();
        let width = columns.len();

        let init = B::State::initial();

        let cells = vec![vec![init; width]; height];

        Board {
            desc_rows: rows,
            desc_cols: columns,
            cells,
        }
    }

    pub fn height(&self) -> usize {
        self.desc_rows.len()
    }

    pub fn width(&self) -> usize {
        self.desc_cols.len()
    }
}

#[cfg(test)]
mod tests {
    use super::BlackState::Undefined;
    use super::{BlackBlock, Board, Description};

    #[test]
    fn u_letter() {
        // X   X
        // X   X
        // X X X
        let rows = vec![
            Description::new(vec![BlackBlock(1), BlackBlock(1)]),
            Description::new(vec![BlackBlock(1), BlackBlock(1)]),
            Description::new(vec![BlackBlock(3)]),
        ];
        let columns = vec![
            Description::new(vec![BlackBlock(3)]),
            Description::new(vec![BlackBlock(1)]),
            Description::new(vec![BlackBlock(3)]),
        ];

        let board = Board::with_descriptions(rows, columns);
        assert_eq!(board.cells.len(), 3);
        assert_eq!(board.cells[0], [Undefined, Undefined, Undefined]);
    }
}
