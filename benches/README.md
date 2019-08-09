# Performance comparison

The basis of the work was done by Jan Wolter in his [survey](https://webpbn.com/survey).

To better understand my own solver's ability I have used the same techniques and puzzles
to prepare a report. To adjust my own machine's performance, I ran several solvers
from the survey on the same puzzles - they marked with the prefix `_my` in the report.

My solver for all the tests was build with `cargo build --release --no-default-features --features=clap`.


## [Black-and-white](perf.csv)

### How did I run the puzzles

Puzzles were exported from the [export page](https://webpbn.com/export.cgi)

```
for id in 1 6 16 21 23 27 65 436 529 803 1611 1694 2040 2413 2556 2712 3541 4645 6574 6739 7604 8098 9892 10088 10810 12548 18297 22336; do
    echo $id;
    curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=syro&go=1" > puzzles/$id.syro
    curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=nin&go=1" > puzzles/$id.nin
    curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=cwd&go=1" > puzzles/$id.cwd
done
```

and then were run with these simple bash loops:

##### Syromolotov

```
for id in $(ls puzzles/*.syro); do
    echo $id;
    time jsolver-1.2-src/jsolver -n 2 $id;
done
```

##### Wolter

```
for id in $(ls puzzles/*.nin); do
    echo $id;
    time pbnsolve-1.09/pbnsolve -u -x1800 $id;
done
```

##### BGU

```
for id in $(ls puzzles/*.nin); do
    echo $id;
    time java -jar bgusolver_cmd_102.jar -file $id -maxsolutions 2 -timeout 1800;
done
```

##### Tamura/Copris

```
for id in $(ls puzzles/*.cwd); do
    echo $id;
    time scala-2.10.7/bin/scala -cp copris-nonogram-v1-2.jar nonogram.Solver $id;
done
```

##### nonogrid

```
for id in $(ls puzzles/*.nin); do
    echo $id;
    time target/release/nonogrid $id --timeout=1800 --max-solutions=2
done
```


## [Colored](perf-color.csv)

##### Comparison was made with Wolter's solver only

```
for id in 47 220 1503 2257 4940 5193 2684 2073 4364 2817 4809 2814 3149 4445 2984 2498 3620 672; do
    echo $id;
    curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=olsak&go=1" > puzzles/$id.g
    time pbnsolve-1.09/pbnsolve -u -x1800 puzzles/$id.g
    time target/release/nonogrid puzzles/$id.g --timeout=1800 --max-solutions=2
done
```


## [Memory consumption (MiB)](memory.csv)

Use this script to automatize runs.

```
#!/bin/bash -e
# Example: `bash check-limits.sh 23 50000` (48.83 MiB)

ulimit -Sv $2
for i in {1..300}; do
  #echo $i
  target/release/nonogrid puzzles/$1.nin --timeout=3600 --max-solutions=2 >/dev/null
done
```


### Export various formats

```
for fmt in $(curl -s https://webpbn.com/export.cgi | grep -oP 'name="fmt" value="\K([^"]+)'); do
    echo "Downloading puzzle $ID with format $fmt"
    curl -s https://webpbn.com/export.cgi --data "id=$ID&fmt=$fmt&go=1" > ${ID}.${fmt}
done
```
