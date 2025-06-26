//	renderer/mod.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

//  this file is used as target in the util script
pub mod font;

#[macro_use]
pub mod color;
pub use color::Color;


pub mod renderer;
pub use renderer::RENDERER;
pub use renderer::Render;

pub(crate) fn init() -> Result<(), ()> {
    renderer::init()
}