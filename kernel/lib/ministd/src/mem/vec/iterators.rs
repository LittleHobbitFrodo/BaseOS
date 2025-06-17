//  mem/vec/iterators.rs (ministd crate)
//  this file originally belonged to baseOS project
//      on OS template on which to build


use core::{marker::PhantomData, ptr::{null, null_mut}};

/*use crate::mem::vec::Vec;

use core::ptr::NonNull;

pub struct VecIter<'l, T: Sized + 'l> {
    data: NonNull<T>,
    start: u32,
    end: u32,
    _marker: PhantomData<&'l T>,
}

impl<'l, T: Sized + 'l> VecIter<'l, T> {
    pub const fn new(vec: &Vec<T>) -> Self {
        if vec.is_empty() {
            Self {
                data: unsafe { NonNull::new_unchecked(null_mut()) },
                    //  some workaround lol
                start: 0,
                end: 0,
                _marker: PhantomData,
            }
        } else {
            Self {
                data: unsafe { NonNull::new_unchecked(vec.as_ptr() as *mut T) },
                start: 0,
                end: vec.len() as u32,
                _marker: PhantomData,
            }
        }
    }
}

pub struct VecIterMut<'l, T: Sized + 'l> {
    data: *mut T,
    start: u32,
    end: u32,
    _marker: PhantomData<&'l mut T>,
}

impl<'l, T: Sized + 'l> VecIterMut<'l, T> {
    pub const fn new(vec: &'l mut Vec<T>) -> Self {
        if vec.is_empty() {
            Self {
                data: null_mut(),
                start: 0,
                end: 0,
                _marker: PhantomData
            }
        } else {
            Self {
                data: vec.as_mut_ptr(),
                start: 0,
                end: vec.len() as u32,
                _marker: PhantomData,
            }
        }
    }
}


impl<'l, T: Sized + 'l> Iterator for VecIter<'l, T> {
    type Item = &'l T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let ret = unsafe { self.data.add(self.start as usize).as_ref() };
            self.start += 1;
            Some(ret)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let dif = (self.end - self.start) as usize;
        (dif, Some(dif))
    }

}*/