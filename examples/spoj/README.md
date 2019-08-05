# Adaptation of the solver for [this problem](https://www.spoj.com/problems/JCROSS/)


Compile

```
rustup run 1.14.0 rustc -C opt-level=3 -C lto <(grep -v clippy main.rs) -o main
```

Test

```
time ./main < puzzles.txt
```
