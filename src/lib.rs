#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unused_results)]
#![warn(unused_qualifications, unused_import_braces)]

pub mod block;
pub mod board;
pub(crate) mod cache;
pub mod parser;
pub mod render;
pub mod solver;
pub mod utils;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
