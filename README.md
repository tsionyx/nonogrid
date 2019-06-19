# Yet another nonograms solver. Now in Rust!

## Usage

### Solve puzzles from http://webpbn.com (should be build with --features=web)

```
# solve puzzle http://webpbn.com/5933
nonogrid -w 5933
```

### Solve locally saved puzzles from http://webpbn.com

```
# solve puzzle http://webpbn.com/2992
wget 'http://webpbn.com/XMLpuz.cgi?id=2992' -O 2992.xml
nonogrid -p 2992.xml
```

### Solve my own TOML-based format

```
nonogrid -b examples/hello.toml
```


## Development mode

```
RUST_BACKTRACE=1 RUST_LOG=nonogrid=info cargo run -- --my examples/hello.toml
```


## Features

By default the `--no-default-features --features="clap std_time colored env_logger"` are enabled but you can disable almost anything
to speed up and/or shrink the size of the application.

### Arguments parsing

To support command-line arguments, the `clap` feature is enabled by default.
To use the `nonogrid` binary you have to included it anyway, or binary crate will not compile.
It can be disabled when using the solver as a library in another projects, [e.g.](https://github.com/tsionyx/nono/blob/8e2f8f27/Cargo.toml#L19)

### Timeout (std_time)

By default you can provide the `--timeout` option to stop backtracking after reaching the specified time limit.
You can disable this feature and the timeout will simply be ignored.

```
cargo run --no-default-features --features=clap -- -p puzzles/2040.xml
```

### Colored nonograms

By default the feature `colored` is enabled to allow to print colored nonograms with real terminal colors.
You can disable this feature:

```
cargo run --no-default-features --features="clap env_logger" -- -p puzzles/2192.xml
```

### Logging

To support pretty formatted logs the `env_logger` crate is enabled by default.
As alwasys, you can disable it by skipping one in the list of features.


### HTTP client

Solved puzzles can be automatically downloaded from the Internet with the `reqwest` library
but it requires too many dependencies and increases compile time, so it's optional by default.
Enable it as simple as:

```
cargo run --features=web -- --webpbn-online=5933
```

### Threading

By default, the solver and all the algorithms are single-threaded. To use the structures
in multi-threaded environment, provide the `threaded` feature. In essense, this feature
replaces every occurence of `Rc/RefCell` with `Arc/RwLock`.
