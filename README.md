# Breadth-first `zip`

Property-tested and verified to return every possible value exactly one in a strictly increasing sum of indices.

## Why?

For whenever you have a multiple iterators and want to cover every possible value from overall smallest to largest.

E.g. for three instances of `0..3`, you'd get this:
```
0 0 0 # sum = 0
0 0 1 # sum = 1
0 1 0
1 0 0
0 0 2 # sum = 2
0 1 1
0 2 0
1 0 1
1 1 0
2 0 0
0 1 2 # sum = 3
0 2 1
1 0 2
1 1 1
1 2 0
2 0 1
2 1 0
0 2 2 # sum = 4
1 1 2
1 2 1
2 0 2
2 1 1
2 2 0
1 2 2 # sum = 5
2 1 2
2 2 1
2 2 2 # sum = 6
```
Inputs can be any non-empty iterator, even combining different sizes.
