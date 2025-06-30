//  init.rs
//  this file originally belonged to baseOS project
//      on OS template on which to build

//  module for kernel initialization


use ministd::{dbg, io};
use ministd::{println, print, locked_println, eprintln, init};
use ministd::{Box, Array, Vec, String};
use core::fmt::Write;

fn init() -> Result<(), ()> {

    if let Err(_) = init::renderer() {
        panic!("failed to initialize renderer");
    }

    if let Err(_) = init::allocator() {
        panic!("failed to initialize heap");
    }

    println!("hello world!");

    let mut string: String = String::from("hello world!");

    let x = 69;
    write!(string, "x: {x}").expect("failed to parse");

    println!("string: {string}");

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
