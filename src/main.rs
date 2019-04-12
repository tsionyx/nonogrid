pub mod block;
pub mod board;
pub(crate) mod cache;
pub mod parser;
pub mod render;
pub mod solver;
pub(crate) mod utils;

use block::binary::BinaryBlock;
use block::multicolor::ColoredBlock;
use block::Block;
use board::Board;
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
            "-b, --my [PATH]                 'path to custom-formatted nonogram file'
             -p, --webpbn [PATH]             'path to Jan Wolter's http://webpbn.com XML-formatted file'
             -w, --webpbn-online [ID]        'id of the http://webpbn.com puzzle'
             -n, --nonograms-org [PATH]      'path to HTML file from http://www.nonograms.org'
             -o, --nonograms-org-online [ID] 'id of the http://www.nonograms.org/ puzzle'
             ",
        )
        .group(ArgGroup::with_name("source").required(true).args(&[
            "my",
            "webpbn",
            "webpbn-online",
            "nonograms-org",
            "nonograms-org-online",
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

    match source {
        Source::Own => run(parser::MyFormat::read_local(&path), search_options),
        Source::WebPbn => run(parser::WebPbn::read_local(&path), search_options),
        Source::WebPbnOnline => run(parser::WebPbn::read_remote(&path), search_options),
        Source::NonogramsOrg => run(parser::NonogramsOrg::read_local(&path), search_options),
        Source::NonogramsOrgOnline => run(parser::NonogramsOrg::read_remote(&path), search_options),
    }
}

fn run<P>(board_parser: Result<P, String>, search_options: SearchOptions)
where
    P: BoardParser,
{
    let board_parser = board_parser.unwrap();
    match board_parser.infer_scheme() {
        PuzzleScheme::BlackAndWhite => {
            run_with_block::<BinaryBlock, _>(&board_parser, search_options)
        }
        PuzzleScheme::MultiColor => {
            run_with_block::<ColoredBlock, _>(&board_parser, search_options)
        }
    }
}

#[allow(dead_code)]
fn example_set_line_callback<B, R>(renderer: &R, is_column: bool, index: usize)
where
    B: Block,
    R: Renderer<B>,
{
    println!(
        "Set {}-th {}",
        index,
        if is_column { "column" } else { "row" }
    );
    println!("{}", renderer.render())
}

fn run_with_block<B, P>(board_parser: &P, search_options: SearchOptions)
where
    B: 'static + Block + Display,
    B::Color: DynamicColor + Display,
    P: BoardParser,
{
    let board = board_parser.parse::<B>();
    let board = Rc::new(RefCell::new(board));
    //let callback_renderer = ShellRenderer::with_board(Rc::clone(&board));
    //board
    //    .borrow_mut()
    //    .set_callback_on_set_line(move |is_column, index| {
    //        example_set_line_callback(&callback_renderer, is_column, index)
    //    });

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
                Board::restore_with_callback(Rc::clone(&board), solution.clone());
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
    let nonograms_org_path = matches.value_of("nonograms-org");
    let nonograms_org_id = matches.value_of("nonograms-org-online");

    if let Some(webpbn_path) = webpbn_path {
        return (Source::WebPbn, webpbn_path.to_string());
    } else if let Some(webpbn_id) = webpbn_id {
        value_t_or_exit!(matches, "webpbn-online", u16);
        return (Source::WebPbnOnline, webpbn_id.to_string());
    } else if let Some(nonograms_org_path) = nonograms_org_path {
        return (Source::NonogramsOrg, nonograms_org_path.to_string());
    } else if let Some(nonograms_org_id) = nonograms_org_id {
        value_t_or_exit!(matches, "nonograms-org-online", u16);
        return (Source::NonogramsOrgOnline, nonograms_org_id.to_string());
    } else if let Some(my_path) = my_path {
        return (Source::Own, my_path.to_string());
    }
    unreachable!("No valid source found");
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
    NonogramsOrg,
    NonogramsOrgOnline,
}
