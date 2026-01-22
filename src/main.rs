extern crate libc;
extern crate pancurses;
extern crate signal_hook;
extern crate snowflake_bounce;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use pancurses::*;
use signal_hook::consts::signal::{SIGINT, SIGQUIT, SIGTERM, SIGWINCH};
use signal_hook::flag;

use snowflake_bounce::Bouncer;

fn main() {
    // Enable UTF-8 locale for Unicode snowflakes
    unsafe {
        libc::setlocale(libc::LC_ALL, std::ffi::CString::new("").unwrap().as_ptr());
    }

    let window = snowflake_bounce::ncurses_init();

    // Set up signal handlers
    let exit_signal = Arc::new(AtomicBool::new(false));
    let resize_signal = Arc::new(AtomicBool::new(false));

    flag::register(SIGWINCH, Arc::clone(&resize_signal)).unwrap();
    flag::register(SIGINT, Arc::clone(&exit_signal)).unwrap();
    flag::register(SIGTERM, Arc::clone(&exit_signal)).unwrap();
    flag::register(SIGQUIT, Arc::clone(&exit_signal)).unwrap();

    let mut bouncer = Bouncer::new();

    // Main event loop
    loop {
        // Handle window resize
        if resize_signal.swap(false, Ordering::Relaxed) {
            snowflake_bounce::resize_window();
            bouncer.resize();
        }

        // Handle exit signals
        if exit_signal.swap(false, Ordering::Relaxed) {
            snowflake_bounce::finish();
        }

        // Handle keypresses
        if let Some(Input::Character('q')) = window.getch() {
            snowflake_bounce::finish();
        }

        // Update and draw
        bouncer.update();
        bouncer.draw(&window);
    }
}
