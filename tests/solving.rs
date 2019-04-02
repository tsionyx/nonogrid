#[cfg(feature = "web")]
use nonogrid::parser::{NetworkReader, NonogramsOrg, WebPbn};
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

#[test]
#[cfg(feature = "web")]
/// http://www.nonograms.org/nonograms/i/4353
fn nonograms_org_extract() {
    let nop = NonogramsOrg::read_remote("4353").unwrap();
    assert_eq!(nop.infer_scheme(), PuzzleScheme::BlackAndWhite);
    assert_eq!(nop.encoded().len(), 40);

    let (colors, solution) = nop.decipher();
    assert_eq!(colors, ["000000"]);
    assert_eq!(
        solution,
        [
            [1, 1, 1, 0],
            [0, 0, 1, 1],
            [1, 0, 1, 0],
            [0, 1, 1, 0],
            [1, 1, 1, 0],
            [1, 0, 1, 0],
        ]
    );
}

#[test]
#[cfg(feature = "web")]
/// http://www.nonograms.org/nonograms2/i/4374
fn nonograms_org_extract_colored() {
    let nop = NonogramsOrg::read_remote("4374").unwrap();
    assert_eq!(nop.infer_scheme(), PuzzleScheme::MultiColor);
    assert_eq!(nop.encoded().len(), 45);

    let (colors, solution) = nop.decipher();
    assert_eq!(colors, ["fbf204", "000000", "f4951c"]);
    assert_eq!(
        solution,
        [
            [0, 0, 0, 1, 0],
            [1, 0, 0, 1, 1],
            [1, 3, 3, 0, 0],
            [2, 3, 3, 0, 0],
            [3, 3, 0, 0, 0],
        ]
    );
}

#[test]
#[cfg(feature = "web")]
/// http://www.nonograms.ru/nonograms/i/23342
fn nonograms_org_not_found_on_org_but_found_on_ru() {
    let nop = NonogramsOrg::read_remote("23342").unwrap();
    assert_eq!(nop.infer_scheme(), PuzzleScheme::BlackAndWhite);
    assert_eq!(nop.encoded().len(), 846);
}

#[test]
#[cfg(feature = "web")]
fn nonograms_org_not_found() {
    let msg = NonogramsOrg::read_remote("444444").err().unwrap();
    assert_eq!(msg, "Not found cypher in HTML content");
}
