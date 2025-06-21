//	mem/text.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build


pub use core::fmt::write;

/// formats and renders stuff onto the screen
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!(*$crate::renderer::RENDERER.lock(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        _ = writeln!(*$crate::renderer::RENDERER.lock(), $($arg)*);
    }};
}

/// uses lock renderer to print to screen
#[macro_export]
macro_rules! locked_print {
    ($guard:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        _ = write!($guard, $($arg)*);
    }};
}

#[macro_export]
macro_rules! locked_println {
    ($guard:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!($guard, $($arg)*);
    }};
}
