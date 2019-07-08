#[cfg(feature = "threaded")]
use nonogrid::{block::binary::BinaryColor, render::ShellRenderer};

#[cfg(feature = "ini")]
mod ini {
    use log::warn;

    use nonogrid::parser::{BoardParser, LocalReader, MyFormat};
    use nonogrid::{
        block::binary::BinaryBlock,
        block::multicolor::ColoredBlock,
        parser::PuzzleScheme,
        solver::{line, probing::*, propagation},
        utils::rc::MutRc,
    };

    #[test]
    #[cfg(feature = "threaded")]
    fn hello() {
        use super::*;
        use nonogrid::render::Renderer;

        let f = MyFormat::read_local("examples/hello.toml").unwrap();
        let board = f.parse::<BinaryBlock>();
        let board = MutRc::new(board);

        let line_callback_renderer = ShellRenderer::with_board(MutRc::clone(&board));
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

        let color_callback_renderer = ShellRenderer::with_board(MutRc::clone(&board));
        board.write().set_callback_on_change_color(move |point| {
            println!("Changing the {:?}", point,);
            println!("{}", color_callback_renderer.render())
        });

        warn!("Solving with simple line propagation");
        let solver = propagation::Solver::new(MutRc::clone(&board));
        solver.run::<line::DynamicSolver<_>>().unwrap();

        let board = board.read();

        assert!(board.is_solved_full());
        assert_eq!(board.solution_rate(), 1.0);

        let b = BinaryColor::Black;
        let w = BinaryColor::White;
        assert_eq!(board.get_column(0), vec![b; 7]);
        assert_eq!(
            board.get_column(board.width() - 1),
            vec![b, b, b, b, b, w, b]
        );
    }

    #[test]
    fn pony() {
        let f = MyFormat::read_local("examples/MLP.toml").unwrap();
        let board = f.parse::<BinaryBlock>();
        let board = MutRc::new(board);

        warn!("Solving with simple line propagation");
        let solver = propagation::Solver::new(MutRc::clone(&board));
        solver.run::<line::DynamicSolver<_>>().unwrap();

        {
            let board = board.read();
            assert_eq!(board.solution_rate(), 0.0);
            assert!(!board.is_solved_full());
        }

        let solver = FullProbe1::with_board(MutRc::clone(&board));
        solver.run_unsolved::<line::DynamicSolver<_>>().unwrap();

        {
            let board = board.read();
            assert_eq!(board.solution_rate(), 1.0);
            assert!(board.is_solved_full());
        }
    }

    #[test]
    fn uk_flag() {
        let p = MyFormat::read_local("examples/UK.toml").unwrap();
        assert_eq!(p.infer_scheme(), PuzzleScheme::MultiColor);

        let board = p.parse::<ColoredBlock>();
        let board = MutRc::new(board);

        warn!("Solving with simple line propagation");
        let solver = propagation::Solver::new(MutRc::clone(&board));
        solver.run::<line::DynamicSolver<_>>().unwrap();

        let board = board.read();
        assert!(board.is_solved_full());
        assert_eq!(board.solution_rate(), 1.0);
    }
}

#[cfg(feature = "web")]
mod web {
    use log::warn;

    use nonogrid::parser::{BoardParser, NetworkReader, NonogramsOrg};
    use nonogrid::{
        block::binary::BinaryBlock,
        block::multicolor::ColoredBlock,
        parser::PuzzleScheme,
        solver::{line, propagation},
        utils::rc::MutRc,
    };

    #[test]
    #[cfg(feature = "xml")]
    fn webpbn_18() {
        use nonogrid::parser::WebPbn;
        let p = WebPbn::read_remote("18").unwrap();
        assert_eq!(p.infer_scheme(), PuzzleScheme::MultiColor);

        let board = p.parse::<ColoredBlock>();
        let board = MutRc::new(board);

        warn!("Solving with simple line propagation");
        let solver = propagation::Solver::new(MutRc::clone(&board));
        solver.run::<line::DynamicSolver<_>>().unwrap();

        let board = board.read();
        assert!(board.is_solved_full());
        assert_eq!(board.solution_rate(), 1.0);
    }

    #[test]
    /// http://www.nonograms.org/nonograms/i/4353
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
    /// http://www.nonograms.org/nonograms2/i/4374
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
    /// http://www.nonograms.ru/nonograms/i/23342
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
    /// http://www.nonograms.org/nonograms/i/6
    fn nonograms_org_solve() {
        let p = NonogramsOrg::read_remote("6").unwrap();
        assert_eq!(p.infer_scheme(), PuzzleScheme::BlackAndWhite);

        let board = p.parse::<BinaryBlock>();
        let board = MutRc::new(board);

        let solver = propagation::Solver::new(MutRc::clone(&board));
        solver.run::<line::DynamicSolver<_>>().unwrap();

        let board = board.read();
        assert!(board.is_solved_full());
        assert_eq!(board.solution_rate(), 1.0);
    }
}
