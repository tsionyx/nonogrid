mod block;
mod board;
mod cache;
mod parser;
mod render;
mod solver;
mod utils;

use block::binary::BinaryBlock;
use block::multicolor::ColoredBlock;
use block::Block;
use parser::{BoardParser, LocalReader, NetworkReader, PuzzleScheme};
use render::{Renderer, ShellRenderer};
use solver::line::{DynamicColor, DynamicSolver};
use solver::probing::FullProbe1;

use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use clap::{App, ArgGroup, ArgMatches};
use log::Level;

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

    let search_options = search_options_from_args(&matches);
    let (source, path) = source_from_args(&matches);

    // FIXME: lack of dynamic dispatching entails this shit.
    //        Box<dyn BoardParser> also does not work due to
    //        error[E0038]: the trait `parser::BoardParser` cannot be made into an object
    //        note: method `parse` has generic type parameters
    if let Source::Own = source {
        let board_parser = parser::MyFormat::read_local(&path).unwrap();
        match board_parser.infer_scheme() {
            PuzzleScheme::BlackAndWhite => run::<BinaryBlock, _>(&board_parser, search_options),
            PuzzleScheme::MultiColor => run::<ColoredBlock, _>(&board_parser, search_options),
        }
    } else {
        let board_parser = match source {
            Source::WebPbn => parser::WebPbn::read_local(&path),
            Source::WebPbnOnline => parser::WebPbn::read_remote(&path),
            _ => panic!("No parser matched"),
        }
        .unwrap();
        match board_parser.infer_scheme() {
            PuzzleScheme::BlackAndWhite => run::<BinaryBlock, _>(&board_parser, search_options),
            PuzzleScheme::MultiColor => run::<ColoredBlock, _>(&board_parser, search_options),
        }
    }
}

fn run<B, P>(board_parser: &P, search_options: SearchOptions)
where
    B: Block + Display,
    B::Color: DynamicColor + Display,
    P: BoardParser,
{
    let board = board_parser.parse::<B>();
    let board = Rc::new(RefCell::new(board));

    let r = ShellRenderer::with_board(Rc::clone(&board));

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

        board.borrow().print_cache_info();
        if log_enabled!(Level::Warn) {
            let search_tree = backtracking.search_tree.borrow();
            if !search_tree.is_empty() {
                println!("Searching progress: {}", search_tree);
            }
        }
    }
}

fn source_from_args(matches: &ArgMatches) -> (Source, String) {
    let my_path = matches.value_of("my");
    let webpbn_path = matches.value_of("webpbn");
    let webpbn_id = matches.value_of("webpbn-online");

    if let Some(webpbn_path) = webpbn_path {
        return (Source::WebPbn, webpbn_path.to_string());
    } else if let Some(webpbn_id) = webpbn_id {
        value_t_or_exit!(matches, "webpbn-online", u16);
        return (Source::WebPbnOnline, webpbn_id.to_string());
    } else if let Some(my_path) = my_path {
        return (Source::Own, my_path.to_string());
    }
    panic!("No valid source found");
}

type SearchOptions = (Option<usize>, Option<u32>, Option<usize>);

fn search_options_from_args(matches: &ArgMatches) -> SearchOptions {
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

enum Source {
    Own,
    WebPbn,
    WebPbnOnline,
}
