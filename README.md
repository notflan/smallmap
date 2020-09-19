# smallmap
A small byte sized table map.

Designed for instances where you want a map with small keys (e.g. primitive).
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
| `HashMap`       | 126,418 |
| `smallmap::Map` | 9,416   |

## u8
| Which           | ns/iter |
|-----------------|---------|
| `HashMap`       | 15      |
| `smallmap::Map` | 2       |

# License
 Dunno yet. Maybe MIT haven't decided...
