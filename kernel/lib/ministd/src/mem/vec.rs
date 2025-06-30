//  mem/vec/mod.rs (ministd crate)
//  this file originally belonged to baseOS project
//      on OS template on which to build


use core::borrow::{Borrow, BorrowMut};
use core::fmt::Debug;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ptr::drop_in_place;
use core::slice::{self, from_raw_parts, from_raw_parts_mut};
use core::ops::{Bound::*, Index, IndexMut, RangeBounds, Deref, DerefMut};
use core::cmp::Ordering::*;

use crate::mem::DynamicBuffer;
use crate::TryClone;


/// A contiguous growable array type, written as Vec<T>, short for ‘vector’
/// - this implementation will also allow you to tweak memory management using generic parameters
/// 
/// ### Generic parameters
/// 1. `T`: datatype of each element
/// 2. `STEP`: indicates how much will vector grow
///     - geometrical growth is used by default
pub struct Vec<T: Sized, const STEP: usize = 0> {
    data: DynamicBuffer<T, STEP>,
}

impl<T: Sized, const STEP: usize> Vec<T, STEP> {


    /// Expands the `capacity` of the vector by `STEP`
    /// - this function always reallocates memory
    /// - **panics** if allocation fails
    #[inline(always)]
    pub fn expand(&mut self) {
        self.data.expand();
    }

    /// Tries to expand the `capacity` of the vector by `STEP`
    /// - this function always reallocated memory
    /// - returns `Err` if allocation fails
    #[inline(always)]
    pub fn try_expand(&mut self) -> Result<(), ()> {
        self.data.try_expand()
    }

    /// Constructs new empty `Vec<T>`
    pub const fn new() -> Self {
        Self {
            data: DynamicBuffer::empty(),
        }
    }


    /// Constructs new empty `Vec` with at least the specified capacity allocated
    /// - **panics** if allocation fails
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: DynamicBuffer::with_capacity(capacity),
        }
    }


    /// Tries to construct new empty `Vec<T>` with at least the specified capacity allocated
    /// - returns `Err` if allocation fails
    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ()> {
        Ok(Self {
            data: DynamicBuffer::try_with_capacity(capacity)?,
        })
    }

    /// Reserves capacity for at least `additional` more elements
    /// - **panics** if allocation fails
    /// - **null checking** is done internally via `DynamicBuffer`
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            self.data.resize(self.len() + additional);
        }

    }

    /// Tries to reserve capacity for at least `additional` more elements
    /// - returns `Err` if allocation fails
    /// - **null checking** is done internally via `DynamicBuffer`
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
    /// - **null checking** is done internally via `DynamicBuffer`
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            self.data.resize_exact(self.len() + additional);
        }
    }

    /// Tries to reserve the minimum capacity for at least `additional` more elements
    /// - returns `Err` if allocation fails
    /// - **null checking** is done internally via `DynamicBuffer`
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
        self.data.size += 1;
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

        self.data.size += 1;

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
            self.data.size += 1;
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
                self.data.size = size as u32;
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
                self.data.size = size as u32;
            }

            self.data.try_resize_exact(size)
        } else {
            Ok(())
        }
    }


    /// Removes and drops the element at `index`
    /// - **panics** if `index > self.len()`
    /// - **no-op** if `self.is_empty()`
    /// - preserves ordering of the vector
    ///   - this is `O(n)` operation
    pub fn remove(&mut self, index: usize) {
        if self.capacity() > 0 {
            if index >= self.len() {
                panic!("index out of bounds");
            }

            unsafe {

                let ptr = self.data.data().add(index).as_ptr();

                drop_in_place(ptr);

                core::ptr::copy(ptr.add(1), ptr, self.len() - index - 1);

            }

            self.data.size -= 1;
        }
    }

    /// Removes and returns the element at `index`
    /// - **panics** if `index > self.len() || self.is_empty()`
    /// - preserves ordering of the vector
    ///   - this is `O(n)` operation
    pub fn remove_ret(&mut self, index: usize) -> T {
        if self.capacity() > 0 {
            if index >= self.len() {
                panic!("index out of bounds");
            }

            let ret;

            self.data.size -= 1;

            unsafe {

                let ptr = self.data.data().add(index).as_ptr();

                ret = ptr.read();

                core::ptr::copy(ptr.add(1), ptr, self.len() - index);

            }

            ret

        } else {
            panic!("vector does not contain any data");
        }
    }


    /// Inserts `val` into the vector at `index` index
    /// - shifts all elements - this is `O(n)` operation
    /// - **panics** if allocation fails or `index > self.len()`
    /// - if `index == self.len()`, pushes instead
    pub fn insert(&mut self, index: usize, val: T) {

        let len = self.len();

        if index > len {
            panic!("index is out of bounds");
        } else if index == len {
            self.push(val);
            return;
        }

        if len == self.capacity() {
            self.expand();
        }

        unsafe {
            let ptr = self.data.as_ptr().add(index);

            core::ptr::copy(ptr, ptr.add(1), len - index);

            ptr.write(val);
        }

        self.data.size += 1;

    }

    /// Tries to insert `val` into the vector at `index` index
    /// - shifts all elements - this is `O(n)` operation
    /// - returns `val` if allocation fails or `index > self.len()`
    /// - if `index == self.len()`, pushes instead
    pub fn try_insert(&mut self, index: usize, val: T) -> Result<(), T> {

        let len = self.len();

        if index > len {
            return Err(val);
        } else if index == len {
            return self.try_push(val);
        }

        if len == self.capacity() {
            if let Err(_) = self.try_expand() {
                return Err(val);
            }
        }

        unsafe {
            let ptr = self.data.data().add(index).as_ptr();

            core::ptr::copy(ptr, ptr.add(1), len - index);

            ptr.write(val);
        }

        self.data.size += 1;

        Ok(())

    }

    /// Drops the last element of the vector
    /// - **no-op** if `self.is_empty()`
    /// - does not affect `capacity`
    pub fn pop(&mut self) {

        if self.len() > 0 {
            unsafe {
                drop_in_place(self.data.data().add(self.len() - 1).as_ptr());
            }
            self.data.size -= 1;
        }
    }

    /// Removes and returns the last element of the vector
    /// - **no-op** if `self.is_empty()`
    /// - does not affect `capacity`
    pub fn pop_ret(&mut self) -> Option<T> {

        if self.len() > 0 {
            self.data.size -= 1;
            Some(unsafe { self.data.data().add(self.len()).read() })
        } else {
            None
        }
    }

    /// Removes last `n` elements of the vector
    /// - does not affect `capacity`
    pub fn pop_n(&mut self, n: usize) {
        if self.len() > 0 {

            if n <= self.len() {
                
                unsafe {
                    drop_in_place(self.as_mut_slice_unchecked());
                }
                
                self.data.size = 0;

            } else {

                self.data.size -= n as u32;

                unsafe {
                    let ptr = self.data.as_ptr().add(self.len());
                    drop_in_place(slice::from_raw_parts_mut(ptr, n));
                }

            }
            
        }
    }

    /// Drops the last element if the vector if `f` returns `true`
    pub fn pop_if<F>(&mut self, f: F)
    where F: Fn(&T) -> bool {
        let last = match self.last() {
            Some(l) => l,
            None => return,
        };



        if f(last) {
            
            unsafe {
                drop_in_place(self.data.data().add(self.len() - 1).as_ptr());
            }

            self.data.size -= 1;
        }
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty
    /// - **panics** if allocation fails
    pub fn append(&mut self, other: &mut Vec<T>) {
        if other.is_empty() {
            return;
        }

        self.reserve(other.len());

        unsafe {
            let mut ptr = other.data.data();
            for _ in 0..other.len() {
                self.push(ptr.read());
                ptr = ptr.add(1);
            }
            other.set_len(0);
        }
    }

    /// Append all elements from `other` to `self`, leaving `other` empty
    /// - returns `Err` if allocation fails
    pub fn try_append(&mut self, other: &mut Vec<T>) -> Result<(), ()> {
        if other.is_empty() {
            return Ok(());
        }

        self.try_reserve(other.len())?;

        unsafe {
            let mut ptr = other.data.data();
            for _ in 0..other.len() {
                self.push(ptr.read());
                ptr = ptr.add(1);
            }
            other.set_len(0);
        }

        Ok(())

    }

    







    /// Forces the length of the vector to new_len.
    /// - this will not construct and/or modify `capacity`
    /// - this function does not check for any boundaries (including `capacity`)
    #[inline(always)]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.data.size = len as u32;
    }

    /// Clears the vector, removing all values.
    /// - note that this method has no effect on the allocated `capacity`
    pub fn clear(&mut self) {
        if core::mem::needs_drop::<T>() && self.len() > 0 {
            unsafe {
                drop_in_place(self.as_mut_slice_unchecked().as_mut_ptr())
            }
        }
        self.data.size = 0;
    }

    /// Consumes and leaks the `Vec`, returning mutable reference to its data
    /// - **panics** if has no data
    /// - does not shrink the `capacity`
    /// - dropping the returned reference may result in memory leak
    pub fn leak<'l>(self) -> &'l mut [T] {
        if self.capacity() > 0 {
            panic!("Vec does not contain any data");
        }

        let m = ManuallyDrop::new(self);

        unsafe { from_raw_parts_mut(m.data.data().as_ptr(), m.len()) }

    }

    /// Returns the remaining spare capacity of the vector as a slice of MaybeUninit<T>
    pub fn spare_capacity_mut(&mut self) -> Option<&mut [MaybeUninit<T>]> {
        if self.capacity() - self.len() > 0 {
            Some(unsafe { from_raw_parts_mut(self.data.data().add(self.len()).as_ptr() as *mut MaybeUninit<T>, self.capacity() - self.len()) })
        } else {
            None
        }
    }












    /// Returns number of elements in the vector
    pub const fn len(&self) -> usize {
        self.data.size as usize
    }

    /// Returns number of elements allocated by the vector
    pub const fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Checks whether the vector is empty (`size == 0`)
    pub const fn is_empty(&self) -> bool {
        self.data.size == 0
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
        if self.capacity() > 0 {
            Some(unsafe { from_raw_parts(self.data.data().as_ptr(), self.len()) })
        } else {
            None
        }
    }

    /// returns contents of the vector as mutable slice
    /// - or `None` if vector does not have any contents
    pub const fn as_mut_slice(&self) -> Option<&mut [T]> {
        if self.capacity() > 0 {
            Some(unsafe { from_raw_parts_mut(self.data.data().as_ptr(), self.len()) })
        } else {
            None
        }
    }


    /// returns contents of the vector as slice without checking for NULL
    /// - safety: use only if you are 1000% sure it will not be a disaster ._.
    pub const unsafe fn as_slice_unchecked(&self) -> &[T] {
        unsafe { from_raw_parts(self.data.data().as_ptr(), self.len())}
    }

    /// returns contents of the vector as mutable slice without checking for NULL
    /// - safety: use only if you are 1000% sure it will not be a disaster ._.
    pub const unsafe fn as_mut_slice_unchecked(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.data.data().as_ptr(), self.len())}
    }

    /// Returns the last element of the vector or `None`
    pub const fn last(&self) -> Option<&T> {
        if self.len() > 0 {
            return Some(unsafe { self.data.data().add(self.len() - 1).as_ref() })
        } else {
            None
        }
    }

    /// Returns the last element of the vector or `None`
    pub const fn last_mut(&mut self) -> Option<&mut T> {
        if self.len() > 0 {
            return Some(unsafe { self.data.data().add(self.len() - 1).as_mut() })
        } else {
            None
        }
    }


    /// Checks RangeBound for this vector
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
        let (start, end) = self.handle_bounds(&range);

        if start > self.len() || end > self.len() {
            return None;
        }

        Some(unsafe { from_raw_parts(self.data.data().add(start).as_ptr(), end - start) })
        
    }

    /// Returns an mutable subslice from the vector
    /// - or `None` if vector has no data or out of bounds
    pub fn get_mut<R>(&self, range: R) -> Option<&mut [T]>
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);

        if start > self.len() || end > self.len() {
            return None;
        }

        Some(unsafe { from_raw_parts_mut(self.data.data().add(start).as_ptr(), end - start) })
    }

    /// Returns an sublice from the vector withou doing bounds check
    /// - does not check if vector has any data
    pub unsafe fn get_unchecked<R>(&self, range: R) -> &[T]
    where R: RangeBounds<usize> {

        let (start, end) = self.handle_bounds(&range);
        unsafe {
            from_raw_parts(self.data.data().add(start).as_ptr(), end - start)
        }
    }

    /// Returns an mutable sublice from the vector withou doing bounds check
    /// - **panics** if has no data
    pub unsafe fn get_unchecked_mut<R>(&mut self, range: R) -> &[T]
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);
        unsafe {
            from_raw_parts_mut(self.data.data().add(start).as_ptr(), end - start)
        }
    }

    /// Returns iterator for this vector
    #[inline(always)]
    pub fn iter<'l>(&'l self) -> core::slice::Iter<'l, T> {
        self.as_slice().expect("Vec has no data").iter()
    }

    pub fn iter_mut<'l>(&'l mut self) -> core::slice::IterMut<'l, T> {
        self.as_mut_slice().expect("Vec has no data").iter_mut()
    }


}

impl<T: Sized + Default, const STEP: usize> Vec<T, STEP> {

    /// Removes and drops element at `index`
    /// - **panics** if `index > self.len()`
    /// - does not preserve ordering of the vector
    ///   - assigns `T::default()` to the element
    /// - this is `O(1)` operation
    pub fn swap_remove(&mut self, index: usize) {
        if self.capacity() == 0 || index >= self.len() {
            if self.capacity() == 0 {
                panic!("vector has no data");
            } else {
                panic!("index out of bounds");
            }
        }

        unsafe {
            let ptr = self.data.data().add(index).as_ptr();

            drop_in_place(ptr);

            ptr.write(T::default())
        }
    }

    /// Removes and returns element at `index`
    /// - **panics** if `index > self.len() || self.is_empty()`
    /// - does not preserve ordering of the vector
    ///   - assigns `T::default()` to the element
    /// - this is `O(1)` operation
    pub fn swap_remove_ret(&mut self, index: usize) -> T {
        if self.capacity() == 0 || index >= self.len() {
            if self.capacity() == 0 {
                panic!("vector has no data");
            } else {
                panic!("index out of bounds");
            }
        }

        let ret;

        unsafe {
            let ptr = self.data.data().add(index).as_ptr();

            ret = ptr.read();

            ptr.write(T::default())
        }

        ret
    }

}


impl<T: Sized, const STEP: usize> AsRef<[T]> for Vec<T, STEP> {
    /// **panics** if has no data
    fn as_ref(&self) -> &[T] {
        self.as_slice().expect("Vec has no data")
    }
}

impl<T: Sized, const STEP: usize> AsMut<[T]> for Vec<T, STEP> {
    /// **panics** if has no data
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice().expect("Vec has no data")
    }
}

impl<T: Sized, const STEP: usize> Borrow<[T]> for Vec<T, STEP> {
    /// **panics** if has no data
    fn borrow(&self) -> &[T] {
        self.as_slice().expect("Vec has no data")
    }
}

impl<T: Sized, const STEP: usize> BorrowMut<[T]> for Vec<T, STEP> {
    /// **panics** if has no data
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice().expect("Vec has no data")
    }
}

impl<T: Sized, const STEP: usize> Drop for Vec<T, STEP> {
    fn drop(&mut self) {
        if self.capacity() > 0 {
            unsafe {
                drop_in_place(from_raw_parts_mut(self.data.data().as_ptr(), self.len()));
            }
        }
    }
}

impl<T: Sized, const STEP: usize> Index<usize> for Vec<T, STEP> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if index < self.len() {
            unsafe {
                return self.data.data().add(index).as_ref();
            }
        }

        if self.len() == 0 {
            panic!("vector has no data");
        } else {
            panic!("Vec[]: out of bounds");
        }
    }
}

impl<T: Sized, const STEP: usize> IndexMut<usize> for Vec<T, STEP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.len() {
            unsafe {
                return self.data.data().add(index).as_mut();
            }
        }

        if self.len() == 0 {
            panic!("vector has no data");
        } else {
            panic!("Vec[]: out of bounds");
        }
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
        })

    }
}

impl<T: Sized, const STEP: usize> Deref for Vec<T, STEP> {
    type Target = [T];
    #[inline]
    /// Does not check for null at all
    fn deref(&self) -> &Self::Target {
        unsafe { self.as_slice_unchecked() }
    }
}

impl<T: Sized, const STEP: usize> DerefMut for Vec<T, STEP> {
    #[inline]
    /// Does not check for null at all
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_mut_slice_unchecked() }
    }
}

impl<T: Sized, const STEP: usize> Default for Vec<T, STEP> {
    #[inline(always)]
    /// Equivalent of `Vec::new()`
    fn default() -> Self {
        Self::new()
    }
}


impl<T: Sized, const STEP: usize> Debug for Vec<T, STEP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        _ = writeln!(f, "Vec: ptr: {:p}, size: {}, capacity: {}", self.data.data().as_ptr(), self.data.size, self.capacity());
        Ok(())
    }
}

