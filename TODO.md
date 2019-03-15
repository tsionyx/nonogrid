### Comparison with python solver (simple line solver)

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


### Comparison with python solver (probing solver)

| puzzle name | contradictions (python/rust) | python (PyPy), sec | rust (debug), sec | rust (release), sec | gain, times |
|-------------|------------------------------|-------------------:|------------------:|--------------------:|:-----------:|
| -b MLP      | 429/?                        | 3.200..4.617       | 3.404..3.982      | 0.122..0.162        | 19..38      |
| -p 2040     | 204/?                        | 1.922..2.349       | 2.384..3.500      | 0.095..0.124        | 15..25      |


- remove contradiction mode?

- play with CHOOSE_STRATEGY and the order of colors for given points

- add backtracking; compare with python (all the others)

- colored; compare with python (all the colored)

- web-assembly and JS rendering(SVG?)

- other formats (nonograms.org)?
