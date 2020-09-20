//! Map entries.
//!
//! The API is similar to that of `BTreeMap` and `HashMap`'s `Entry` types.
use super::*;

/// Varient of [`Entry`](Entry) that already contains a value.
#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V>(pub(crate) &'a mut Option<(K,V)>);

impl<'a, K, V> OccupiedEntry<'a, K, V>
where K: Collapse
{
    /// Get a reference to the value
    pub fn get(&self) -> &V
    {
	&self.0.as_ref().unwrap().1
    }
    /// Get a mutable reference to the value
    pub fn get_mut(&mut self) -> &mut V
    {
	&mut self.0.as_mut().unwrap().1
    }
    /// Consume this instance, returning the held mutable reference to the value
    pub fn into_mut(self) -> &'a mut V
    {
	&mut self.0.as_mut().unwrap().1
    }
    /// A reference to the key
    pub fn key(&self) -> &K
    {
	&self.0.as_ref().unwrap().0
    }
    /// Replace the held value with another, yielding the old one
    pub fn insert(&mut self, value: V) -> V
    {
	std::mem::replace(&mut self.0.as_mut().unwrap().1, value)
    }
    /// Remove this entry from the `Map`, yielding the removed value
    pub fn remove(self) -> V
    {
	self.remove_entry().1
    }
    /// Remove this entry from the `Map`, yielding the removed key-value pair.
    pub fn remove_entry(self) -> (K, V)
    {
	self.0.take().unwrap()
    }
}

/// Varient of [`Entry`](Entry) that does not contain a value.
#[derive(Debug)]
pub struct VacantEntry<'a,K,V>(pub(crate) &'a mut Option<(K,V)>, pub(crate) K);

impl<'a, K, V> VacantEntry<'a, K, V>
where K: Collapse
{
    /// Insert a value into this empty slot, retuning a mutable reference to the new value.
    pub fn insert(self, value: V) -> &'a mut V
    {
	*self.0 = Some((self.1, value));
	&mut self.0.as_mut().unwrap().1
    }

    /// Consume this instance, returning the held key.
    pub fn into_key(self) -> K
    {
	self.1
    }

    /// A reference to the held key
    pub fn key(&self) -> &K
    {
	&self.1
    }
}

/// Represents a space in a `Map` that may or may not contains a value.
#[derive(Debug)]
pub enum Entry<'a, K, V>
{
    /// This entry slot does not yet contain a value
    Vacant(VacantEntry<'a, K, V>),
    /// This entry slot does contain a value
    Occupied(OccupiedEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where K: Collapse
{
    /// Run this closure on a mutable reference to the internal value if it is present, otherwise do nothing.
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Entry<'a, K, V>
    {
	if let Self::Occupied(occuped) = &mut self {
	    f(occuped.get_mut())
	}
	self
    }
    
    /// A reference to the key
    pub fn key(&self) -> &K
    {
	match self {
	    Entry::Vacant(v) => v.key(),
	    Entry::Occupied(o) => o.key(),
	}
    }

    /// Insert into the entry if it is empty the value returned by the closure and return a mutable reference to the new value, otherwise return a mutable reference to the already present value.
    pub fn or_insert_with<F: FnOnce() -> V>(self, with: F) -> &'a mut V
    {
	match self {
	    Entry::Occupied(o) => o.into_mut(),
	    Entry::Vacant(v) => v.insert(with())
	}
    }

    /// Insert into the entry this value if it is empty and return a mutable reference to the new value, otherwise return a mutable reference to the already present value.
    #[inline] pub fn or_insert(self, value: V) -> &'a mut V
    {
	self.or_insert_with(|| value)
    }

}

impl<'a, K, V> Entry<'a, K, V>
where K: Collapse,
      V: Default
{
    /// Insert into the entry the default value if it is empty and return a mutable reference to the new value, otherwise return a mutable reference to the already present value.
    #[inline] pub fn or_default(self) -> &'a mut V
    {
	self.or_insert_with(Default::default)
    }
}
