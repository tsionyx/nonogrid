#[cfg(feature = "ini")]
mod ini {
    use std::f64;

    use log::warn;

    use nonogrid::{
        parser::{LocalReader, MyFormat, PuzzleScheme},
        BinaryBlock, BoardParser, ColoredBlock, FullProbe, LineSolver, ProbeSolver,
        PropagationSolver,
    };

    #[test]
    fn hello() {
        use nonogrid::{
            render::{Renderer, ShellRenderer},
            BinaryColor,
        };

        let f = MyFormat::read_local("examples/hello.toml").unwrap();
        let board = f.parse_rc::<BinaryBlock>();

        let line_callback_renderer = ShellRenderer::with_board(board.clone());
        board
            .write()
            .set_callback_on_set_line(move |is_column, index| {
                println!(
                    "Set {}-th {}",
                    index,
                    if is_column { "column" } else { "row" }
                );
                println!("{}", line_callback_renderer.render())
            });

        let color_callback_renderer = ShellRenderer::with_board(board.clone());
        board.write().set_callback_on_change_color(move |point| {
            println!("Changing the {:?}", point,);
            println!("{}", color_callback_renderer.render())
        });

        warn!("Solving with simple line propagation");
        let mut solver = PropagationSolver::new(board.clone());
        solver.run::<LineSolver<_>>(None).unwrap();

        let board = board.read();

        assert!(board.is_solved_full());
        assert!((board.solution_rate() - 1.0).abs() < f64::EPSILON);

        let b = BinaryColor::Black;
        let w = BinaryColor::White;
        assert_eq!(board.get_column(0), vec![b; 7].into());
        assert_eq!(
            board.get_column(board.width() - 1),
            vec![b, b, b, b, b, w, b].into()
        );
    }

    #[test]
    fn pony() {
        let f = MyFormat::read_local("examples/MLP.toml").unwrap();
        let board = f.parse_rc::<BinaryBlock>();

        warn!("Solving with simple line propagation");
        let mut solver = PropagationSolver::new(board.clone());
        solver.run::<LineSolver<_>>(None).unwrap();

        {
            let board = board.read();
            assert!(board.solution_rate() < f64::EPSILON);
            assert!(!board.is_solved_full());
        }

        let mut solver = FullProbe::with_board(board.clone());
        solver.run_unsolved::<LineSolver<_>>().unwrap();

        {
            let board = board.read();
            assert!((board.solution_rate() - 1.0).abs() < f64::EPSILON);
            assert!(board.is_solved_full());
        }
    }

    #[test]
    fn uk_flag() {
        let p = MyFormat::read_local("examples/UK.toml").unwrap();
        assert_eq!(p.infer_scheme(), PuzzleScheme::MultiColor);

        let board = p.parse_rc::<ColoredBlock>();

        warn!("Solving with simple line propagation");
        let mut solver = PropagationSolver::new(board.clone());
        solver.run::<LineSolver<_>>(None).unwrap();

        let board = board.read();
        assert!(board.is_solved_full());
        assert!((board.solution_rate() - 1.0).abs() < f64::EPSILON);
    }
}

#[cfg(feature = "web")]
mod web {
    use std::f64;

    use log::warn;

    use nonogrid::{
        parser::{NetworkReader, NonogramsOrg, PuzzleScheme},
        BinaryBlock, BoardParser, ColoredBlock, LineSolver, PropagationSolver,
    };

    #[test]
    #[cfg(feature = "xml")]
    fn webpbn_18() {
        use nonogrid::parser::WebPbn;
        let p = WebPbn::read_remote("18").unwrap();
        assert_eq!(p.infer_scheme(), PuzzleScheme::MultiColor);

        let board = p.parse_rc::<ColoredBlock>();

        warn!("Solving with simple line propagation");
        let mut solver = PropagationSolver::new(board.clone());
        solver.run::<LineSolver<_>>(None).unwrap();

        let board = board.read();
        assert!(board.is_solved_full());
        assert!((board.solution_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    /// <http://www.nonograms.org/nonograms/i/4353>
    fn nonograms_org_extract() {
        let nop = NonogramsOrg::read_remote("4353").unwrap();
        assert_eq!(nop.infer_scheme(), PuzzleScheme::BlackAndWhite);
        assert_eq!(nop.encoded().len(), 40);

        let (colors, solution) = nop.decipher();
        assert_eq!(colors, ["000000"]);
        assert_eq!(
            solution,
            [
                [1, 1, 1, 0],
                [0, 0, 1, 1],
                [1, 0, 1, 0],
                [0, 1, 1, 0],
                [1, 1, 1, 0],
                [1, 0, 1, 0],
            ]
        );
    }

    #[test]
    /// <http://www.nonograms.org/nonograms2/i/4374>
    fn nonograms_org_extract_colored() {
        let nop = NonogramsOrg::read_remote("4374").unwrap();
        assert_eq!(nop.infer_scheme(), PuzzleScheme::MultiColor);
        assert_eq!(nop.encoded().len(), 45);

        let (colors, solution) = nop.decipher();
        assert_eq!(colors, ["fbf204", "000000", "f4951c"]);
        assert_eq!(
            solution,
            [
                [0, 0, 0, 1, 0],
                [1, 0, 0, 1, 1],
                [1, 3, 3, 0, 0],
                [2, 3, 3, 0, 0],
                [3, 3, 0, 0, 0],
            ]
        );
    }

    #[test]
    /// <http://www.nonograms.ru/nonograms/i/23342>
    fn nonograms_org_not_found_on_org_but_found_on_ru() {
        let nop = NonogramsOrg::read_remote("23342").unwrap();
        assert_eq!(nop.infer_scheme(), PuzzleScheme::BlackAndWhite);
        assert_eq!(nop.encoded().len(), 846);
    }

    #[test]
    fn nonograms_org_not_found() {
        let msg = NonogramsOrg::read_remote("444444").err().unwrap();
        assert_eq!(msg.0, "Not found cypher in HTML content");
    }

    #[test]
    /// <http://www.nonograms.org/nonograms/i/6>
    fn nonograms_org_solve() {
        let p = NonogramsOrg::read_remote("6").unwrap();
        assert_eq!(p.infer_scheme(), PuzzleScheme::BlackAndWhite);

        let board = p.parse_rc::<BinaryBlock>();

        let mut solver = PropagationSolver::new(board.clone());
        solver.run::<LineSolver<_>>(None).unwrap();

        let board = board.read();
        assert!(board.is_solved_full());
        assert!((board.solution_rate() - 1.0).abs() < f64::EPSILON);
    }
}

#[cfg(feature = "web")]
mod detect_webpbn_formats {
    use reqwest::blocking::Client;

    use nonogrid::{
        parser::PuzzleScheme, BinaryBlock, BoardParser, ColoredBlock, DetectedParser, FullProbe,
        LineSolver, ProbeSolver,
    };

    fn request_puzzle(id: u32, fmt: &str) -> String {
        let id = id.to_string();
        let params = [("id", id.as_str()), ("fmt", fmt), ("go", "1")];
        let client = Client::new();
        let req = client
            .post("https://webpbn.com/export.cgi")
            .form(&params)
            .send()
            .unwrap();

        req.text().unwrap()
    }

    fn solve_puzzle_scheme(id: u32, fmt: &str, scheme: PuzzleScheme) -> String {
        let content = request_puzzle(id, fmt);
        print!("{}", &content);

        let p = DetectedParser::with_content(&content).unwrap();
        assert_eq!(p.infer_scheme(), scheme);

        match scheme {
            PuzzleScheme::BlackAndWhite => {
                let board = p.parse_rc::<BinaryBlock>();
                // warn!("Solving with simple line propagation");
                // let mut solver = propagation::Solver::new(MutRc::clone(&board));
                // solver.run::<line::DynamicSolver<_>>(None).unwrap();

                let mut solver = FullProbe::with_board(board.clone());
                solver.run_unsolved::<LineSolver<_>>().unwrap();

                let board = board.read();
                assert!(board.is_solved_full());
            }
            PuzzleScheme::MultiColor => {
                let board = p.parse_rc::<ColoredBlock>();
                let mut solver = FullProbe::with_board(board.clone());
                solver.run_unsolved::<LineSolver<_>>().unwrap();

                let board = board.read();
                assert!(board.is_solved_full());
            }
        };

        content
    }

    fn solve_puzzle(id: u32, fmt: &str) -> String {
        solve_puzzle_scheme(id, fmt, PuzzleScheme::BlackAndWhite)
    }

    #[test]
    fn faase_65() {
        let content = solve_puzzle(65, "faase");
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&"width 34"));
        assert!(content.contains(&"height 40"));
        assert!(content.contains(&"rows"));
        assert!(content.contains(&"columns"));
        // columns + rows + control rows
        assert_eq!(content.len(), 34 + 40 + 4);
    }

    #[test]
    fn ish_436() {
        let content = solve_puzzle(436, "ish");
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&"# row clues"));
        assert!(content.contains(&"# column clues"));
        assert!(content.contains(&"# Copyright 2006 by Jan Wolter"));
        assert!(content.contains(&""));
        // columns + rows + control rows
        assert_eq!(content.len(), 40 + 35 + 6);
    }

    #[test]
    fn keen_529() {
        let content = solve_puzzle(529, "keen");
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&""));
        // columns + rows + control rows
        assert_eq!(content.len(), 45 + 45 + 1);
    }

    #[test]
    fn makhorin_803() {
        let content = solve_puzzle(803, "makhorin");
        let content: Vec<_> = content.lines().collect();
        assert!(content[0].starts_with('*'));
        assert!(content[1].starts_with('*'));
        assert!(content.contains(&"&"));
        // columns + rows + control rows
        assert_eq!(content.len(), 50 + 45 + 3);
    }

    #[test]
    fn nin_1611() {
        let content = solve_puzzle(1611, "nin");
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&"55 60"));
        // columns + rows + control rows
        assert_eq!(content.len(), 55 + 60 + 1);
    }

    #[test]
    fn olsak_1694() {
        let content = solve_puzzle(1694, "olsak");
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&"#d"));
        assert!(content[3].contains("#FFFFFF"));
        assert!(content[3].contains("white"));
        assert!(content[4].contains("#000000"));
        assert!(content[4].contains("black"));
        assert!(content.contains(&": rows"));
        assert!(content.contains(&": columns"));
        // columns + rows + description + control rows + colors
        assert_eq!(content.len(), 45 + 50 + 2 + 3 + 2);
    }

    #[test]
    fn olsak_color_2814() {
        let content = solve_puzzle_scheme(2814, "olsak", PuzzleScheme::MultiColor);
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&"#d"));
        assert!(content[3].contains("#FFFFFF"));
        assert!(content[3].contains("white"));
        assert!(content[4].contains("#000000"));
        assert!(content[4].contains("black"));
        assert!(content[5].contains("#4040FF"));
        assert!(content[5].contains("blue"));
        assert!(content.contains(&": rows"));
        assert!(content.contains(&": columns"));
        // columns + rows + description + control rows + colors
        assert_eq!(content.len(), 45 + 50 + 2 + 3 + 3);
    }

    #[test]
    fn ss_2040() {
        let content = solve_puzzle(2040, "ss");
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&"width 55"));
        assert!(content.contains(&"height 60"));
        assert!(content.contains(&""));
        assert!(content.contains(&"rows"));
        assert!(content.contains(&"columns"));
        // columns + rows + description + control rows
        assert_eq!(content.len(), 55 + 60 + 4 + 6);
    }

    #[test]
    fn syro_2413() {
        let content = solve_puzzle(2413, "syro");
        let content: Vec<_> = content.lines().collect();
        assert!(content.contains(&"#"));
        // columns + rows + control rows
        assert_eq!(content.len(), 20 + 20 + 2);
    }
}
