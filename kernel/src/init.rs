//  init.rs
//  this file originally belonged to baseOS project
//      on OS template on which to build

//  module for kernel initialization

use ministd::mem::dynamic_buffer::DynamicBuffer;
use ministd::renderer::RENDERER;
use core::mem::MaybeUninit;
use ministd::{io};
use ministd::{println, locked_println, init};
use ministd::{Box, Array, Vec, String};

fn init() -> Result<(), ()> {

    if let Err(_) = init::renderer() {
        panic!("failed to initialize renderer");
    }

    if let Err(_) = init::allocator() {
        panic!("failed to initialize heap");
    }

    let mut v: Vec<u32> = Vec::new();

    v.push(69);
    v.push(420);

    for (i, item) in v.iter().enumerate() {
        println!("{i}: {item}");
    }

    Ok(())

}

#[unsafe(no_mangle)]
extern "C" fn _start() {

    io::int::disable();

    if let Err(_) = init() {
        panic!("failed to initialize the kernel");
    }

    ministd::hang();
}
