//	io/mod.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build



//  this module provides basic IO functionalities
//      such as [`in`] and [`out`] instructions and better text rendering

use core::arch::asm;

#[macro_use]
pub mod text;


pub mod int {
    use core::arch::asm;
    
    #[inline(always)]
    pub fn disable() {
        unsafe { asm!("cli"); }
    }

    #[inline(always)]
    pub fn enable() {
        unsafe { asm!("sti"); }
    }
}

pub fn outb(port: u16, data: u8) {
    unsafe {
        asm!("out %0, %1",
        in("al") data,
        in("dx") port,
        options(nostack));
    }
}

pub fn inb(port: u16) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("in %1, %0",
        out("al") ret,
        in("dx")port,
        options(nostack));
    }
    ret
}

pub fn outw(port: u16, data: u16) {
    unsafe {
        asm!("out %0, %1",
        in("ax") data,
        in("dx") port,
        options(nostack));
    }
}

pub fn inw(port: u16) -> u16 {
    let mut ret: u16;
    unsafe {
        asm!("in %1, %0",
        out("ax") ret,
        in("dx") port,
        options(nostack));
    }
    ret
}

pub fn outd(port: u16, data: u32) {
    unsafe {
        asm!("out %0, %1",
        in("eax") data,
        in("dx") port,
        options(nostack));
    }
}

pub fn ind(port: u16) -> u32 {
    let mut ret: u32;
    unsafe {
        asm!("in %1, %0",
        out("eax") ret,
        in("dx") port,
        options(nostack));
    }
    ret
}

pub fn outq(port: u16, data: u64) {
    unsafe {
        asm!("out %0, %1",
        in("rax") data,
        in("dx") port,
        options(nostack));
    }
}

pub fn inq(port: u16) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!("in %1, %0",
        out("rax") ret,
        in("dx") port,
        options(nostack));
    }
    ret
}


pub fn wait() {
    //  wait aprox. nanosecond
    outb(0x80, 0);
}