//  mem/string.rs (ministd crate)
//  this file originally belonged to baseOS project
//      on OS template on which to build

use core::mem::ManuallyDrop;

use core::{alloc::{GlobalAlloc, Layout}, ops::RangeBounds, ptr::{self, null_mut, NonNull}, slice};

use crate::{convert::{strify, strify_mut, Align, IsAligned}, Array, ALLOCATOR};

use core::convert::{TryFrom, TryInto};

use core::ops::Bound::{Included, Excluded, Unbounded};


/// `std::String`-like string implementation supporting ASCII (or possibly extended ASCII)
/// 
/// implementation details:
/// - `capacity` and `size` are of type `u32`
/// - `pointer` and `size` must be aligned to `String::ALIGN`
///   - feel free to change the constant if you want **(must be power of 2)**
/// - `capacity` growth is geometrical (exponencial = 8 -> 16 -> 32 -> 64 ...)
///   - `capacity` is always greater or equal to `Self::ALIGN` for faster copying
/// 
/// recomendation:
/// - since most methods can cause panic please use its `try` alternatives
///   - on the other hand using it panicking functions can be a good idea in some cases
pub struct String {
    data: Option<NonNull<u8>>,
    size: u32,
    cap: u32,
}

impl String {

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

    /// sets the alignement of `String` data and its `capacity`
    pub const ALIGN: usize = size_of::<usize>();

    /// function used to create `Layout` for `String`
    /// 
    /// mind that it does not check if capacity is aligned
    #[inline(always)]
    pub const fn mk_layout(capacity: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(capacity, Self::ALIGN) }
    }

    #[inline(always)]
    const fn capacity_next(cap: u32) -> u32 {
        cap.next_power_of_two()
    }

    /// function used to initialize capacity for data allocation
    const fn init_capacity(capacity: u32) -> u32 {
        if capacity < Self::ALIGN as u32 {
            Self::ALIGN as u32
        } else {
            capacity.next_power_of_two()
        }
    }

    /// creates string instance with no data
    pub const fn new() -> Self {
        Self {
            data: None,
            size: 0,
            cap: 0
        }
    }

    /// creates new string instance with allocated data
    /// - panics if allocation fails
    /// 
    /// use only if allocation failure would cause panic anyway
    /// 
    /// **NOTE**: allocated data is uninitialized
    pub fn with_capacity(mut capacity: usize) -> Self {

        capacity = String::init_capacity(capacity as u32) as usize;
        let data = unsafe { ALLOCATOR.alloc(String::mk_layout(capacity)) };

        assert!(!data.is_null(), "failed to allocate memory for String");

        Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            size: 0,
            cap: capacity as u32,
        }
    }


    /// tries to allocate data for string
    /// - returns `Err` if allocation fails
    /// 
    /// **NOTE**: allocated data is uninitialized
    pub fn try_with_capacity(mut capacity: usize) -> Result<Self, ()> {
        capacity = String::init_capacity(capacity as u32) as usize;
        let data = unsafe {ALLOCATOR.alloc(String::mk_layout(capacity))};
        
        if data.is_null() {
            return Err(());
        }

        Ok(Self {
            data: Some(unsafe { NonNull::new_unchecked(data) }),
            size: 0,
            cap: capacity as u32,
        })
    }


    /// returns raw parts (`(pointer, length, capacity)`) by consuming string
    /// - use of this function is discouraged
    /// 
    /// panics if string is empty
    pub fn into_raw_parts(self) -> (*mut u8, u32, usize) {
        let me = ManuallyDrop::new(self);
        if let Some(data) = me.data {
            (data.as_ptr(), me.size, me.cap as usize)
        } else {
            panic!("trying to consume empty String");
        }
    }

    /// creates `String` from raw parts
    /// 
    /// panics if:
    /// - pointer is null
    /// - pointer or capacity is not aligned to `String::ALIGN`
    /// - `length` > `capacity`
    pub unsafe fn from_raw_parts(ptr: *mut u8, length: usize, capacity: usize) -> Self {
        if (ptr as usize).is_not_aligned(Self::ALIGN) || capacity.is_not_aligned(Self::ALIGN) {
            panic!("pointer or capacity is misaligned");
        }
        assert!(length <= capacity);
        Self {
            data: NonNull::new(ptr),    //  checks for NULL
            size: length as u32,
            cap: capacity as u32
        }
    }

    /// creates `String` from raw parts
    /// 
    /// returns error if:
    /// - pointer is null
    /// - pointer or capacity is not aligned to `String::ALIGN`
    /// - `length` > `capacity`
    pub unsafe fn try_from_raw_parts(ptr: *mut u8, length: usize, capacity: usize) -> Result<Self, ()> {
        if (ptr as usize).is_not_aligned(Self::ALIGN) || capacity.is_not_aligned(Self::ALIGN) {
            return Err(());
        }
        if length <= capacity {
            return Err(());
        }
        Ok(Self {
            data: NonNull::new(ptr),    //  checks for NULL
            size: length as u32,
            cap: capacity as u32
        })
    }


    /// creates `String` from raw parts
    /// 
    /// ## criteria:
    /// - `ptr` != null
    /// - `length` < `capacity`
    /// - `capacity` is aligned to `String::ALIGN`
    /// 
    /// **FAILURE TO COMPLY WITH THESE CRITERIA MAY RESULT IN UNDEFINED BEHAVIOUR**
    #[inline]
    pub const unsafe fn from_raw_parts_unchecked(ptr: *mut u8, length: usize, capacity: usize) -> Self {
        Self {
            data: Some(unsafe { NonNull::new_unchecked(ptr) }),
            size: length as u32,
            cap: capacity as u32
        }
    }


    /// appends string
    /// - panics if allocation fails
    /// 
    pub fn push_str(&mut self, string: &str) {
        self.reserve(string.len());
        let data = self.data.unwrap().as_ptr();

        unsafe { core::ptr::copy_nonoverlapping(string.as_ptr(), data.add(self.len()), string.len()); }

        self.size += string.len() as u32;
    }

    /// tries to append string
    /// - returns `Err` if allocation fails
    pub fn try_push_str(&mut self, string: &str) -> Result<(), ()> {
        self.try_reserve(string.len())?;
        let data = self.data.unwrap().as_ptr();

        unsafe { core::ptr::copy_nonoverlapping(string.as_ptr(), data.add(self.len()), string.len()); }

        self.size += string.len() as u32;

        Ok(())
    }

    /// tries to append character to the string
    /// - panics if allocation fails
    pub fn push(&mut self, c: u8) {
        if let Some(data) = self.data {
            if self.len() + 1 < self.capacity() {

                unsafe { data.as_ptr().add(1).write(c); }
                self.size += 1;

            } else {

                let cap = Self::capacity_next(self.cap);
                let d = unsafe { ALLOCATOR.realloc(data.as_ptr(), Self::mk_layout(self.cap as usize), cap as usize)};

                if d.is_null() {
                    panic!("failed to allocate memory for String");
                }

                unsafe { d.add(self.len()).write(c); }

                self.size += 1;
                self.cap = cap;
                self.data = Some(unsafe { NonNull::new_unchecked(d) });

            }
        } else {
            // allocate data
            let d = unsafe { ALLOCATOR.alloc(Self::mk_layout(Self::ALIGN)) };

            if d.is_null() {
                panic!("failed to allocate memory for String");
            }

            unsafe { d.write(c); }
            
            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.size = 1;
            self.cap = Self::ALIGN as u32;
        }
    }

    /// tries to append character to the string
    /// - returns `Err` if allocation fails
    pub fn try_push(&mut self, c: u8) -> Result<(), ()> {
        if let Some(data) = self.data {
            if self.len() + 1 < self.capacity() {

                unsafe { data.as_ptr().add(1).write(c); }
                self.size += 1;
                Ok(())

            } else {

                let cap = Self::capacity_next(self.cap);
                let d = unsafe { ALLOCATOR.realloc(data.as_ptr(), Self::mk_layout(self.cap as usize), cap as usize)};

                if d.is_null() {
                    return Err(());
                }

                unsafe { d.add(self.len()).write(c); }

                self.size += 1;
                self.cap = cap;
                self.data = Some(unsafe { NonNull::new_unchecked(d) });

                Ok(())
            }
        } else {
            // allocate data
            let d = unsafe { ALLOCATOR.alloc(Self::mk_layout(Self::ALIGN)) };

            if d.is_null() {
                return Err(());
            }

            unsafe { d.write(c); }
            
            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.size = 1;
            self.cap = Self::ALIGN as u32;

            Ok(())
        }
    }


    /// reserves at least `add` more bytes on the heap
    /// 
    /// does not affect data or length
    pub fn reserve(&mut self, add: usize) {
        let mut required = self.size + add as u32;

        if required > self.cap {

            required = Self::init_capacity(required);

            let d: *mut u8;

            if let Some(data) = self.data {
                d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr(), String::mk_layout(self.capacity()), required as usize)
                };
            } else {
                d = unsafe {
                    ALLOCATOR.alloc(String::mk_layout(required as usize))
                };
            }

            if d.is_null() {
                panic!("failed to allocate memory for String");
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });

            self.cap = required;

        }
    }

    /// tries to reserve at least `add` more bytes on the heap
    /// - returns `Err` if allocation fails
    /// 
    /// does not affect String data or length
    pub fn try_reserve(&mut self, add: usize) -> Result<(), ()> {
        let mut required = self.size + add as u32;

        if required > self.cap {

            required = String::init_capacity(required);

            let d: *mut u8;

            if let Some(data) = self.data {
                d = unsafe { ALLOCATOR.realloc(data.as_ptr(), String::mk_layout(self.capacity()), required as usize) };
            } else {
                d = unsafe { ALLOCATOR.alloc(String::mk_layout(required as usize)) };
            }

            if d.is_null() {
                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });

            self.cap = required;
        }
        Ok(())
    }

    /// reserves at least `add` bytes on the heap
    /// - panics if allocation fails
    /// 
    /// unlike `reserve` does not overallocate memory
    pub fn reserve_exact(&mut self, add: usize) {
        let required = (self.size + add as u32).align(Self::ALIGN as u32);

        if required > self.cap {
            let d: *mut u8;

            if let Some(data) = self.data {
                d = unsafe { ALLOCATOR.realloc_layout(data.as_ptr(), String::mk_layout(self.capacity()), String::mk_layout(required as usize)) };
            } else {
                d = unsafe { ALLOCATOR.alloc(String::mk_layout(required as usize)) };
            }

            if d.is_null() {
                panic!("failed to allocate memory for String");
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.cap = required;
    
        }
    }

    /// tries to reserve at least `add` bytes in the heap
    /// - returns `Err` if allocation fails
    /// 
    /// unlike `reserve` does not overallocate memory
    pub fn try_reserve_exact(&mut self, add: usize) -> Result<(), ()> {
        let required = (self.size + add as u32).align(Self::ALIGN as u32);

        if required > self.cap {
            let d: *mut u8;

            if let Some(data) = self.data {
                d = unsafe { ALLOCATOR.realloc_layout(data.as_ptr(), String::mk_layout(self.capacity()), String::mk_layout(required as usize)) };
            } else {
                d = unsafe { ALLOCATOR.alloc(String::mk_layout(required as usize)) };
            }

            if d.is_null() {

                return Err(());
            }

            self.data = Some(unsafe { NonNull::new_unchecked(d) });
            self.cap = required;
        }

        Ok(())
    }

    /// shrinks the `capacity` of the string to fit `len`
    /// - panics if allocation fails
    /// 
    /// the calculated capacity is aligned to `size_of::<usize>()`
    /// - this function is optimized to reallocate data only if its worth it
    pub fn shrink_to_fit(&mut self) {

        if let Some(data) = self.data {
            let wanted = self.size.align(Self::ALIGN as u32);

            if self.cap - wanted > Self::ALIGN as u32 * 2 {
                let d = unsafe {
                    ALLOCATOR.realloc(data.as_ptr(), String::mk_layout(self.capacity()), wanted as usize)
                };
                if d.is_null() {
                    panic!("failed to allocate memory for String");
                }
                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.cap = wanted;
            }
        }
    }

    /// tries to shrink the data to `len`
    /// - returns `Err` if allocation fails
    /// 
    /// if old capacity < new capacity this is no-op
    /// 
    /// the calculated capacity is aligned to `size_of::<usize>()`
    /// - this function is optimized and may not reallocate the data if the difference between old and new capacity is small
    pub fn try_shrink_to_fit(&mut self) -> Result<(), ()> {

        if let Some(data) = self.data {
            let wanted = self.size.align(Self::ALIGN as u32);

            if self.cap - wanted > Self::ALIGN as u32 * 2 {
                let d = unsafe { ALLOCATOR.realloc(data.as_ptr(), Self::mk_layout(self.capacity()), wanted as usize) };
                if d.is_null() {
                    return Err(());
                }
                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.cap = wanted;
            }
        }
        Ok(())
    }

    /// shrinks the capacity to specified size
    /// 
    /// if old capacity < new capacity this is no-op
    /// 
    /// the calculated capacity is aligned to `size_of::<usize>()`
    /// - this function is optimized and may not reallocate the data if the difference between old and new capacity is small
    pub fn shrink_to(&mut self, capacity: usize) {
        if let Some(data) = self.data {
            let wanted = capacity.align(Self::ALIGN);

            let diff = match self.cap.checked_sub(wanted as u32) {
                Some(val) => val,
                None => return,
            };

            if diff > Self::ALIGN as u32 * 2 {
                let d = unsafe { ALLOCATOR.realloc(data.as_ptr(), Self::mk_layout(self.capacity()), wanted) };

                if d.is_null() {
                    panic!("failed to allocate memory for String");
                }

                self.data = Some(unsafe { NonNull::new_unchecked(d) });
                self.cap = wanted as u32;
            }
        }
    }

    /// shortens string to specific size
    /// - if `has no data` || `len > new_size` => **no-op**
    /// - this has no effect on allocated `capacity`
    #[inline]
    pub fn truncate(&mut self, size: usize) {
        if self.data.is_some() && self.len() > size {
            self.size = size as u32;
        }
    }

    /// removes the last character and returns it
    /// - if no other characters are present returns `None`
    ///   - does not dealocate data
    /// - `capacity` is left unchanged
    pub fn pop(&mut self) -> Option<u8> {
        if let Some(data) = self.data {
            if self.len() > 1 {
                self.size -= 1;
                Some( unsafe { data.as_ptr().add(self.len()).read() })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// removes character at position and returns it
    /// - returns `b'\0'` if has no data
    /// - this is an `O(n)` operation
    /// - if `index >= self.len() || self.data.is_none()` => **no-op**
    pub fn remove(&mut self, index: usize) -> u8 {
        if let Some(data) = self.data {
            if self.len() > index {
                let ret = unsafe { data.as_ptr().add(index).read() };
                for i in index..self.len() {
                    unsafe {
                        data.as_ptr().add(i).write(data.as_ptr().add(i+1).read())
                    }
                }
                self.size -= 1;

                ret
            } else {
                b'\0'
            }
        } else {
            b'\0'
        }
    }

    /// inserts character at certain position
    /// - this is an `O(n)` operation
    /// - returns `Err` if allocation failed or if `index` is out of range
    /// - if has no data => **no-p**
    pub fn insert(&mut self, index: usize, c: u8) -> Result<(), ()> {
        if self.data.is_some() {
            self.try_reserve(1)?;
            self.size += 1;
            let data = self.data.unwrap().as_ptr();
            for i in (index+1 .. self.len()).rev() {
                unsafe {
                    data.add(i).write(data.add(i-1).read());
                }
            }
            unsafe { data.add(index).write(c) }
        }
        Ok(())
    }

    /// inserts string at certain position
    /// - this is an `O(n)` operation
    /// - returns `Err` if allocation fails
    /// - if has no data => **no-op**
    pub fn insert_str(&mut self, index: usize, string: &str) -> Result<(), ()> {
        if self.data.is_some() {

            self.try_reserve(string.len())?;

            let data = self.data.unwrap();

            unsafe {
                let src = data.add(index).as_ptr();
                let dest = data.add(index + string.len()).as_ptr();
                core::ptr::copy(src, dest, self.len() - index);

                core::ptr::copy_nonoverlapping(string.as_ptr(), src, string.len());
            }

            self.size += string.len() as u32;

        }
        Ok(())
    }

    /// replaces the specified range with given string
    /// - returns `Err` if allocation fails
    ///   - and if range is out of range
    /// - if string has no data => **no-op**
    pub fn replace_range<R>(&mut self, range: R, replacement: &str) -> Result<(), ()> 
    where R: RangeBounds<usize> {
        if let Some(data) = self.data {
            let start = match range.start_bound() {
                Included(&n) => n,
                Excluded(&n) => n.saturating_sub(1),
                Unbounded => 0,
            };
            let end = match range.end_bound() {
                Included(&n) => n,
                Excluded(&n) => n + 1,
                Unbounded => self.len(),
            };

            if start >= self.len() || end >= self.len() {
                return Err(());
            }

            let repl = replacement.as_bytes();
            for (i, item) in unsafe { core::slice::from_raw_parts_mut(data.as_ptr().add(start), end - start) }.iter_mut().enumerate() {
                *item = repl[i % repl.len()];
            }
        }

        Ok(())
    }


    /// consumes the `String` and leaks the memory
    /// - user can choose its lifetime
    /// - dropping the reference will cause memory leak
    pub unsafe fn leak<'l>(self) -> Option<&'l mut [u8]> {
        if let Some(data) = self.data {
            Some(unsafe { core::slice::from_raw_parts_mut(data.as_ptr(), self.size as usize) })
        } else {
            None
        }
    }

    /// returns subslice of the content
    /// - returns `None` if range is out of range or string does not contain any data
    pub fn get<R>(&self, range: R) -> Option<&[u8]>
    where R: RangeBounds<usize> {

        if let Some(data) = self.data {

            let (start, end) = self.handle_bounds(&range);

            if start >= self.len() || end >= self.len() {
                return None;
            }

            Some(unsafe {
                slice::from_raw_parts(data.as_ptr().add(start), end - start)
            })

        } else {
            None
        }
    }

    /// returns slice of bytes without checking whether string is empty or long enough
    pub unsafe fn get_unchecked<R>(&self, range: R) -> &[u8]
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);
        unsafe { slice::from_raw_parts(self.data.expect("String is empty").add(start).as_ptr(), end - start) }

    }

    /// returns mutable subslice of the content
    /// - returns `None` if range is out of range or string is empty
    pub fn get_mut<R>(&self, range: R) -> Option<&mut [u8]>
    where R: RangeBounds<usize> {

        if let Some(data) = self.data {

            let (start, end) = self.handle_bounds(&range);

            if start >= self.len() || end >= self.len() {
                return None;
            }

            Some(unsafe {
                slice::from_raw_parts_mut(data.as_ptr().add(start), end - start)
            })

        } else {
            return None;
        }
    }

    /// returns mutable subslice of bytes without checking whether string is empty or long enough
    pub unsafe fn get_mut_unchecked<R>(&self, range: R) -> &mut [u8]
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);

        unsafe { slice::from_raw_parts_mut(self.data.expect("String is empty").add(start).as_ptr(), end - start) }
    }

    /// returns `str` subslice from the string
    pub fn get_str<R>(&self, range: R) -> Option<&str>
    where R: RangeBounds<usize> {
        
        if let Some(data) = self.data {
            let (start, end) = self.handle_bounds(&range);

            if start >= self.len() || end >= self.len() {
                return None;
            }

            Some(unsafe {
                strify(slice::from_raw_parts(data.as_ptr().add(start), end - start))
            })
        } else {
            None
        }
    }

    /// returns subslice `str` without checking whether string is empty or long enough
    pub unsafe fn get_str_unchecked<R>(&self, range: R) -> &str
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);

        strify(unsafe { slice::from_raw_parts(self.data.expect("String is empty").add(start).as_ptr(), end - start) })
    }

    /// returns subslice from the String as `&mut str`
    pub fn get_mut_str<R>(&self, range: R) -> Option<&mut str>
    where R: RangeBounds<usize> {
        
        if let Some(data) = self.data {
            let (start, end) = self.handle_bounds(&range);

            if start >= self.len() || end >= self.len() {
                return None;
            }

            Some(unsafe {
                strify_mut(slice::from_raw_parts_mut(data.as_ptr().add(start), end - start))
            })
        } else {
            None
        }
    }

    pub unsafe fn get_str_mut_unchecked<R>(&self, range: R) -> &mut str
    where R: RangeBounds<usize> {
        let (start, end) = self.handle_bounds(&range);

        strify_mut(unsafe { slice::from_raw_parts_mut(self.data.expect("String is empty").add(start).as_ptr(), end - start) })
    }

}

impl String {
    /// returns length of the string in bytes
    /// - which is also character count for ASCII
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.size as usize
    }

    /// returns number of allocated bytes in the string
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.cap as usize
    }

    /// returns whether string is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// removes all characters from the `String`
    /// - does not affect `capacity`
    /// - if has no data => **no-op**
    #[inline(always)]
    pub fn clear(&mut self) {
        self.size = 0;
    }

    /// returns string's data as `&str`
    /// - panics if string is empty
    pub const fn as_str(&self) -> &str {
        strify(unsafe { slice::from_raw_parts(self.data.expect("String is empty").as_ptr(), self.size as usize) })
    }

    /// returns string data as `&mut str`
    pub const fn as_mut_str(&mut self) -> &mut str {
        strify_mut(unsafe { slice::from_raw_parts_mut(self.data.expect("String is empty").as_ptr(), self.size as usize) })
    }

    /// returns string data as `&[u8]`
    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.expect("String is empty").as_ptr(), self.size as usize) }
    }

    /// returns String contents as `&mut [u8]`
    pub const fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data.expect("String is empty").as_ptr(), self.size as usize) }
    }

    /// converts content of the `String` to pointer
    #[inline]
    pub const fn as_ptr(&self) -> *const u8 {
        if let Some(data) = self.data {
            data.as_ptr()
        } else {
            ptr::null()
        }
    }

    /// converts content of the `String` to mutable pointer
    #[inline]
    pub const fn as_mut_ptr(&self) -> *mut u8 {
        if let Some(data) = self.data {
            data.as_ptr()
        } else {
            null_mut()
        }
    }

    /// returns character at certain index
    /// - returns `b'\0'` if out of bounds or string is empty
    pub const fn at(&self, index: usize) -> Option<u8> {
        if let Some(data) = self.data {
            if index < self.size as usize {
                Some(unsafe { *data.add(index).as_ptr() })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// returns character at certain index
    /// - returns `b'\0'` if out of bounds or string is empty
    #[inline]
    pub const unsafe fn at_unchecked(&self, index: usize) -> u8 {
        unsafe { *self.data.expect("String is empty").add(index).as_ptr() }
    }

    /// returns mutable reference to character at certain index
    pub const fn at_mut(&mut self, index: usize) -> Option<&mut u8> {
        if let Some(data) = self.data {
            if index < self.size as usize {
                Some(unsafe { data.add(index).as_mut() })
            } else {
                None
            }
        } else {
            None
        }
    }
    /// returns mutable reference to character while not checking if string is empty
    #[inline]
    pub const unsafe fn at_mut_unchecked(&self, index: usize) -> &mut u8 {
        unsafe { self.data.expect("String is empty").add(index).as_mut() }
    }

    pub fn iter<'l>(&'l self) -> core::slice::Iter<'l, u8> {
        self.as_bytes().iter()
    }

    pub fn iter_mut<'l>(&'l mut self) -> core::slice::IterMut<'l, u8> {
        self.as_bytes_mut().iter_mut()
    }

}


impl TryFrom<&str> for String {
    type Error = ();
    /// returns error if failed to allocate data
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut string = String::try_with_capacity(value.len())?;

        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), string.data.unwrap().as_ptr(), value.len()) };

        string.size = value.len() as u32;
        Ok(string)

    }
}

impl TryFrom<&String> for String {
    type Error = ();
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let mut string = String::try_with_capacity(value.len())?;

        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), string.as_mut_ptr(), value.len()); }

        string.size = value.len() as u32;
        Ok(string)
    }
}

impl core::cmp::PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
    fn ne(&self, other: &Self) -> bool {
        self.as_str().ne(other.as_str())
    }
}

impl core::cmp::PartialOrd for String {
    fn ge(&self, other: &Self) -> bool {
        self.as_str().ge(other.as_str())
    }
    fn gt(&self, other: &Self) -> bool {
        self.as_str().gt(other.as_str())
    }
    fn le(&self, other: &Self) -> bool {
        self.as_str().le(other.as_str())
    }
    fn lt(&self, other: &Self) -> bool {
        self.as_str().lt(other.as_str())
    }
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl core::hash::Hash for String {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl core::ops::Deref for String {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl core::ops::DerefMut for String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_bytes_mut()
    }
}

impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::ops::Index<usize> for String {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(data) = self.data {
            if self.size > index as u32 {
                return unsafe { data.add(index).as_ref() };
            } else {
                panic!("index is out of bounds");
            }
        } else {
            panic!("String is empty");
        }
    }
}

impl core::ops::IndexMut<usize> for String {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(data) = self.data {
            if self.size > index as u32 {
                return unsafe { data.add(index).as_mut() };
            } else {
                panic!("index is out of bounds");
            }
        } else {
            panic!("String is empty");
        }
    }
}

impl<'l> IntoIterator for &'l String {
    type IntoIter = core::slice::Iter<'l, u8>;
    type Item = &'l u8;
    fn into_iter(self) -> Self::IntoIter {
        self.as_bytes().iter()
    }
}


impl Drop for String {
    #[inline]
    fn drop(&mut self) {
        if let Some(data) = self.data {
            unsafe { ALLOCATOR.dealloc(data.as_ptr(), String::mk_layout(self.capacity())) };
        }
    }
}

impl core::fmt::Display for String {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl core::fmt::Debug for String {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "string: {self}\nlen: {}\ncap: {}", self.size, self.cap)
    }
}

impl core::fmt::Write for String {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.try_push(c as u8).map_err(|_| core::fmt::Error)
    }
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.try_push_str(s).map_err(|_| core::fmt::Error)
    }
}


impl Default for String {
    #[inline]
    fn default() -> Self {
        Self {
            data: None,
            size: 0,
            cap: 0
        }
    }
}

