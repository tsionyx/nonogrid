# Performance comparison

The basis of the work was done by Jan Wolter in his [survey](https://webpbn.com/survey).

To better understand my own solver's ability I have used the same techniques and puzzles
to prepare a report. To adjust my own machine's performance, I ran several solvers
from the survey on the same puzzles - they marked with the prefix `_my` in the report.

## [Black-and-white](perf.csv)

### How did I run the puzzles

Puzzles were exported from the [export page](https://webpbn.com/export.cgi) and then
were run with these simple bash loops:

##### export

```
id=24
curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=syro&go=1" > puzzles/$id.syr
curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=nin&go=1" > puzzles/$id.nin
curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=cwd&go=1" > puzzles/$id.cwd
```

##### Syromolotov

```
for i in $(ls puzzles/*.syr); do
    echo $i;
    time jsolver-1.2-src/jsolver -n 2 $i;
done
```

##### Wolter

```
for i in $(ls puzzles/*.nin); do
    echo $i;
    time pbnsolve-1.09/pbnsolve -u -x1800 $i;
done
```

##### BGU

```
for i in $(ls puzzles/*.nin); do
    echo $i;
    time java -jar bgusolver_cmd_102.jar -file $i -maxsolutions 2 -timeout 1800;
done
```

##### Tamura/Copris

```
for i in $(ls puzzles/*.cwd); do
    echo $i;
    time scala-2.10.7/bin/scala -cp copris-nonogram-v1-2.jar nonogram.Solver $i;
done
```

##### nonogrid

Default XML-format is too verbose and can affect the performance for very easy puzzle,
so it is better to use my TOML format for it.

The easiest format to convert into my format is (for black-and-white puzzles)
is Syromolotov's format. This simple script can do the trick:

```
#!/bin/sh -e

printf '[clues]\nrows="""'
cat $1 | sed 's/#/"""\ncolumns="""/' | sed \$d
```

So we run it like this:

```
for i in $(ls puzzles/*.syr); do
    echo $i;
    time target/release/nonogrid -b <(sh benches/syr2toml.sh $i) --timeout=1800 --max-solutions=2
done
```


## [Colored](perf-color.csv)

##### Comparison was made with Wolter's solver only

```
for i in 47 220 1503 2257 4940 5193 2684 2073 4364 2817 4809 2814 3149 4445 2984 2498 3620 672; do
    echo $i;
    time pbnsolve-1.09/pbnsolve -u -x1800 puzzles/$i.xml;
done
```


## [Memory consumption (Mbytes)](memory.csv)

```
#!/bin/bash -e
# Example: `bash check-limits.sh 23 50000`

ulimit -Sv $2
for i in {1..300}; do
  #echo $i
  target/release/nonogrid -p puzzles/$1.xml --timeout=3600 --max-solutions=2 >/dev/null
done
```

# Results
