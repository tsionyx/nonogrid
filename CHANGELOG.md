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
