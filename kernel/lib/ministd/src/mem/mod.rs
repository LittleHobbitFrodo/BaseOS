//	mem/mod.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

/// this file provides basic memory-related functionalities



pub const KB: usize = 1024;
pub const MB: usize = 1024 * 1024;
pub const GB: usize = 1024 * 1024 * 1024;


pub use core::mem::needs_drop;

pub mod readonly;
pub use readonly::ReadOnly;
pub mod alloc;
pub mod boxed;
pub mod array;
pub mod dynamic_buffer;


pub mod string;
pub mod vec;

pub use dynamic_buffer::DynamicBuffer;



#[derive(Copy, Clone)]
pub struct Region {
    start: usize,
    size: usize,
}

impl Region {
    pub const fn new(start: usize, size: usize) -> Self {
        Self {
            start,
            size,
        }
    }

    pub const fn empty() -> Self {
        Self {
            start: 0,
            size: 0,
        }
    }

    #[inline]
    pub fn move_to(&mut self, place: usize) {
        self.start = place;
    }
    #[inline]
    pub fn resize(&mut self, size: usize) {
        self.size = size;
    }
    #[inline]
    pub fn enlarge(&mut self, by: usize) {
        self.size += by;
    }
    #[inline]
    pub fn shrink(&mut self, by: usize) -> Result<(), ()> {
        if let Some(sub) = self.size.checked_sub(by) {
            self.size = sub;
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

}