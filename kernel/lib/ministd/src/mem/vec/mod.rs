//  mem/vec/mod.rs (ministd crate)
//  this file originally belonged to baseOS project
//      on OS template on which to build

use core::{alloc::{GlobalAlloc, Layout, LayoutError}, ascii, fmt::Write, hint::spin_loop, mem::ManuallyDrop, ops::RangeBounds, ptr::{self, drop_in_place, null, null_mut, NonNull}, slice};

use core::ops::Bound::{Included, Excluded, Unbounded};

use core::cmp::Ordering::{self, Less, Greater, Equal};

use crate::{hang, TryClone, ALLOCATOR};

use core::borrow::{Borrow, BorrowMut};

use core::ops::{Deref, DerefMut};

pub mod iterators;

pub use iterators::*;

//  TODO: investigate Layout and reallocating (not all structs are power-of-two-sized)

/*/// A contiguous growable array type, written as Vec<T>, short for ‘vector’.
/// 
/// implementation details:
/// - `capacity` and `size` are of type `u32`
/// - `capacity` growth is geometrical (exponencial => 8 -> 16 -> 32 -> 64 ...)
///   - `capacity * size_of::<T>()` is actually allocated
/// - `capacity` is always greater or equal to `size_of::<T>()`
pub struct Vec<T: Sized> {
    data: Option<NonNull<T>>,
    size: u32,
    cap: u32,
}

impl<T: Sized> Vec<T> {

    #[inline]
    const fn mk_layout_unchecked(capacity: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(size_of::<T>() * capacity, align_of::<T>()) }
    }

    /// function used to create `Layout` for `Vec` under the hood
    #[inline]
    pub const fn mk_layout(capacity: usize) -> Result<Layout, LayoutError> {
        Layout::array::<T>(capacity)
    }

    const fn init_capacity(capacity: u32) -> u32 {
        if capacity < size_of::<T>() as u32 {
            size_of::<T>() as u32
        } else {
            capacity.next_power_of_two()
        }
    }

    #[inline]
    fn handle_bounds<R>(&self, range: &R) -> (usize, usize)
    where R: RangeBounds<usize> {

        (match range.start_bound() {
            Excluded(&val) => val + 1,
            Included(&val) => val,
            Unbounded => 0,
        },
        match range.end_bound() {
            Included(&val) => val + 1,
            Excluded(&val) => val,
            Unbounded => self.len(),
        })
    }

    pub const fn new() -> Self {
        Self {
            data: None,
            size: 0,
            cap: 0
        }
    }

    /// allocates new data for Vec
    /// - allocated data is not initialized
    /// - panics if allocation fails
    /// - use only if allocation failure would cause panic anyway
    pub fn with_capacity(mut capacity: usize) -> Self {
        capacity = Self::init_capacity(capacity as u32) as usize;

        let data = unsafe {
            ALLOCATOR.alloc(Self::mk_layout_unchecked(capacity))
        } as *mut T;

        if data.is_null() {
            panic!("failed to allocate memory for Vec");
        }

        Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            size: 0,
            cap: capacity as u32,
        }
    }

    /// allocates new data for Vec
    /// - allocated data is not initialized
    /// - returns `Err` if allocation fails
    ///   - `Layout` creation failure is possible too, but should not ever occur
    pub fn try_with_capacity(mut capacity: usize) -> Result<Self, ()> {
        capacity = Self::init_capacity(capacity as u32) as usize;

        let layout = match Self::mk_layout(capacity) {
            Ok(l) => l,
            Err(_) => return Err(()),
        };

        let data = unsafe { ALLOCATOR.alloc(layout) } as *mut T;

        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            size: 0,
            cap: capacity as u32,
        })
    }

    /// constructs vector from raw parts
    /// - panics if anything goes wrong
    /// 
    /// please do not use this function unless absolutely necessary
    /// 
    /// safety:
    /// - `ptr` must be **non-null** and must be aligned to `align_of::<T>()`
    /// - `length` must be less or equal than `capacity`
    /// - `capacity` must be greater than 0
    pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Self {
        if !ptr.is_aligned() || ptr.is_null() || capacity == 0 || length > capacity {
            if ptr.is_null() {
                panic!("pointer is null");
            } else if !ptr.is_aligned() {
                panic!("pointer is not aligned");
            } else if capacity == 0 {
                panic!("capacity is zero");
            } else if length > capacity {
                panic!("length > capacity");
            } else {
                panic!("unknown error");
            }
        }

        Self {
            data: Some(unsafe { NonNull::new_unchecked(ptr) }),
            size: length as u32,
            cap: capacity as u32,
        }
    }

    /// constructs vector from raw parts
    /// - panics if anything goes wrong
    /// 
    /// safety:
    /// - `ptr` must be **non-null** and must be aligned to `align_of::<T>()`
    /// - `length` must be less or equal than `capacity`
    /// - `capacity` must be greater than 0
    pub unsafe fn try_from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Result<Self, ()> {
        if !ptr.is_aligned() || ptr.is_null() || capacity == 0 || length > capacity {
            Err(())
        } else {
            Ok(Self {
                data: Some(unsafe { NonNull::new_unchecked(ptr) }),
                size: length as u32,
                cap: capacity as u32
            })
        }
    }

    /// returns each part of the vector while consuming itself
    pub unsafe fn into_raw_parts(self) -> (*mut T, usize, usize) {
        let me = ManuallyDrop::new(self);
        if let Some(data) = me.data {
            (data.as_ptr(), me.size as usize, me.cap as usize)
        } else {
            (null_mut(), 0, 0)
        }
    }



    /// reserves at least `add` more elements on the heap
    /// - panics if allocation fails
    /// - after this operation, capacity will be greater than or equal to `self.len() + add`
    /// - use only if allocation failure would cause panic anyway
    /// 
    /// use only if allocation failure would cause panic anyway
    /// 
    /// does not affect data or length
    pub fn reserve(&mut self, add: usize) {
        let mut required = self.size + add as u32;

        if required > self.cap {
            required = Self::init_capacity(required);

            let d: *mut T;

            if let Some(data) = self.data {
                d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr() as *mut u8, Self::mk_layout_unchecked(self.cap as usize), required as usize)
                } as *mut T;
            } else {
                d = unsafe {
                    ALLOCATOR.alloc(Self::mk_layout_unchecked(required as usize))
                } as *mut T;
            }

            if d.is_null() {
                panic!("failed to allocate data for Vec");
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.cap = required;
        }
    }

    /// reserves at least `add` more elements on the heap
    /// - after this operation, capacity will be greater than or equal to `self.len() + add`
    /// - returns `Err` if allocation fails
    /// 
    /// does not affect data or length
    pub fn try_reserve(&mut self, add: usize) -> Result<(), ()> {
        let mut required = self.size + add as u32;

        if required > self.cap {
            required = Self::init_capacity(required);

            let d: *mut T;

            if let Some(data) = self.data {
                let layout = Self::mk_layout_unchecked(self.capacity());

                d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr() as *mut u8, layout, required as usize)
                } as *mut T;
            } else {
                let layout = Self::mk_layout_unchecked(required as usize);

                d = unsafe {
                    ALLOCATOR.alloc(layout)
                } as *mut T;
            }

            if d.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.cap = required;
        }
        Ok(())
    }

    /// reserves at least `add` more elements on the heap
    /// - after this operation, capacity will be greater than or equal to `self.len() + add`
    /// - returns `Err` if allocation fails
    /// - use only if allocation failure would cause panic anyway
    /// 
    /// use only if allocation failure would cause panic anyway
    /// 
    /// does not affect data or length
    /// 
    /// unlike `reserve` does not overallocate memory
    pub fn reserve_exact(&mut self, add: usize) {
        let required = self.size + add as u32;

        if required > self.cap {

            let d: *mut T;

            if let Some(data) = self.data {
                d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr() as *mut u8, Self::mk_layout_unchecked(self.cap as usize), required as usize)
                } as *mut T;
            } else {
                d = unsafe {
                    ALLOCATOR.alloc(Self::mk_layout_unchecked(required as usize))
                } as *mut T;
            }

            if d.is_null() {
                panic!("failed to allocate data for Vec");
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.cap = required;
        }
    }

    /// reserves at least `add` more elements on the heap
    /// - after this operation, capacity will be greater than or equal to `self.len() + add`
    /// - returns `Err` if allocation fails
    /// 
    /// does not affect data or length
    pub fn try_reserve_exact(&mut self, add: usize) -> Result<(), ()> {
        let required = self.size + add as u32;

        if required > self.cap {

            let d: *mut T;

            if let Some(data) = self.data {
                let layout = Self::mk_layout_unchecked(self.capacity());

                d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr() as *mut u8, layout, required as usize)
                } as *mut T;
            } else {
                let layout = Self::mk_layout_unchecked(required as usize);

                d = unsafe {
                    ALLOCATOR.alloc(layout)
                } as *mut T;
            }

            if d.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.cap = required;
        }
        Ok(())
    }


    /// shrinks the `capacity` of the string to fit its `len`
    /// - panics if allocation fails
    /// - use only if allocation failure would cause panic anyway
    /// 
    /// this function is optimized to reallocate data only if its worth it
    pub fn shrink_to_fit(&mut self) {
        if let Some(data) = self.data {
            if self.cap - self.size > size_of::<T>() as u32 * 2 {

                let old = Self::mk_layout_unchecked(self.capacity());
                let new = Self::mk_layout_unchecked(self.size as usize);
                let d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr() as *mut u8, old, new.size())
                } as *mut T;

                if d.is_null() {
                    panic!("failed to allocate memory for Vec");
                }

                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.cap = self.size;
            }
        }
    }

    /// shrinks the `capacity` of the string to fit its `len`
    /// - returns `Err` if failed to allocate data
    /// 
    /// this function is optimized to reallocate data only if its worth it
    pub fn try_shrink_to_fit(&mut self) -> Result<(), ()> {
        if let Some(data) = self.data {
            if self.cap - self.size > size_of::<T>() as u32 * 2 {

                let old = Self::mk_layout_unchecked(self.capacity());
                let d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr() as *mut u8, old, size_of::<T>() * self.size as usize)
                } as *mut T;

                if d.is_null() {
                    return Err(());
                }

                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.cap = self.size;
            }
            Ok(())
        } else {
            Ok(())
        }
    }


    /// shrinks the `capacity` of the string to fit `size`
    /// - panics if allocation fails
    /// - use only if allocation failure would cause panic anyway
    /// - `size` must be less than `capacity` else is no-op
    /// 
    /// this function is optimized to reallocate data only if its worth it
    pub fn shrink_to(&mut self, len: usize) {

        if len > 0 && len < self.cap as usize {
            if let Some(data) = self.data {
                if self.cap - len as u32 > size_of::<T>() as u32 * 2 {

                    let old = Self::mk_layout_unchecked(self.capacity());
                    let new = Self::mk_layout_unchecked(len);
                        //  new is create only to get size
                    let d = unsafe {
                        ALLOCATOR.realloc(data.as_ptr() as *mut u8, old, new.size())
                    } as *mut T;

                    if d.is_null() {
                        panic!("failed to allocate memory for Vec");
                    }

                    self.data = Some(unsafe { NonNull::new_unchecked(d) });
                    self.cap = self.size;
                }
            }
        }
    }

    /// shrinks the `capacity` of the string to fit `size`
    /// - returns `Err` if allocation fails
    /// - `size` must be less than `capacity` else is no-op
    /// 
    /// this function is optimized to reallocate data only if its worth it
    pub fn try_shrink_to(&mut self, len: usize) -> Result<(), ()> {
        if len > 0 && len < self.cap as usize {
            if let Some(data) = self.data {
                if self.cap - len as u32 > size_of::<T>() as u32 * 2 {

                    let old = Self::mk_layout_unchecked(self.capacity());
                    let new = Self::mk_layout_unchecked(len);
                        //  new is create only to get size
                    let d = unsafe {
                        ALLOCATOR.realloc(data.as_ptr() as *mut u8, old, new.size())
                    } as *mut T;

                    if d.is_null() {
                        return Err(());
                    }

                    self.data = Some(unsafe { NonNull::new_unchecked(d) });
                    self.cap = self.size;
                }
            }
        }
        Ok(())
    }

    /// shortens the vector to `size` while dropping other elements
    /// - if has no data or `self.len() < size` => **no-op**
    pub fn truncate(&mut self, len: usize) {
        if self.data.is_some() {
            if len < self.size as usize {
                unsafe { drop_in_place(self.get_unchecked_mut(len..self.len())) };
                self.size = len as u32;
            }
        }
    }

    /// removes element at some index and returns it
    /// - `index` must be less or equal than len
    /// - element at `index` will be assigned to `default`
    /// - panics if index is out of bounds of bounds or vector is empty
    /// - this is an `O(n)` operation
    pub fn remove(&mut self, index: usize) -> T {

        if self.data.is_some() && index < self.len() {
            unsafe {
                let ptr = self.data.unwrap().add(index);
                let val = ptr.read();
                core::ptr::copy(ptr.add(1).as_ptr(), ptr.as_ptr(), self.len() - index - 1);
                self.size -= 1;

                val
            }
        } else {
            if self.data.is_none() {
                panic!("Vec is empty");
            } else {
                panic!("index is out of bounds");
            }
        }

    }

    /// removes element at some index and returns it
    /// - `index` must be less or equal than len
    /// - returns `Err` if index is out of bounds or vector is empty
    /// - this is an `O(n)` operation
    pub fn try_remove(&mut self, index: usize) -> Result<T, ()> {
        if self.data.is_some() && index < self.len() {
            unsafe {
                let ptr = self.data.unwrap().add(index);
                let val = ptr.read();
                core::ptr::copy(ptr.add(1).as_ptr(), ptr.as_ptr(), self.len() - index - 1);
                self.size -= 1;

                Ok(val)
            }
        } else {
            Err(())
        }
    }

    /// removes element at some index without returning it
    /// - `index` must be less or equal than len
    /// - returns `Err` if index is out of bounds or vector is empty
    /// - this is an `O(n)` operation
    pub fn delete(&mut self, index: usize) -> Result<(), ()> {
        if self.data.is_some() && index < self.len() {
            unsafe {
                let ptr = self.data.unwrap().add(index);
                drop_in_place(ptr.as_ptr());
                core::ptr::copy(ptr.add(1).as_ptr(), ptr.as_ptr(), self.len() - index - 1);
                self.size -= 1;

                Ok(())
            }
        } else {
            Err(())
        }
    }

    /// forces `len` of the vector to certain value
    /// - the new `len` must be less or equal to `capacity`
    /// - definition of this method differs from the stdlib definition
    /// - returns `Err` if len is out of bounds
    /// 
    /// safety:
    /// - use of this function may lead to uninitialized objects at the end of the vector
    /// - look for `set_len_assign` for safer alternative
    pub unsafe fn set_len(&mut self, len: usize) -> Result<(), ()> {
        if self.data.is_some() && self.cap >= len as u32 {
            self.size = len as u32;
            Ok(())
        } else {
            Err(())
        }
    }

    /// returns subslice from vector
    pub fn get<R>(&self, range: R) -> Option<&[T]>
    where R: RangeBounds<usize> {
        if let Some(data) = self.data {
            let (start, end) = self.handle_bounds(&range);

            if start >= self.len() || end >= self.len() {
                return None;
            }

            Some(unsafe { slice::from_raw_parts(data.add(start).as_ptr(), end - start) })
        } else {
            None
        }
    }

    /// returns sublice from vector
    /// - does not do bound checks
    pub unsafe fn get_unchecked<R>(&self, range: R) -> &[T]
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);
        unsafe { slice::from_raw_parts(self.data.unwrap().add(start).as_ptr(), end - start) }
    }

    /// returns mutable subslice from vector
    pub fn get_mut<R>(&mut self, range: R) -> Option<&mut [T]>
    where R: RangeBounds<usize> {
        if let Some(data) = self.data {
            let (start, end) = self.handle_bounds(&range);

            if start >= self.len() || end >= self.len() {
                return None;
            }

            Some(unsafe { slice::from_raw_parts_mut(data.add(start).as_ptr(), end - start) })

        } else {
            None
        }
    }

    pub unsafe fn get_unchecked_mut<R>(&mut self, range: R) -> &mut [T]
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);
        unsafe { slice::from_raw_parts_mut(self.data.unwrap().add(start).as_ptr(), end - start) }
    }

    /// appends element to the vector
    /// - panics if allocation fails
    pub fn push(&mut self, val: T) {
        self.reserve(1);
        unsafe {
            let dest = self.data.unwrap().add(self.len());
            dest.as_ptr().write(val)
        };
        self.size += 1;
    }

    /// appends element to the vector
    /// - returns `Err` if allocation fails
    pub fn try_push(&mut self, val: T) -> Result<(), ()> {
        self.try_reserve(1)?;
        unsafe {
            let dest = self.data.unwrap().add(self.len());
            dest.as_ptr().write(val)
        };
        self.size += 1;
        Ok(())
    }

    /// tries to push within capacity
    /// / if fails returns `val`
    pub fn push_within_capacity(&mut self, val: T) -> Result<(), T> {
        if self.size < self.cap {
            let ptr = self.data.unwrap();

            unsafe { ptr.add(self.len()).as_ptr().write(val) };
            self.size += 1;

            Ok(())
            
        } else {
            Err(val)
        }
    }

    /// pops from the vector
    /// - unlike `std::Vec::pop` does not return the popped value
    ///   - popped value is dropped
    ///   - to get the value use `pop_get`
    /// - does not affect `capacity`
    /// 
    /// - returns `Err` if fails
    pub fn pop(&mut self) -> Result<(), ()> {
        if self.data.is_some() && self.size > 0 {
            let element = unsafe { self.data.unwrap().add(self.len()-1)  };
            unsafe { drop_in_place(element.as_ptr() ); }
            self.size -= 1;
            Ok(())
        } else {
            Err(())
        }
    }

    /// pops from the vector and returns the popped value
    /// - does not affect `capacity`
    /// 
    /// - returns `Err` if fails
    pub fn pop_get(&mut self) -> Result<T, ()> {
        if self.data.is_some() && self.size > 0 {
            let element = unsafe { self.data.unwrap().add(self.len()-1) };
            self.size -= 1;
            Ok(unsafe { element.read() })
        } else {
            Err(())
        }
    }


    /// appends all elements from `other` to `self`
    /// - leaves `other` empty
    /// - returns `Err` if fails
    pub fn append(&mut self, other: &mut Vec<T>) -> Result<(), ()> {
        if other.is_empty() {
            return Ok(())
        }
        self.try_reserve(other.len())?;
        let data = unsafe { self.data.unwrap().add(self.len()) };
        let odata = other.data.unwrap();
        for i in 0..other.size as usize {
            unsafe { data.add(i).write(odata.add(i).read()) }
        }

        other.drop_all();

        Ok(())
    }

    /// removes all values from the vector
    pub fn clear(&mut self) {
        unsafe { drop_in_place(self.as_mut_slice_unchecked()); }
        self.size = 0;
    }

    /// resizes `Vec` to `size` while assigning calling `f` on each new element
    /// - if Vec is shrinking => drops all values
    /// - returns `Err` if allocation fails
    pub fn resize_with<F>(&mut self, size: usize, mut f: F) -> Result<(), ()>
    where F: FnMut() -> T {

        if self.data.is_none() {
            //  allocate and set values

            let cap = Self::init_capacity(size as u32) as usize;

            let d = unsafe {
                ALLOCATOR.alloc(Self::mk_layout_unchecked(cap))
            } as *mut T;

            if d.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });

            self.cap = cap as u32;
            self.size = size as u32;

            for i in unsafe { slice::from_raw_parts_mut(d, size) } {
                *i = f();
            }

            Ok(())

        } else {

            match (self.size as usize).cmp(&size) {
                Less => {

                    let required = Self::init_capacity(size as u32) as usize;

                    let start = self.size as usize;

                    if self.cap < required as u32 {
                        //  expand allocated data
                        let d = unsafe {
                            ALLOCATOR.realloc(self.data.unwrap().as_ptr() as *mut u8, Self::mk_layout_unchecked(self.cap as usize), required)
                        } as *mut T;

                        if d.is_null() {
                            return Err(());
                        }

                        self.data = Some(unsafe { NonNull::new_unchecked(d) });
                        self.cap = required as u32;
                    }

                    self.size = size as u32;

                    for i in unsafe { self.get_unchecked_mut(start..self.size as usize) } {
                        *i = f();
                    }

                },
                Equal => return Ok(()),
                Greater => {
                    unsafe { drop_in_place(self.get_unchecked_mut(self.size as usize..size)); }
                    self.size = size as u32;
                }
            }

            Ok(())
        }
    }

    /// returns reference to the allocated data while consuming the vector
    pub fn leak<'l>(self) -> &'l mut [T] {
        if let Some(data) = self.data {
            unsafe { slice::from_raw_parts_mut(data.as_ptr(), self.len()) }
        } else {
            panic!("Vec does not contain any data");
        }
    }



    /// drops all elements and deallocate vector memory
    pub fn drop_all(&mut self) {
        if self.size > 0 {
            unsafe { drop_in_place(self.as_mut_slice_unchecked()); }
        }
        if let Some(data) = self.data {
            unsafe {
                ALLOCATOR.dealloc(data.as_ptr() as *mut u8, Self::mk_layout_unchecked(self.capacity()));
            }
        }
        self.data = None;
        self.size = 0;
        self.cap = 0;
    }

}


impl<T: Copy + Clone> Vec<T> {
    /// forces `len` of vector and assigns uninitialized elements to `val`
    /// - the new `len` must be less or equal to `capacity`
    pub fn set_len_assign(&mut self, len: usize, val: T) -> Result<(), ()> {
        if self.data.is_some() && self.cap >= len as u32 {
            let slice = match self.get_mut(self.len()..len as usize) {
                Some(s) => s,
                None => return Err(()),
            };
            for i in slice {
                *i = val;
            }
            Ok(())
        } else {
            Err(())
        }
    }
    
}

impl<T: Default> Vec<T> {

    /// resizes vector while assigning `default` to new element
    /// - returns `Err` if allocation fails
    pub fn resize(&mut self, size: usize) -> Result<(), ()> {
        if self.data.is_none() {
            //  allocate and set values

            let cap = Self::init_capacity(size as u32) as usize;
            
            let d = unsafe {
                ALLOCATOR.alloc(Self::mk_layout_unchecked(cap))
            } as *mut T;

            if d.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });

            self.cap = cap as u32;
            self.size = size as u32;

            for i in unsafe { slice::from_raw_parts_mut(d, size) } {
                *i = T::default();
            }

            Ok(())

        } else {

            match (self.size as usize).cmp(&size) {
                Less => {

                    let required = Self::init_capacity(size as u32) as usize;

                    let start = self.size as usize;

                    if self.cap < required as u32 {
                        //  expand allocated data
                        let d = unsafe {
                            ALLOCATOR.realloc(self.data.unwrap().as_ptr() as *mut u8, Self::mk_layout_unchecked(self.cap as usize), required)
                        } as *mut T;

                        if d.is_null() {
                            return Err(());
                        }

                        self.data = Some(unsafe { NonNull::new_unchecked(d) });
                        self.cap = required as u32;
                    }

                    self.size = size as u32;

                    for i in unsafe { self.get_unchecked_mut(start..self.size as usize) } {
                        *i = T::default();
                    }

                },
                Equal => return Ok(()),
                Greater => {
                    unsafe { drop_in_place(self.get_unchecked_mut(self.size as usize..size)); }
                    self.size = size as u32;
                }
            }

            Ok(())
        }
    }

    /// removes element at some index and returns it
    /// - panics if index is out of the bounds of bounds or vector is empty
    /// - `index` must be less or equal than len
    /// - does not preserve ordering of the remaining object but is `O(1)`
    ///   - to preserve it, use `remove`
    /// - value of the element at the `index` position is set to `default`
    pub fn swap_remove(&mut self, index: usize) -> T {

        if self.data.is_some() && index < self.len() {
            unsafe {
                let ptr = self.data.unwrap().add(index);
                let val = ptr.read();
                ptr.write(T::default());
                self.size -= 1;
                
                val
            }
        } else {
            if self.data.is_none() {
                panic!("Vec is empty");
            } else {
                panic!("index is out of bounds");
            }
        }

    }

    /// removes element at some index and returns it
    /// - returns `Err` if index is out of bounds of bounds or vector is empty
    /// - `index` must be less or equal than len
    /// - does not preserve ordering of the remaining object but is `O(1)`
    ///   - to preserve it, use `remove`
    /// - value of the element at the `index` position is set to `default`
    pub fn try_swap_remove(&mut self, index: usize) -> Result<T, ()> {
        if self.data.is_some() && index < self.len() {
            unsafe {
                let ptr = self.data.unwrap().add(index);
                let val = ptr.read();
                ptr.write(T::default());
                self.size -= 1;
                
                Ok(val)
            }
        } else {
            Err(())
        }
    }

    /// removes element at some index without returning it
    /// - returns `Err` if index is out of bounds of bounds or vector is empty
    /// - `index` must be less or equal than len
    /// - does not preserve ordering of the remaining object but is `O(1)`
    ///   - to preserve it, use `remove`
    /// - value of the element at the `index` position is set to `default`
    pub fn swap_delete(&mut self, index: usize) -> Result<(), ()> {
        if self.data.is_some() && index < self.len() {
            unsafe {
                let ptr = self.data.unwrap().add(index);
                drop_in_place(ptr.as_ptr());
                ptr.write(T::default());
                self.size -= 1;
                
                Ok(())
            }
        } else {
            Err(())
        }
    }


}

impl<T: Sized> Vec<T> {

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// returns the number of elements allocated in vector
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.cap as usize
    }

    /// returns the length of the vector
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.size as usize
    }

    /// returns vector content as slice of T
    pub const fn as_slice(&self) -> Option<&[T]> {
        if let Some(data) = self.data {
            Some(unsafe { slice::from_raw_parts(data.as_ptr(), self.size as usize) })
        } else {
            None
        }
    }

    /// returns vector content as mutable slice of T
    pub const fn as_mut_slice(&mut self) -> Option<&mut [T]> {
        if let Some(data) = self.data {
            Some(unsafe { slice::from_raw_parts_mut(data.as_ptr(), self.size as usize) })
        } else {
            None
        }
    }

    /// returns vector content as slice of T
    /// - panics if vector has no content
    pub unsafe fn as_slice_unchecked(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.unwrap().as_ptr(), self.size as usize) }
    }

    /// returns vector content as mutalbe slice of T
    /// - panics if vector has no content
    pub const unsafe fn as_mut_slice_unchecked(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.unwrap().as_ptr(), self.size as usize) }
    }

    /// returns pointer to vector content
    pub const fn as_ptr(&self) -> *const T {
        if let Some(data) = self.data {
            data.as_ptr()
        } else {
            null()
        }
    }

    /// returns mutable pointer to vector content
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        if let Some(data) = self.data {
            data.as_ptr()
        } else {
            null_mut()
        }
    }


    pub fn iter<'l>(&self) -> VecIter<'l, T> {
        VecIter::new(&self)
    }

    pub fn iter_mut<'l>(&'l mut self) -> VecIterMut<'l, T> {
        VecIterMut::new(self)
    }


}

impl<T> AsMut<[T]> for Vec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        if let Some(data) = self.data {
            unsafe { slice::from_raw_parts_mut(data.as_ptr(), self.len()) }
        } else {
            panic!("Vec does not contain any data");
        }
    }
}

impl<T> AsRef<[T]> for Vec<T> {
    fn as_ref(&self) -> &[T] {
        if let Some(data) = self.data {
            unsafe { slice::from_raw_parts(data.as_ptr(), self.len()) }
        } else {
            panic!("Vec does not contain any data");
        }
    }
}

impl<T> Borrow<[T]> for Vec<T> {
    fn borrow(&self) -> &[T] {
        if let Some(data) = self.data {
            unsafe { slice::from_raw_parts(data.as_ptr(), self.len()) }
        } else {
            panic!("Vec does not cotain any data");
        }
    }
}

impl<T> BorrowMut<[T]> for Vec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        if let Some(data) = self.data {
            unsafe { slice::from_raw_parts_mut(data.as_ptr(), self.len()) }
        } else {
            panic!("Vec does not cotain any data");
        }
    }
}


impl<T: Clone> Clone for Vec<T> {
    fn clone(&self) -> Self {
        if let Some(data) = self.data {
            let d = unsafe {
                ALLOCATOR.alloc(Self::mk_layout_unchecked(self.capacity()))
            } as *mut T;

            if d.is_null() {
                panic!("failed to allocate memory for Vec");
            }

            for (i, item) in unsafe { slice::from_raw_parts_mut(d, self.len()) }.iter_mut().enumerate() {
                *item = unsafe { data.add(i).as_ref().clone() };
            }

            Self {
                data: Some(unsafe { NonNull::new_unchecked(d) }),
                size: self.size,
                cap: self.cap
            }

        } else {
            Self {
                data: None,
                size: 0,
                cap: 0
            }
        }
    }
}

impl<T: TryClone> TryClone for Vec<T> {
    type Error = ();
    fn try_clone(&self) -> Result<Self, Self::Error>
        where Self: Sized {
        if let Some(data) = self.data {
            let d = unsafe {
                ALLOCATOR.alloc(Self::mk_layout_unchecked(self.capacity()))
            } as *mut T;

            if d.is_null() {
                return Err(());
            }

            for (i, item) in unsafe { slice::from_raw_parts_mut(d, self.len()) }.iter_mut().enumerate() {
                *item = match unsafe { data.add(i).as_ref().try_clone() } {
                    Ok(it) => it,
                    Err(_) => return Err(()),
                };
            }

            Ok( Self {
                data: Some(unsafe { NonNull::new_unchecked(d) }),
                size: self.size,
                cap: self.cap
            })

        } else {
            Ok( Self {
                data: None,
                size: 0,
                cap: 0
            })
        }
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self {
            data: None,
            size: 0,
            cap: 0,
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        if let Some(data) = self.data {
            unsafe { slice::from_raw_parts(data.as_ptr(), self.len()) }
        } else {
            panic!("Vec does not contain any data");
        }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(data) = self.data {
            unsafe { slice::from_raw_parts_mut(data.as_ptr(), self.len()) }
        } else {
            panic!("Vec does not contain any data");
        }
    }
}

impl<T> Drop for Vec<T> {

    fn drop(&mut self) {

        if let Some(data) = self.data {

            crate::println!("dropping values in Vec");

            if core::mem::needs_drop::<T>() {
                //  drop all values if needed
                for i in unsafe { self.as_mut_slice_unchecked() } {
                    unsafe { drop_in_place(i); }
                }
            }

            unsafe {
                //  deallocate memory
                ALLOCATOR.dealloc(data.as_ptr() as *mut u8, Self::mk_layout_unchecked(self.capacity()));
            };

        }

    }

}

*/