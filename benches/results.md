# Black-and-White puzzles

## Comparison with python solver

### Simple line solvable puzzles

| puzzle name | lines solved (python/rust) | python (PyPy), sec | rust (debug), sec | rust (release), sec | gain, times |
|-------------|----------------------------|-------------------:|------------------:|--------------------:|:-----------:|
| -b einstein | 1793/1679                  | 0.685..0.755       | 0.359..0.393      | 0.0114..0.0127      | 54..66      |
| -p 2992     | 2779/2634                  | 0.701..0.910       | 0.809..0.815      | 0.0214..0.0277      | 25..42      |
| -p 5933     | 3461/3192                  | 0.861..0.995       | 1.165..1.230      | 0.0313..0.0427      | 20..32      |
| -p 10564    | 2828/2914                  | 0.749..0.939       | 0.783..0.863      | 0.0257..0.0305      | 25..36      |
| -p 18845    | 1897/1366                  | 0.824..0.985       | 0.287..0.313      | 0.0082..0.0116      | 71..120     |
| -n 4438     | 3047                       | 1.061..1.216       | unimplemented     | unimplemented       | N/A         |
| -n 5114     | 5274                       | 1.940..2.137       | unimplemented     | unimplemented       | N/A         |
| -n 5178     | 3421                       | 1.146..1.380       | unimplemented     | unimplemented       | N/A         |
| -n 19043    | 4608                       | 1.043..1.286       | unimplemented     | unimplemented       | N/A         |


### Probing solver

| puzzle name | contradictions (python/rust) | python (PyPy), sec | rust (debug), sec | rust (release), sec | gain, times |
|-------------|------------------------------|-------------------:|------------------:|--------------------:|:-----------:|
| -b MLP      | 429/?                        | 3.200..4.617       | 3.404..3.982      | 0.122..0.162        | 19..38      |
| -p 2040     | 204/?                        | 1.922..2.349       | 2.384..3.500      | 0.095..0.124        | 15..25      |



## Hardest backtracking puzzles

### SAT solver, more than 10 seconds

| puzzle_id | solve time /
|-----------|------------/
| **9892**  | 26         /
| **12548** | 136        /
| 16900     | 12         /
| 19080     | 367        /
| **22336** | 263        /
| 25385     | 278        /
| 25588     | 185        /
| 25820     | 119271 (43412 for 1-st solution)
| 26520     | 47628      /
| 30532     | 12         /
| 30654     | 1148       /
| 32013     | 12         /
| 32291     | 58         /


# Colored puzzles

### SAT solver, more than 10 seconds

| puzzle_id | solve time, sec | colors (w/o blank) |
|-----------|----------------:|--------------------|
| **672**   | 44              | 3                  |
| **2498**  | 57              | 4                  |
| 3114      | 32              | 3                  |
| **4445**  | 24              | 3                  /
| 7541      | 55              | 4                  /
| 7778      | 14              | 2                  /
| 8337      | 33              | 4                  /
| 8880      | 17              | 4                  /
| 9786      | 53              | 2                  /
| 10585     | 435             | 4                  /
| 16838     | 695             | 2                  /
| 22027     | 160             | 4                  /
| 25158     | 41              | 4                  /
| 26810     | 73              | 4                  /
| 27097     | 77              | 4                  /
| 29469     | 15              | 2                  /
| 29826     | 24              | 3                  /
| 31812     | 39              | 3                  /

**Bold** puzzles are from http://webpbn.com/survey/.


# http://www.nonograms.org puzzles

28075 puzzles were run. All the puzzles are line solvable and has single solution.

## Distribution of solve times

```
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

## Top 6 (>=0.05 sec)

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
