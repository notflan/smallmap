# smallmap
A small table map using single byte key indecies. Designed for maps with tiny keys.

Pages are stored as 256 entry key-value arrays which are indexed by the byte key index. The key is compared for collision check and on collision the next page is checked or inserted if needed.
`smallmap` does not ever need to allocate more than 1 page for types which all invariants can be represented as unique bytes.

## Usage
The API is a similar subset to `HashMap`, containing the same `insert`, `get`, and `entry` functions:

``` rust
fn max_char(chars: &str) -> (char, usize)
{
    let mut map = Map::new();
    for x in chars.chars() {
		*map.entry(x).or_insert(0usize) += 1;	
    }

	map.into_iter().max_by_key(|&(_, v)| v).unwrap_or_default()
}
```

## Use cases
Designed for instances where you want a small map with relatively trivial keys (e.g. primitive type).
Performance can greately outpace hash-based by an order of magnitude or more in these cases.

### Maybe use if

* You have small keys
* Your map is not at risk of Denial of Service attacks.
* Your keys will have a lot of collisions

### Don't use if

* You have complex keys
* Denial of service is a concern
* Your map will contain a large volume of entries
* Your keys may have a large number of collisions when represented as `u8`.


# Benchmarks
Some crude and basic benchmarks

## char

| Which           | ns/iter |
|-----------------|---------|
| `HashMap`       | 16      |
| `smallmap::Map` | 7       |

## Iterating a string's chars and counting each

| Which           | ns/iter |
|-----------------|---------|
| `HashMap`       | 8,418   |
| `BTreeMap`      | 9,742   |
| `smallmap::Map` | 4,416   |

## u8
| Which           | ns/iter |
|-----------------|---------|
| `HashMap`       | 15      |
| `smallmap::Map` | 2       |

# License
MIT licensed
