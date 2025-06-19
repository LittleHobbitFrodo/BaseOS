//	mem/dynamic_buffer.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::{null_mut, NonNull};
use core::alloc::{Layout, GlobalAlloc};
use crate::{ALLOCATOR, TryClone};



/// Dynamic buffer has only ne task: memory management
/// 
/// however it is not much useful on its own...
/// 
/// It simply allocates memory like vector would, but does not work with its content
/// - this also means that no elements will be dropped
/// 
/// ### Generic parameters
/// 1. **T**: T defines the type that is allocated
/// 2. **STEP**: indicates how many elements should be preallocated
///     - set to 0 to enable **geometrical growth**
pub struct DynamicBuffer<T: Sized, const STEP: usize = 4> {
    data: Option<NonNull::<u8>>,
    layout: Layout,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T: Sized, const STEP: usize> DynamicBuffer<T, STEP> {

    /// Constructs empty DynamicBuffer with no allocated data
    pub const fn empty() -> Self {
        Self {
            data: None,
            layout: Layout::new::<T>(),
            cap: 0,
            _marker: PhantomData,
        }
    }

    /// Constructs `DynamicBuffer<T>` with some elements allocated
    /// - **panics** if allocation fails
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = Self::cap_next(capacity);

        let l = Self::layout_for_exact(cap);
        let data = unsafe {
            ALLOCATOR.alloc(l)
        };

        assert!(!data.is_null(), "failed to allocate data for Vec");

        Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: l,
            cap: cap,
            _marker: PhantomData
        }
    }

    /// Tries to construct `DynamicBuffer<T>` with some elements allocated
    /// - returns `Err` if allocation fails
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ()> {
        let cap = Self::cap_next(capacity);

        let l = Self::layout_for_exact(cap);
        let data = unsafe {
            ALLOCATOR.alloc(l)
        };

        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: l,
            cap: cap,
            _marker: PhantomData
        })
    }

    /// Constructs `DynamicBuffer<T>` with some elements allocated and zeroed memory
    /// - **panics** if allocation fails
    pub fn with_capacity_zeroed(capacity: usize) -> Self {
        let cap = Self::cap_next(capacity);

        let l = Self::layout_for_exact(cap);
        let data = unsafe {
            ALLOCATOR.alloc_zeroed(l)
        };

        assert!(!data.is_null(), "failed to allocate data for Vec");

        Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: l,
            cap: cap,
            _marker: PhantomData
        }
    }

    /// Tries to construct `DynamicBuffer<T>` with some elements allocated and zeroed memory
    /// - returns `Err` if allocation fails
    pub fn try_with_capaity_zeroed(capacity: usize) -> Result<Self, ()> {
        let cap = Self::cap_next(capacity);

        let l = Self::layout_for_exact(cap);
        let data = unsafe {
            ALLOCATOR.alloc(l)
        };

        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: l,
            cap: cap,
            _marker: PhantomData
        })
    }

    /// Resizes (reallocates) the buffer to certain size
    /// - `size` is aligned to `STEP`
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - **panics** if allocation fails
    pub fn resize(&mut self, mut size: usize) {
        size = Self::cap_next(size);

        if let Some(data) = self.data {

            if self.capacity() != size {
                let l = Self::layout_for_exact(size);
                let d = unsafe {
                    ALLOCATOR.realloc_layout(data.as_ptr() as *mut u8, self.layout, l)
                };

                assert!(!d.is_null(), "failed to reallocate memory");

                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.layout = l;
                self.cap = size;

            }
            
        } else {
            //  allocate new memory
            let l = Self::layout_for_exact(size);
            let data = unsafe {
                ALLOCATOR.alloc(l)
            };

            assert!(!data.is_null(), "failed to allocate memory");

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.cap = size;
        }
    }

    /// Tries to resize (reallocate) the buffer to certain size
    /// - `size` is aligned to `STEP`
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - **panics** if allocation fails
    pub fn try_resize(&mut self, mut size: usize) -> Result<(), ()> {
        size = Self::cap_next(size);

        if let Some(data) = self.data {

            if self.capacity() != size {
                let l = Self::layout_for_exact(size);
                let d = unsafe {
                    ALLOCATOR.realloc_layout(data.as_ptr() as *mut u8, self.layout, l)
                };

                if d.is_null() {
                    return Err(());
                }

                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.layout = l;
                self.cap = size;

            }

            Ok(())
            
        } else {
            //  allocate new memory
            let l = Self::layout_for_exact(size);
            let data = unsafe {
                ALLOCATOR.alloc(l)
            };

            if data.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.cap = size;

            Ok(())

        }
    }

    /// Resizes (reallocates) the buffer to exact size
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - **panics** if allocation fails
    pub fn resize_exact(&mut self, size: usize) {

        if let Some(data) = self.data {

            if self.capacity() != size {
                let l = Self::layout_for_exact(size);
                let d = unsafe {
                    ALLOCATOR.realloc_layout(data.as_ptr() as *mut u8, self.layout, l)
                };

                assert!(!d.is_null(), "failed to reallocate memory");

                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.layout = l;
                self.cap = size;

            }
            
        } else {
            //  allocate new memory
            let l = Self::layout_for_exact(size);
            let data = unsafe {
                ALLOCATOR.alloc(l)
            };

            assert!(!data.is_null(), "failed to allocate memory");

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.cap = size;
        }
    }

    /// Tries to resize (reallocate) the buffer to exact size
    /// - **no-op** if `capacity` would be the same`
    /// - if `self.is_empty()` allocates new data
    /// - **panics** if allocation fails
    pub fn try_resize_exact(&mut self, size: usize) -> Result<(), ()> {

        if let Some(data) = self.data {

            if self.capacity() != size {
                let l = Self::layout_for_exact(size);
                let d = unsafe {
                    ALLOCATOR.realloc_layout(data.as_ptr() as *mut u8, self.layout, l)
                };

                if d.is_null() {
                    return Err(());
                }

                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.layout = l;
                self.cap = size;

            }

            Ok(())
            
        } else {
            //  allocate new memory
            let l = Self::layout_for_exact(size);
            let data = unsafe {
                ALLOCATOR.alloc(l)
            };

            if data.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.cap = size;

            Ok(())

        }
    }


    /// Expands the `capacity` by `STEP` elements
    /// - this function always reallocates memory
    /// - **panics** if allocation fails
    pub fn expand(&mut self) {

        let wanted = self.capacity() + STEP;
        let l = Self::layout_for_exact(wanted);

        let data = unsafe { if let Some(data) = self.data {
            ALLOCATOR.realloc_layout(data.as_ptr(), self.layout, l)
        } else {
            ALLOCATOR.alloc(l)
        }};

        
        assert!(!data.is_null(), "failed to allocate memory");

        self.data = Some(unsafe { NonNull::new_unchecked(data) });
        self.layout = l;
        self.cap = wanted;
    }

    /// Tries to expanf the `capacity` by `STEP` elements
    /// - this function always reallocates memory
    /// - returns `Err` if allocation fails
    pub fn try_expand(&mut self) -> Result<(), ()> {

        let wanted = self.capacity() + STEP;
        let l = Self::layout_for_exact(wanted);

        let data = unsafe { if let Some(data) = self.data {
            ALLOCATOR.realloc_layout(data.as_ptr(), self.layout, l)
        } else {
            ALLOCATOR.alloc(l)
        }};

        if data.is_null() {
            return Err(());
        }

        self.data = Some(unsafe { NonNull::new_unchecked(data) });
        self.layout = l;
        self.cap = wanted;

        Ok(())
    }

    /// Expands the `capacity` by `STEP * steps` elements
    /// - this function always reallocates memory
    /// - **panics** if allocation fails
    pub fn expand_by(&mut self, steps: usize) {

        let wanted = self.capacity() + (STEP * steps);
        let l = Self::layout_for_exact(wanted);

        let data = unsafe { if let Some(data) = self.data {
            ALLOCATOR.realloc_layout(data.as_ptr(), self.layout, l)
        } else {
            ALLOCATOR.alloc(l)
        }};

        
        assert!(!data.is_null(), "failed to allocate memory");

        self.data = Some(unsafe { NonNull::new_unchecked(data) });
        self.layout = l;
        self.cap = wanted;
    }

    /// Tries to expanf the `capacity` by `STEP * steps` elements
    /// - this function always reallocates memory
    /// - returns `Err` if allocation fails
    pub fn try_expand_by(&mut self, steps: usize) -> Result<(), ()> {

        let wanted = self.capacity() + (STEP * steps);
        let l = Self::layout_for_exact(wanted);

        let data = unsafe { if let Some(data) = self.data {
            ALLOCATOR.realloc_layout(data.as_ptr(), self.layout, l)
        } else {
            ALLOCATOR.alloc(l)
        }};

        if data.is_null() {
            return Err(());
        }

        self.data = Some(unsafe { NonNull::new_unchecked(data) });
        self.layout = l;
        self.cap = wanted;

        Ok(())
    }

    pub fn from_raw_parts(ptr: NonNull<T>, layout: Layout) -> Self {
        Self {
            data: Some(unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut u8) }),
            layout,
            cap: layout.size()/size_of::<T>(),
            _marker: PhantomData
        }
    }

    pub fn into_raw_parts(self) -> (NonNull<T>, Layout) {
        let m = ManuallyDrop::new(self);
        let ptr = m.data.expect("DynamicBuffer does not contain any data")
    .as_ptr() as *mut T;
        (unsafe { NonNull::new_unchecked(ptr) }, m.layout)
    }


}


impl<T: Sized, const STEP: usize> DynamicBuffer<T, STEP> {

    /// Returns the `STEP` constant for this instance
    pub const fn step(&self) -> usize {
        STEP
    }

    pub const fn has_data(&self) -> bool {
        self.data.is_some()
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
        let size = unsafe {
            size_of::<T>().unchecked_mul(capacity)
        };

        unsafe { Layout::from_size_align_unchecked(size, align_of::<T>()) }
    }

    /// aligns capacity to generic `STEP`
    /// - returns number of elements
    pub const fn cap_next(cap: usize) -> usize {
        if STEP == 0 {
            cap.next_power_of_two()
        } else {
            cap.next_multiple_of(STEP)
        }
    }

    /// Returns layout describing allocated memory
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    /// Returns number of elements allocated in the buffer
    pub const fn capacity(&self) -> usize {
        self.cap
    }

    /// Indicates if no data is allocated
    pub const fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    /// Returns pointer to allocated data
    pub const fn as_ptr(&self) -> *mut T {
        if let Some(data) = self.data {
            data.as_ptr() as *mut T
        } else {
            null_mut()
        }
    }

    /// Returns pointer to data as `NonNull`
    pub const fn data(&self) -> Option<NonNull<T>> {
        if let Some(data) = self.data {
            Some(unsafe { NonNull::new_unchecked(data.as_ptr() as *mut T) })
        } else {
            None
        }
    }


}


impl<T: Sized, const STEP: usize> Drop for DynamicBuffer<T, STEP> {
    fn drop(&mut self) {
        if let Some(data) = self.data {
            unsafe {
                ALLOCATOR.dealloc(data.as_ptr(), self.layout);
            }
        }
    }
}

impl<T: Sized, const STEP: usize> Clone for DynamicBuffer<T, STEP> {
    /// `DynamicBuffer::clone()` does **not copy** any data
    fn clone(&self) -> Self {
        let data = unsafe {
            ALLOCATOR.alloc(self.layout())
        };

        assert!(!data.is_null(), "failed to allocate memory");

        Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: self.layout(),
            cap: self.capacity(),
            _marker: PhantomData,
        }

    }
}


impl<T: Sized, const STEP: usize> TryClone for DynamicBuffer<T, STEP> {
    type Error = ();
    /// `DynamicBuffer::try_clone()` does **not copy** any data
    fn try_clone(&self) -> Result<Self, Self::Error>
    where Self: Sized, Self::Error: Default {
        let data = unsafe {
            ALLOCATOR.alloc(self.layout())
        };

        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: self.layout(),
            cap: self.capacity(),
            _marker: PhantomData,
        })

    }
}