//! Because we can't #derive big arrays on stable smh
use super::*;
use core::{
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
    use std::marker::PhantomData;
    
    impl<K,V> serde::Serialize for Page<K,V>
    where K:Serialize, V: Serialize
    {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
            S: Serializer,
	{
	    use serde::ser::SerializeSeq;
	    let mut seq = serializer.serialize_seq(Some(MAX))?;
            for element in self.0.iter() {
		seq.serialize_element(element)?;
            }
            seq.end()
	}
    }

    struct PageVisitor<K,V>(PhantomData<Page<K,V>>);
    
    impl<'de, K, V> de::Visitor<'de> for PageVisitor<K,V> 
    where K: Deserialize<'de>,
	  V: Deserialize<'de>
    {
	type Value = Page<K,V>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array of 256 `Option<(K,V)>` elements")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where
	    A: serde::de::SeqAccess<'de>
	{
	    let mut elems = init::blank_page();
	    let mut i=0usize;
	    while let Some(optkv) = seq.next_element()?
	    {
		elems[i] = optkv;
		i+=1;
	    }
	    if i==MAX {
		Ok(Page(elems))
	    } else {
		use serde::de::Error;
		Err(A::Error::custom(format!("Expected {} elemts, got {}", MAX, i)))
	    }
	}
    }

    
    impl<'de, K, V> Deserialize<'de> for Page<K,V>
    where K: Deserialize<'de>,
	  V: Deserialize<'de>
    {
	fn deserialize<D>(deserializer: D) -> Result<Page<K,V>, D::Error>
	where
            D: serde::de::Deserializer<'de>,
	{
            deserializer.deserialize_seq(PageVisitor(PhantomData))
	}
    }

};


#[cfg(feature="serde")] 
#[cfg(test)]
mod serde_tests
{
    use crate::*;
    #[test]
    fn serde()
    {
	let map = smallmap! {
	    {101 => 102},
	    {10111 => 10222},
	};
	let string = serde_json::to_string(&map).expect("ser failed");
	println!("String {:?}", string);
	let map2 = serde_json::from_str(&string[..]).expect("de failed");
	assert_eq!(map, map2);
    }
}
