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

/// Entry point for the snowflake-bounce animation.
///
/// Sets up locale for Unicode rendering, initializes ncurses,
/// installs signal handlers for resize/exit, and runs the main
/// animation loop until the user quits.
fn main() {
    // Enable UTF-8 locale so Unicode glyphs render correctly in ncurses.
    unsafe {
        libc::setlocale(libc::LC_ALL, std::ffi::CString::new("").unwrap().as_ptr());
    }

    // Initialize the main ncurses window.
    let window = snowflake_bounce::ncurses_init();

    // Shared flags toggled by POSIX signals.
    let exit_signal = Arc::new(AtomicBool::new(false));
    let resize_signal = Arc::new(AtomicBool::new(false));

    // Flip flags whenever the corresponding signal is delivered.
    flag::register(SIGWINCH, Arc::clone(&resize_signal)).unwrap();
    flag::register(SIGINT, Arc::clone(&exit_signal)).unwrap();
    flag::register(SIGTERM, Arc::clone(&exit_signal)).unwrap();
    flag::register(SIGQUIT, Arc::clone(&exit_signal)).unwrap();

    // Animated logo state.
    let mut bouncer = Bouncer::new();

    // Main event loop: react to signals, keys, and update animation.
    loop {
        // Handle terminal resize.
        if resize_signal.swap(false, Ordering::Relaxed) {
            snowflake_bounce::resize_window();
            bouncer.resize();
        }

        // Handle termination signals.
        if exit_signal.swap(false, Ordering::Relaxed) {
            snowflake_bounce::finish();
        }

        // Handle non-blocking key input.
        if let Some(Input::Character(c)) = window.getch() {
            match c {
                'q' => snowflake_bounce::finish(),
                'c' => bouncer.cycle_color(),
                's' => bouncer.cycle_symbol(),
                'f' => bouncer.set_middle_finger(),
                _ => {}
            }
        }

        // Advance physics and redraw.
        bouncer.update();
        bouncer.draw(&window);
    }
}
