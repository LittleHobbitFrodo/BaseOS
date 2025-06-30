//  init.rs
//  this file originally belonged to baseOS project
//      on OS template on which to build

//  module for kernel initialization


use ministd::{dbg, io};
use ministd::{println, print, locked_println, eprintln, init};
use ministd::{Box, Array, Vec, String, HashMap};
use core::fmt::Write;

fn init() -> Result<(), ()> {

    if let Err(_) = init::renderer() {
        panic!("failed to initialize renderer");
    }

    if let Err(_) = init::allocator() {
        panic!("failed to initialize heap");
    }

    println!("hello world!");

    let mut h: HashMap<usize, String> = HashMap::new();

    h.insert(1, String::from("hello world!"));

    let get = h.get(&1).expect("failed to get");

    println!("get: {}", get);


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
