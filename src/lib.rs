// The list was generated with the command
//   $ rustc -W help | grep ' allow ' | awk '{print $1}' | tr - _ | sort | xargs -I{} echo '#![warn({})]'
#![warn(absolute_paths_not_starting_with_crate)]
#![warn(anonymous_parameters)]
// #![warn(box_pointers)]
#![warn(deprecated_in_future)]
// #![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(indirect_structural_match)]
#![warn(keyword_idents)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_doc_code_examples)]
//#![warn(missing_docs)]  // TODO
#![warn(non_ascii_idents)]
#![warn(private_doc_tests)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
// #![warn(unreachable_pub)] // TODO
#![warn(unstable_features)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_labels)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
// recommendations
#![forbid(unsafe_code)]
#![deny(clippy::mem_forget)]
// suppress some pedantic warnings
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::must_use_candidate)]
#![cfg_attr(test, allow(clippy::wildcard_imports))]
// TODO: remove the following suppression:
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

pub use self::{
    block::{
        base::{color::ColorId, Block, Color, Description},
        binary::{BinaryBlock, BinaryColor},
        multicolor::ColoredBlock,
    },
    board::Board,
    parser::{BoardParser, DetectedParser},
    solver::{
        line::{DynamicColor, DynamicSolver as LineSolver},
        probing::{FullProbe1 as FullProbe, ProbeSolver},
        propagation::Solver as PropagationSolver,
        run as solve,
    },
};

mod block;
mod board;
mod cache;
pub mod parser;
pub mod render;
mod solver;
mod utils;

pub type RcBoard<B> = utils::rc::MutRc<Board<B>>;
