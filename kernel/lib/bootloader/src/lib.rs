//	bootloader.rs (bootloader crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build

#![no_std]

//  put bootloader requests here

pub use limine_rs;

use limine_rs::request::{FramebufferRequest, HhdmRequest,
    MemoryMapRequest, RequestsEndMarker,
    RequestsStartMarker};


#[unsafe(link_section = ".requests_start_marker")]
pub static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
pub static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();






//  REQUESTS NEEDED BY LOCAL MINISTD CRATE
//  please do no change names or delete it

///  reserved by [`ministd::renderer`]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::with_revision(4);

/// These two variables are used by the default implementation of [`mem::heap::init()`]
///   - exactly by [`mem::find_heap_region`] (in the main crate)
/// - By changing the function you can modify this request
#[unsafe(link_section = ".requests")]
pub static HHDM: HhdmRequest = HhdmRequest::new();
#[unsafe(link_section = ".requests")]
pub static MEMMAP: MemoryMapRequest = MemoryMapRequest::new();


