#[cfg(feature = "web")]
use nonogrid::parser::{NetworkReader, WebPbn};
use nonogrid::{
    block::binary::{BinaryBlock, BinaryColor},
    block::multicolor::ColoredBlock,
    parser::{BoardParser, LocalReader, MyFormat, PuzzleScheme},
    solver::{line, probing::*, propagation},
};

#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn hello() {
    let f = MyFormat::read_local("examples/hello.toml").unwrap();
    let board = f.parse::<BinaryBlock>();
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
    let f = MyFormat::read_local("examples/MLP.toml").unwrap();
    let board = f.parse::<BinaryBlock>();
    let board = Rc::new(RefCell::new(board));

    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<line::DynamicSolver<_>>().unwrap();

    {
        let board = board.borrow();
        assert_eq!(board.solution_rate(), 0.0);
        assert!(!board.is_solved_full());
    }

    let solver = FullProbe1::with_board(Rc::clone(&board));
    solver.run_unsolved::<line::DynamicSolver<_>>().unwrap();

    {
        let board = board.borrow();
        assert_eq!(board.solution_rate(), 1.0);
        assert!(board.is_solved_full());
    }
}

#[test]
fn uk_flag() {
    let p = MyFormat::read_local("examples/UK.toml").unwrap();
    assert_eq!(p.infer_scheme(), PuzzleScheme::MultiColor);

    let board = p.parse::<ColoredBlock>();
    let board = Rc::new(RefCell::new(board));

    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<line::DynamicSolver<_>>().unwrap();

    let board = board.borrow();
    assert!(board.is_solved_full());
    assert_eq!(board.solution_rate(), 1.0);
}

#[test]
#[cfg(feature = "web")]
fn webpbn_18() {
    let p = WebPbn::read_remote("18").unwrap();
    assert_eq!(p.infer_scheme(), PuzzleScheme::MultiColor);

    let board = p.parse::<ColoredBlock>();
    let board = Rc::new(RefCell::new(board));

    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<line::DynamicSolver<_>>().unwrap();

    let board = board.borrow();
    assert!(board.is_solved_full());
    assert_eq!(board.solution_rate(), 1.0);
}
