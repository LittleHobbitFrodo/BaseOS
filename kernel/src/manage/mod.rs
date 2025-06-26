//	manage/mod.rs
//	this file originally belonged to baseOS project
//		an OS template on which to build


use ministd::{renderer::{RENDERER}, RwLock, hang};
use core::panic::{PanicInfo};
use ministd::{locked_print, locked_println};
use ministd::convert::strify;

pub mod kernel_state;
pub use kernel_state::*;




/// indicates the state of the kernel
/// - set to `Panic` in the default panic handler
pub static KERNEL_STATE: RwLock<KernelState> = RwLock::new(KernelState::Init(KernelInitState::Base));


#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {

    if RENDERER.is_locked() {
        unsafe { RENDERER.force_unlock() }
    }
    let mut rend = RENDERER.lock();
    if rend.column() > 0 {
        rend.endl();
    }

    unsafe { KERNEL_STATE.force_write_unlock() };
    let state = *KERNEL_STATE.read();
    *KERNEL_STATE.write() = KernelState::Panic;

    //let msg = info.message().as_str();
    let msg = match info.message().as_str() {
        Some(m) => Some(m),
        None => None,
    };
    let location = info.location().unwrap();


    rend.set_color(0xff0000);

    locked_println!(rend, "PANIC occured at {}:{}:{}", location.file(), location.line(), location.column());

    match state {
        KernelState::Init(i) => {
            locked_println!(rend, "while initializing {}", strify(i.as_str()) );
        },
        KernelState::Runtime(r) => {
            locked_println!(rend, "at runtime task {}", strify(r.as_str()));
        },
        KernelState::Shutdown(s) => {
            locked_println!(rend, "at shutdown task {}", strify(s.as_str()));
        },
        KernelState::Panic => {
            locked_println!(rend, "already in panic");
        },
    }

    match msg {
        Some(m) => {
            locked_print!(rend, "error message: ");
            rend.println(m.as_bytes());
        },
        None => locked_println!(rend, "No error message given!"),
    }

    hang();
}
