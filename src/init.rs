//! Blank page


#[inline(always)] pub const fn blank_page<K,V>() -> [Option<(K,V)>; super::MAX]
{
    //stable doesn't let us use [None; MAX], so...
    [None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,
     None,None,None,None,None,None,None,None,]
}
