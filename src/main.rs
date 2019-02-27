mod board;
mod parser;
mod render;
mod solver;
mod utils;

use parser::BoardParser;
use render::{Renderer, ShellRenderer};
use solver::line::DynamicSolver;
use solver::propagation;

use std::cell::RefCell;
use std::rc::Rc;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate priority_queue;

use clap::{App, Arg};

fn main() {
    env_logger::init();

    let matches = App::new("Nonogrid")
        .version("0.1.0")
        .about("Nonogram solver")
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("Path to nonogram descriptions file")
                .index(1),
        )
        .get_matches();

    let path_to_file = matches.value_of("INPUT").unwrap();
    let board = parser::MyFormat::read_board(path_to_file);
    let board = Rc::new(RefCell::new(board));

    let r = ShellRenderer {
        board: Rc::clone(&board),
    };
    // println!("{}", r.render());
    println!("Solving...");
    propagation::solve::<_, DynamicSolver<_>>(Rc::clone(&board), None, None, false).unwrap();
    println!("{}", r.render());
}
