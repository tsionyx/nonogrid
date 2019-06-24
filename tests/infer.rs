use nonogrid::parser::{BoardParser, LocalReader, MyFormat, Paletted, PuzzleScheme};

#[test]
fn infer_own_black_and_white() {
    let f = MyFormat::read_local("examples/MLP.toml").unwrap();
    assert_eq!(f.infer_scheme(), PuzzleScheme::BlackAndWhite)
}

#[test]
fn get_colors_own() {
    let f = MyFormat::read_local("examples/UK.toml").unwrap();

    let colors = &[
        ("b".to_string(), '*', "blue".to_string()),
        ("r".to_string(), '%', "red".to_string()),
    ];
    assert_eq!(f.get_colors(), colors);

    let palette = f.get_palette();
    assert_eq!(palette.get_default(), Some("B".to_string()));
    assert_eq!(palette.id_by_name("W"), Some(1));
    assert_eq!(palette.id_by_name("B"), Some(2));
    assert_eq!(palette.id_by_name("b"), Some(4));
    assert_eq!(palette.id_by_name("r"), Some(8));
}

#[cfg(feature = "web")]
#[cfg(feature = "xml")]
mod webpbn {
    use nonogrid::parser::{BoardParser, Paletted, PuzzleScheme};
    use nonogrid::parser::{NetworkReader, WebPbn};

    #[test]
    fn infer_pbn_black_and_white() {
        let f = WebPbn::read_remote("1").unwrap();
        assert_eq!(f.infer_scheme(), PuzzleScheme::BlackAndWhite)
    }

    #[test]
    #[cfg(feature = "web")]
    fn infer_pbn_color() {
        let f = WebPbn::read_remote("18").unwrap();
        assert_eq!(f.infer_scheme(), PuzzleScheme::MultiColor)
    }

    #[test]
    fn get_pbn_colors() {
        let s = WebPbn::read_remote("18").unwrap();

        let colors = &[
            ("black".to_string(), 'X', "000000".to_string()),
            ("green".to_string(), '%', "00B000".to_string()),
            ("red".to_string(), '*', "FF0000".to_string()),
            ("white".to_string(), '.', "FFFFFF".to_string()),
        ];
        assert_eq!(s.get_colors(), colors);

        let palette = s.get_palette();
        assert_eq!(palette.get_default(), Some("black".to_string()));
        assert_eq!(palette.id_by_name("black"), Some(2));
        assert_eq!(palette.id_by_name("green"), Some(4));
        assert_eq!(palette.id_by_name("red"), Some(8));
        assert_eq!(palette.id_by_name("white"), Some(1));
    }
}
