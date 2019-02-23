mod board;
mod reader;
mod render;
mod solver;
mod utils;

use render::{Renderer, ShellRenderer};
use solver::line::DynamicSolver;
use solver::propagation;

use std::cell::RefCell;
use std::rc::Rc;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate priority_queue;

fn main() {
    env_logger::init();

    let board = reader::MyFormat::read_board("examples/hello.toml");
    let board = Rc::new(RefCell::new(board));
    let r = ShellRenderer {
        board: Rc::clone(&board),
    };
    println!("{}", r.render());
    println!("Solving...");
    propagation::solve::<_, DynamicSolver<_>>(Rc::clone(&board), None, None, false).unwrap();
    println!("{}", r.render());
}
