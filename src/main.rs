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

use clap::{App, ArgGroup};

fn main() {
    env_logger::init();

    let matches = App::new("Nonogrid")
        .version("0.1.0")
        .about("Nonogram solver")
        .args_from_usage(
            "--my [PATH]     'path to custom-formatted nonogram file'
             --webpbn [PATH] 'path to Jan Wolter's webpbn.com XML-formatted file'",
        )
        .group(
            ArgGroup::with_name("source")
                .required(true)
                .args(&["my", "webpbn"]),
        )
        .get_matches();

    let my_path = matches.value_of("my");
    let webpbn_path = matches.value_of("webpbn");

    let board = if let Some(webpbn_path) = webpbn_path {
        parser::WebPbn::read_board(webpbn_path)
    } else {
        parser::MyFormat::read_board(my_path.unwrap())
    };
    let board = Rc::new(RefCell::new(board));

    let r = ShellRenderer {
        board: Rc::clone(&board),
    };
    // println!("{}", r.render());
    println!("Solving...");
    propagation::solve::<_, DynamicSolver<_>>(Rc::clone(&board), None, None, false).unwrap();
    println!("{}", r.render());
}
