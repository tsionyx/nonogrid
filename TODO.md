## Experiments
- reduce board by removing solved lines:
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
- SAT:
  - ignore solved lines (reduce the number of block position variables)
  - review and rustdoc
- initial clues validation (validate_descriptions_size, validate_colors)
- port SvgRenderer from pynogram
- [blotted puzzles](https://webpbn.com/19407)
- other formats:
  - https://github.com/Izaron/Nonograms/raw/master/puzzles
- docs.rs (+from README)
- SVG, XML, [cmd](https://docs.python.org/3/library/cmd.html)


## Refactoring
- `solver::run_with_default`
- replace Color with ColorScheme, move Block inside ColorScheme
- rewrite backtracking `search` with Result
- remove probing and propagation structures: make it `impl Probe for Board`
