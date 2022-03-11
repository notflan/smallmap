//! # smallmap
//! A small table map with a byte sized key index.
//!
//! With a key type which all invariants can be represented as unique bytes, searching this map is a single index dereference.
//! With only a few bytes it is still very efficient.
//!
//! ## Usage
//! The API is a similar subset to `HashMap`, containing the same `insert`, `get`, and `entry` functions:
//!
//! ```
//! # use smallmap::Map;
//! fn max_char(chars: &str) -> (char, usize)
//! {
//!     let mut map = Map::new();
//!     for x in chars.chars() {
//! 	    *map.entry(x).or_insert(0usize) += 1;	
//!     }
//! 
//!     map.into_iter().max_by_key(|&(_, v)| v).unwrap_or_default()
//! }
//! ```
//!
//! ## Use cases
//! Designed for instances where you want a small map with small key types.
//! Performance greately outpaces complex hash-based maps in these cases.
//!
//! ###  When not to use
//! Generally don't use this if your key would have a lot of collisions being represents in 8 bits, otherwise it might be a faster alternative to hash-based maps. You should check yourself before sticking with this crate instead of `std`'s vectorised map implementations.

#![cfg_attr(nightly, feature(test))] 
#![cfg_attr(nightly, feature(drain_filter))] 
#![cfg_attr(nightly, feature(const_fn_trait_bound))] 
#![cfg_attr(nightly, feature(never_type))] 

#[cfg(nightly)] extern crate test;

const MAX: usize = 256;

use std::borrow::Borrow;

pub mod iter;
use iter::*;
pub mod entry;
pub use entry::Entry;

pub mod space;

pub mod primitive;
pub use primitive::Primitive;

mod init;

mod private {
    pub trait Sealed{}
}

/// A smallmap set.
///
/// Can be used to quickly insert or remove a key only, with no value; and can be used to see if this key is present.
///
/// Any map type with a zero-sized value is essentially a set.
pub type Set<T> = Map<T,()>;

/// A helper macro for creating `Map` instances with or without pre-set entries.
///
/// # Create empty map
/// With no parameters this just calls `Map::new()`.
/// ```
/// # use smallmap::*;
/// let map: Map<i32, i32> = smallmap!();
/// let map2: Map<i32, i32> = Map::new();
/// assert_eq!(map, map2);
/// ```
/// # Create with key-value pairs
/// You can specify some entries to pre-insert in the format `{key => value}`.
/// ```
/// # use smallmap::*;
/// let map = smallmap! {
///   {"Key" => 1},
///   {"Key two" => 2},
///   {"Key three" => 3},
///   {"Key four" => 4},
/// };
/// ```
#[macro_export ]macro_rules! smallmap {
    () => {
	$crate::Map::new()
    };
    ($({$key:expr => $value:expr}),* $(,)?) => {
	{
	    let mut map = $crate::Map::new();
	    $(
		map.insert($key, $value);
	    )*
		map
	}
    }
}


/// Trait for types that can be used as `Map` keys.
///
/// Implementors should try to minimise collisions by making `collapse` return a relatively unique value if possible.
/// But it is not required.
/// It is automatically implemented for types implementing the `Hash` trait.
/// A simple folding implementation is provided for byte slices here [`collapse_iter()`](collapse_iter).
///
/// The default implementation has integer types implement this through the modulo of itself over 256, whereas byte slice types implement it through an XOR fold over itself. It doesn't matter though, the programmer is free to implement it how she chooses.
pub trait Collapse: Eq
{
    /// Create the index key for this instance. This is similar in use to `Hash::hash()`.
    fn collapse(&self) -> u8;
}

/// A single page in a `Map`. Contains up to 256 key-value entries.
#[repr(transparent)]
pub struct Page<TKey,TValue>([Option<(TKey, TValue)>; MAX]);

mod page_impls;

impl<K,V> Page<K,V>
where K: Collapse
{
    /// Create a new blank page
    #[cfg(nightly)] 
    pub const fn new() -> Self
    {
	Self(init::blank_page())
    }
    /// Create a new blank page
    #[cfg(not(nightly))]
    pub fn new() -> Self
    {
	Self(init::blank_page())
    }
    
    /// The number of entries currently in this page
    ///
    /// This is a count that iterates over all slots, if possible store it in a temporary instead of re-calling it many times.
    pub fn len(&self) -> usize
    {
	self.0.iter().map(Option::as_ref).filter_map(std::convert::identity).count()
    }

    /// An iterator over all entries currently in this page
    pub fn iter(&self) -> PageElements<'_, K,V>
    {
	PageElements(self.0.iter())
    }

    /// A mutable iterator over all entries currently in this page
    pub fn iter_mut(&mut self) -> PageElementsMut<'_, K,V>
    {
	PageElementsMut(self.0.iter_mut())
    }
    
    fn search<Q: ?Sized>(&self, key: &Q) -> &Option<(K,V)>
    where Q: Collapse
    {
	&self.0[usize::from(key.collapse())]
    }
    fn search_mut<Q: ?Sized>(&mut self, key: &Q) -> &mut Option<(K,V)>
    where Q: Collapse
    {
	&mut self.0[usize::from(key.collapse())]
    }

    fn replace(&mut self, k: K, v: V) -> Option<(K,V)>
    {
	std::mem::replace(&mut self.0[usize::from(k.collapse())], Some((k,v)))
    }
}

impl<K: Collapse, V> std::iter::FromIterator<(K, V)> for Map<K,V>
{
    fn from_iter<I: IntoIterator<Item=(K, V)>>(iter: I) -> Self
    {
	//TODO: Optimise this
	let mut this = Self::new();
	for (key, value) in iter.into_iter()
	{
	    this.insert(key, value);
	}
	this
    }
}

impl<K,V> IntoIterator for Page<K,V>
where K: Collapse
{
    type Item= (K,V);
    type IntoIter = IntoPageElements<K,V>;

    /// Consume this `Page` into an iterator of all values currently in it.
    fn into_iter(self) -> Self::IntoIter
    {
	IntoPageElements(self.0, 0)
    }
}


impl<K,V> Default for Page<K,V>
where K: Collapse
{
    #[inline]
    fn default() -> Self
    {
	Self::new()
    }
}

/// A small hashtable-like map with byte sized key indecies.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
// TODO: Replace with `SmallVec<[Page<TKey, TValue>; 1]>` when feature that adds `smallvec` is enabled (this will allocate the first page on the stack, and the rest on the heap.
pub struct Map<TKey, TValue>(Vec<Page<TKey,TValue>>);

impl<K,V> Map<K,V>
{
    /// Returns the currently allocated size of the map in bytes (including currently unused reserved space.)
    #[inline(always)]
    #[allow(dead_code)] // Used in test cases, but compiler still warns about it
    pub(crate) fn internal_size_bytes(&self) -> usize
    {
	self.0.capacity() * std::mem::size_of::<Page<K,V>>()
	//self.0.iter().map(std::mem::size_of_val).sum::<usize>()
    }
}

impl<K,V> Map<K,V>
where K: Collapse
{
    fn new_page(&mut self) -> &mut Page<K,V>
    {
	let len = self.0.len();
	self.0.push(Page::new());
	&mut self.0[len]
    }
    #[inline(always)] fn fuck_entry(&mut self, key: K) -> Option<Entry<'_, K, V>>
    {
	for page in self.0.iter_mut()
	{
	    let re = page.search_mut(&key);
	    match  re {
		Some((ref ok, _)) if key.eq(ok.borrow()) => {
		    return Some(Entry::Occupied(entry::OccupiedEntry(re)));
		},
		None => {
		    return Some(Entry::Vacant(entry::VacantEntry(re, key)));
		},
		_ => (),
	    }
	}
	None
    }

    /// Get an `Entry` for the `key` that lets you get or insert the value
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V>
    {
	// somehow this is faster than using index, even though here we search twice????? i don't know why but there you go
	if let None =  self.0.iter()
	    .filter(|x| x.search(&key).as_ref().and_then(|(k, v)| if k==&key {None} else {Some((k,v))}).is_none())
	    .next() {
		self.new_page();
	    }
	self.fuck_entry(key).unwrap()
    }
    /// Remove all empty pages from this instance.
    pub fn clean(&mut self)
    {
	#[cfg(nightly)] 
	self.0.drain_filter(|x| x.len() <1);
	#[cfg(not(nightly))]
	{
	    let mut i = 0;
	    while i != self.0.len() {
		if self.0[i].len() <1 {
		    self.0.remove(i);
		} else {
		    i += 1;
		}
	    }
	}
    }

    /// The number of entries currently in this map
    ///
    /// This is an iterating count over all slots in all current pages, if possible store it in a temporary instead of re-calling it.
    pub fn len(&self) -> usize
    {
	self.pages().map(Page::len).sum()
    }
    /// Is this map empty
    pub fn is_empty(&self) -> bool
    {
	self.0[0].iter().next().is_none()
    }
    /// The number of pages currently in this map
    pub fn num_pages(&self) -> usize
    {
	self.0.len()
    }
    /// Consume the instance, returning all pages.
    pub fn into_pages(self) -> Vec<Page<K,V>>
    {
	self.0
    }
    /// An iterator over all pages
    pub fn pages(&self) -> Pages<'_, K, V>
    {
	iter::Pages(self.0.iter())
    }

    /// A mutable iterator over all pages
    pub fn pages_mut(&mut self) -> PagesMut<'_, K, V>
    {
	iter::PagesMut(self.0.iter_mut())
    }

    /// An iterator over all elements in the map
    pub fn iter(&self) -> Iter<'_, K, V>
    {
	Iter(None, self.pages())
    }

    /// A mutable iterator over all elements in the map
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V>
    {
	IterMut(None, self.pages_mut())
    }

    /// Create a new empty `Map`
    pub fn new() -> Self
    {
	Self(vec![Page::new()])
    }

    /// Create a new empty `Map` with a specific number of pages pre-allocated
    pub fn with_capacity(pages: usize) -> Self
    {
	#[cold] fn cap_too_low() -> !
	{
	    panic!("Got 0 capacity, this is invalid.")
	}
	
	if pages == 0 {
	    cap_too_low()
	}
	let mut p = Vec::with_capacity(pages);
	p.push(Page::new());
	Self(p)
    }

    /// Get a mutable reference of the value corresponding to this key if it is in the map.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where K: Borrow<Q>,
	  Q: Collapse + Eq
    {
	for page in self.0.iter_mut()
	{
	    match page.search_mut(key) {
		Some((ref ok, ov)) if key.eq(ok.borrow()) => {
		    return Some(ov);
		},
		_ => (),
	    }
	}
	None
    }

    /// Search the map for entry corresponding to this key
    #[inline] pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where K: Borrow<Q>,
	  Q: Collapse + Eq
    {
	self.get(key).is_some()
    }

    /// Get a reference of the value corresponding to this key if it is in the map.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where K: Borrow<Q>,
	  Q: Collapse + Eq
    {
	for page in self.0.iter()
	{
	    match page.search(key) {
		Some((ref ok, ov)) if key.eq(ok.borrow()) => {
		    return Some(ov);
		},
		_ => (),
	    }
	}
	None
    }

    /// Remove the entry corresponding to this key in the map, returning the value if it was present
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where K: Borrow<Q>,
	  Q: Collapse + Eq
    {
	for page in self.0.iter_mut()
	{
	    let v = page.search_mut(key);
	    match v {
		Some((ref ok, _)) if key.eq(ok.borrow()) => {
		    return v.take().map(|(_, v)| v);
		},
		_ => (),
	    }
	}
	None
    }

    /// Insert a new key-value entry into this map, returning the pervious value if it was present
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    {
	for page in self.0.iter_mut()
	{
	    match page.search_mut(&key) {
		Some((ref ok, ov)) if ok.eq(&key) => {
		    return Some(std::mem::replace(ov, value));
		},
		empty @ None => {
		    return empty.replace((key, value))
			.map(|(_, v)| v);
		},
		_ => (),
	    }
	}

	let mut page = Page::new();
	page.replace(key, value);
	self.0.push(page);
	None
    }
    
    /// Consume this `Map` by swapping its keys and values around.
    pub fn reverse(self) -> Map<V,K>
    where V: Collapse
    {
	let mut output = Map::with_capacity(self.num_pages());

	for (k,v) in self.into_iter()
	{
	    output.insert(v, k);
	}

	output
    }
}

impl<K: Collapse, V> IntoIterator for Map<K,V>
{
    type Item= (K,V);
    type IntoIter = IntoIter<K,V>;

    /// Consume this map into an iterator over all currently inserted entries
    fn into_iter(self) -> Self::IntoIter
    {
	IntoIter(None, self.0.into_iter())
    }
}

impl<K: Collapse, V> std::iter::Extend<(K,V)> for Map<K,V>
{
    fn extend<T: IntoIterator<Item = (K,V)>>(&mut self, iter: T) {
	// we can probably optimise this better, right?
	for (key, value) in iter.into_iter()
	{
	    self.insert(key,value);
	}
    }
}

use std::hash::{Hash, Hasher,};
impl<T: Hash+ Eq> Collapse for T
{
    fn collapse(&self) -> u8 {
	struct CollapseHasher(u8);
	macro_rules! hash_type {
	    
	    ($nm:ident, u8) => {
		#[inline(always)] fn $nm(&mut self, i: u8)
		{
		    self.0 ^= i;
		}
	    };
	    
	    ($nm:ident, i8) => {
		#[inline(always)] fn $nm(&mut self, i: i8)
		{
		    self.0 ^= i as u8;
		}
	    };
	    
	    ($nm:ident, $ty:tt) => {
		#[inline] fn $nm(&mut self, i: $ty)
		{
		    self.0 ^= (i % MAX as $ty) as u8;
		}
	    };
	}
	impl Hasher for CollapseHasher
	{
	    #[inline] fn finish(&self) -> u64
	    {
		self.0 as u64
	    }
	    #[inline] fn write(&mut self, buffer: &[u8])
	    {
		self.0 ^= collapse(buffer);
	    }
	    hash_type!(write_u8, u8);
	    hash_type!(write_i8, i8);
	    hash_type!(write_i16, i16);
	    hash_type!(write_u16, u16);
	    hash_type!(write_i32, i32);
	    hash_type!(write_u32, u32);
	    hash_type!(write_i64, i64);
	    hash_type!(write_u64, u64);
	    hash_type!(write_u128, u128);
	    
	    hash_type!(write_isize, isize);
	    hash_type!(write_usize, usize);
	}

	let mut h = CollapseHasher(0);
	self.hash(&mut h);
	h.0
    }
}
#[cfg(test)]
mod tests;

/// Collapse a slice of bytes with an XOR fold
#[inline] pub fn collapse<T: AsRef<[u8]>>(bytes: T) -> u8
{
    bytes.as_ref().iter().copied().fold(0, |a, b| a ^ b)
}

/// Collapse an iterator of bytes with an XOR fold
#[inline] pub fn collapse_iter<T: IntoIterator<Item=u8>>(bytes: T) -> u8
{
    bytes.into_iter().fold(0, |a, b| a ^ b)
}
