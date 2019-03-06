mod board;
mod parser;
mod render;
mod solver;
mod utils;

use parser::BoardParser;
use render::{Renderer, ShellRenderer};
use solver::line::DynamicSolver;
use solver::probing;
use solver::propagation;

use std::cell::RefCell;
use std::rc::Rc;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate priority_queue;

use cached::Cached;
use clap::{App, ArgGroup};

fn main() {
    env_logger::init();

    let matches = App::new("Nonogrid")
        .version("0.1.0")
        .about("Nonogram solver")
        .args_from_usage(
            "-b, --my [PATH]     'path to custom-formatted nonogram file'
             -p, --webpbn [PATH] 'path to Jan Wolter's http://webpbn.com XML-formatted file'
             -w, --webpbn-online [ID] 'id of the http://webpbn.com puzzle'",
        )
        .group(ArgGroup::with_name("source").required(true).args(&[
            "my",
            "webpbn",
            "webpbn-online",
        ]))
        .get_matches();

    let my_path = matches.value_of("my");
    let webpbn_path = matches.value_of("webpbn");
    let webpbn_id = matches.value_of("webpbn-online");

    let board = if let Some(webpbn_path) = webpbn_path {
        parser::WebPbn::read_board(webpbn_path)
    } else if let Some(webpbn_id) = webpbn_id {
        value_t_or_exit!(matches, "webpbn-online", u16);
        parser::WebPbn::get_board(webpbn_id)
    } else {
        parser::MyFormat::read_board(my_path.unwrap())
    };
    let board = Rc::new(RefCell::new(board));

    let r = ShellRenderer {
        board: Rc::clone(&board),
    };
    // println!("{}", r.render());
    warn!("Solving with simple line propagation");
    let solver = propagation::Solver::new(Rc::clone(&board));
    solver.run::<DynamicSolver<_>>().unwrap();
    println!("{}", r.render());

    {
        warn!("Trying to solve with probing");
        let solver = probing::FullProbe1::new(Rc::clone(&board));
        solver.run::<DynamicSolver<_>>().unwrap();

        let cache = solver.cache.borrow();
        if cache.cache_size() > 0 {
            warn!(
                "Cache size: {}, hits: {}, misses: {}",
                cache.cache_size(),
                cache.cache_hits().unwrap_or(0),
                cache.cache_misses().unwrap_or(0),
            );
        }
    }
}
