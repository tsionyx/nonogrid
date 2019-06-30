#[macro_use]
extern crate log;

use std::fmt::Display;
use std::fs;
use std::io::{stdin, Read};
use std::str::FromStr;

use clap::{value_t, App, Arg, ArgMatches};
use log::Level;

use block::{binary::BinaryBlock, multicolor::ColoredBlock, Block};
use board::Board;
use parser::{BoardParser, NetworkReader, ParseError, PuzzleScheme};
use render::{Renderer, ShellRenderer};
use solver::{
    line::{DynamicColor, DynamicSolver},
    probing::FullProbe1,
};
use utils::rc::MutRc;

pub mod block;
pub mod board;
pub(crate) mod cache;
pub mod parser;
pub mod render;
pub mod solver;
pub(crate) mod utils;

fn main() -> Result<(), ParseError> {
    #[cfg(feature = "env_logger")]
    env_logger::init();

    let matches = App::new("nonogrid")
        .version("0.1.0")
        .about("Efficient nonogram solver")
        .arg(
            Arg::with_name("INPUT")
                .help("The nonogram file or puzzle ID to solve. When no input is present, read from the stdin.")
                .index(1)
        )
        .arg(
            Arg::with_name("webpbn").help("Solve puzzle from http://webpbn.com with specified ID")
                .short("w").long("webpbn").requires("INPUT")
        )
        .arg(
            Arg::with_name("nonograms-org").help("Solve puzzle from http://www.nonograms.org/ with specified ID")
                .short("o").long("nonograms-org").requires("INPUT").conflicts_with("webpbn")
        )
        .args_from_usage(
            "-m, --max-solutions=[THRESHOLD] 'Stop searching after finding enough solutions'
             -t, --timeout=[SECONDS] 'Stop searching after specified timeout in seconds'
             -d, --max-depth=[DEPTH] 'Stop searching after reaching specified search depth'",
        )
        .get_matches();

    let search_options = search_options_from_args(&matches);
    let (source, content) = content_from_args(&matches)?;

    match source {
        Source::LocalFile => run(
            &parser::DetectedParser::with_content(content)?,
            search_options,
        ),
        Source::WebPbn => run(&parser::WebPbn::read_remote(&content)?, search_options),
        Source::NonogramsOrg => run(
            &parser::NonogramsOrg::read_remote(&content)?,
            search_options,
        ),
    };
    Ok(())
}

fn run<P>(board_parser: &P, search_options: SearchOptions)
where
    P: BoardParser,
{
    match board_parser.infer_scheme() {
        PuzzleScheme::BlackAndWhite => {
            run_with_block::<BinaryBlock, _>(board_parser, search_options)
        }
        PuzzleScheme::MultiColor => run_with_block::<ColoredBlock, _>(board_parser, search_options),
    }
}

fn run_with_block<B, P>(board_parser: &P, search_options: SearchOptions)
where
    B: 'static + Block + Display,
    B::Color: DynamicColor + Display,
    P: BoardParser,
{
    let board = board_parser.parse::<B>();
    let board = MutRc::new(board);
    let r = ShellRenderer::with_board(MutRc::clone(&board));

    let backtracking = solver::run::<_, DynamicSolver<_>, FullProbe1<_>>(
        MutRc::clone(&board),
        search_options.0,
        search_options.1,
        search_options.2,
    )
    .unwrap();
    println!("{}", r.render());

    if let Some(backtracking) = backtracking {
        let solutions = &backtracking.solutions;
        if !solutions.is_empty() && (!board.read().is_solved_full() || solutions.len() > 1) {
            println!("Backtracking found {} solutions:", solutions.len());
            for solution in solutions.iter() {
                Board::restore_with_callback(MutRc::clone(&board), solution.clone());
                println!("{}", r.render());
            }
        }

        board.read().print_cache_info();
        if log_enabled!(Level::Warn) {
            let search_tree = backtracking.search_tree.read();
            if !search_tree.is_empty() {
                println!("Searching progress: {:?}", search_tree);
            }
        }
    }
}

fn content_from_args(matches: &ArgMatches) -> Result<(Source, String), ParseError> {
    let input_id = matches.value_of("INPUT");

    if matches.is_present("webpbn") {
        return Ok((
            Source::WebPbn,
            input_id
                .expect("INPUT should be present in --webpbn mode")
                .to_string(),
        ));
    }

    if matches.is_present("nonograms-org") {
        return Ok((
            Source::NonogramsOrg,
            input_id
                .expect("INPUT should be present in --nonograms-org mode")
                .to_string(),
        ));
    }

    let content = if let Some(input_file) = input_id {
        fs::read_to_string(input_file)?
    } else {
        warn!("Reading from stdin...");
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer)?;
        buffer
    };

    Ok((Source::LocalFile, content))
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
    LocalFile,
    WebPbn,
    NonogramsOrg,
}
