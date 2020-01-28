// Generated with
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
#![warn(unsafe_code)]
#![warn(unstable_features)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_labels)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

pub mod block;
pub mod board;
pub(crate) mod cache;
pub mod parser;
pub mod render;
pub mod solver;
pub mod utils;
