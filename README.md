# Ultimate nonograms solver written in Rust language.

```
$ wget -qO- https://webpbn.com/export.cgi --post-data "id=32480&fmt=nin&go=1" | cargo run
      Finished dev [unoptimized + debuginfo] target(s) in 0.06s
       Running `target/debug/nonogrid`
# # # # # #                                         6 1
# # # # # #     1                                   5 6 7 6 7   8           6   8 1                             1
# # # # # #     2               1             1 8   4 5 5 5 5 9 2 9 7 6   1 136 116 7 7   1   101               2
# # # # # #   1 3 3   3         1015  1 9 7 7 6 236 5 4 4 4 4 5 5 5 6 7 7 6 9 117 9 5 2 9 7 8 4 10        3   3 3 1
# # # # # #   1 3 151 21  1 33132 111138232323231 237 5 5 5 5 5 5 7 101322237 8 1 6 5 5 5 131 132 13331   211 153 1
# # # # # #   1 2 1 222 27302 14113 142 109 8 109 9 2 8 6 7 8 7 6 7 8 7 6 8 2 9 9 108 9 102 153 14152 30272 221 2 1
            . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
          1 . . . . . . . . . . . . . . . . . . . . . . . . . . ⬛ . . . . . . . . . . . . . . . . . . . . . . . . . .
      2 3 2 . . . . . . . . . . . . . . . . . . . . . ⬛ ⬛ . . ⬛ ⬛ ⬛ . . ⬛ ⬛ . . . . . . . . . . . . . . . . . . . . .
      2 9 2 . . . . . . . . . . . . . . . . . ⬛ ⬛ . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . ⬛ ⬛ . . . . . . . . . . . . . . . . .
          17. . . . . . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . . . . . .
      2 212 . . . . . . . . . . . . . ⬛ ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ ⬛ . . . . . . . . . . . . .
          25. . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . .
      2 252 . . . . . . . . . . ⬛ ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ . . . . . . . . . .
          31. . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . .
2 112 2 112 . . . . . . . ⬛ ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ . ⬛ ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ . . . . . . .
      113 11. . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . ⬛ ⬛ ⬛ . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . .
      9 1 9 . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . ⬛ . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . .
    1 8 8 1 . . . . . ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ . . . . .
        1111. . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . .
        9 9 . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . .
    1 277 1 . . . ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ . . .
        3310. . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
        338 . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . .
        328 . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . .
      9 249 . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . .
9 1 7 102 9 . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ .
    107 9 10. . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
    8 7 7 8 . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . .
    7 7 8 7 . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
    8 7 8 8 . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ .
      7 237 . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
      6 226 . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . .
      7 207 . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
      8 198 . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ .
    7 7 8 7 . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
    6 7 7 8 . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . .
    8 7 8 10. . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
    8 7 8 11. . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . .
        2222. . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . .
        2120. . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . .
        2120. . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . .
        2119. . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . .
    2 19162 . . . ⬛ ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ ⬛ . . .
        8 8 . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . .
        1111. . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . .
    1 13131 . . . . . ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ . . . . .
    111 1 11. . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ . . . . . . . . . . . . . ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . .
    1 12121 . . . . . . . ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ . . . . . . .
        1313. . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . . . . . . . .
      2 292 . . . . . . . . ⬛ ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ . . . . . . . .
      1 311 . . . . . . . . ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ . . . . . . . .
      2 252 . . . . . . . . . . . ⬛ ⬛ . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . ⬛ ⬛ . . . . . . . . . . .
      1 251 . . . . . . . . . . . ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ . . . . . . . . . . .
      2 192 . . . . . . . . . . . . . ⬛ ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ . . . . . . . . . . . . .
  1 2 112 1 . . . . . . . . . . . . . ⬛ . . . ⬛ ⬛ . . ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ ⬛ . . ⬛ ⬛ . . . ⬛ . . . . . . . . . . . . .
1 2 3 3 2 1 . . . . . . . . . . . . . . . . . . ⬛ . ⬛ ⬛ . ⬛ ⬛ ⬛ . ⬛ ⬛ ⬛ . ⬛ ⬛ . ⬛ . . . . . . . . . . . . . . . . . .
    1 1 1 1 . . . . . . . . . . . . . . . . . . . . ⬛ . . . ⬛ . . . ⬛ . . . ⬛ . . . . . . . . . . . . . . . . . . . .
            . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .

```

## Features

- solves binary (blank-and-white) and colored (<32 colors) nonograms;

- supports wide variety of formats:
  - own TOML-based format ([example](examples/hello.toml)) (with `ini` feature);
  - [webpbn](https://webpbn.com)-s primary [XML format](https://webpbn.com/pbn_fmt.html) (with `xml` feature);
  - some of the other formats that can be [exported from webpbn](https://webpbn.com/export.cgi):
    _faase, ish, keen, makhorin, nin, olsak, ss, syro_. All of them, except _olsak_,
    supports only black-and-white puzzles;
  - encoded format of https://nonograms.org.

- combines several solving methods to achieve speed for various puzzle types:
  - very simple puzzles are solved line-by-line (`line` + `propagation`);
  - if the puzzle cannot be solved, the `probing` phase begins, where assumptions
  are made about every unsolved cell following by the analysis of the impact they bring;
  - if the puzzle not solvable even here, the searching algorithms are enabled:
  by default `backtracking` is used that colors a cell, then another one, and go on,
  until the solution(s) is found. But there is another option (with `sat` feature):
  special SAT-solver, that uses the results of previous phases to more effectively
  explore the solution space.


By default the `--features="clap std_time env_logger ini"` are enabled but you can disable almost anything
to speed up and/or shrink the size of the binary.


### Arguments parsing

To support command-line arguments, the `clap` feature is enabled by default.
You can disable it, but then you will not able to set solving timeout or maximum number of solutions to find.
It also can be disabled when using the solver as a library in another projects,
[e.g.](https://github.com/tsionyx/nono/blob/8e2f8f27/Cargo.toml#L19)


### Timeout (std_time)

By default you can provide the `--timeout` option to stop backtracking after reaching the specified time limit.
You can disable this feature (`std_time`) and the timeout option will simply be ignored.


### Logging support

Logging is possible if you provide environment variable `RUST_LOG=nonogrid=<log_level>`.
For example, in the [benchmarks script](benches/batch.sh), the `RUST_LOG=nonogrid=warn` used
to inspect the intermediate results of solving. You can disable the option by **not** providing
the `--features=env_logger` while building.


### TOML puzzles parsing support

[My custom TOML-based format](examples/hello.toml) is supported by default via feature `ini`.
It can be disabled when using the solver as a library in another projects,
[e.g.](https://github.com/tsionyx/nono/blob/8e2f8f27/Cargo.toml#L19)


### SAT

By default, the backtracking algorithm used for solving hard puzzles.
The feature `sat` allows to use the [SAT](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
solver for such a job.
The most of hard puzzles are solved significantly faster with this option.

The latest benchmarks show that the SAT-solver is very effective
for the hardest webpbn puzzles (actually, only two puzzles are found
that solves longer than an hour: [25820](https://webpbn.com/25820)
and [26520](https://webpbn.com/26520)).


### XML puzzles parsing support

The [Jan Wolter's XML format](https://webpbn.com/pbn_fmt.html) supported via feature `xml`.
You can enable it by building with the `--features=xml`.


### Colored nonograms

You can enable the feature `colored` to allow to print colored nonograms with real terminal colors:

```
wget -qO- https://webpbn.com/export.cgi --post-data "fmt=olsak&go=1&id=2192" |
cargo run --no-default-features --features=colored
```


### Logging

To support pretty formatted logs the `env_logger` crate is enabled by default.
As alwasys, you can disable it by skipping one in the list of features.


### HTTP client

Solved puzzles can be automatically downloaded from the Internet with the `reqwest` library
but it requires too many dependencies and increases compile time, so it's optional by default.
Enable it as simple as:

```
cargo run --features=web,xml -- --webpbn 5933
```


### Threading

By default, the solver and all the algorithms are single-threaded. To use the solver's structures
in multi-threaded environment, provide the `threaded` feature. In essense, this feature
replaces every occurence of `Rc/RefCell` with `Arc/RwLock`.


### Probing tweaking

When the 'logical' solving (`line/propagation`) gets stuck, the `probing` phase starting which tries every variant
for every unsolved cells. It does this by calculating the priority for each cell:

```
P = N + R + C,

where 0<=N<=4 - number of neighbours which are solved cells or puzzle edges.
For example, the cell which has all 4 heighboring cells solved, has N = 4.
The upper left cell of the puzzle without any neighbours solved, has N = 2,
since it has 2 edges of the puzzle.

0<=R<=1 - row solution rate, the ratio of solved cells in the row to total number of cells (width)
0<=C<=1 - column solution rate, the ratio of solved cells in the column to total number of cells (height)
```

By default every cell with `P>=0` checked, but you can customize the threshold by specifying
the `LOW_PRIORITY` environment variable.

For example, running
```
LOW_PRIORITY=1 nonogrid puzzles/6574.xml
```

can be solved 3 times faster than standard way, by skipping the probing of cells with `P < 1`.


## Usage examples

### Solve locally saved puzzles from https://webpbn.com (XML format)

```
cargo build --features="xml"

# solve puzzle https://webpbn.com/2992
wget 'https://webpbn.com/XMLpuz.cgi?id=2992' -O 2992.xml
target/debug/nonogrid 2992.xml

# with pipe
wget -qO- 'https://webpbn.com/XMLpuz.cgi?id=2992' | target/debug/nonogrid
```

### Solve puzzles from https://webpbn.com (with embedded HTTP-client)

```
cargo build --features="web,xml"

# solve puzzle https://webpbn.com/5933
target/debug/nonogrid -w 5933
```

### Solve locally saved puzzles from https://nonograms.org

```
cargo build

# solve puzzle https://webpbn.com/2992
wget -qO- 'https://www.nonograms.org/nonograms/i/2581' | grep 'var d=' > 2581.js
target/debug/nonogrid 2581.js

# with pipe
wget -qO- 'https://www.nonograms.org/nonograms/i/2581' | target/debug/nonogrid
```

### Solve puzzles from https://nonograms.org (with embedded HTTP-client)

```
cargo build --features="web"

# solve puzzle https://www.nonograms.org/nonograms/i/13588
target/debug/nonogrid -o 13588

# solve puzzle https://www.nonograms.org/nonograms2/i/10270
target/debug/nonogrid -o 10270
```

### Solve other formats

#### [TOML format](examples)

```
cargo build

target/debug/nonogrid examples/hello.toml
```

#### Webpbn's [exportable formats](https://webpbn.com/export.cgi)

```
wget -qO- https://webpbn.com/export.cgi --post-data "fmt=syro&go=1&id=2040" |
cargo run --no-default-features
```


## Development

### See the INFO logs and unfold backtrace on panic

```
RUST_BACKTRACE=1 RUST_LOG=nonogrid=info cargo run -- examples/hello.toml
```
