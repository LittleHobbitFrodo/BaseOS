//	mem/dynamic_buffer.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::{NonNull, copy_nonoverlapping};
use core::alloc::{Layout, GlobalAlloc};
use crate::{ALLOCATOR, TryClone};


/// returns the minimum of 3 values
#[inline(always)]
fn min_3(v1: usize, v2: usize, v3: usize) -> usize {
    core::cmp::min(core::cmp::min(v1, v2), v3)
}



/// Dynamic buffer has only ne task: memory management
/// - it is not much useful on its own...
/// 
/// It simply allocates memory like vector would, but does not work with its content
/// - this also means that no elements will be dropped
/// 
/// ## Implementation details
/// - uses `self.capacity() > 0` to check if any data is allocated
///   - `self.data` is set to `NonNull::dangling()` if not
/// - `self.size` is only used for copying of elements and is not modified
/// - `drop()` will only deallocate buffer, no elements are dropped
/// 
/// ### Generic parameters
/// 1. **T**: defines the type that is allocated
/// 2. **STEP**: indicates how many elements should be preallocated
///     - set to 0 to enable **geometrical growth**
pub struct DynamicBuffer<T: Sized, const STEP: usize = 4> {
    data: NonNull::<u8>,
    cap: u32,
    pub size: u32,
    _marker: PhantomData<T>,
}

impl<T: Sized, const STEP: usize> DynamicBuffer<T, STEP> {

    /// returns `Layout` describing memory layout for `self`
    /// - use `DynamicBuffer::layout_for_exact(capacity)` for other instances
    const fn layout(&self) -> Layout {
        Self::layout_for_exact(self.capacity())
    }

    /// Constructs empty DynamicBuffer with no allocated data
    pub const fn empty() -> Self {
        Self {
            data: NonNull::dangling(),
            cap: 0,
            size: 0,
            _marker: PhantomData,
        }
    }

    /// Constructs `DynamicBuffer<T>` with some elements allocated
    /// - **panics** if allocation fails
    /// - `size = 0`
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = Self::new_capacity(capacity);

        let l = Self::layout_for_exact(cap);

        let data = unsafe { ALLOCATOR.alloc(l) };

        assert!(!data.is_null(), "failed to allocate data");

        Self {
            data: unsafe { NonNull::new_unchecked(data) },
            cap: cap as u32,
            size: 0,
            _marker: PhantomData
        }
    }

    /// Tries to construct `DynamicBuffer<T>` with some elements allocated
    /// - returns `Err` if allocation fails
    /// - `size = 0`
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ()> {
        let cap = Self::new_capacity(capacity);

        let l = Self::layout_for_exact(cap);

        let data = unsafe { ALLOCATOR.alloc(l) };

        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: unsafe { NonNull::new_unchecked(data) },
            cap: cap as u32,
            size: 0,
            _marker: PhantomData
        })
    }

    /// Constructs `DynamicBuffer<T>` with some elements allocated and zeroed memory
    /// - **panics** if allocation fails
    /// - `size = 0`
    pub fn with_capacity_zeroed(capacity: usize) -> Self {

        let cap = Self::new_capacity(capacity);

        let l = Self::layout_for_exact(cap);
        
        let data = unsafe { ALLOCATOR.alloc_zeroed(l) };

        assert!(!data.is_null(), "failed to allocate data");

        Self {
            data: unsafe { NonNull::new_unchecked(data) },
            cap: cap as u32,
            size: 0,
            _marker: PhantomData
        }
    }

    /// Tries to construct `DynamicBuffer<T>` with some elements allocated and zeroed memory
    /// - returns `Err` if allocation fails
    /// - `size = 0`
    pub fn try_with_capaity_zeroed(capacity: usize) -> Result<Self, ()> {

        let cap = Self::new_capacity(capacity);

        let l = Self::layout_for_exact(cap);

        let data = unsafe { ALLOCATOR.alloc(l) };

        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: unsafe { NonNull::new_unchecked(data) },
            cap: cap as u32,
            size: 0,
            _marker: PhantomData
        })
    }

    /// Resizes (reallocates) the buffer to certain size
    /// - `size` is aligned to `STEP`
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - **panics** if allocation fails
    /// - **Copies exactly `self.size` elements to the new location**
    pub fn resize(&mut self, size: usize) {

        if self.capacity() == size {
            return
        }

        let wanted = Self::new_capacity(size);

        let layout = Self::layout_for_exact(wanted);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        assert!(!new.is_null(), "failed to allocate memory");

        if self.capacity() > 0 {
            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), wanted);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = wanted as u32;

    }



    /// Tries to resize (reallocate) the buffer to certain size
    /// - `size` is aligned to `STEP`
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - returns `Err` if allocation fails
    /// - **Copies exactly `self.size` elements to the new location**
    pub fn try_resize(&mut self, size: usize) -> Result<(), ()> {

        if self.capacity() == size {
            return Ok(())
        }

        let wanted = Self::new_capacity(size);

        let layout = Self::layout_for_exact(wanted);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        if new.is_null() {
            return Err(());
        }

        if self.capacity() > 0 {

            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), wanted);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = wanted as u32;

        Ok(())

    }

    /// Resizes (reallocates) the buffer to exact size
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - **panics** if allocation fails
    /// - **Copies exactly `self.size` elements to the new location**
    pub fn resize_exact(&mut self, size: usize) {

        if size == self.capacity() {
            return
        }

        let layout = Self::layout_for_exact(size);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        assert!(!new.is_null(), "failed to allocate memory");

        if self.capacity() > 0 {
            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), size);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = size as u32;

    }

    /// Tries to resize (reallocate) the buffer to exact size
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - **panics** if allocation fails
    /// - **Copies exactly `self.size` elements to the new location**
    pub fn try_resize_exact(&mut self, size: usize) -> Result<(), ()> {

        let layout = Self::layout_for_exact(size);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        if new.is_null() {
            return Err(());
        }

        if self.capacity() > 0 {
            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), size);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = size as u32;

        Ok(())

    }


    /// Expands the `capacity` by `STEP` elements
    /// - this function always reallocates memory
    /// - **panics** if allocation fails
    /// - **Copies exactly `self.size` elements to the new location**
    pub fn expand(&mut self) {

        let wanted = Self::cap_next(self.capacity());

        let layout = Self::layout_for_exact(wanted);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        assert!(!new.is_null(), "failed to allocate memory");

        if self.capacity() > 0 {
            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), wanted);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = wanted as u32;

    }

    /// Tries to expand the `capacity` by `STEP` elements
    /// - this function always reallocates memory
    /// - returns `Err` if allocation fails
    /// - **Copies exactly `self.size` elements to the new location**
    pub fn try_expand(&mut self) -> Result<(), ()> {

        let wanted = Self::cap_next(self.capacity());

        let layout = Self::layout_for_exact(wanted);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        if new.is_null() {
            return Err(());
        }

        if self.capacity() > 0 {
            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), wanted);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = wanted as u32;

        Ok(())

    }

    /// Expands the `capacity` by `STEP * steps` elements
    /// - this function always reallocates memory
    /// - **panics** if allocation fails
    pub fn expand_by(&mut self, steps: usize) {

        let wanted = Self::cap_next(self.capacity() + (STEP * steps));

        let layout = Self::layout_for_exact(wanted);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        assert!(!new.is_null(), "failed to allocate memory");

        if self.capacity() > 0 {
            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), wanted);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = wanted as u32;

    }

    /// Tries to expanf the `capacity` by `STEP * steps` elements
    /// - this function always reallocates memory
    /// - returns `Err` if allocation fails
    pub fn try_expand_by(&mut self, steps: usize) -> Result<(), ()> {

        let wanted = Self::cap_next(self.capacity() + (STEP * steps));

        let layout = Self::layout_for_exact(wanted);

        let new = unsafe { ALLOCATOR.alloc(layout) };

        if new.is_null() {
            return Err(());
        }

        if self.capacity() > 0 {
            if self.size > 0 {
                unsafe {
                    //  eliminate buffer overflow
                    let copy_size = min_3(self.size as usize, self.capacity(), wanted);
                    copy_nonoverlapping(self.data.as_ptr(), new, copy_size * size_of::<T>());
                }
            }
            unsafe { ALLOCATOR.dealloc(self.data.as_ptr(), self.layout()); }
        }

        self.data = unsafe { NonNull::new_unchecked(new) };
        self.cap = wanted as u32;

        Ok(())

    }

    /// Constructs new `DynamicBuffer` from raw parts
    /// - **warning**: may be potentially unsafe
    pub fn from_raw_parts(ptr: NonNull<T>, layout: Layout) -> Self {
        Self {
            data: unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) },
            cap: (layout.size()/size_of::<T>()) as u32,
            size: 0,
            _marker: PhantomData
        }
    }

    /// 
    pub fn into_raw_parts(self) -> (NonNull<T>, usize) {

        if self.capacity() == 0 {
            panic!("no memory allocated");
        }

        let m = ManuallyDrop::new(self);
        let ptr = m.data.as_ptr() as *mut T;
        (unsafe { NonNull::new_unchecked(ptr) }, m.capacity())
    }


}


impl<T: Sized, const STEP: usize> DynamicBuffer<T, STEP> {

    /// Returns the `STEP` constant for this instance
    pub const fn step(&self) -> usize {
        STEP
    }

    pub const fn has_data(&self) -> bool {
        self.capacity() > 0
    }

    /// Describes memory layout for some capacity
    pub const fn layout_for(capacity: usize) -> Layout {
        let size = unsafe {
            size_of::<T>().unchecked_mul(Self::cap_next(capacity))
        };
        
        unsafe { Layout::from_size_align_unchecked(size, align_of::<T>()) }
    }

    /// Describes memory layout for some capacity without aligning to `STEP`
    pub const fn layout_for_exact(capacity: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(size_of::<T>().unchecked_mul(capacity), align_of::<T>()) }
    }

    /// aligns the capacity up to next generic `STEP`
    /// - result is greater than `STEP`
    /// - returns number of elements
    pub const fn cap_next(cap: usize) -> usize {
        if STEP == 0 {
            (cap + 1).next_power_of_two()
        } else {
            (cap + 1).next_multiple_of(STEP)
        }
    }

    /// aligns the capacity to generic `STEP`
    /// - result is equal or greater than `STEP`
    /// - returns number of elements
    pub const fn new_capacity(cap: usize) -> usize {
        if STEP == 0 {
            cap.next_power_of_two()
        } else {
            cap.next_multiple_of(STEP)
        }
    }

    /// Returns number of elements allocated in the buffer
    pub const fn capacity(&self) -> usize {
        self.cap as usize
    }

    /// Indicates if no data is allocated
    pub const fn is_empty(&self) -> bool {
        self.capacity() == 0
    }

    /// Returns pointer to allocated data
    pub const fn as_ptr(&self) -> *mut T {
        self.data.as_ptr() as *mut T
    }

    /// Returns pointer to data as `NonNull`
    pub const fn data(&self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.data.as_ptr() as *mut T) }
    }


}


impl<T: Sized, const STEP: usize> Drop for DynamicBuffer<T, STEP> {
    fn drop(&mut self) {
        if self.capacity() > 0 {
            unsafe {
                ALLOCATOR.dealloc(self.data.as_ptr(), self.layout());
            }
        }
    }
}

impl<T: Sized, const STEP: usize> Clone for DynamicBuffer<T, STEP> {
    /// `DynamicBuffer::clone()` does **not copy** any data
    fn clone(&self) -> Self {
        if self.capacity() == 0 {
            Self {
                data: NonNull::dangling(),
                cap: 0,
                size: 0,
                _marker: PhantomData,
            }
        } else {
            let data = unsafe {
                ALLOCATOR.alloc(self.layout())
            };

            assert!(!data.is_null(), "failed to allocate memory");

            Self {
                data: unsafe { NonNull::new_unchecked(data) },
                cap: self.capacity() as u32,
                size: self.size,
                _marker: PhantomData,
            }
        }
    }
}


impl<T: Sized, const STEP: usize> TryClone for DynamicBuffer<T, STEP> {
    type Error = ();
    /// `DynamicBuffer::try_clone()` does **not copy** any data
    fn try_clone(&self) -> Result<Self, Self::Error>
    where Self: Sized, Self::Error: Default {

        if self.capacity() == 0 {
            Ok(Self {
                data: NonNull::dangling(),
                cap: 0,
                size: 0,
                _marker: PhantomData,
            })
        } else {

            let data = unsafe {
                ALLOCATOR.alloc(self.layout())
            };

            if data.is_null() {
                return Err(());
            }

            Ok(Self {
                data: unsafe { NonNull::new_unchecked(data) },
                cap: self.capacity() as u32,
                size: self.size,
                _marker: PhantomData,
            })
        }

    }
}