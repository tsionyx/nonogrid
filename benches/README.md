# Performance comparison

The basis of the work done by Jan Wolter in his [survey](https://webpbn.com/survey).

To better understand my own solver's ability I have used the same techniques and puzzles
to prepare a report. To adjust my own machine's performance, I ran several solvers
from the survey on the same puzzles - they marked with the prefix `_my` in the report.


## https://webpbn.com puzzles

### [Black-and-white](perf.csv)

```
bash benches/cmp.sh $PWD
```


### [Colored](perf-color.csv)

##### Comparison made with Wolter's solver only

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
# only 27898 of them are valid at 5 Nov 2021
nohup bash benches/batch.sh webpbn {1..35500} >benches/batch.log 2>&1 &
bash benches/batch.sh stat benches/batch.log 9 --details
```

Most of the hardest puzzles are not unique, so we need to find at least two solutions to prove it.
So, the solution ends when the second solution is found or if we found proof that the first solution is the only.
However, for puzzles that take hours to solve, it can be helpful to know when
the first solution has been found, and therefore, how long will it take to find the second one.

Out of 27898 puzzles:
- 23214 puzzles are line solvable (with propagation of changes):
  - `less benches/batch.log | grep -B1 'Total:' | grep -oP '\d+ points' | sort -nr | uniq -c`
  - 0 is the minimum number of points (empty descriptions set of puzzles: #845, #7693, etc.);
  - 13517 is the maximum number of points to solve (#31651);

- 3956 are solvable with the probing (unique solution found):
  - `less benches/batch.log | grep -B1 'Total:' | grep 'Cache rows' | wc -l`

- 728 cannot be solved with probing and was tried to solve with SAT:
  - `less benches/batch.log | grep -B1 'Total:' | grep -oP '\d+-th solution found' | sort -nr | uniq -c`
  - 88 of them has unique solution (#25820 is one of them, and it is the hardest puzzle to solve, see the table below);
  - 104 has two solutions;
  - 227 has more than 1000 solutions;
  - #32722 has the highest complexity of probing (`less benches/batch.log | grep -oP 'hits=\d+' | grep -oP '\d+' | sort -nr | head`);


#### with SAT solver

```
cargo build --release --no-default-features --features="args std_time logger sat"
```

##### black-and-white (>=10 seconds)

| puzzle_id | solve time, sec | after first, sec | solutions | SAT variables | SAT clauses |
|-----------|----------------:|-----------------:|----------:|--------------:|------------:|
| 9892*     |              28 |                0 |   >10_000 |        13_887 |     425_270 |
| 10088*    |              10 |                0 |   >10_000 |        23_483 |   1_025_585 |
| 12548*    |             141 |                0 |   >10_000 |        13_685 |     503_999 |
| 16900     |              15 |                0 |   >10_000 |        32_295 |   1_751_050 |
| 18297*    |              11 |                0 |         3 |         9_708 |     275_166 |
| 19080     |             398 |                0 |   >10_000 |        24_673 |   1_272_743 |
| 22336*    |             269 |                0 |   >10_000 |        67_137 |   5_461_726 |
| 25385     |             295 |                0 |   >10_000 |        21_334 |     923_394 |
| 25588     |             187 |                0 |   >10_000 |        21_017 |     993_331 |
| 25820     |         103_996 |           61_619 |         1 |        43_668 |   2_948_833 |
| 26520     |          41_161 |                0 | >10_000 (294_182 seconds) | 29_223 | 1_696_163 |
| 30532     |              13 |                0 |   >10_000 |        15_212 |     444_144 |
| 30654     |           1_243 |               56 |   >10_000 |        33_384 |   1_138_340 |
| 32013     |              13 |                0 |   >10_000 |         9_468 |     297_855 |
| 32291     |              61 |                0 |         2 |        14_421 |     543_978 |
| [Knotty**](https://webpbn.com/survey/puzzles/) | 42_931 | 0 | >=2 | 11_076 |     230_271 |
| [Meow**](https://webpbn.com/survey/puzzles/)   | ++ | N/A | ? |     35_036 |   1_872_216 |
| [Faase**](https://webpbn.com/survey/puzzles/)  | ++ | N/A | ? |     89_918 |   9_968_433 |

`++` - search was not completed after 1 week of training (>0.6M seconds)

##### colored (>=10 seconds)

| puzzle_id | solve time, sec | after first, sec | solutions | SAT variables | SAT clauses | colors (w/o blank) |
|-----------|----------------:|-----------------:|----------:|--------------:|------------:|-------------------:|
| 672*      |              23 |                1 |   >10_000 |        32_027 |   1_483_859 | 3
| 2498*     |              42 |                0 |   >10_000 |        42_352 |   2_259_671 | 4
| 3114      |              29 |                0 |   >10_000 |        28_886 |     995_422 | 3
| 9786      |              24 |                0 |   >10_000 |        16_094 |     581_210 | 2
| 10585     |             275 |                0 |   >10_000 |        28_224 |     829_213 | 4
| 16838     |             260 |                1 |   >10_000 |        70_240 |   6_752_766 | 2
| 25158     |              35 |                0 |   >10_000 |        20_433 |     660_340 | 4
| 26810     |              28 |                0 |   >10_000 |        45_695 |   2_351_897 | 4
| 34250     |             252 |                0 |   >10_000 |        37_899 |   1_996_812 | 4

#### with custom backtracking

```
cargo build --release --no-default-features --features="args std_time logger"
```

##### black-and-white (>=10 seconds)

| puzzle_id | solve time, sec | depth reached, levels  |
|-----------|----------------:|-----------------------:|
| 3867*     |             149 |                     21 |
| 8098*     |              19 |                      8 |
| 9892*     |             103 |                     22 |
| 12548*    |               + |                     46 |
| 13480     |               + |                     41 |
| 16900     |              83 |                     30 |
| 18297*    |             326 |                     14 |
| 22336*    |               + |                     12 |
| 25385     |               + |                     40 |
| 25820     |               + |                     35 |
| 26520     |               + |                     50 |
| 27174     |              36 |                      7 |
| 30509     |              24 |                     98 |
| 30654     |               3 |                    372 |
| 30681     |              67 |                     30 |
| 32291     |              36 |                      6 |
| [Knotty**](https://webpbn.com/survey/puzzles/) | + | 27 |
| [Meow**](https://webpbn.com/survey/puzzles/)   | + | 28 |
| [Faase**](https://webpbn.com/survey/puzzles/)  | + | 43 |

##### colored (>=10 seconds)

| puzzle_id | solve time, sec |  depth reached, levels  | colors (w/o blank) |
|-----------|----------------:|------------------------:|-------------------:|
| 672*      |               + |                      70 | 3
| 3085      |             991 |                      25 | 3
| 3114      |             173 |                      18 | 3
| 3149*     |               + |                      37 | 4
| 3620*     |               + |                      22 | 4
| 4445*     |              23 |                      13 | 3
| 7778      |              33 |                      42 | 2
| 10585     |               + |                      27 | 4
| 11546     |              15 |                      36 | 4
| 12831     |               + |                      22 | 4
| 14717     |               + |                      33 | 3
| 15101     |               + |                      33 | 4
| 16552     |               + |                      26 | 4
| 16838     |               + |                      37 | 2
| 16878     |               + |                      35 | 2
| 17045     |              77 |                      23 | 3
| 18290     |           1_846 |                      34 | 4
| 26810     |               + |                      29 | 4
| 29436     |               + |                      24 | 4
| 29826     |              22 |                      21 | 3
| 30640     |             667 |                      28 | 4
| 31812     |              27 |                     523 | 3
| 33745     |               + |                      47 | 4
| 34250     |               + |                      16 | 4
| 34722     |               + |                      26 | 4

`+` - search was not completed after 1 hour

`*` - puzzles from http://webpbn.com/survey/.

`**` - puzzles are not in public access but can be downloaded at https://webpbn.com/survey/puzzles/


## Export puzzle $ID in every supported format

```
for fmt in $(curl -s https://webpbn.com/export.cgi | grep -oP 'name="fmt" value="\K([^"]+)'); do
    echo "======== Downloading puzzle $ID with format $fmt ========"
    curl -s https://webpbn.com/export.cgi --data "id=$ID&fmt=$fmt&go=1" > ${ID}.${fmt}
done
```

The colored puzzles are only supported in the following formats:
- _xml_: colored block represented as _'color="NAME"'_;
- _pnm_;
- _xls_?;
- _olsak_: colored block represented as the letters for every color: _'n:%  #00B000   green'_;
- _cwc_: colored blocks represented as the numbers (1..N)
- _crossa_: colored blocks represented as the numbers (1..N)

The blotted puzzles (not implemented yet) are only supported in the following formats:
- _xml_: blotted block represented as _'0'_;
- _csv_: blotted block represented as _'#'_.



## http://www.nonograms.org puzzles

49686 puzzles run. All the puzzles are line solvable and has single solution.

### Distribution of solve times

```
$ nohup bash benches/batch.sh nonograms.org {1..50000} 2>&1 > benches/batch-norg.log &
$ less benches/batch-norg.log | grep 'Total' | awk '{print $2}' | sort -r | uniq -c
     1 0.16
     1 0.12
     1 0.09
     2 0.06
     9 0.05
    16 0.04
    72 0.03
   341 0.02
  2229 0.01
 47014 0.00
```

### Top 31 (>=0.04 sec)

| puzzle_id | solve time, sec | solve time   |    size | colors (w/o blank) |
|-----------|----------------:|:-------------|---------|-------------------:|
| 4462*     |       0.05-0.08 | ###+         | 140x150 |   3
| 9596*     |       0.05-0.06 | ###          | 139x166 |  10
| 15118     |            0.04 | ##           | 140x130 |   2 (black and white)
| 17921     |       0.04-0.05 | ##+          |  110x70 |   2 (black and white)
| 18417     |       0.04-0.05 | ##+          | 148x120 |   2 (black and white)
| 19043     |            0.04 | ##           |   96x96 |   2 (black and white)
| 20689     |            0.05 | ##+          | 100x107 |   4
| 21251     |       0.03-0.05 | ##+          | 175x140 |   2 (black and white)
| 21259     |       0.04-0.05 | ##+          | 150x140 |   2 (black and white)
| 21272     |       0.06-0.09 | ###++        | 149x153 |   2 (black and white)
| 21553     |       0.05-0.08 | ###+         | 200x200 |   5
| 22118     |       0.04-0.05 | ##+          | 145x200 |  10
| 22340     |            0.04 | ##           |   97x97 |   2 (black and white)
| 31862     |       0.04-0.05 | ##+          |  150x93 |   4
| 33190     |       0.05-0.06 | ###          | 125x200 |   9
| 36040     |       0.05-0.06 | ###          | 120x120 |   3
| 39385     |       0.03-0.05 | ##+          |  80x120 |   9
| 39792     |            0.04 | ##           |  70x100 |   8
| 40330     |       0.04-0.05 | ##+          |   90x94 |   4
| 41095     |       0.05-0.06 | ###          | 185x195 |   7
| 41103     |       0.04-0.05 | ##+          | 158x200 |  10
| 43693     |       0.04-0.05 | ##+          |  200x15 |   2 (black and white)
| 44129     |       0.04-0.05 | ##+          |  194x33 |   2 (black and white)
| 45290     |       0.04-0.05 | ##+          | 130x160 |   2 (black and white)
| 47574(ru) |       0.03-0.05 | ##+          | 128x180 |   2 (black and white)
| 47617(ru) |       0.12-0.17 | ######+++    | 140x200 |   2 (black and white)
| 47623(ru) |       0.05-0.07 | ###+         | 160x190 |   2 (black and white)
| 47648(ru) |       0.15-0.24 | ########++++ | 150x200 |   2 (black and white)
| 47650(ru) |       0.09-0.14 | ####+++      | 160x180 |   2 (black and white)
| 48172     |       0.05-0.07 | ###+         | 200x120 |   2 (black and white)
| 48723     |       0.06-0.07 | ###+         | 100x171 |   2 (black and white)


`*` - puzzles also mentioned in [this C++ solver post](
https://izaron.github.io/post/solving-colored-japanese-crosswords-with-the-speed-of-light/#what-decreases-the-execution-time).
