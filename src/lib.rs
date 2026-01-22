extern crate pancurses;
extern crate rand;
extern crate term_size;

use pancurses::*;

use rand::distributions::{Distribution, Standard};

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;

// RNG boilerplate (kept from original)
thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}

fn rng<T>() -> T
where
    Standard: Distribution<T>,
{
    RNG.with(|rng| (*rng).borrow_mut().r#gen::<T>())
}

pub struct Bouncer {
    x: i32,      // Current X position
    y: i32,      // Current Y position
    prev_x: i32, // Previous X (for erasing)
    prev_y: i32, // Previous Y (for erasing)
    dx: i32,     // Velocity X (+1 or -1)
    dy: i32,     // Velocity Y (+1 or -1)
    color: i16,  // Current color (0-7)
    max_x: i32,  // Right boundary
    max_y: i32,  // Bottom boundary
}

impl Bouncer {
    pub fn new() -> Self {
        let (max_y, max_x) = get_term_size();

        // Start at random position (with padding to avoid edges)
        let start_x = rng::<i32>() % (max_x - 4) + 2;
        let start_y = rng::<i32>() % (max_y - 4) + 2;

        Bouncer {
            x: start_x,
            y: start_y,
            prev_x: start_x,
            prev_y: start_y,
            dx: if rng::<bool>() { 1 } else { -1 }, // Random direction
            dy: if rng::<bool>() { 1 } else { -1 },
            color: COLOR_BLUE,
            max_x: max_x - 1,
            max_y: max_y - 1,
        }
    }
    fn change_color(&mut self) {
        let colors = [
            COLOR_GREEN,
            COLOR_BLUE,
            COLOR_WHITE,
            COLOR_YELLOW,
            COLOR_CYAN,
            COLOR_MAGENTA,
            COLOR_RED,
        ];
        self.color = colors[rng::<usize>() % colors.len()];
    }

    pub fn update(&mut self) {
        // Save current position before moving
        self.prev_x = self.x;
        self.prev_y = self.y;

        // Apply velocity
        self.x += self.dx;
        self.y += self.dy;

        // Bounce off left/right walls
        if self.x <= 0 {
            self.x = 0; // Clamp to boundary
            self.dx = -self.dx; // Reverse direction
            self.change_color(); // Visual feedback
        } else if self.x >= self.max_x {
            self.x = self.max_x;
            self.dx = -self.dx;
            self.change_color();
        }

        // Bounce off top/bottom walls
        if self.y <= 0 {
            self.y = 0;
            self.dy = -self.dy;
            self.change_color();
        } else if self.y >= self.max_y {
            self.y = self.max_y;
            self.dy = -self.dy;
            self.change_color();
        }
    }
    pub fn draw(&self, window: &Window) {
        // Erase old position (overwrite with space)
        window.mvaddstr(self.prev_y, self.prev_x, " ");

        // Draw new position
        window.attron(COLOR_PAIR(self.color as chtype));
        window.mvaddstr(self.y, self.x, "❄");
        window.attroff(COLOR_PAIR(self.color as chtype));

        window.refresh(); // Actually display changes
        napms(50); // ~20 FPS (50ms per frame)
    }
    pub fn resize(&mut self) {
        let (lines, cols) = get_term_size();
        self.max_y = lines as i32 - 1;
        self.max_x = cols as i32 - 1;

        // Reset position if we are now outside bounds
        if self.x >= self.max_x {
            self.x = self.max_x - 1;
        }
        if self.y >= self.max_y {
            self.y = self.max_y - 1;
        }
    }
}

/// Get terminal dimensions safely
pub fn get_term_size() -> (i32, i32) {
    match term_size::dimensions() {
        Some((width, height)) => {
            // Enforce minimum size to prevent crashes
            let width = width.max(10);
            let height = height.max(10);
            (height as i32, width as i32) // Return (rows, cols)
        }
        None => (24, 80), // Fallback to classic terminal size
    }
}

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
pub fn resize_window() {
    endwin();
    initscr();
}

/// Clean exit: restore terminal state
pub fn finish() {
    curs_set(1); // Show cursor again
    endwin(); // End ncurses mode
    std::process::exit(0);
}
