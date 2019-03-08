use nonogrid::{
    board::{BinaryBlock, BinaryColor},
    parser::{BoardParser, MyFormat},
    solver::{line, probing::*, propagation},
};

#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn hello() {
    let board = MyFormat::<BinaryBlock>::read_board("examples/hello.toml");
    let board = Rc::new(RefCell::new(board));

    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<line::DynamicSolver<_>>().unwrap();

    let board = board.borrow();

    assert!(board.is_solved_full());
    assert_eq!(board.solution_rate(), 1.0);

    let b = BinaryColor::Black;
    let w = BinaryColor::White;
    assert_eq!(board.get_column(0), vec![b; 7]);
    assert_eq!(
        board.get_column(board.width() - 1),
        vec![b, b, b, b, b, w, b]
    );
}

#[test]
fn pony() {
    let board = MyFormat::<BinaryBlock>::read_board("examples/MLP.toml");
    let board = Rc::new(RefCell::new(board));

    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<line::DynamicSolver<_>>().unwrap();

    {
        let board = board.borrow();
        assert!(!board.is_solved_full());
        assert_eq!(board.solution_rate(), 0.0);
    }

    let solver = FullProbe1::new(Rc::clone(&board));
    solver.run::<line::DynamicSolver<_>>().unwrap();

    {
        let board = board.borrow();
        assert!(board.is_solved_full());
        assert_eq!(board.solution_rate(), 1.0);
    }
}
