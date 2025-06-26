//	renderer/color.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build


//  this file provides simple [`Color`] structure to use with the renderer

use core::fmt::Display;

use crate::renderer::{Render, RENDERER};

#[derive(Copy, Clone)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Copy, Clone)]
union Col {
    rgb: Rgb,
    int: u32,
}

#[derive(Copy, Clone)]
pub struct Color {
    value: Col
}

impl Color {
    #[inline(always)]
    pub fn as_int(&self) -> u32 {
        unsafe { self.value.int }
    }
    #[inline(always)]
    pub fn as_rgb(&self) -> Rgb {
        unsafe { self.value.rgb }
    }
    #[inline(always)]
    pub fn set_int(&mut self, val: u32) {
        self.value.int = val;
    }
    #[inline(always)]
    pub fn set_rgb(&mut self, val: Rgb) {
        self.value.rgb = val;
    }
}


impl Color {

    pub const fn new(value: u32) -> Self {
        Self { value: Col {int: value} }
    }
    pub const fn new_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { value: Col { rgb: Rgb { r: red, g: green, b: blue } } }
    }
    pub const fn new_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {value: Col { int: red as u32 | (green as u32) << 8 | (blue as u32) << 16 | (alpha as u32) << 24 } }
    }

    #[inline(always)]
    pub fn red(&self) -> u8 {
        unsafe { self.value.rgb.r }
    }
    #[inline(always)]
    pub fn green(&self) -> u8 {
        unsafe { self.value.rgb.g }
    }
    #[inline(always)]
    pub fn blue(&self) -> u8 {
        unsafe { self.value.rgb.b }
    }

    #[inline]
    pub fn set_red(&mut self, red: u8) {
        self.value.rgb.r = red;
    }
    #[inline]
    pub fn set_green(&mut self, green: u8) {
        self.value.rgb.g = green;
    }
    #[inline]
    pub fn set_blue(&mut self, blue: u8) {
        self.value.rgb.b = blue;
    }

}

