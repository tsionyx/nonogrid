use nonogrid::parser::{InferScheme, LocalReader, MyFormat, PuzzleScheme};
#[cfg(feature = "web")]
use nonogrid::parser::{NetworkReader, WebPbn};

use std::collections::HashMap;

#[test]
fn infer_own_black_and_white() {
    let s = MyFormat::read_local("examples/MLP.toml").unwrap();
    assert_eq!(MyFormat::infer_scheme(&s), PuzzleScheme::BlackAndWhite)
}

#[test]
#[cfg(feature = "web")]
fn infer_pbn_black_and_white() {
    let s = WebPbn::read_remote("1").unwrap();
    assert_eq!(WebPbn::infer_scheme(&s), PuzzleScheme::BlackAndWhite)
}

#[test]
#[cfg(feature = "web")]
fn infer_pbn_color() {
    let s = WebPbn::read_remote("18").unwrap();
    assert_eq!(WebPbn::infer_scheme(&s), PuzzleScheme::MultiColor)
}

#[test]
#[cfg(feature = "web")]
fn get_pbn_colors() {
    let mut colors = HashMap::new();
    colors.insert("black".to_string(), ('X', "000000".to_string()));
    colors.insert("white".to_string(), ('.', "FFFFFF".to_string()));
    colors.insert("green".to_string(), ('%', "00B000".to_string()));
    colors.insert("red".to_string(), ('*', "FF0000".to_string()));

    let s = WebPbn::read_remote("18").unwrap();
    assert_eq!(WebPbn::get_colors(&s), colors)
}
