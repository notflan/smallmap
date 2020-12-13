//! Contains `Collapse` impls for primitive types through a newtime shim.
//!
//! # Why
//! Such wrappers are a workaround for the lack of template specialisation available in Rust so far, as the generic `impl<T: Hash> Collapse<T> for T` still requires computing the hash of the internal types before reducing to the `u8` page index.
//! For primitive types, this is unnessisary and causes a (very slight) performance loss.
//!
//! If/when Rust gets specialisation, this will be unneeded.
use super::*;
use std::num::*;

/// Sealed trait allowing for wrapping primitive types with a more efficient implemntation for the `Collapse` trait.
/// This should not be used for much directly, instead use the newtype shim `Primitive<T>`.
pub trait PrimitiveCollapse: private::Sealed
{
    fn collapse(&self) -> u8;
}

/// Shim for primitive types to efficiently implement `Collapse`.
///
/// # Notes
/// This newtype is transparent. It is safe to `mem::transmute`() from `Primitive<T>` to `T` and vice versa.
/// However, if `T` does *not* implement `PrimitiveCollapse`, it is undefined behaviour.
///
/// Also, the `collapse()` output from this structure is not guaranteed to be the same as the `collapse()` output from the inner value, so the following code is very unsafe and such patterns should only be used if the programmer is absolutely sure there will be absolutely no difference between `T::collapse` and `Self::collapse`:
/// ```
/// # use smallmap::{Map, Primitive};
/// # use std::mem;
///
///  let mut map: Map<u8, ()> = Map::new();
///  map.insert(120, ());
///
///  let map: Map<Primitive<u8>, ()> = unsafe { mem::transmute(map) };
///  assert_eq!(map.get(&120.into()).copied(), Some(()));
/// ```
/// This code pretty much only works with `u8`. and `i8`.
///
/// However unsafe, it is possible these values will line up in your use case. In which case, it is an acceptable pattern.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Primitive<T>(T);

impl<T: PrimitiveCollapse+ Eq> Collapse for Primitive<T>
{
    #[inline(always)] fn collapse(&self) -> u8 {
	self.0.collapse()
    }
}

impl<T: PrimitiveCollapse+ Eq> Primitive<T>
{
    /// Wrap this primitive 
    #[cfg(nightly)] #[inline] pub const fn new(value: T) -> Self
    {
	Self(value)
    }
    /// Wrap this primitive 
    #[cfg(not(nightly))] #[inline] pub fn new(value: T) -> Self
    {
	Self(value)
    }
    /// Consume into the inner primitive
    #[inline] pub fn into_inner(self) -> T
    {
	self.0
    }
    /// Get the inner primitive
    ///
    /// # Notes
    /// Only useful if the inner type does not implement `Copy`, which is extremely unlickely.
    /// You should almost always use `into_inner` instead.
    #[inline] pub fn inner(&self) -> &T
    {
	&self.0
    }
    /// Get a mutable reference to the inner ptimitive.
    #[inline] pub fn inner_mut(&mut self) -> &mut T
    {
	&mut self.0
    }
    
    /// Same as `into_inner`, except only for `Copy` types.
    ///
    /// # Notes
    /// The only use of this function is that it is `const fn` on nightly.
    /// If you're not using a version of rustc that supports generic `const fn`, this method is identical to `into_inner`.
    #[cfg(nightly)] #[inline] pub const fn into_inner_copy(self) -> T
    where T: Copy
    {
	self.0
    }
    #[cfg(not(nightly))] #[inline(always)] #[deprecated = "This function should only be used on Rust nightly. Please use `into_inner` instead"] pub fn into_inner_copy(self) -> T
    where T: Copy
    {
	self.0
    }
}

impl<T> From<T> for Primitive<T>
    where T: PrimitiveCollapse + Eq
{
    #[inline] fn from(from: T) -> Self
    {
	Self::new(from)
    }
}

macro_rules! prim {
    ($name:ty) => {	
	impl private::Sealed for $name{}
	impl PrimitiveCollapse for $name
	{
	    #[inline(always)] fn collapse(&self) -> u8 {
		(*self) as u8
	    }
	}
    };
    ($name:ty: +) => {	
	impl private::Sealed for $name{}
	impl PrimitiveCollapse for $name
	{
	    #[inline(always)] fn collapse(&self) -> u8 {
		self.get() as u8
	    }
	}
    };
    ($name:ty: ^) => {	
	impl private::Sealed for $name{}
	impl PrimitiveCollapse for $name
	{
	    #[inline(always)] fn collapse(&self) -> u8 {
		super::collapse(<$name>::to_ne_bytes(*self))
	    }
	}
    };
    ($name:ty: ^+) => {	
	impl private::Sealed for $name{}
	impl PrimitiveCollapse for $name
	{
	    #[inline(always)] fn collapse(&self) -> u8 {
		super::collapse(self.get().to_ne_bytes())
	    }
	}
    };
    ($name:ty: fn {$($block:tt)*}) => {
	impl private::Sealed for $name{}
	impl PrimitiveCollapse for $name
	{
	    #[inline(always)] fn collapse(&self) -> u8 {
		$($block)+
	    }
	}
    };
    ($name:ty: {$($block:tt)*}) => {
	impl private::Sealed for $name{}
	impl PrimitiveCollapse for $name
	{
	    $($block)+
	}
    };

}

prim!(u8);
prim!(i8);
prim!(u16: ^);
prim!(i16: ^);
prim!(u32: ^);
prim!(i32: ^);
prim!(u64: ^);
prim!(i64: ^);
prim!(u128: ^);
prim!(i128: ^);
prim!(isize: ^);
prim!(usize: ^);

prim!(NonZeroU8: +);
prim!(NonZeroI8: +);
prim!(NonZeroU16: ^+);
prim!(NonZeroI16: ^+);
prim!(NonZeroU32: ^+);
prim!(NonZeroI32: ^+);
prim!(NonZeroU64: ^+);
prim!(NonZeroI64: ^+);
prim!(NonZeroU128: ^+);
prim!(NonZeroI128: ^+);
prim!(NonZeroIsize: ^+);
prim!(NonZeroUsize: ^+);

prim!((): fn {
    0
});

#[cfg(nightly)] 
prim!(!: {
    fn collapse(&self) -> u8
    {
	*self
    }
});
