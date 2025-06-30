//  mem/string.rs (ministd crate)
//  this file originally belonged to baseOS project
//      on OS template on which to build

use crate::mem::DynamicBuffer;
use core::{array::IntoIter, fmt::{Debug, Display, Write}, mem::ManuallyDrop, ops::{Deref, DerefMut, Index, IndexMut, RangeBounds}, ptr::{self, copy_nonoverlapping}, slice};
use crate::convert::{strify, strify_mut};

use core::ops::Bound::*;
/// A ASCII–encoded, growable string.
/// - this implementation will also allow you to tweak memory management using generic parameter
/// 
/// **note**: implementation of the `Drop` trait is not needed for the memory is deallocated by `DynamicBuffer::drop()` automatically
pub struct String<const STEP: usize = 0> {
    data: DynamicBuffer<u8, 0>,
}

impl<const STEP: usize> String<STEP> {

    /// Expands the `capacity` of the vector by `STEP`
    /// - this function always reallocates memory
    /// - **panics** if allocation fails
    #[inline(always)]
    pub fn expand(&mut self) {
        self.data.expand();
    }

    /// Tries to expand the `capacity` of the vector by `STEP`
    /// - this function always reallocates memory
    /// - returns `Err` if allocation fails
    #[inline(always)]
    pub fn try_expand(&mut self) -> Result<(), ()> {
        self.data.try_expand()
    }


    /// Creates a new empty `String`
    /// - no data is allocated
    pub const fn new() -> Self {
        Self {
            data: DynamicBuffer::empty(),
        }
    }


    /// Creates new `String` with at least the specified capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: DynamicBuffer::with_capacity(capacity),
        }
    }

    /// Tries to create new `String` with at least the specified capacity
    /// - returns `Err` if allocation fails
    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, ()> {
        Ok(Self {
            data: DynamicBuffer::try_with_capacity(capacity)?
        })
    }




    /// Appends a given string slice onto the end of this `String`
    /// - **panics** if allocation fails
    pub fn push_str(&mut self, string: &str) {
        
        self.reserve(string.len());

        unsafe {
            let ptr = self.data.as_ptr().add(self.len());

            ptr::copy_nonoverlapping(string.as_ptr(), ptr, string.len());
        }
        self.data.size += string.len() as u32;
    }

    /// Appends a given string slice onto the end of this `String` without checking bounds
    /// - **safety** - misuse may cause buffer overflow and/or undefined behaviour
    ///   - use only if you are 100% sure overflow will not happen
    pub unsafe fn push_str_unchecked(&mut self, string: &str) {
        unsafe {
            let ptr = self.data.as_ptr().add(self.len());

            ptr::copy_nonoverlapping(string.as_ptr(), ptr, string.len());
        }
        self.data.size += string.len() as u32;
    }

    /// Tries to append a given string slice onto the end of this `String`
    /// - returns `Err` if allocation fails
    pub fn try_push_str(&mut self, string: &str) -> Result<(), ()> {

        self.try_reserve(string.len())?;

        unsafe {
            let ptr = self.data.as_ptr().add(self.len());

            ptr::copy_nonoverlapping(string.as_ptr(), ptr, string.len());
        }
        self.data.size += string.len() as u32;

        Ok(())
    }

    /// Appends the given character to the end of the `String`
    /// - **panics** if allocation fails
    pub fn push(&mut self, c: u8) {
        if self.len() == self.capacity() {
            self.expand();
        }

        unsafe {

            self.data.as_ptr().add(self.len()).write(c);
        }
        self.data.size += 1;
    }

    /// Pushes character withouch checking bounds
    /// - **safety** - misuse may cause buffer overflow and/or undefined behavoiur
    ///   - use only if you are 100% sure overflow will not happen
    #[inline]
    pub unsafe fn push_unchecked(&mut self, c: u8) {
        unsafe {
            self.data.as_ptr().add(self.len()).write(c);
        }
        self.data.size += 1;
    }

    /// Tries to push the given character to the end of the `String`
    /// - returns `Err` if allocation fails
    pub fn try_push(&mut self, c: u8) -> Result<(), ()> {
        if self.len() == self.capacity() {
            self.try_expand()?;
        }

        unsafe {
            self.data.as_ptr().add(self.len()).write(c);
        }

        self.data.size += 1;

        Ok(())

    }

    /// Removes the last character from the `String`
    /// - does not affect `capacity`
    #[inline]
    pub fn pop(&mut self) {
        self.data.size = self.data.size.saturating_sub(1);
    }

    /// Removes the last character from the `String` and returns it
    /// - does not affect `capacity`
    #[inline]
    pub fn pop_ret(&mut self) -> Option<u8> {
        if self.len() > 0 {
            self.data.size -= 1;
            Some(unsafe { self.data.as_ptr().add(self.len()).read() })
        } else {
            None
        }
    }

    /// Removes last `n` characters from the string
    /// - does not affect `capacity`
    #[inline(always)]
    pub fn pop_n(&mut self, n: usize) {
        self.data.size = self.data.size.saturating_sub(n as u32);
    }


    /// Reserves capacity for at least `add` more characters
    /// - **panics** if allocation fails
    /// - capacity will be greater than or equal to `self.len() + add.len()`
    #[inline]
    pub fn reserve(&mut self, add: usize) {
        if self.len() + add > self.capacity() {
            self.data.resize(self.len() + add);
        }
    }


    /// Tries to reserve capacity for at least `add` more characters
    /// - returns `Err` if allocation fails
    /// - capacity will be greater than or equal to `self.len() + add.len()`
    #[inline]
    pub fn try_reserve(&mut self, add: usize) -> Result<(), ()> {
        if self.len() + add > self.capacity() {
            self.data.try_resize(self.len() + add)?;
        }
        Ok(())
    }
    
    /// Removes character at the `index` position
    /// - **panics** if index is out of bounds
    /// - this os `O(n)` operation
    /// - **no-op** if empty
    pub fn remove(&mut self, index: usize) {

        if self.len() > 0 {
            if index > self.len() {
                panic!("index out of bounds");
            }

            self.data.size -= 1;

            unsafe {
                let start = self.data.as_ptr().add(index);
                ptr::copy(start.add(1), start, self.len() - index);
            }

        }
    }


    /// Inserts character at the `index` position
    /// - **panics** if allocation fails
    /// - pushes the character if `index >= self.len()`
    /// - this is `O(n)` operation
    pub fn insert(&mut self, index: usize, c: u8) {

        let len = self.len();

        if index >= len {
            self.push(c);
            return;
        }

        if len == self.capacity() {
            self.expand();
        }

        unsafe {
            let ptr = self.data.as_ptr().add(index);

            ptr::copy(ptr, ptr.add(1), len - index);

            ptr.write(c)
        }

        self.data.size += 1;

    }

    /// Tries to insert character at the `index` position
    /// - returns `Err` if allocation fails
    /// - this is `O(n)` operation
    /// - pushes the character if `index >= self.len()`
    pub fn try_insert(&mut self, index: usize, c: u8) -> Result<(), ()> {

        let len = self.len();

        if index >= len {
            return self.try_push(c);
        }

        if len == self.capacity() {
            self.try_expand()?;
        }

        unsafe {
            let ptr = self.data.as_ptr().add(index);

            ptr::copy(ptr, ptr.add(1), len - index);

            ptr.write(c)
        }

        self.data.size += 1;
        Ok(())
    } 

    /// Inserts string slice at the `index` position
    /// - **panics** if allocation fails
    /// - this is `O(n)` operation
    pub fn insert_str(&mut self, index: usize, string: &str) {

        let len = self.len();

        if index >= len {
            self.push_str(string);
            return;
        }

        self.reserve(string.len());

        unsafe {
            let ptr = self.data.as_ptr().add(index);

            ptr::copy(ptr, ptr.add(string.len()), len - index);

            ptr::copy_nonoverlapping(string.as_ptr(), ptr, string.len());
        }

        self.data.size += string.len() as u32;

    }

    /// Tries to insert string at the `index` position
    /// - returns `Err` if allocation fails
    /// - this is `O(n)` operation
    /// - pushes the character if `index >= self.len()`
    pub fn try_insert_str(&mut self, index: usize, string: &str) -> Result<(), ()> {

        let len = self.len();

        if index >= len {
            return self.try_push_str(string);
        }

        self.try_reserve(string.len())?;

        unsafe {
            let ptr = self.data.as_ptr().add(index);

            ptr::copy(ptr, ptr.add(string.len()), len - index);

            ptr::copy_nonoverlapping(string.as_ptr(), ptr, string.len());
        }

        self.data.size += string.len() as u32;

        Ok(())

    }

    /// Removes all characters from the `String`
    /// - does not affect `capacity`
    pub const fn clear(&mut self) {
        self.data.size = 0;
    }


    /// Removes substring from the `String`
    /// - **panics** if allocation fails or out of bounds or if empty
    /// - this is an `O(n)` operation
    pub fn remove_str<R>(&mut self, range: R)
    where R: RangeBounds<usize> {

        let len = self.len();

        let (start, end) = self.handle_bounds(&range);

        if start > len || end > len {
            if len == 0 {
                panic!("String is empty");
            } else {
                panic!("slice out of bounds");
            }
        }

        let count = end - start;

        unsafe {
            let ptr = self.data.as_ptr().add(start);

            ptr::copy(ptr.add(count), ptr, len - start);
        }

        self.data.size -= count as u32;

    }

    /// Consumes and leaks the String, returning a mutable reference to the contents, &'a mut str
    /// - the caller can freely choose lifetime of the reference
    /// - dropping the reference will cause memory leak
    pub fn leak<'l>(self) -> &'l mut str {
        let m = ManuallyDrop::new(self);
        if m.is_empty() {
            panic!("String has no data");
        }

        unsafe { strify_mut(slice::from_raw_parts_mut(m.data.as_ptr(), m.len())) }
    }

}




impl<const STEP: usize> String<STEP> {

    /// Returns the number of ASCII characters (bytes) of the string
    pub const fn len(&self) -> usize {
        self.data.size as usize
    }

    /// Returns the constant generic `STEP` of this instance
    pub const fn step(&self) -> usize {
        STEP
    }

    /// Returns the `String`s capacity in bytes
    pub const fn capacity(&self) -> usize {
        self.data.capacity()
    }


    /// Returns a byte slice of this `String`’s contents
    /// - **panics** if empty
    pub const fn as_bytes(&self) -> &[u8] {
        if self.data.has_data() {
            unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len()) }
        } else {
            panic!("String has no data");
        }
    }

    /// Returns a byte slice of this `String`’s content without checking for NULL
    /// - **safety** - even if the `String` does not contain any data, the pointer is valid
    ///   - misuse may cause undefined behavoiur
    ///   - use only if you are 100% sure that the string contains value
    pub const unsafe fn as_bytes_unchecked(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len()) }
    }

    /// Returns a byte slice of this `String`’s content
    /// - returns `None` if empty
    pub const fn as_bytes_checked(&self) -> Option<&[u8]> {
        if self.data.has_data() {
            Some(unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len()) })
        } else {
            None
        }
    }

    /// Returns a mutable byte slice of this `String`’s contents
    /// - **panics** if empty
    pub const fn as_bytes_mut(&mut self) -> &mut [u8] {
        if self.data.has_data() {
            unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.len()) }
        } else {
            panic!("String has no data");
        }
    }

    /// Returns a mutable byte slice of this `String`’s content without checking for NULL
    /// - **safety** - even if the `String` does not contain any data, the pointer is valid
    ///   - misuse may cause undefined behavoiur
    ///   - use only if you are 100% sure that the string contains value
    pub const unsafe fn as_bytes_mut_unchecked(&self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.len()) }
    }

    /// Returns a mutable byte slice of this `String`’s content
    /// - returns `None` if empty
    pub const fn as_bytes_mut_checked(&self) -> Option<&mut [u8]> {
        if self.data.has_data() {
            Some(unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.len()) })
        } else {
            None
        }
    }


    /// Extracts a string slice containing the entire `String`
    /// - **panics** if empty
    pub const fn as_str(&self) -> &str {
        if self.data.has_data() {
            strify(unsafe { slice::from_raw_parts(self.data.as_ptr(), self.data.size as usize) })
        } else {
            panic!("String has no data");
        }
    }

    /// Extracts a string slice containing the entire `String` without checking for NULL
    /// - **safety** - even if the `String` does not contain any data, the pointer is valid
    ///   - misuse may cause undefined behavoiur
    ///   - use only if you are 100% sure that the string contains value
    pub const unsafe fn as_str_unchecked(&self) -> &str {
        strify(unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len()) })
    }

    /// Extracts a string slice containing the entire `String`
    /// - returns `None` if empty
    pub const fn as_str_checked(&self) -> Option<&str> {
        if self.data.has_data() {
            Some(strify(unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len()) }))
        } else {
            None
        }
    }

    /// Converts a String into a mutable string slice
    /// - **panics** if empty
    pub const fn as_mut_str(&mut self) -> &mut str {
        if self.data.has_data() {
            strify_mut(unsafe { slice::from_raw_parts_mut(self.data.data().as_ptr(), self.data.size as usize) })
        } else {
            panic!("String has no data");
        }
    }

    /// Converts a String into a mutable string slice without checking for NULL
    /// - **safety** - even if the `String` does not contain any data, the pointer is valid
    ///   - misuse may cause undefined behavoiur
    ///   - use only if you are 100% sure that the string contains value
    pub const unsafe fn as_mut_str_unchecked(&self) -> &mut str {
        strify_mut(unsafe {  slice::from_raw_parts_mut(self.data.as_ptr(), self.len())})
    }

    /// Converts a String into a mutable string
    /// - returns `None` if the `String` has no data
    pub const fn as_mut_str_checked(&self) -> Option<&mut str> {
        if self.data.has_data() {
            Some(strify_mut(unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.len()) }))
        } else {
            None
        }
    }


    /// Checks whether the `String` has length of zero
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }


    /// Checks `RangeBounds` for this vector
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

    #[inline(always)]
    pub fn iter<'l>(&'l self) -> core::slice::Iter<'l, u8> {
        self.as_bytes_checked().expect("String is empty").into_iter()
    }

    pub fn iter_mut<'l>(&'l mut self) -> core::slice::IterMut<'l, u8> {
        self.as_bytes_mut_checked().expect("String is empty").into_iter()
    }


}


impl<const STEP: usize> Display for String<STEP> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const STEP: usize> Debug for String<STEP> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "size: {}, capacity: {}, conatent: {self}", self.len(), self.capacity())
    }
}


impl<const STEP: usize> PartialEq<str> for String<STEP> {

    #[inline]
    fn eq(&self, other: &str) -> bool {
        if let Some(s) = self.as_str_checked() {
            s.eq(other)
        } else {
            false
        }
    }

    #[inline]
    fn ne(&self, other: &str) -> bool {
        if let Some(s) = self.as_str_checked() {
            s.ne(other)
        } else {
            true
        }
    }
}

impl<const STEP: usize> PartialEq<&str> for String<STEP> {

    #[inline]
    fn eq(&self, other: &&str) -> bool {
        if let Some(s) = self.as_str_checked() {
            s.eq(*other)
        } else {
            false
        }
    }

    #[inline]
    fn ne(&self, other: &&str) -> bool {
        if let Some(s) = self.as_str_checked() {
            s.ne(*other)
        } else {
            true
        }
    }
}

impl<const STEP: usize> PartialOrd<&str> for String<STEP> {
    fn ge(&self, other: &&str) -> bool {
        let s = if let Some(s) = self.as_str_checked() {
            s
        } else {
            return true;
        };

        s.ge(other)

    }

    fn le(&self, other: &&str) -> bool {
        let s = if let Some(s) = self.as_str_checked() {
            s
        } else {
            return other.len() == 0;
        };

        s.le(other)

    }

    fn lt(&self, other: &&str) -> bool {
        let s = if let Some(s) = self.as_str_checked() {
            s
        } else {
            return false;
        };

        s.lt(other)

    }

    fn gt(&self, other: &&str) -> bool {
        let s = if let Some(s) = self.as_str_checked() {
            s
        } else {
            return other.len().gt(&0);
        };

        s.gt(other)

    }

    fn partial_cmp(&self, other: &&str) -> Option<core::cmp::Ordering> {
        let s = if let Some(s) = self.as_str_checked() {
            s
        } else {
            return other.len().partial_cmp(&0)
        };

        s.partial_cmp(other)
    }

}


impl<const STEP: usize> PartialEq<String> for String<STEP> {

    fn eq(&self, other: &String) -> bool {
        let s1 = if let Some(s) = self.as_str_checked() {
            s
        } else {
            return if other.len() == 0 {
                true
            } else {
                false
            }
        };

        let s2 = if let Some(s) = other.as_str_checked() {
            s
        } else {
            return false;
        };

        s1.eq(s2)
    }

    fn ne(&self, other: &String) -> bool {
        let s1 = if let Some(s) = self.as_str_checked() {
            s
        } else {
            return if other.len() == 0 {
                false
            } else {
                true
            }
        };

        let s2 = if let Some(s) = other.as_str_checked() {
            s
        } else {
            return true;
        };

        s1.ne(s2)
    }
}

impl<const STEP: usize> Clone for String<STEP> {
    fn clone(&self) -> Self {

        let data = self.data.clone();

        unsafe {
            copy_nonoverlapping(self.data.as_ptr(), data.as_ptr(), self.len());
        }


        Self { data }
    }
}

impl<const STEP: usize> From<&str> for String<STEP> {
    fn from(value: &str) -> Self {
        
        let mut data = DynamicBuffer::with_capacity(value.len());

        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), data.as_ptr(), value.len());
        }

        data.size = value.len() as u32;

        Self { data }

    }
}

impl<const STEP: usize> Deref for String<STEP> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        if let Some(s) = self.as_str_checked() {
            s
        } else {
            panic!("String is empty");
        }
    }
}

impl<const STEP: usize> DerefMut for String<STEP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(s) = self.as_mut_str_checked() {
            s
        } else {
            panic!("String is empty");
        }
    }
}

impl<const STEP: usize> Index<usize> for String<STEP> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        if self.len() > 0 && index < self.len(){
            unsafe {
                self.data.as_ptr().add(index).as_ref().unwrap_unchecked()
            }
        } else {
            if self.is_empty() {
                panic!("String is empty");
            } else {
                panic!("index out of bounds");
            }
        }
    }
}

impl<const STEP: usize> IndexMut<usize> for String<STEP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if self.len() > 0 && index < self.len(){
            unsafe {
                self.data.as_ptr().add(index).as_mut().unwrap_unchecked()
            }
        } else {
            if self.is_empty() {
                panic!("String is empty");
            } else {
                panic!("index out of bounds");
            }
        }
    }
}

impl<const STEP: usize> Default for String<STEP> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}


impl<const STEP: usize> Write for String<STEP> {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.try_push(c as u8).map_err(|_| core::fmt::Error)
    }
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.try_push_str(s).map_err(|_| core::fmt::Error)
    }
}

