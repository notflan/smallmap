//! Space-efficient small maps and sets
//!
//! To make an entirely space efficient `Map` (i.e. size of each page is 256 bytes, there is never more than 1 page), the following must be true:
//!
//! * The key must be 8 bits wide and subject to the *null pointer optimisation*
//! * The value must be a ZST.
//!
//! This leaves pretty much only `std::num::NonZeroU8` and `std::num::NonZeroI8` as entirely space-efficient key candidates.
//! The restriction on values also means the only entirely space-efficient smallmaps are sets, enable to encode only if a key is present, with no extra information. (See `std::collections::HashSet`).
use super::*;

/// A set of only non-zero bytes.
///
/// This type is entirely space efficient and will only ever allocate `256` bytes of memory.
pub type NonZeroByteSet = Set<core::num::NonZeroU8>;

/// A set of non-zero signed 8-bit integers.
///
/// This type is entirely space efficient and will only ever allocate `256` bytes of memory.
pub type NonZeroI8Set = Set<core::num::NonZeroI8>;

/// A set of non-zero unsigned 8-bit integers.
///
/// This type is entirely space efficient and will only ever allocate `256` bytes of memory.
pub type NonZeroU8Set = NonZeroByteSet;

#[cfg(test)]
mod tests
{
    use super::*;

    /// Returns the currently allocated space of `map`.
    pub fn space_of<K, V>(map: &Map<K,V>) -> usize
    {
	map.internal_size_bytes()
    }

    /// Check the allocation size of types in this module
    mod allocsz {
	use super::*;

	/// Assert the allocated space (in bytes) of `map` is equal to `expected`.
	pub fn assert_space_of<T, K, V>(map: T, expected: usize)
	    where T: std::borrow::Borrow<Map<K,V>>
	{
	    let sz = space_of(map.borrow());
	    assert_eq!(sz, expected, "unexpected full allocated size of type {}: expected {expected}, got {sz}.", std::any::type_name::<Map<K,V>>());
	}

	/// Create a test that asserts the allocation size of a type is equal to a specific number of bytes
	///
	/// # Usage
	/// ```
	/// # use super::*;
	/// size_test!(non_zero_byte_set, NonZeroByteSet, 256); // Creates a test function, named `non_zero_byte_set`, that asserts the type `NonZeroByteSet` allocates exactly 256 bytes.
	/// ```
	macro_rules! size_test {
	    ($name:ident, $type:ty, $num:expr) => {
		#[test]
		fn $name() {
		    assert_space_of(<$type>::new(), $num);
		}
	    }
	}

	size_test!(non_zero_byte_set, NonZeroByteSet, 256);
	size_test!(non_zero_u8_set, NonZeroU8Set, 256);
	size_test!(non_zero_i8_set, NonZeroI8Set, 256);
    }
}
