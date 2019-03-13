mod board;
mod cache;
mod parser;
mod render;
mod solver;
mod utils;

use board::{Block, Board};
use parser::BoardParser;
use render::{Renderer, ShellRenderer};
use solver::line::DynamicSolver;
use solver::probing::FullProbe1;

use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use clap::{App, ArgGroup, ArgMatches};

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
        .arg_from_usage(
            "-m, --max-solutions=[THRESHOLD] 'Stop searching after finding enough solutions'",
        )
        .arg_from_usage(
            "-t, --timeout=[SECONDS] 'Stop searching after specified timeout in seconds'",
        )
        .arg_from_usage(
            "-d, --max-depth=[DEPTH] 'Stop searching after reaching specified search depth'",
        )
        .get_matches();

    let board = board_from_args(&matches);
    let board = Rc::new(RefCell::new(board));
    let search_options = search_options_from_args(&matches);

    let r = ShellRenderer {
        board: Rc::clone(&board),
    };

    let backtracking = solver::run::<_, DynamicSolver<_>, FullProbe1<_>>(
        Rc::clone(&board),
        search_options.0,
        search_options.1,
        search_options.2,
    )
    .unwrap();
    println!("{}", r.render());

    if let Some(backtracking) = backtracking {
        let solutions = &backtracking.solutions;
        if !solutions.is_empty() && (!board.borrow().is_solved_full() || solutions.len() > 1) {
            println!("Backtracking found {} solutions:", solutions.len());
            for solution in solutions.iter() {
                board.borrow_mut().restore(solution.clone());
                println!("{}", r.render());
            }
        }
        backtracking.print_cache_info();
    }
}

fn board_from_args<B>(matches: &ArgMatches) -> Board<B>
where
    B: Block,
{
    let my_path = matches.value_of("my");
    let webpbn_path = matches.value_of("webpbn");
    let webpbn_id = matches.value_of("webpbn-online");

    if let Some(webpbn_path) = webpbn_path {
        parser::WebPbn::read_board(webpbn_path)
    } else if let Some(webpbn_id) = webpbn_id {
        value_t_or_exit!(matches, "webpbn-online", u16);
        parser::WebPbn::get_board(webpbn_id)
    } else {
        parser::MyFormat::read_board(my_path.unwrap())
    }
}

fn search_options_from_args(matches: &ArgMatches) -> (Option<usize>, Option<u32>, Option<usize>) {
    (
        parse_arg::<usize>(&matches, "max-solutions"),
        parse_arg::<u32>(&matches, "timeout"),
        parse_arg::<usize>(&matches, "max-depth"),
    )
}

fn parse_arg<T>(matches: &ArgMatches, name: &str) -> Option<T>
where
    T: FromStr,
{
    if matches.is_present(name) {
        let value = value_t!(matches, name, T).unwrap_or_else(|e| e.exit());
        return Some(value);
    }

    None
}
