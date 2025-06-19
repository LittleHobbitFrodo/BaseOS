//  mem/vec/mod.rs (ministd crate)
//  this file originally belonged to baseOS project
//      on OS template on which to build


use core::ptr::drop_in_place;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::ops::{Bound::*, Index, IndexMut, RangeBounds, Deref, DerefMut};

use crate::mem::dynamic_buffer::DynamicBuffer;
use crate::TryClone;


/// A contiguous growable array type, written as Vec<T>, short for ‘vector’.
/// 
/// ## Implementation details
/// ### Generic parameters
/// 1. `T`: datatype of each element
/// 2. `STEP`: indicates how much will vector grow
///     - geometrical growth is used by default
pub struct Vec<T: Sized, const STEP: usize = 0> {
    data: DynamicBuffer<T, STEP>,
    size: usize,
}

impl<T: Sized, const STEP: usize> Vec<T, STEP> {

    /// Constructs new empty `Vec<T>`
    pub const fn new() -> Self {
        Self {
            data: DynamicBuffer::empty(),
            size: 0,
        }
    }


    /// Constructs new empty `Vec` with at least the specified capacity allocated
    /// - **panics** if allocation fails
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: DynamicBuffer::with_capacity(capacity),
            size: 0,
        }
    }


    /// Tries to construct new empty `Vec<T>` with at least the specified capacity allocated
    /// - returns `Err` if allocation fails
    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ()> {
        Ok(Self {
            data: DynamicBuffer::try_with_capacity(capacity)?,
            size: 0,
        })
    }

    /// Reserves capacity for at least `additional` more elements
    /// - **panics** if allocation fails
    #[inline]
    pub fn reserve(&mut self, additional: usize) {

        if self.len() + additional > self.capacity() {
            self.data.resize(self.len() + additional);
        }

    }

    /// Tries to reserve capacity for at least `additional` more elements
    /// - returns `Err` if allocation fails
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {
        if self.len() + additional > self.capacity() {
            self.data.try_resize(self.len() + additional)
        } else {
            Ok(())
        }
    }

    /// Reserves the minimum capacity for at least `additional` more elements
    /// - **panics** if allocation fails
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            self.data.resize_exact(self.len() + additional);
        }
    }

    /// Tries to reserve the minimum capacity for at least `additional` more elements
    /// - returns `Err` if allocation fails
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), ()> {
        if self.len() + additional > self.capacity() {
            self.data.try_resize_exact(self.len() + additional)
        } else {
            Ok(())
        }
    }

    /// Appends one element at the end of the vector
    /// - **panics** if allocation fails
    pub fn push(&mut self, val: T) {
        if self.len() == self.capacity() {
            self.data.expand();
        }
        
        unsafe {
            self.data.as_ptr().add(self.len()).write(val);
        }
        self.size += 1;
    }

    /// Tries to append one element at the end of the vector
    /// - returns `Err` if allocation fails
    ///     - in this case returns the ownership of `val`
    pub fn try_push(&mut self, val: T) -> Result<(), T> {
        if self.len() == self.capacity() {
            if let Err(_) = self.data.try_expand() {
                return Err(val);
            }
        }
        unsafe {
            self.data.as_ptr().add(self.len()).write(val);
        }

        self.size += 1;

        Ok(())

    }

    /// Appends the vector if there is enough spare space in allocated memory
    pub fn push_within_capacity(&mut self, val: T) -> Result<(), T> {
        if self.len() == self.capacity() {
            return Err(val);
        } else {
            unsafe {
                self.data.as_ptr().add(self.len()).write(val);
            }
            self.size += 1;
            Ok(())
        }
    }

    /// Shrinks the capacity of the vector as much as possible
    /// - **panics** if allocation fails
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.data.resize_exact(self.len());
    }

    /// Shrinks the capacity of the vector as much as possible
    /// - returns `Err` if allocation fails
    #[inline]
    pub fn try_shrink_to_fit(&mut self) -> Result<(), ()> {
        self.data.try_resize_exact(self.len())
    }

    /// Shrinks the vector to some size while dropping all elements that will not be preserved
    /// - **panics** if allocation fails
    pub fn shrink_to(&mut self, size: usize) {
        if self.capacity() > size {
            if size < self.len() {
                unsafe {
                    drop_in_place(from_raw_parts_mut(self.data.as_ptr()
                    .add(size), self.len() - size));
                }
            }
            self.data.resize_exact(size);
        }
    }

    /// Tries to shrink the vector to some size while dropping all elements that will not be preserved
    /// - returns `Err` if allocation fails
    pub fn try_shrink_to(&mut self, size: usize) -> Result<(), ()> {
        if self.capacity() > size {
            if size < self.len() {
                unsafe {
                    drop_in_place(from_raw_parts_mut(self.data.as_ptr()
                    .add(size), self.len()));
                }
            }

            self.data.try_resize_exact(size)
        } else {
            Ok(())
        }
    }


    /// Forces the length of the vector to new_len.
    /// - this will not construct and/or modify `capacity`
    /// - this function does not check for any boundaries (including `capacity`)
    #[inline(always)]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.size = len;
    }

    /// Clears the vector, removing all values.
    /// - note that this method has no effect on the allocated capacity of the vector.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.size = 0;
    }












    /// Returns number of elements in the vector
    pub const fn len(&self) -> usize {
        self.size
    }

    /// Returns number of elements allocated by the vector
    pub const fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Checks whether the vector is empty
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Checks if vector has any allocated data
    pub const fn has_data(&self) -> bool {
        self.data.has_data()
    }

    pub const fn step(&self) -> usize {
        STEP
    }




    /// returns contents of the vector as slice
    /// - or `None` if vector does not have any contents
    pub const fn as_slice(&self) -> Option<&[T]> {
        if let Some(data) = self.data.data() {
            Some(unsafe { from_raw_parts(data.as_ptr(), self.len()) })
        } else {
            None
        }
    }

    /// returns contents of the vector as mutable slice
    /// - or `None` if vector does not have any contents
    pub const fn as_mut_slice(&self) -> Option<&mut [T]> {
        if let Some(data) = self.data.data() {
            Some(unsafe { from_raw_parts_mut(data.as_ptr(), self.len()) })
        } else {
            None
        }
    }


    /// returns contents of the vector as slice
    /// - **panics** if vector does not have any contents
    pub const fn as_slice_unchecked(&self) -> &[T] {
        unsafe { from_raw_parts(self.data.data()
        .expect("Vec does not contain any data").as_ptr(), self.len())}
    }

    /// returns contents of the vector as mutable slice
    /// - **panics** if vector does not have any contents
    pub const fn as_mut_slice_unchecked(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.data.data()
        .expect("Vec does not contain any data").as_ptr(), self.len())}
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

    /// Returns an subslice from the vector
    /// - or `None` if vector has no data or out of bounds
    pub fn get<R>(&self, range: R) -> Option<&[T]>
    where R: RangeBounds<usize> {
        if let Some(data) = self.data.data() {
            let (start, end) = self.handle_bounds(&range);

            if start > self.len() || end > self.len() {
                return None;
            }

            Some(unsafe { from_raw_parts(data.add(start).as_ptr(), end - start) })
        } else {
            None
        }
    }

    /// Returns an mutable subslice from the vector
    /// - or `None` if vector has no data or out of bounds
    pub fn get_mut<R>(&self, range: R) -> Option<&mut [T]>
    where R: RangeBounds<usize> {
        if let Some(data) = self.data.data() {
            let (start, end) = self.handle_bounds(&range);

            if start > self.len() || end > self.len() {
                return None;
            }

            Some(unsafe { from_raw_parts_mut(data.add(start).as_ptr(), end - start) })
        } else {
            None
        }
    }

    /// Returns an sublice from the vector withou doing bounds check
    /// - **panics** if has no data
    pub unsafe fn get_unchecked<R>(&self, range: R) -> &[T]
    where R: RangeBounds<usize> {
        if let Some(data) = self.data.data() {
            let (start, end) = self.handle_bounds(&range);
            unsafe {
                from_raw_parts(data.add(start).as_ptr(), end - start)
            }
        } else {
            panic!("Vec does not contain any data");
        }
    }

    /// Returns an mutable sublice from the vector withou doing bounds check
    /// - **panics** if has no data
    pub unsafe fn get_unchecked_mut<R>(&mut self, range: R) -> &[T]
    where R: RangeBounds<usize> {
        if let Some(data) = self.data.data() {
            let (start, end) = self.handle_bounds(&range);
            unsafe {
                from_raw_parts_mut(data.add(start).as_ptr(), end - start)
            }
        } else {
            panic!("Vec does not contain any data");
        }
    }

    /// Returns iterator for this vector
    #[inline(always)]
    pub fn iter<'l>(&'l self) -> core::slice::Iter<'l, T> {
        self.as_slice_unchecked().iter()
    }

    pub fn iter_mut<'l>(&'l mut self) -> core::slice::IterMut<'l, T> {
        self.as_mut_slice_unchecked().iter_mut()
    }


}


impl<T: Sized, const STEP: usize> Drop for Vec<T, STEP> {
    fn drop(&mut self) {
        if let Some(data) = self.data.data() {
            unsafe {
                drop_in_place(from_raw_parts_mut(data.as_ptr(), self.len()));
            }
        }
    }
}

impl<T: Sized, const STEP: usize> Index<usize> for Vec<T, STEP> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(data) = self.data.data() {
            if index < self.len() {
                unsafe {
                    return data.add(index).as_ref();
                }
            }
        }

        panic!("Vec[]: out of bounds");
    }
}

impl<T: Sized, const STEP: usize> IndexMut<usize> for Vec<T, STEP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(data) = self.data.data() {
            if index < self.len() {
                unsafe {
                    return data.add(index).as_mut();
                }
            }
        }
        panic!("Vec[]: out of bounds");
    }
}

impl<T: Sized + Clone, const STEP: usize> Clone for Vec<T, STEP> {
    fn clone(&self) -> Self {

        let db = self.data.clone();

        let new_slice = unsafe { from_raw_parts_mut(db.as_ptr(), self.len()) };
        let old_slice = unsafe { from_raw_parts(self.data.as_ptr(), self.len()) };

        for (i, item) in new_slice.iter_mut().enumerate() {
            *item = old_slice[i].clone();
        }

        Self {
            data: db,
            size: self.len(),
        }

    }
}

impl<T: Sized + TryClone, const STEP: usize> TryClone for Vec<T, STEP> {
    type Error = ();

    fn try_clone(&self) -> Result<Self, Self::Error>
    where Self: Sized, Self::Error: Default {
        
        let db = self.data.try_clone()?;

        let new_slice = unsafe { from_raw_parts_mut(db.as_ptr(), self.len()) };
        let old_slice = unsafe { from_raw_parts(self.data.as_ptr(), self.len()) };

        //  copy all elements (DynamicBuffer does not do that)
        for (i, item) in new_slice.iter_mut().enumerate() {
            *item = match old_slice[i].try_clone() {
                Ok(i) => i,
                Err(_) => {
                    unsafe { drop_in_place(new_slice[0..i].as_mut_ptr()) }
                    return Err(());
                },
            };
        }

        Ok(Self {
            data: db,
            size: self.len(),
        })

    }
}

impl<T: Sized, const STEP: usize> Deref for Vec<T, STEP> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice_unchecked()
    }
}

impl<T: Sized, const STEP: usize> DerefMut for Vec<T, STEP> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice_unchecked()
    }
}

impl<T: Sized, const STEP: usize> Default for Vec<T, STEP> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}





/*use core::{alloc::{GlobalAlloc, Layout, LayoutError}, ascii, fmt::Write, hint::spin_loop, mem::ManuallyDrop, ops::RangeBounds, ptr::{self, drop_in_place, null, null_mut, NonNull}, slice};

use core::ops::Bound::{Included, Excluded, Unbounded};

use core::cmp::Ordering::{self, Less, Greater, Equal};

use crate::{TryClone, ALLOCATOR, convert::Align};

use core::borrow::{Borrow, BorrowMut};

use core::ops::{Deref, DerefMut};




/// Custom implementation of `std::Vec` - dynamic array
/// 
/// - Uses exponencial growth
pub struct Vec<T: Sized> {
    data: Option<NonNull<T>>,
    layout: Layout, //  memory layout
    size: u32,  //  size of the vector
    cap: u32,   //  capacity in elements
}


impl<T: Sized> Vec<T> {

    const fn layout_for(capacity: usize) -> Layout {
        let size = unsafe {
            size_of::<T>().unchecked_mul(Self::cap_next(capacity as u32) as usize)
        };
        
        unsafe { Layout::from_size_align_unchecked(size, align_of::<T>()) }
    }

    const fn layout_for_exact(capacity: usize) -> Layout {
        let size = unsafe {
            size_of::<T>().unchecked_mul(capacity)
        };

        unsafe { Layout::from_size_align_unchecked(size, align_of::<T>()) }
    }

    /// aligns capacity for exponencional growth
    pub(crate) const fn cap_next(cap: u32) -> u32 {
        cap.next_power_of_two()
    }

    /// Constructs empty `Vec<T>` with no data
    pub const fn new() -> Self {
        Self {
            data: None,
            layout: Layout::new::<T>(),
            size: 0,
            cap: 0,
        }
    }

    /// Constructs `Vec<T>` with some elements allocated
    /// - **panics** if allocation fails
    pub fn with_capaity(capacity: usize) -> Self {
        let cap = Self::cap_next(capacity as u32);
        let l = Self::layout_for(capacity);
        let data = unsafe {
            ALLOCATOR.alloc(l)
        } as *mut T;

        assert!(!data.is_null(), "failed to allocate data for Vec");

        Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: l,
            size: 0,
            cap: ,
        }
    }

    /// Tries to construct `Vec<T>` with some capacity
    /// - returns `Err` if allocation fails
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ()> {
        let l = Self::layout_for(capacity);
        let data = unsafe {
            ALLOCATOR.alloc(l)
        } as *mut T;

        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            layout: l,
            size: 0,
        })
    }

    pub unsafe fn from_raw_parts(ptr: NonNull<T>, len: usize, layout: Layout) -> Self {
        Self {
            data: Some(ptr),
            layout,
            size: len as u32,
        }
    }

    /// Reserves capacity for at least additional more elements 
    /// - capacity will be greater or equal to `self.len() + additional`
    /// - **panics** if allocation fails
    pub fn reserve(&mut self, additional: usize) {

        if self.data.is_some() {

            if self.len() + additional > self.capacity() {
                //  reallocate data

                let l = Self::layout_for(self.len() + additional);
                let data = unsafe {
                    let d = self.data.unwrap().as_ptr();
                    ALLOCATOR.realloc_layout(d as *mut u8, self.layout, l)
                } as *mut T;

                assert!(!data.is_null(), "failed to reallocate data for Vec");

                self.data = Some(unsafe { NonNull::new_unchecked(data) });
                self.layout = l;

            }
            //  else OK

        } else {
            //  allocate data

            let l = Self::layout_for(additional);

            let data = unsafe {
                ALLOCATOR.alloc(l)
            } as *mut T;

            assert!(!data.is_null(), "failed to allocate data for Vec");

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.size = 0;
        }

    }

    /// Tries to reserve capacity for at least additional more elements 
    /// - capacity will be greater or equal to `self.len() + additional`
    /// - returns `Err` if allocation fails
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {

        if self.data.is_some() {

            if self.len() + additional > self.capacity() {
                //  reallocate data

                let l = Self::layout_for(self.len() + additional);
                let data = unsafe {
                    let d = self.data.unwrap().as_ptr();
                    ALLOCATOR.realloc_layout(d as *mut u8, self.layout, l)
                } as *mut T;

                if data.is_null() {
                    return Err(());
                }

                self.data = Some(unsafe { NonNull::new_unchecked(data) });
                self.layout = l;

            }

            Ok(())

        } else {
            //  allocate data

            let l = Self::layout_for(additional);

            let data = unsafe {
                ALLOCATOR.alloc(l)
            } as *mut T;

            if data.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.size = 0;

            Ok(())
        }

    }



    /// Reserves minimum capacity for at least additional more elements 
    /// - capacity will be greater or equal to `self.len() + additional`
    /// - **panics** if allocation fails
    pub fn reserve_exact(&mut self, additional: usize) {

        if self.data.is_some() {

            if self.len() + additional > self.capacity() {
                //  reallocate data

                let l = Self::layout_for_exact(self.len() + additional);
                let data = unsafe {
                    let d = self.data.unwrap().as_ptr();
                    ALLOCATOR.realloc_layout(d as *mut u8, self.layout, l)
                } as *mut T;

                assert!(!data.is_null(), "failed to reallocate data for Vec");

                self.data = Some(unsafe { NonNull::new_unchecked(data) });
                self.layout = l;

            }
            //  else OK

        } else {
            //  allocate data

            let l = Self::layout_for_exact(additional);

            let data = unsafe {
                ALLOCATOR.alloc(l)
            } as *mut T;

            assert!(!data.is_null(), "failed to allocate data for Vec");

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.size = 0;
        }

    }

    /// Tries to reserve minimum capacity for at least additional more elements 
    /// - capacity will be greater or equal to `self.len() + additional`
    /// - **panics** if allocation fails
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), ()> {

        if self.data.is_some() {

            if self.len() + additional > self.capacity() {
                //  reallocate data

                let l = Self::layout_for_exact(self.len() + additional);
                let data = unsafe {
                    let d = self.data.unwrap().as_ptr();
                    ALLOCATOR.realloc_layout(d as *mut u8, self.layout, l)
                } as *mut T;

                if data.is_null() {
                    return Err(());
                }

                self.data = Some(unsafe { NonNull::new_unchecked(data) });
                self.layout = l;

            }
            //  else OK

            Ok(())

        } else {
            //  allocate data

            let l = Self::layout_for_exact(additional);

            let data = unsafe {
                ALLOCATOR.alloc(l)
            } as *mut T;

            if data.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(data) });
            self.layout = l;
            self.size = 0;

            Ok(())
        }

    }


    pub fn push(&mut self, val: T) {

        if self.len() == self.capacity() {
            self.reserve(1);
        }

        unsafe {
            self.data.unwrap().add(self.len()).write(val)
        }
        self.size += 1;

    }




}


impl<T: Sized> Vec<T> {

    /// Reveals size of the vector in elements
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.size as usize
    }

    /// Returns count of allocated
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.layout.size() as usize
    }


    /// Returns content of the vector as slice
    /// - panics if vector does not contain any data
    #[inline(always)]
    pub const fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.data.expect("Vec does not contain any data")
            .as_ptr(), self.len())
        }
    }

    /// Returns content of the vector as mutable slice
    /// - panics if vector does not contain any data
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.data.expect("Vec does not contain any data")
            .as_ptr(), self.len())
        }
    }

    /// Gets range of elements from the vector
    /// - returns `None` if vector does not contain any data or range is out of bounds
    pub fn get<R>(&self, range: R) -> Option<&[T]>
    where R: RangeBounds<usize> {

        if let Some(data) = self.data {

            let (start, end) = self.handle_bounds(&range);

            if start > self.len() || end > self.len() {
                return None;
            }

            Some(unsafe { slice::from_raw_parts(data.add(start).as_ptr(), end - start) })
        } else {
            None
        }
    }

    /// Gets range of mutable elements from the vector
    /// - returns `None` if vector does not contain any data ot range is out of bounds
    pub fn get_mut<R>(&mut self, range: R) -> Option<&mut [T]>
    where R: RangeBounds<usize> {

        if let Some(data) = self.data {

            let (start, end) = self.handle_bounds(&range);

            if start > self.len() || end > self.len() {
                return None;
            }

            Some(unsafe { slice::from_raw_parts_mut(data.add(start).as_ptr(), end - start) })

        } else {
            None
        }

    }


    /// Indicates if vector size is 0
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Indicates if vector has any data allocated
    #[inline(always)]
    pub fn has_data(&self) -> bool {
        self.data.is_some()
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



}

impl<T: Sized> Drop for Vec<T> {
    fn drop(&mut self) {
        if let Some(data) = self.data {
            unsafe {
                drop_in_place(slice::from_raw_parts_mut(data.as_ptr(), self.len()));
                ALLOCATOR.dealloc(data.as_ptr() as *mut u8, self.layout);
            }
        }
    }
}

impl<T: Sized> Default for Vec<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}*/


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