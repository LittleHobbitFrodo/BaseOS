//	sync/rosync.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build




/// # Read Only Sync
/// The [`RoSync`] structure offers read only access for any thread with no locking or reference counting mechanism
/// 
/// ### Use case
/// Make static read-only data accessible without [`unsafe`] block
pub struct RoSync<T> {
    data: T,
}

unsafe impl<T> Sync for RoSync<T> {}
unsafe impl<T> Send for RoSync<T> {}

impl<T> RoSync<T> {
    pub const fn new(value: T) -> Self {
        Self { data: value }
    }

    #[inline(always)]
    pub fn as_ref(&self) -> &T {
        &self.data
    }

    #[inline(always)]
    pub fn borrow(&self) -> &T {
        &self.data
    }
}

impl<T> core::ops::Deref for RoSync<T> {
    type Target=T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}


impl<T> Drop for RoSync<T> {
    fn drop(&mut self) {
        if crate::mem::needs_drop::<T>() {
            //  drop T here
            unsafe {core::ptr::drop_in_place(&mut self.data as *mut T);}
        }
    }
}