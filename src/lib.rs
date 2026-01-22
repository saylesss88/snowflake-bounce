extern crate pancurses;

use pancurses::*;

/// Initialize ncurses with sensible defaults
pub fn ncurses_init() -> Window {
    let window = initscr(); // Initialize the screen

    // Configure terminal behavior
    window.nodelay(true); // getch() doesn't block (returns None immediately)
    noecho(); // Don't echo keypresses to screen
    cbreak(); // Disable line buffering (get chars immediately)
    curs_set(0); // Hide the cursor

    // Set up colors
    if has_colors() {
        start_color(); // Enable color mode
        use_default_colors(); // Use terminal's default bg/fg

        // Initialize color pairs (foreground color, background -1 = default)
        for i in 0..8 {
            init_pair(i, i, -1);
        }
    }

    window.refresh(); // Apply all changes
    window
}

/// Clean exit: restore terminal state
pub fn finish() {
    curs_set(1); // Show cursor again
    endwin(); // End ncurses mode
    std::process::exit(0);
}
