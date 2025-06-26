//  init.rs
//  this file originally belonged to baseOS project
//      on OS template on which to build

//  module for kernel initialization


use ministd::{dbg, io};
use ministd::{println, locked_println, init};
use ministd::{Box, Array, Vec, String};

fn init() -> Result<(), ()> {

    if let Err(_) = init::renderer() {
        panic!("failed to initialize renderer");
    }

    if let Err(_) = init::allocator() {
        panic!("failed to initialize heap");
    }

    println!("hello world!");

    

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
