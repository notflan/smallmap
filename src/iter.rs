//! Iterator types for `Map`
use super::*;

pub struct Pages<'a, K, V>(pub(crate) std::slice::Iter<'a, Page<K,V>>);

impl<'a, K, V> Iterator for Pages<'a,K,V>
{
    type Item = &'a Page<K,V>;

    #[inline] fn next(&mut self) -> Option<Self::Item> {
	self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
	self.0.size_hint()
    }
}

pub struct PagesMut<'a, K, V>(pub(crate) std::slice::IterMut<'a, Page<K,V>>);

impl<'a, K, V> Iterator for PagesMut<'a,K,V>
{
    type Item = &'a mut Page<K,V>;

    #[inline] fn next(&mut self) -> Option<Self::Item> {
	self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
	self.0.size_hint()
    }
}

impl<'a, K, V> ExactSizeIterator for PagesMut<'a,K,V>{}
impl<'a, K, V> std::iter::FusedIterator for PagesMut<'a,K,V>{}

pub struct PageElements<'a, K, V>(pub(crate) std::slice::Iter<'a, Option<(K,V)>>);

impl<'a, K, V> Iterator for PageElements<'a,K,V>
{
    type Item = &'a (K,V);

    #[inline] fn next(&mut self) -> Option<Self::Item> {
	self.0.next().map(Option::as_ref).flatten()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
	(0, self.0.size_hint().1)
    }
}
impl<'a, K, V> std::iter::FusedIterator for PageElements<'a,K,V>{}


pub struct PageElementsMut<'a, K, V>(pub(crate) std::slice::IterMut<'a, Option<(K,V)>>);

impl<'a, K, V> Iterator for PageElementsMut<'a,K,V>
{
    type Item = &'a mut (K,V);

    #[inline] fn next(&mut self) -> Option<Self::Item> {
	self.0.next().map(Option::as_mut).flatten()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
	(0, self.0.size_hint().1)
    }
}
impl<'a, K, V> std::iter::FusedIterator for PageElementsMut<'a,K,V>{}

pub struct IntoPageElements<K,V>(pub(crate) [Option<(K,V)>; MAX], pub(crate) usize);

impl<K,V> Iterator for IntoPageElements<K,V>
{
    type Item = (K,V);
    fn next(&mut self) -> Option<Self::Item> {
	loop {
	    if self.1 >= self.0.len() {
		return None;
	    } else {
		match self.0[self.1].take() {
		    Some(value) => return Some(value),
		    None => self.1+=1,
		}
	    }
	}
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
	(0, Some(self.0.len()))
    }
}
impl<K, V> std::iter::FusedIterator for IntoPageElements<K,V>{}

pub struct Iter<'a, K, V>(pub(crate) Option<PageElements<'a,K,V>>, pub(crate) Pages<'a, K,V>);

impl<'a, K,V> Iterator for Iter<'a, K,V>
where K: Collapse
{
    type Item = &'a (K,V);
    fn next(&mut self) -> Option<Self::Item> {
	loop {
	    if let Some(ref mut page) = self.0 {
		if let Some(elem) = page.next() {
		    return Some(elem);
		}
	    }
	    if let Some(next_page) = self.1.next() {
		self.0.replace(next_page.iter());
	    } else {
		return None;
	    }
	}
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
	(0, self.1.size_hint().1.map(|x| x * MAX))
    }
}
impl<'a, K: Collapse, V> std::iter::FusedIterator for Iter<'a, K,V>{}


pub struct IterMut<'a, K, V>(pub(crate) Option<PageElementsMut<'a,K,V>>, pub(crate) PagesMut<'a, K,V>);

impl<'a, K,V> Iterator for IterMut<'a, K,V>
where K: Collapse
{
    type Item = &'a mut (K,V);
    fn next(&mut self) -> Option<Self::Item> {
	loop {
	    if let Some(ref mut page) = self.0 {
		if let Some(elem) = page.next() {
		    return Some(elem);
		}
	    }
	    if let Some(next_page) = self.1.next() {
		self.0.replace(next_page.iter_mut());
	    } else {
		return None;
	    }
	}
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
	(0, self.1.size_hint().1.map(|x| x * MAX))
    }
}
impl<'a, K: Collapse, V> std::iter::FusedIterator for IterMut<'a, K,V>{}



pub struct IntoIter<K, V>(pub(crate) Option<IntoPageElements<K,V>>,  pub(crate) std::vec::IntoIter<Page<K,V>>);

impl<K, V> Iterator for IntoIter<K,V>
   where K: Collapse
{
    type Item = (K,V);
    fn next(&mut self) -> Option<Self::Item> {
	loop {
	    if let Some(ref mut page) = self.0 {
		if let Some(elem) = page.next() {
		    return Some(elem);
		}
	    }
	    if let Some(next) = self.1.next() {
		self.0.replace(next.into_iter());
	    } else {
		return None;
	    }
	}
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
	(0, self.1.size_hint().1.map(|x| x * MAX))
    }
}

impl<K: Collapse, V> std::iter::FusedIterator for IntoIter<K,V>{}
