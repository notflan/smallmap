//! Entry API
use super::*;

#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V>(pub(crate) &'a mut Option<(K,V)>);

impl<'a, K, V> OccupiedEntry<'a, K, V>
where K: Collapse
{
    pub fn get(&self) -> &V
    {
	&self.0.as_ref().unwrap().1
    }
    pub fn get_mut(&mut self) -> &mut V
    {
	&mut self.0.as_mut().unwrap().1
    }
    pub fn into_mut(self) -> &'a mut V
    {
	&mut self.0.as_mut().unwrap().1
    }
    pub fn key(&self) -> &K
    {
	&self.0.as_ref().unwrap().0
    }
    pub fn insert(&mut self, value: V) -> V
    {
	std::mem::replace(&mut self.0.as_mut().unwrap().1, value)
    }
    pub fn remove(self) -> V
    {
	self.remove_entry().1
    }
    pub fn remove_entry(self) -> (K, V)
    {
	self.0.take().unwrap()
    }
}

#[derive(Debug)]
pub struct VacantEntry<'a,K,V>(pub(crate) &'a mut Option<(K,V)>, pub(crate) K);

impl<'a, K, V> VacantEntry<'a, K, V>
where K: Collapse
{
    pub fn insert(self, value: V) -> &'a mut V
    {
	*self.0 = Some((self.1, value));
	&mut self.0.as_mut().unwrap().1
    }

    pub fn into_key(self) -> K
    {
	self.1
    }

    pub fn key(&self) -> &K
    {
	&self.1
    }
}

#[derive(Debug)]
pub enum Entry<'a, K, V>
{
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where K: Collapse
{
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Entry<'a, K, V>
    {
	if let Self::Occupied(occuped) = &mut self {
	    f(occuped.get_mut())
	}
	self
    }

    pub fn key(&self) -> &K
    {
	match self {
	    Entry::Vacant(v) => v.key(),
	    Entry::Occupied(o) => o.key(),
	}
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, with: F) -> &'a mut V
    {
	match self {
	    Entry::Occupied(o) => o.into_mut(),
	    Entry::Vacant(v) => v.insert(with())
	}
    }

    #[inline] pub fn or_insert(self, value: V) -> &'a mut V
    {
	self.or_insert_with(|| value)
    }

}

impl<'a, K, V> Entry<'a, K, V>
where K: Collapse,
      V: Default
{
    #[inline] pub fn or_default(self) -> &'a mut V
    {
	self.or_insert_with(Default::default)
    }
}
