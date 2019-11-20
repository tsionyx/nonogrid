# Performance comparison

The basis of the work was done by Jan Wolter in his [survey](https://webpbn.com/survey).

To better understand my own solver's ability I have used the same techniques and puzzles
to prepare a report. To adjust my own machine's performance, I ran several solvers
from the survey on the same puzzles - they marked with the prefix `_my` in the report.

My solver for all the tests was build with `cargo build --release --no-default-features --features=clap,sat`.


## https://webpbn.com puzzles

### [Black-and-white](perf.csv)

#### How did I run the puzzles

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


### [Colored](perf-color.csv)

##### Comparison was made with Wolter's solver only

```
for id in 47 220 1503 2257 4940 5193 2684 2073 4364 2817 4809 2814 3149 4445 2984 2498 3620 672; do
    echo $id;
    curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=olsak&go=1" > puzzles/$id.g
    time pbnsolve-1.09/pbnsolve -u -x1800 puzzles/$id.g
    time target/release/nonogrid puzzles/$id.g --timeout=1800 --max-solutions=2
done
```


### [Memory consumption (MiB)](memory.csv)

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


### Test [random puzzles](https://webpbn.com/survey/#rand)

```
wget -q -O- https://webpbn.com/survey/rand30.tgz | tar -xz

cargo build --release --features=sat

for i in {1..5000}; do
    echo "#$i"
    RUST_LOG=nonogrid=warn /usr/bin/time -f 'Total: %U' target/release/nonogrid 30x30-2/rand$i --max-solutions=2 >/dev/null
done 2>&1 | tee rand.log

grep -oP 'Total: \K(.+)' rand.log | sort -n | nl -ba | less
```

| 0 - 0.09 | 0.10 - 0.19 | 0.20 - 0.49 | 0.50 - 0.99 | 1.00 - 3.99 | 4.00 - 9.99 | 10.00 - 29.99 | 30.00 - 59.99 | 60 - 120 | 120+ |
|---------:|------------:|------------:|------------:|------------:|------------:|--------------:|--------------:|---------:|-----:|
|     4230 |         283 |         300 |         112 |          61 |          10 |             4 |              0|         0|     0|


### Hardest backtracking puzzles

```
nohup bash benches/batch.sh webpbn {1..34000} >batch.log 2>&1 &
bash benches/batch.sh stat batch.log 10
```

#### black-and-white (with SAT solver, >=10 seconds)

| puzzle_id | solve time |
|-----------|------------|
| **9892**  | 26         |
| **12548** | 136        |
| 16900     | 12         |
| 19080     | 367        |
| **22336** | 263        |
| 25385     | 278        |
| 25588     | 185        |
| 25820     | 119271 (43412 for 1-st solution)
| 26520     | 47628      |
| 30532     | 12         |
| 30654     | 1148       |
| 32013     | 12         |
| 32291     | 58         |

#### colored (with SAT solver, >=10 seconds)

| puzzle_id | solve time, sec | colors (w/o blank) |
|-----------|----------------:|--------------------|
| **672**   | 44              | 3                  |
| **2498**  | 57              | 4                  |
| 3114      | 32              | 3                  |
| **4445**  | 24              | 3                  |
| 7541      | 55              | 4                  |
| 7778      | 14              | 2                  |
| 8337      | 33              | 4                  |
| 8880      | 17              | 4                  |
| 9786      | 53              | 2                  |
| 10585     | 435             | 4                  |
| 16838     | 695             | 2                  |
| 22027     | 160             | 4                  |
| 25158     | 41              | 4                  |
| 26810     | 73              | 4                  |
| 27097     | 77              | 4                  |
| 29469     | 15              | 2                  |
| 29826     | 24              | 3                  |
| 31812     | 39              | 3                  |

**Bold** puzzles are from http://webpbn.com/survey/.


## Export puzzle $ID in every supported format

```
for fmt in $(curl -s https://webpbn.com/export.cgi | grep -oP 'name="fmt" value="\K([^"]+)'); do
    echo "Downloading puzzle $ID with format $fmt"
    curl -s https://webpbn.com/export.cgi --data "id=$ID&fmt=$fmt&go=1" > ${ID}.${fmt}
done
```



## http://www.nonograms.org puzzles

28075 puzzles were run. All the puzzles are line solvable and has single solution.

### Distribution of solve times

```
$ nohup bash benches/batch.sh nonograms.org {1..28200} 2>&1 > batch-norg.log &
$ less batch-norg.log | grep 'Total' | awk '{print $2}' | sort -r | uniq -c
     1 0.09
     1 0.08
     2 0.06
     2 0.05
     8 0.04
    18 0.03
    83 0.02
   412 0.01
 27548 0.00
```

### Top 6 (>=0.05 sec)

| puzzle_id | solve time, sec | colors (w/o blank) |
|-----------|----------------:|--------------------|
| **4462**  | 0.05 +          | 3
| **9596**  | 0.09 +          | 10
| 20689     | 0.05 +          | 4
| 21251     | 0.06 +          | 1 (black)
| 21272     | 0.06 +          | 1 (black)
| 21553     | 0.08 +          | 5


**Bold** puzzles also found in [this C++ solver post](
https://izaron.github.io/post/solving-colored-japanese-crosswords-with-the-speed-of-light/#what-decreases-the-execution-time).
