# Performance comparison

The basis of the work was done by Jan Wolter in his [survey](https://webpbn.com/survey).

To better understand my own solver's ability I have used the same techniques and puzzles
to prepare a report. To adjust my own machine's performance, I ran several solvers
from the survey on the same puzzles - they marked with the prefix `_my` in the report.

My solver for all the tests was build with `cargo build --release --no-default-features --features=args,sat`.


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
done 2>&1 | tee benches/rand.log

grep -oP 'Total: \K(.+)' benches/rand.log | sort -n | nl -ba | less
```

| 0 - 0.09 | 0.10 - 0.19 | 0.20 - 0.49 | 0.50 - 0.99 | 1.00 - 3.99 | 4.00 - 9.99 | 10.00 - 29.99 | 30.00 - 59.99 | 60 - 120 | 120+ |
|---------:|------------:|------------:|------------:|------------:|------------:|--------------:|--------------:|---------:|-----:|
|     4230 |         283 |         300 |         112 |          61 |          10 |             4 |              0|         0|     0|


### Hardest backtracking puzzles

```
nohup bash benches/batch.sh webpbn {1..35000} >benches/batch.log 2>&1 &
bash benches/batch.sh stat benches/batch.log 10 --details
```

#### black-and-white (with SAT solver, >=10 seconds)

| puzzle_id | solve time, sec | SAT variables | SAT clauses |
|-----------|----------------:|--------------:|------------:|
| 9892*     | 33              |        13_887 |     425_270 |
| 10088*    | 10              |        23_483 |   1_025_585 |
| 12548*    | 137             |        13_685 |     503_999 |
| 16900     | 12              |        32_295 |   1_751_050 |
| 18297*    | 10              |         9_708 |     275_166 |
| 19080     | 363             |        24_673 |   1_272_743 |
| 22336*    | 259             |        67_137 |   5_461_726 |
| 25385     | 279             |        21_334 |     923_394 |
| 25588     | 187             |        21_017 |     993_331 |
| 25820     | 106498 (42230 for 1-st solution) | 43_668 | 2_948_833 |
| 26520     | 41161           |        29_223 |   1_696_163 |
| 30532     | 13              |        15_212 |     444_144 |
| 30654     | 1342            |        33_384 |   1_138_340 |
| 32013     | 13              |         9_468 |     297_855 |
| 32291     | 67              |        14_421 |     543_978 |
| [Knotty**](https://webpbn.com/survey/puzzles/) | + | 11_076 |   230_271 |
| [Meow**](https://webpbn.com/survey/puzzles/)   | + | 35_036 | 1_872_216 |
| [Faase**](https://webpbn.com/survey/puzzles/)  | + | 89_918 | 9_968_433 |

#### colored (with SAT solver, >=10 seconds)

| puzzle_id | solve time, sec | SAT variables | SAT clauses | colors (w/o blank) |
|-----------|----------------:|--------------:|------------:|-------------------:|
| 672*      | 37              |        32_027 |   1_483_859 | 3
| 2498*     | 44              |        42_352 |   2_259_671 | 4
| 3114      | 25              |        28_886 |     995_422 | 3
| 9786      | 51              |        16_094 |     581_210 | 2
| 10585     | 414             |        28_224 |     829_213 | 4
| 16838     | 311             |        70_240 |   6_752_766 | 2
| 25158     | 33              |        20_433 |     660_340 | 4
| 26810     | 33              |        45_695 |   2_351_897 | 4

&ast; - puzzles from http://webpbn.com/survey/.

** - puzzles are not in public access but can be downloaded at https://webpbn.com/survey/puzzles/


## Export puzzle $ID in every supported format

```
for fmt in $(curl -s https://webpbn.com/export.cgi | grep -oP 'name="fmt" value="\K([^"]+)'); do
    echo "Downloading puzzle $ID with format $fmt"
    curl -s https://webpbn.com/export.cgi --data "id=$ID&fmt=$fmt&go=1" > ${ID}.${fmt}
done
```



## http://www.nonograms.org puzzles

29896 puzzles were run. All the puzzles are line solvable and has single solution.

### Distribution of solve times

```
$ nohup bash benches/batch.sh nonograms.org {1..30000} 2>&1 > benches/batch-norg.log &
$ less benches/batch-norg.log | grep 'Total' | awk '{print $2}' | sort -r | uniq -c
     3 0.06
     9 0.04
     9 0.03
    56 0.02
   302 0.01
 29517 0.00
```

### Top 12 (>=0.04 sec)

| puzzle_id | solve time, sec | colors (w/o blank) |
|-----------|----------------:|-------------------:|
| 4462*     | 0.04            | 3
| 9596*     | 0.04            | 10
| 17921     | 0.04            | 1 (black)
| 18417     | 0.04            | 1 (black)
| 19043     | 0.04            | 1 (black)
| 20689     | 0.04            | 4
| 21251     | 0.06            | 1 (black)
| 21259     | 0.04            | 1 (black)
| 21272     | 0.06            | 1 (black)
| 21424     | 0.04            | 10
| 21553     | 0.06            | 5
| 22118     | 0.04            | 10


&ast; - puzzles also mentioned in [this C++ solver post](
https://izaron.github.io/post/solving-colored-japanese-crosswords-with-the-speed-of-light/#what-decreases-the-execution-time).
