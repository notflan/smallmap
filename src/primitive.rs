//! Contains `Collapse` impls for primitive types through a newtime wrapper.
//! 
//! Such wrappers are a workaround for the lack of template specialisation available in Rust so far, as the generic `impl<T: Hash> Collapse<T> for T` still requires computing the hash of the internal types before reducing to the `u8` page index.
//! For primitive types, this is unnessisary and causes a (very slight) performance loss.
use super::*;
use std::num::*;

pub trait PrimitiveCollapse: private::Sealed
{
    fn collapse(&self) -> u8;
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default, Ord, PartialOrd)]
pub struct Primitive<T>(T);

impl<T: PrimitiveCollapse+ Eq> Collapse for Primitive<T>
{
    #[inline(always)] fn collapse(&self) -> u8 {
	self.0.collapse()
    }
}

impl<T: PrimitiveCollapse+ Eq> Primitive<T>
{
    #[const_fn]
    #[inline] pub const fn new(value: T) -> Self
    {
	Self(value)
    }
    #[inline] pub fn into_inner(self) -> T
    {
	self.0
    }
    #[const_fn]
    #[inline] pub fn inner(&self) -> &T
    {
	&self.0
    }
    #[inline] pub fn inner_mut(&mut self) -> &mut T
    {
	&mut self.0
    }
    #[const_fn]
    #[inline] pub const fn copy_into_inner(&self) -> T
    where T: Copy
    {
	self.0
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
