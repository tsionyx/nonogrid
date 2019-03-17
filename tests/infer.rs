use nonogrid::parser::{InferScheme, LocalReader, MyFormat, PuzzleScheme};
#[cfg(feature = "web")]
use nonogrid::parser::{NetworkReader, WebPbn};

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
