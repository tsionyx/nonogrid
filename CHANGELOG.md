# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2019-02-12
### Added
- Initial release!
- Parse board from custom TOML-based format
- Store the board
- Print out the board in the terminal
- Solve a single black-and-white line


## [0.2.0] - 2019-03-15
### Added
- Solve the black-and-white board with the line solving then probing then backtracking
- Parse boards from https://webpbn.com


## [0.3.0] - 2019-04-03
### Added
- Support for multi-colored boards
- Parse boards from https://nonograms.org


## [0.3.1] - 2019-04-12
### Added
- callbacks support

### Fixed
- optimization of probing queue: use different strategies depending of queue size
- optimize NonogramsOrg parsing and ShellRenderer with caching

### Updated
- replace `Option::unwrap` with `Option::expect("error message")`
- many refactoring of references
- refactoring with clippy


## [0.4.0] - 2019-05-09
### Added
- ability to use Board in threaded environment (--features=threaded)

### Fixed
- WASM runtime support: remove Instant::now() calls

### Updated
- fix imports to use `crate::` instead of `super::`
- switch to upstream priority-queue (after merging https://github.com/garro95/priority-queue/pull/14)


## [0.4.1] - 2019-05-19
### Added
-  add `ColorDesc::rgb_value` to use in web renderer

### Fixed
- restore backtracking timeouts with `Instant::now()` by enabling `std_time` default feature

### Updated
- published on crates.io repository


## [0.5.0] - 2019-07-03
### Added
- olsak is the new default format for webpbn puzzles
- support for [more formats](https://webpbn.com/export.cgi/) (faase, ish, keen, makhorin, ss, syro, nin)
- _LOW_PRIORITY_ environment variable to prevent probing the cells with possibly low impact
- [more checks](src/lib.rs)
- implement `Debug` for all the structures
- [travis](https://travis-ci.org/tsionyx/nonogrid) tests support
- 'stat' mode for [batch.sh](benches/batch.sh) script
- performance comparison [results as csv](benches)
- [article](doc/README.md) on [habr.com]((https://habr.com/ru/post/454586/))

### Fixed
- remove `Color::is_updated_with` and `Board::diff` to improve performance
- disable `backtracking::Solver::explored_paths` to improve performance
- improve `backtracking::SearchTree::debug` to prevent high memory usage
- add 'repository' in crate metadata

### Updated
- move all but the core dependencies into optional features (clap, env_logger, ini, xml, colored)
- iterators refactoring
- [bench results](benches/results.md)
- merge 'benches/batch-nonograms.org.sh' and 'benches/batch-webpbn.sh' into [single script](benches/batch.sh)
- remove unnecessary `pub`


## [0.5.1] - 2019-07-03

### Fixed
- correctly parse the colors starting with '#', e.g. _#FF00FF_


## [0.5.1] - 2019-08-14

### Added
- [spoj example](examples/spoj/) with the algorithm adaptation for
[this problem](https://www.spoj.com/problems/JCROSS/)
- crate metadata to `--help` with help of `clap`'s macros.

### Fixed
- README examples
- get rid of some allocations by preventing premature `collect`-ing
- do not store `Err("Bad line")` in cache anymore
- use iterators instead of `Vec`'s where possible
- optimize `propagation::Solver::update_line`
- clippy errors about `f64` comparison: use `f64::EPSILON`

### Updated
- move solutions cache from `Board` to `propagation::Solver`
- make the `Priority` abstraction instead of `OrderedFloat<f64>`
- make the `ProbeResult` abstraction instead of `Option`
to represent NewInfo/Contradiction variants
- benchmarks results (run some very long-solving puzzles)
- use SmallVec for Point's neighbours
- lower the logs level for some of `probing` and `backtracking` events
- do not create the propagation solver `with_point` anymore, just run with (or without) a Point
- dependencies
- callback test as closure
