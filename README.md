# Yet another nonograms solver. Now in Rust!

## Solve puzzles from http://webpbn.com

```
# solve puzzle http://webpbn.com/5933
nonogrid -w 5933
```

## Solve locally saved puzzles from http://webpbn.com

```
# solve puzzle http://webpbn.com/2992
wget 'http://webpbn.com/XMLpuz.cgi?id=2992' -O 2992.xml
nonogrid -p 2992.xml
```

## Solve my own TOML-based format

```
nonogrid -b examples/hello.toml
```


## Solve in development mode

```
RUST_BACKTRACE=1 RUST_LOG=nonogrid=info cargo run -- --webpbn-online=5933
```
