//! Space-efficient small maps and sets
//!
//! To make an entirely space efficient `Map` (i.e. size of each page is 256 bytes, there is never more than 1 page), the following must be true:
//!
//! * The key must be 8 bits wide and subject to the *null pointer optimisation*
//! * The value must be a ZST.
//!
//! This leaves pretty much only `NonZeroU8` and `NonZeroI8` as entirely space-efficient key candidates.
//! The restriction on values also means the only entirely space-efficient smallmaps are sets, enable to encode only if a key is present, with no extra information. (See `std::collections::HashSet`).
use super::*;

/// A set of only non-zero bytes.
///
/// This type is entirely space efficient and will only ever allocate `256` bytes of memory.
pub type NonZeroByteSet = Map<std::num::NonZeroU8, ()>;
