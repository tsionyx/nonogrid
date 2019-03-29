## Experiments
- remove solved lines
- optimize solver cache: use bitset
- optimize color solver:
  - cache solution_rate

- reducing board:
  - if there is a fully solved row (column), one can safely remove it (with 'comparing neighbours' check)
  - if the block position is fully revealed, one can replace that block with white (for the whole board).
  Consider applying 'comparing neighbours' procedure.
  - if only one color left after previous two prodecures, then replace MultiColor with BinaryColor
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
  - http://nonograms.org
  - https://github.com/Izaron/Nonograms/raw/master/puzzles


## Optimizations
- line in LineSolver: need Rc?
- remove contradiction mode?
- do I need RefCell for cells? (use https://crates.io/crates/nalgebra or https://crates.io/crates/rulinalg)


## Refactoring
- replace Color with ColorScheme, move Block inside ColorScheme
