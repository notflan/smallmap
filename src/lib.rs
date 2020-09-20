#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn)]
#![feature(drain_filter)]
#![cfg_attr(nightly, feature(test))] 


#![allow(dead_code)]

#[cfg(nightly)] extern crate test;

const MAX: usize = 256;

//TODO: Move test
//TODO: Document
//TODO: Readme
//TODO: LICENSE
//TODO: Publish and upload to githubxc

use std::{
    borrow::Borrow,
};

pub trait Collapse: Eq
{
    fn collapse(&self) -> u8;
}

#[repr(transparent)]
#[derive(Debug,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub struct Page<TKey,TValue>([Option<(TKey, TValue)>; MAX]);

impl<K,V> Page<K,V>
where K: Collapse
{
    /// Create a new blank page
    pub const fn new() -> Self
    {
	Self([None; MAX])
    }
    
    pub fn len(&self) -> usize
    {
	self.0.iter().map(Option::as_ref).filter_map(std::convert::identity).count()
    }

    pub fn iter(&self) -> PageElements<'_, K,V>
    {
	PageElements(self.0.iter())
    }
    
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

impl<K,V> IntoIterator for Page<K,V>
where K: Collapse
{
    type Item= (K,V);
    type IntoIter = IntoPageElements<K,V>;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Map<TKey, TValue>(Vec<Page<TKey,TValue>>);

pub mod iter;
use iter::*;
pub mod entry;
pub use entry::Entry;

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
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V>
    {
	if self.0.iter()
	    .filter(|x| x.search(&key).is_none())
	    .count() == 0 {
		self.new_page();
	    }//so dumb..... SO dumb
	     //will need to completely reimplement all entry::* shit to just have mut reference to Map and then usize indecies for location I guess. Fuck this
	self.fuck_entry(key).unwrap()
    }
    pub fn clean(&mut self)
    {
	self.0.drain_filter(|x| x.len() <1);
    }
    
    pub fn len(&self) -> usize
    {
	self.pages().map(Page::len).sum()
    }
    pub fn num_pages(&self) -> usize
    {
	self.0.len()
    }
    pub fn into_pages(self) -> Vec<Page<K,V>>
    {
	self.0
    }
    pub fn pages(&self) -> Pages<'_, K, V>
    {
	iter::Pages(self.0.iter())
    }
    
    pub fn pages_mut(&mut self) -> PagesMut<'_, K, V>
    {
	iter::PagesMut(self.0.iter_mut())
    }

    pub(crate) fn iter_opaque(&self) -> impl Iterator<Item = &(K, V)> + '_
    {
	self.pages().map(|x| x.iter()).flatten()
    }

    pub fn iter(&self) -> Iter<'_, K, V>
    {
	Iter(None, self.pages())
    }
    
    pub(crate) fn iter_mut_opaque(&mut self) -> impl Iterator<Item = &mut (K, V)> + '_
    {
	self.pages_mut().map(|x| x.iter_mut()).flatten()
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V>
    {
	IterMut(None, self.pages_mut())
    }
    
    pub fn new() -> Self
    {
	Self(vec![Page::new()])
    }

    pub fn with_capacity(pages: usize) -> Self
    {
	let mut p = Vec::with_capacity(pages);
	p.push(Page::new());
	Self(p)
    }

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

    #[inline] pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where K: Borrow<Q>,
	  Q: Collapse + Eq
    {
	self.get(key).is_some()
    }
    
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

    
    fn search_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Option<(K,V)>>
    where K: Borrow<Q>,
	  Q: Collapse + Eq
    {
	for page in self.0.iter_mut()
	{
	    let se = page.search_mut(key);
	    match se {
		Some((ref ok, _)) if key.eq(ok.borrow()) => {
		    return Some(se);
		},
		_ => (),
	    }
	}
	None
    }

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
}

impl<K: Collapse, V> IntoIterator for Map<K,V>
{
    type Item= (K,V);
    type IntoIter = IntoIter<K,V>;

    fn into_iter(self) -> Self::IntoIter
    {
	IntoIter(None, self.0.into_iter())
    }
}


pub trait CollapseMemory: Eq
{
    fn as_memory(&self) -> &[u8];
}
impl<T> Collapse for T
where T: CollapseMemory
{
    fn collapse(&self) -> u8 {
	collapse(self.as_memory())
    }
}


mod primitives;
pub use primitives::*;

mod defaults;
pub use defaults::*;

/// Collapse bytes with default XOR fold
pub fn collapse<T: AsRef<[u8]>>(bytes: T) -> u8
{
    bytes.as_ref().iter().copied().fold(0, |a, b| a ^ b)
}

#[cfg(test)]
mod tests;
