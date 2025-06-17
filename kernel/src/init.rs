//  init.rs
//  this file originally belonged to baseOS project
//      on OS template on which to build

//  module for kernel initialization

use ministd::renderer::RENDERER;
use core::mem::MaybeUninit;
use ministd::{Array, String};
use ministd::{io};
use ministd::{println, locked_println, init};
use ministd::Box;

fn init() -> Result<(), ()> {

    if let Err(_) = init::renderer() {
        panic!("failed to initialize renderer");
    }

    if let Err(_) = init::allocator() {
        panic!("failed to initialize heap");
    }

    let a = unsafe { Array::assume_init(Array::<usize>::new_uninit(8)) };

    println!("whole:");
    for (i, item) in a.iter().enumerate() {
        println!("arr[{i}]: {item}");
    }

    let slice = if let Some(slice) = a.get(2..=6) { slice } else {
        panic!("failed to get slice");
    };

    println!("\n\n\nslice:");
    for (i, item) in slice.iter().enumerate() {
        println!("slice[{i}]: {item}");
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
