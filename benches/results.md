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

### Sqrt strategy, two caches, more than 10 seconds

| puzzle_id | solve time, sec | depth, levels | solutions | final rate |
|-----------|----------------:|--------------:|:---------:|-----------:|
| _3867_    | 126.07          | 21            | 2         | 0.8710     |
| **8098**  | 17.30           | 8             | 1         | 1          |
| **9892**  | 110.68          | 22            | 2         | 0.4385     |
| **12548** | +               | 46            | 0         | 0.1856     |
| 13480     | +               | 40            | 0         | 0.1263     | FIXME: why starting rate r=12.68 in Python?
| 16900     | 136.73          | 30            | 2         | 0.4712     | FIXME: why starting rate r=48.06 in Python?
| **18297** | 451.86          | 14            | 2         | 0.1349     |
| **22336** | +               | 12            | 0         | 0.4313     |
| 25385     | +               | 43            | 0         | 0.2656     | FIXME: Sometimes it solves for hours.
| 25471     | 14.44           | 10            | 1         | 1          |
| 25820     | +               | 33            | 0         | 0.0552     |
| 26520     | +               | 46            | 0         | 0.0836     |
| 27174     | 59.85           | 8             | 1         | 1          |
| 30509     | 25.70           | 98            | 2         | 0          |
| 30681     | 95.64           | 30            | 2         | 0.5464     |
| 32291     | 56.94           | 6             | 2         | 0.4288     |


# Colored puzzles

### Sqrt strategy, two caches, more than 10 seconds

| puzzle_id | solve time, sec | depth, levels | solutions | final rate | colors (w/o blank) |
|-----------|----------------:|--------------:|:---------:|-----------:|--------------------|
| **672**   | +               | 61            | 0         | 0.7638     | 3                  |
| 3085      | 1180.97         | 25            | 2         | 0.8888     | 3                  |
| 3114      | 214.59          | 18            | 2         | 0.8159     | 3                  |
| **3149**  | +               | 35            | 0         | 0.7863     | 4                  |
| **3620**  | +               | 22            | 0         | 0.8581     | 4                  |
| 10585     | +               | 27            | 0         | 0.8585     | 4                  |
| 11546     | 14.94           | 36            | 2         | 0.6069     | 4                  |
| 12831     | +               | 22            | 0         | 0.9075     | 4                  |
| 14717     | +               | 33            | 0         | 0.8398     | 3                  |
| 15101     | +               | 33            | 0         | 0.8762     | 4                  |
| 16552     | +               | 25            | 0         | 0.9640     | 4                  |
| 16838     | +               | 22            | 0         | 0.6915     | 2                  |
| 16878     | +               | 35            | 0         | 0.8524     | 2                  |
| 17045     | 117.05          | 24            | 2         | 0.9598     | 3                  |
| 18290     | 2498.38         | 34            | 2         | 0.3750     | 4                  |
| 25158     | 18.01           | 18            | 2         | 0.8551     | 4                  |
| 25540     | +               | 28            | 0         | 0.8284     | 2                  |
| 26810     | +               | 29            | 0         | 0.7953     | 4                  |
| 29436     | +               | 22            | 0         | 0.8440     | 4                  |
| 29826     | 10.27           | 21            | 2         | 0.8673     | 3                  |
| 30640     | 754.94          | 28            | 2         | 0.9678     | 4                  |
| 31114     | 9.14            | 27            | 2         | 0.1850     | 2                  |
| 31697     | +               | 107           | 0         | 0.6100     | 2                  | FIXME: reached ~18Gb of RAM and was interrupted
| 31812     | 25.15           | 465           | 2         | 0.7273     | 3                  |


`+` means the solving time exceeds 1 hour and was interrupted (`--timeout=3600`)

**Bold** puzzles are from http://webpbn.com/survey/ (_italic_ puzzles are mentioned there too).


# http://www.nonograms.org puzzles

23258 puzzles run. All the puzzles are line solvable and has single solution.

## Distribution of solve times

```
$ less batch-norg.log | grep 'Total' | awk '{print $2}' | sort -r | uniq -c
      1 0.12
      2 0.11
      1 0.10
      1 0.08
      3 0.07
      4 0.06
      3 0.05
     30 0.04
     40 0.03
    207 0.02
    890 0.01
  22222 0.00
```

## Top times (more than 0.05 sec)

| puzzle_id | solve time, sec | colors (w/o blank) |
|-----------|----------------:|--------------------|
| 2617      | 0.00            | 6
| **4462**  | 0.07            | 3
| **9596**  | 0.11            | 10
| 9664      | 0.06            | 1 (black)
| 10509     | 0.04            | 1 (black)
| 10548     | 0.04            | 1 (black)
| 18305     | 0.04            | 8
| 18417     | 0.06            | 1 (black)
| 19043     | 0.06            | 1 (black)
| 20689     | 0.07            | 4
| 21251     | 0.11            | 1 (black)
| 21259     | 0.07            | 1 (black)
| 21272     | 0.10            | 1 (black)
| 21424     | 0.06            | 10
| 21553     | 0.12            | 5
| 21886     | 0.04            | 3
| 22118     | 0.08            | 10
| 22326     | 0.05            | 1 (black)
| 23343     | 0.03            | 7


**Bold** puzzles also found in [this C++ solver post](
https://izaron.github.io/post/solving-colored-japanese-crosswords-with-the-speed-of-light/#what-decreases-the-execution-time).
