# smallmap
A small byte sized table map. (Currently *requires* nightly).

Designed for instances where you want a small map with relatively trivial keys (e.g. primitive type).
Performance greately outpaces hash-based maps in these cases.



# Benchmarks
Some rudamentary benchmarks

## char

| Which           | ns/iter |
|-----------------|---------|
| `HashMap`       | 16      |
| `smallmap::Map` | 7       |

## Iterating a string's chars and incrementing values

| Which           | ns/iter |
|-----------------|---------|
| `HashMap`       | 65,418  |
| `smallmap::Map` | 9,416   |

## u8 (single table)
| Which           | ns/iter |
|-----------------|---------|
| `HashMap`       | 15      |
| `smallmap::Map` | 2       |

# License
 Dunno yet. Maybe MIT haven't decided...
