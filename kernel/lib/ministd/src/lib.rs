//	lib.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

#![no_std]
#![no_main]
//#![deny(static_mut_refs)]


use core::ops::Deref;
/// # MINISTD crate
/// This crate mimics basic functionalities of the STD crate  
/// Each functionality that provides [`init()`] function is meant to be initialized manually in your kernel [`init()`] function
/// 
/// PS: bootloader requests are done in the [`bootloader`] local crate


pub use core::pin::Pin;


//  used modules
pub mod mem;
pub mod renderer;
#[macro_use]
pub mod io;
pub mod convert;
pub mod init;

//  modules
pub use mem::string::String;
pub use mem::boxed::Box;
pub use mem::vec::Vec;
pub use mem::array::Array;
pub use mem::alloc::{self, ALLOCATOR, Allocator};

//  local crates
pub use bootloader;
pub use limine_rs as limine;
pub use buddy_system_allocator as allocator;
pub use spin;


pub use spin::{Mutex, MutexGuard,
    RwLock, RwLockReadGuard, RwLockWriteGuard, RwLockUpgradableGuard,
    Lazy, Barrier, Once};

use core::arch::asm;
use core::hint::spin_loop;
pub use core::convert::{Infallible, From, TryFrom, Into, TryInto};


pub fn hang() -> ! {
    loop {
        io::int::disable();
        unsafe { asm!("hlt"); }
        spin_loop();
    }
}


/// Allows cloning if failure is possible
pub trait TryClone {
    type Error;
    fn try_clone(&self) -> Result<Self, Self::Error>
    where Self: Sized;
}

/// # Nothing
/// 
/// This structure represents ..., well ... nothing  
/// 
/// Usage:
/// - No data while returning `Err` but still needs to be constructed
#[derive(Copy, Clone)]
pub struct Nothing();

impl Default for Nothing {
    #[inline(always)]
    fn default() -> Self {
        Nothing()
    }
}



/// structure used for sigle-threaded immutable data access
pub struct Immutable<T: Sized> {
    data: T,
}

impl<T: Sized> Immutable<T> {
    pub const fn new(val: T) -> Self {
        Self {
            data: val,
        }
    }
}

impl<T: Sized> Deref for Immutable<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
