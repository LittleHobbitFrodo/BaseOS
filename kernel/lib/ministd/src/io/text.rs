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


#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut rend = $crate::renderer::RENDERER.lock();

        let c = rend.color();

        rend.set_color(0xff9a9a);

        let _ = write!(*rend, $($arg)*);

        rend.set_color(c.as_int());
    }};
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut rend = $crate::renderer::RENDERER.lock();

        let c = rend.color();

        rend.set_color(0xff9a9a);

        let _ = writeln!(*rend, $($arg)*);
        
        rend.set_color(c.as_int());
    }};
}

#[macro_export]
macro_rules! locked_eprint {
    ($guard:expr, $($arg:tt)*) => {{
        use core::fmt::Write;

        let c = $guard.color();

        $guard.set_color(0xff9a9a);

        let _ = write!(*$guard, $($arg)*);

        $guard.set_color(c.as_int());
    }};
}

#[macro_export]
macro_rules! locked_eprintln {
    ($guard:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        let c = $guard.color();

        $guard.set_color(0xff9a9a);

        let _ = writeln!(*$guard, $($arg)*);
        
        $guard.set_color(c.as_int());
    }};
}


#[macro_export]
macro_rules! dbg {

    () => {
        $crate::eprintln!("[{}:{}:{}]", core::file!(), core::line!(), core::column!());
    };
    ($val:expr $(,)?) => {{

        let mut rend = $crate::renderer::RENDERER.lock();
        let color = rend.color();
        rend.set_color(0xff9a9a);

        let value = &$val;

        $crate::locked_eprint!(rend, "[{}:{}:{}] {} = {:#?}", core::file!(), core::line!(), core::column!(), core::stringify!($val),
        &&value as &dyn core::fmt::Debug);
        rend.set_color(color.as_int());
    }};
}

#[macro_export]
macro_rules! locked_dbg {

    ($guard:expr) => {
        $crate::locked_eprintln!($guard, "[{}:{}:{}]", core::file!(), core::line!(), core::column!());
    };
    ($guard:expr, $val:expr $(,)?) => {{
        let color = $guard.color();
        $guard.set_color(0xff9a9a);

        let value = &$val;

        $crate::locked_eprint!($guard, "[{}:{}:{}] {} = {:#?}", core::file!(), core::line!(), core::column!(), core::stringify!($val),
        &&value as &dyn core::fmt::Debug);
        $guard.set_color(color.as_int());
    }};
}