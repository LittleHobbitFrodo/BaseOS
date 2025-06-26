//  init.rs
//  this file originally belonged to baseOS project
//      on OS template on which to build

use ministd::{mem::{alloc::*, Region}, println, renderer::{Color, RENDERER}, MutexGuard};
use bootloader::{MEMMAP, HHDM};
use limine_rs::memory_map::EntryType;
use ministd::mem::{MB, GB};

/// Use this function to find an valid spot for heap
/// Feel free to rewrite it!
/// - but do not change the declaration
#[unsafe(no_mangle)]
extern "Rust" fn find_heap_region() -> Result<Region, ()> {

    let hhdm = match HHDM.get_response() {
        Some(res) => res.offset(),
        None => return Err(()),
    } as usize;

    if let Some(res) = MEMMAP.get_response() {
        for i in res.entries() {
            match i.entry_type {
                EntryType::USABLE => {
                    if (i.length as usize) > MB && (i.base as usize) < 4*GB {   //  4GB should be the boundary for HHDM
                        return Ok(Region::new(i.base as usize + hhdm, core::cmp::min(i.length as usize, 2*MB)));
                        //  add HHDM offset to be in virtual address space
                    }
                },
                _ => continue,
            }
        }
    }

    Err(())

}

/// this function is called by the allocator whenever it fails to allocate memory
/// - if it succees (returns Ok) it will try to allocate again  
/// 
/// feel free to rewrite this function but:
/// - be sure you know what are you doing
/// - do not change the declaration
#[unsafe(no_mangle)]
extern "Rust" fn out_of_memory_handler(heap: &mut MutexGuard<Heap>, allocator: &Allocator) -> Result<(), ()> {
    //  use heap to access heap data


    if let Ok(reg) = find_heap_region() {

        //  TODO: push into the regions vector
        unsafe { heap.add_to_heap(reg.start(), reg.start() + reg.size()) }
        Ok(())
    } else {
        Err(())
    }
}


