//	mem/box.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

// the (almost) standard implementation of `Box<T>` structure

use core::alloc::{GlobalAlloc, Layout};
use core::default;
use core::fmt::Display;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ptr::{drop_in_place, NonNull};
use core::ops::{Deref, DerefMut};

use crate::{TryClone, Nothing};
use crate::{mem::alloc::ALLOCATOR};


/// `Box` is used to safely allocate and deallocate memory of type `T`
/// 
/// Use `Array` to allocate arrays
pub struct Box<T: Sized> {
    data: NonNull<T>,
}

impl<T: Sized> Box<T> {

    /// Describes memory layout of single element `Box`
    pub const fn layout() -> Layout {
        Layout::new::<T>()
    }

    /// Allocates memory on heap with `val` value
    /// - panics if allocation fails
    pub fn new(val: T) -> Self {
        Self {
            data: match unsafe { ALLOCATOR.allocate(val) } {
                Ok(data) => data,
                Err(_) => panic!("failed to allocate memory for Box"),
            },
        }
    }

    /// Tries to allocate memory with some value
    /// - returns `Err` if allocation fails
    pub fn try_new(val: T) -> Result<Self, ()> {
        Ok(Self {
            data: unsafe { ALLOCATOR.allocate(val)? },
        })
    }

    /// Allocates memory on heap and leaves it uninitialized
    /// - panics if allocation fails
    pub fn new_uninit() -> Box<MaybeUninit<T>> {
        let data = unsafe {
            ALLOCATOR.alloc(Self::layout())
        } as *mut MaybeUninit<T>;

        assert!(!data.is_null(), "failed to allocate memory for Box");

        Box {
            data: unsafe { NonNull::new_unchecked(data) },
        }
    }

    /// Tries to allocate memory on heap and leaves it uninitialized
    /// - returns `Err` if allocation fails
    pub fn try_new_uninit() -> Result<Box<MaybeUninit<T>>, ()> {
        let data = unsafe {
            ALLOCATOR.alloc(Self::layout())
        } as *mut MaybeUninit<T>;

        if data.is_null() {
            return Err(());
        }

        Ok(Box {
            data: unsafe { NonNull::new_unchecked(data) },
        })
    }

    /// Allocates memory on heap and forces all bytes to 0
    /// - panics if allocation fails
    pub fn new_zeroed() -> Box<MaybeUninit<T>> {
        let data = unsafe {
            ALLOCATOR.alloc(Self::layout())
        } as *mut MaybeUninit<T>;

        assert!(!data.is_null(), "failed to allocate memory for Box");

        Box {
            data: unsafe { NonNull::new_unchecked(data) }
        }
    }

    /// Tries to allocate memory while forcing all bytes to 0
    /// - returns `Err` if allocation fails
    pub fn try_new_zeroed() -> Result<Box<MaybeUninit<T>>, ()> {
        let data = unsafe {
            ALLOCATOR.alloc(Self::layout())
        } as *mut MaybeUninit<T>;

        if data.is_null() {
            return Err(());
        }

        Ok(Box {
            data: unsafe { NonNull::new_unchecked(data) }
        })
    }

    /// Constructs `Box` from `NonNull`
    /// - use `Box::layout::<T>()` to describe Layout
    pub const fn from_non_null(ptr: NonNull<T>) -> Self {
        Self {
            data: ptr,
        }
    }

    /// Converts the `Box` to `NonNull` while consuming the Box
    pub unsafe fn into_non_null(self) -> NonNull<T> {
        let m = ManuallyDrop::new(self);
        m.data
    }

    /// Returns pointer to content of the `Box`
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    /// Returns mutable pointer to content of the `Box`
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_ptr()
    }

    pub unsafe fn leak<'l>(self) -> &'l mut T {
        let mut m = ManuallyDrop::new(self);
        unsafe { m.data.as_mut() }
    }


}

impl<T: Sized> Box<MaybeUninit<T>> {

    /// Tells the compiler to treat this `Box` as initialized
    pub unsafe fn assume_init(self) -> Box<T> {
        let m = ManuallyDrop::new(self);
        Box {
            data: unsafe { NonNull::new_unchecked(m.data.as_ptr() as *mut T) }
        }
    }

}

impl<T: Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            drop_in_place(self.data.as_ptr());
            ALLOCATOR.dealloc(self.data.as_ptr() as *mut u8, Self::layout());
        }
    }
}


impl<T: Display> Display for Box<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", *unsafe { self.data.as_ref() })
    }
}

impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref() }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.data.as_mut() }
    }
}

impl<T> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }
}

impl<T> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }
}

impl<T: Clone> Clone for Box<T> {
    fn clone(&self) -> Self {
        let val = self.as_ref().clone();
        Self {
            data: match unsafe { ALLOCATOR.allocate(val) } {
                Ok(data) => data,
                Err(_) => panic!("failed to allocate memory for Box"),
            }
        }
    }
}

impl<T: Sized + TryClone> TryClone for Box<T>
where T: Sized + TryClone, T::Error: Default {

    type Error = T::Error;

    fn try_clone(&self) -> Result<Self, Self::Error>
    where Self: Sized, Self::Error: Default {
        
        let val = self.as_ref().try_clone()?;
        
        Ok(Self {
            data: match unsafe { ALLOCATOR.allocate(val) }{
                Ok(data) => data,
                Err(_) => return Err(T::Error::default())
            },
        })
    }
}

/*impl<T: Sized + TryClone + Default> TryClone for Box<T> {
    type Error = Nothing;
    fn try_clone(&self) -> Result<Self, Self::Error>
        where Self: Sized {
        let val = match self.as_ref().try_clone() {
            Ok(v) => v,
            Err(_) => return Err(Nothing::default()),
        };

        Ok(Self {
            data: unsafe { ALLOCATOR.allocate(val)? }
        })
    }
}*/