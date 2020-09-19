//! Default implementors
use super::*;

macro_rules! collapse {
    ($ty:ty) => {
	impl CollapseMemory for $ty
	{
	    fn as_memory(&self) -> &[u8]
	    {
		self.as_ref()
	    }
	}
    };
}

collapse!(str);
collapse!(&str);
collapse!(&mut str);
collapse!([u8]);
collapse!(&[u8]);
collapse!(&mut [u8]);
