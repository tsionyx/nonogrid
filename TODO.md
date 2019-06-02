## Experiments
- remove solved lines
- optimize solver cache: use bitset

- reducing board:
  - if there is a fully solved row (column), one can safely remove it (with 'comparing neighbours' check)
  - if the block position is fully revealed, one can replace that block with white (for the whole board).
  Consider applying 'comparing neighbours' procedure.
  - if only one color left after previous two procedures, then replace MultiColor with BinaryColor
    (see puzzles 672, 3085, 10585, 16552, 16878).
  - if one can determine if the cell belongs to the particular block
    one can replace this cell with 'space' and decrement the appropriate blocks.

  comparing neighbours:
  - if for every pair of cells in (previous, next) rows (columns) the intersection
    of possible colors (colors_previous & color_next) does not contain any colors except 'space'.
- play with CHOOSE_STRATEGY and the order of colors for given points


## Features
- web-assembly and JS rendering(SVG/Canvas/WebGL)
- other formats:
  - https://github.com/Izaron/Nonograms/raw/master/puzzles


## Optimizations
- remove contradiction mode?
- move dependencies into optional features (toml, xml, clap, colored)
- load puzzles in other formats (syr for black, olsak for multi) to reduce network latency


## Refactoring
- replace Color with ColorScheme, move Block inside ColorScheme
