//! Implementing `Collapse` for primitive types
use super::*;

macro_rules! collapse {
    (char) => {
	impl Collapse for char
	{
	    #[inline] fn collapse(&self) -> u8
	    {
		(u32::from(*self) % (MAX as u32)) as u8
	    }
	}
    };
    ($type:ty) => {
	impl Collapse for $type
	{
	    #[inline] fn collapse(&self) -> u8
	    {
		const _: &[(); 1] =  &[(); (((MAX as $type) as usize) == MAX) as usize];
		(*self % MAX as Self) as u8
	    }
	}
    };
}

impl Collapse for bool
{
    #[inline] fn collapse(&self) -> u8
    {
	*self as u8
    }    
}
impl Collapse for u8
{
    #[inline] fn collapse(&self) -> u8
    {
	*self
    }    
}

impl Collapse for i8
{
    #[inline] fn collapse(&self) -> u8
    {
	*self as u8
    }    
}
collapse!(char);
collapse!(u16);
collapse!(i16);
collapse!(i32);
collapse!(u32);
collapse!(u64);
collapse!(i64);
collapse!(i128);
collapse!(u128);
