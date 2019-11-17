# Yet another nonograms solver. Now in Rust!

## Usage

### Solve puzzles from http://webpbn.com (should be build with --features=web)

```
# solve puzzle http://webpbn.com/5933
nonogrid -w 5933
```

### Solve locally saved puzzles from http://webpbn.com (should be build with --features=xml)

```
# solve puzzle http://webpbn.com/2992
wget 'http://webpbn.com/XMLpuz.cgi?id=2992' -O 2992.xml
nonogrid 2992.xml

# with pipe
wget 'http://webpbn.com/XMLpuz.cgi?id=2992' -O- | nonogrid
```

### Solve local puzzles

Can solve any supported format:
- [TOML-based](examples) (`--features=ini`)
- nonograms.org HTML page
- webpbn [XML format](https://webpbn.com/pbn_fmt.html) (`--features=xml`)
- variety of webpbn's [exportable formats](https://webpbn.com/export.cgi/)

```
nonogrid examples/hello.toml

wget -qO- https://webpbn.com/export.cgi --post-data "fmt=syro&go=1&id=2040" |
cargo run --no-default-features
```


## Development mode

```
RUST_BACKTRACE=1 RUST_LOG=nonogrid=info cargo run -- examples/hello.toml
```


## Features

By default the `--features="clap std_time env_logger ini"` are enabled but you can disable almost anything
to speed up and/or shrink the size of the application.


### SAT

By default, the backtracking algorithm used for solving hard puzzles.
The feature `sat` allows to use the [SAT](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
solver for such a job.
The most of hard puzzles are solved significantly faster with this option.


### Arguments parsing

To support command-line arguments, the `clap` feature is enabled by default.
You can disable it, but then you will not able to set solving timeout or maximum number of solutions to find.
It also can be disabled when using the solver as a library in another projects, [e.g.](https://github.com/tsionyx/nono/blob/8e2f8f27/Cargo.toml#L19)


### Timeout (std_time)

By default you can provide the `--timeout` option to stop backtracking after reaching the specified time limit.
You can disable this feature and the timeout will simply be ignored.


### TOML puzzles parsing support

[My custom TOML-based format](examples/hello.toml) is supported by default via feature `ini`.
It can be disabled when using the solver as a library in another projects, [e.g.](https://github.com/tsionyx/nono/blob/8e2f8f27/Cargo.toml#L19)


### XML puzzles parsing support

The [Jan Wolter's XML format](https://webpbn.com/pbn_fmt.html) supported via feature `xml`.
You can enable it by building with the `--features=xml`.


### Colored nonograms

You can enable the feature `colored` to allow to print colored nonograms with real terminal colors:

```
wget -qO- https://webpbn.com/export.cgi --post-data "fmt=olsak&go=1&id=2192" |
cargo run --no-default-features --features=colored
```


### Logging

To support pretty formatted logs the `env_logger` crate is enabled by default.
As alwasys, you can disable it by skipping one in the list of features.


### HTTP client

Solved puzzles can be automatically downloaded from the Internet with the `reqwest` library
but it requires too many dependencies and increases compile time, so it's optional by default.
Enable it as simple as:

```
cargo run --features=web,xml -- --webpbn 5933
```


### Threading

By default, the solver and all the algorithms are single-threaded. To use the structures
in multi-threaded environment, provide the `threaded` feature. In essense, this feature
replaces every occurence of `Rc/RefCell` with `Arc/RwLock`.


### Probing tweaking

When the 'logical' solving gets stuck, the 'probing' phase starting which tries every variant
for every unsolved cells. It does this by calculating the priority for each cell:

```
P = N + R + C,

where 0<=N<=4 - number of neighbours which are solved cells or puzzle edges.
For example, the cell which has all 4 heighboring cells solved, has N = 4.
The upper left cell of the puzzle without any neighbours solved, has N = 2,
since it has 2 edges of the puzzle.

0<=R<=1 - row solution rate, the ratio of solved cells in the row to total number of cells (width)
0<=C<=1 - column solution rate, the ratio of solved cells in the column to total number of cells (height)
```

By default every cell with `P>=0` checked, but you can customize the threshold by specifying
the `LOW_PRIORITY` environment variable.

For example, running
```
LOW_PRIORITY=1 nonogrid puzzles/6574.xml
```

can be solved 3 times faster than standard way, by skipping the probing of cells with `P < 1`.
