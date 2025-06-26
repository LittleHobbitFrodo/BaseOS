//	renderer.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

use limine_rs as limine;
use spin::MutexGuard;
use crate::renderer::font;


use crate::renderer::color::Color;

pub const TAB_SIZE: usize = 6;
pub const SPACE_BETWEEN_LINES: u16 = 3;


/// Helper trait for the [`Renderer`]
/// - classic [`core::fmt::Display`] should be prefered
pub trait Render {
    fn render(&self);
    fn render_locked<'l>(&self, guard: &'l mut MutexGuard<Renderer>);
}


//pub static mut RENDERER: Renderer = Renderer::new();
pub static RENDERER: spin::Mutex<Renderer> = spin::Mutex::new(Renderer::new());
//pub static mut RENDERER: SyncCell<Renderer> = SyncCell::new(Renderer::new());


pub struct Renderer {
    row: usize,
    line: usize,
    fb: FrameBuffer,
    col: Color,
    space: u16,
    initialized: bool,
}

unsafe impl Sync for Renderer {}
unsafe impl Send for Renderer {}

impl Renderer {
    pub const fn new() -> Self {
        Self {
            row: 0,
            line: 0,
            fb: FrameBuffer::new(),
            col: Color::new(0xffffff),
            space: 0,
            initialized: false,
        }
    }

    fn init(&mut self, fb: &limine::request::FramebufferRequest) -> Result<(), ()> {
        self.col = Color::new_rgb(255, 255, 255);
        self.row = 0;
        self.line = 0;
        self.space = SPACE_BETWEEN_LINES;
        if let Ok(_) = FrameBuffer::init(&mut self.fb, fb) {
            self.initialized = true;
            Ok(())
        } else {
            self.initialized = false;
            Err(())
        }
    }

    #[inline(always)] pub fn column(&self) -> usize { self.row }
    #[inline(always)] pub fn line(&self) -> usize { self.line }
    #[inline(always)] pub fn fb(&self) -> &FrameBuffer { &self.fb }
    #[inline(always)] pub fn color(&self) -> Color { self.col }
    #[inline(always)] pub fn set_color(&mut self, color: u32) {self.col.set_int(color);}



    fn space(&mut self) {
        self.row += 1;
        if self.row >= self.fb.width {
            self.row = 0;
            self.line += 1;
        }
    }
    #[inline]
    pub fn endl(&mut self) {
        self.line += 1;
        self.row = 0;
    }
    #[inline]
    pub fn tab(&mut self) {
        self.row += TAB_SIZE - (self.row % TAB_SIZE);
        if self.row >= self.fb.width {
            self.endl();
        }
    }

    pub fn clear(&mut self) {
        for i in unsafe { core::slice::from_raw_parts_mut(self.fb.address, self.fb.width*self.fb.height) } {
            i.set_int(0);       //  black
        }
    }

    pub fn set_pos(&mut self, line: usize, row: usize) -> Result<(), ()> {
        if line < self.fb.height && row < self.fb.width {
            self.line = line;
            self.row = row;
            Ok(())
        } else {
            Err(())
        }
    }

    fn rend(&mut self, c: u8) {
        match c {
            0..31 => {
                match c {
                    b'\n' => {
                        self.endl();
                        return;
                    },
                    b'\t' => {
                        self.tab();
                        return;
                    },
                    _ => return,
                }
            },
            _ => {
                let fnt = match crate::renderer::font::FONT.as_ref().get_char(c) {
                    Some(f) => f,
                    None => &font::ERR_CHAR,
                };
                let fb = self.fb();
                let arr = unsafe {fb.address().cast::<u32>().add((self.line * fb.width * (font::FONT_BITS + self.space as usize)) + self.row * font::FONT_BITS)};

                for i in 0..font::FONT_BITS {
                    for ii in 0..font::FONT_BITS {
                        unsafe {arr.add((i as usize * fb.width) + (font::FONT_BITS - ii as usize)).write(self.color().as_int() * ((fnt[i] as u32 >> ii as u32) & 1))};
                    }
                }
                self.row += 1;
                if self.row >= self.fb.height {
                    self.endl();
                }
            }
        }
    }

    #[inline(always)]
    pub fn render(&mut self, c: u8) {
        if self.initialized {
            self.rend(c);
        }
    }

    #[inline(always)]
    pub fn print(&mut self, str: &[u8]) {
        if self.initialized {
            for i in 0..str.len() {
                self.rend(str[i]);
            }
        }
    }
    pub fn println(&mut self, str: &[u8]) {
        if self.initialized {
            for i in 0..str.len() {
                self.rend(str[i]);
            }
            self.endl();
        }
    }
}

impl AsRef<Renderer> for Renderer {
    #[inline(always)]
    fn as_ref(&self) -> &Renderer {
        &self
    }
}

impl AsMut<Renderer> for Renderer {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut Renderer {
        self
    }
}

pub struct FrameBuffer {
    width: usize,
    height: usize,
    address: *mut Color,
    bpp: usize,
}

impl FrameBuffer {
    pub const fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            address: core::ptr::null_mut(),
            bpp: 0,
        }
    }
    pub fn init(&mut self, fb: &limine::request::FramebufferRequest) -> Result<(), ()> {
        if let Some(res) = fb.get_response() {
            if let Some(framebuffer) = res.framebuffers().next() {
                self.bpp = framebuffer.bpp() as usize;
                self.width = framebuffer.width() as usize;
                self.height = framebuffer.height() as usize;
                self.address = framebuffer.addr() as *mut Color;
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn bpp(&self) -> usize { self.bpp }
    pub fn address(&self) -> *mut Color {
        self.address
    }
}


#[inline(always)]
pub fn init() -> Result<(), ()> {
    RENDERER.lock().init(&bootloader::FRAMEBUFFER)
}


impl core::fmt::Write for Renderer {
    #[inline]
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.render(c as u8);
        Ok(())
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s.as_bytes());
        Ok(())
    }

}

impl Renderer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s.as_bytes());
        Ok(())
    }
}