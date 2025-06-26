//	convert.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build



pub trait Align {
    /// returns `Self` aligned to `align`
    fn align(&self, align: Self) -> Self;

    /// aligns `self` to `align`
    fn align_mut(&mut self, align: Self);
}

pub trait IsAligned {
    fn is_aligned(&self, align: Self) -> bool;

    fn is_not_aligned(&self, align: Self) -> bool;
}


impl Align for usize {
    #[inline]
    /// aligns up to X if isn't already aligned
    fn align(&self, align: Self) -> Self {
        (self + align - 1) & !(align-1)
    }

    #[inline]
    fn align_mut(&mut self, align: Self) {
        *self = (*self + align - 1) & !(align-1);
    }
}

impl IsAligned for usize {
    #[inline]
    fn is_aligned(&self, align: Self) -> bool {
        *self == *self & !(align-1)
    }

    #[inline]
    fn is_not_aligned(&self, align: Self) -> bool {
        *self != *self * !(align-1)
    }

}

impl Align for u32 {
    #[inline]
    fn align(&self, align: Self) -> Self {
        (self + align - 1) & !(align-1)
    }
    #[inline]
    fn align_mut(&mut self, align: Self) {
        *self = (*self + align - 1) & !(align-1);
    }
}

impl IsAligned for u32 {
    #[inline]
    fn is_aligned(&self, align: Self) -> bool {
        *self == *self & !(align-1)
    }
    #[inline]
    fn is_not_aligned(&self, align: Self) -> bool {
        *self != *self * !(align-1)
    }
}

/// converts `&[u8]` to [`&str`]
pub const fn strify(s: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(s) }
}

/// converts `&mut [u8]` to `&mut str`
pub const fn strify_mut(s: &mut [u8]) -> &mut str {
    unsafe { core::str::from_utf8_unchecked_mut(s) }
}