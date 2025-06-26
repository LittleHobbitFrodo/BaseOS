//	prelude.rs
//	this file originally belonged to baseOS project
//		an OS template on which to build

//  this file is used as target in the util script

#![no_std]
#![no_main]

//  used core utilities
pub use core::clone::Clone;
pub use core::marker::Copy;
pub use core::mem::{align_of, size_of};


pub mod init;
pub mod manage;
pub mod mem;

//  local crates
pub use ministd;
pub use bootloader;
pub use limine_rs as limine;


pub use ministd::{print, println};

pub use core::panic;

