//! Because we can't #derive big arrays on stable smh
use super::*;
use std::{
    fmt::{self, Debug,},
    hash,
};

impl<K: Clone, V: Clone> Clone for Page<K,V>
{
    fn clone(&self) -> Self
    {
	#[inline(always)] fn copy_slice<T: Clone>(dst: &mut [T], src: &[T])
	{
	    for (d, s) in dst.iter_mut().zip(src.iter())
	    {
		*d = s.clone()
	    }
	}
	let mut new = init::blank_page();
	copy_slice(&mut new[..], &self.0[..]);
	Self(new)
    }
}

impl<K: Debug, V:Debug> Debug for Page<K,V>
{
    #[inline] fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	write!(f, "{:?}", &self.0[..])
    }
}


impl<K: Eq, V: Eq> Eq for Page<K,V>{}
impl<K: PartialEq, V: PartialEq> PartialEq for Page<K,V>
{
    #[inline] fn eq(&self, other: &Self) -> bool
    {
	&self.0[..] == &other.0[..]
    }
}

impl<K: hash::Hash, V: hash::Hash> hash::Hash for Page<K,V> {
    #[inline] fn hash<H: hash::Hasher>(&self, state: &mut H) {
	(&self.0[..]).hash(state)
    }
}

#[cfg(feature="serde")]
const _: () = {
    use serde::*;
    impl<K,V> serde::Serialize for Page<K,V>
    where K:Serialize, V: Serialize
    {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
            S: Serializer,
	{
            serializer.serialize_slice(&self.0[..])
	}
    }
};
