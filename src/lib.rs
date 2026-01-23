extern crate pancurses;
extern crate rand;
extern crate term_size;

use pancurses::{
    cbreak, chtype, curs_set, endwin, has_colors, init_pair, initscr, napms, noecho, start_color,
    use_default_colors, Window, COLOR_BLUE, COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_PAIR,
    COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};
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

// ✅ Add an enum to track which symbol mode we're in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolMode {
    SnowflakeSmall,
    SnowflakeLarge,
    NixOS,
    MiddleFinger,
}

pub struct Bouncer {
    x: i32,               // Current X position
    y: i32,               // Current Y position
    prev_x: i32,          // Previous X (for erasing)
    prev_y: i32,          // Previous Y (for erasing)
    dx: i32,              // Velocity X (+1 or -1)
    dy: i32,              // Velocity Y (+1 or -1)
    color: i16,           // Current color (0-7)
    max_x: i32,           // Right boundary
    max_y: i32,           // Bottom boundary
    pub mode: SymbolMode, // ✅ Current display mode
}

impl Bouncer {
    #[must_use]
    pub fn new() -> Self {
        let (max_y, max_x) = get_term_size();

        // Start at random position (with padding to avoid edges)
        let start_x = rng::<i32>() % (max_x - 50).max(5) + 2; // More padding for NixOS logo
        let start_y = rng::<i32>() % (max_y - 25).max(5) + 2;

        Self {
            x: start_x,
            y: start_y,
            prev_x: start_x,
            prev_y: start_y,
            dx: if rng::<bool>() { 1 } else { -1 },
            dy: if rng::<bool>() { 1 } else { -1 },
            color: COLOR_BLUE,
            max_x: max_x - 1,
            max_y: max_y - 1,
            mode: SymbolMode::NixOS, // Start with big flake
        }
    }

    // ✅ Cycle through symbol modes
    pub const fn cycle_symbol(&mut self) {
        self.mode = match self.mode {
            SymbolMode::SnowflakeSmall => SymbolMode::SnowflakeLarge,
            SymbolMode::SnowflakeLarge => SymbolMode::NixOS,
            SymbolMode::NixOS => SymbolMode::SnowflakeSmall,
            SymbolMode::MiddleFinger => SymbolMode::SnowflakeSmall,
        };
    }

    // method to change color
    pub fn cycle_color(&mut self) {
        self.change_color();
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

        // Get logo dimensions for boundary checking
        let (logo_width, logo_height) = self.get_logo_dimensions();

        // Bounce off left/right walls
        if self.x <= 0 {
            self.x = 0;
            self.dx = -self.dx;
            self.change_color();
        } else if self.x + logo_width >= self.max_x {
            self.x = self.max_x - logo_width;
            self.dx = -self.dx;
            self.change_color();
        }

        // Bounce off top/bottom walls
        if self.y <= 0 {
            self.y = 0;
            self.dy = -self.dy;
            self.change_color();
        } else if self.y + logo_height >= self.max_y {
            self.y = self.max_y - logo_height;
            self.dy = -self.dy;
            self.change_color();
        }
    }
    pub fn set_middle_finger(&mut self) {
        self.mode = SymbolMode::MiddleFinger;
    }

    // ✅ Get dimensions of current logo
    const fn get_logo_dimensions(&self) -> (i32, i32) {
        match self.mode {
            SymbolMode::SnowflakeSmall => (1, 1),
            SymbolMode::SnowflakeLarge => (5, 3),
            SymbolMode::NixOS => (46, 19), // Updated for the full logo
            SymbolMode::MiddleFinger => (2, 1),
        }
    }

    // ✅ Get the logo lines for the current mode
    fn get_logo_lines(&self) -> Vec<&str> {
        match self.mode {
            SymbolMode::SnowflakeSmall => vec!["❄"],
            SymbolMode::SnowflakeLarge => vec!["  ❄  ", " ❄❄❄ ", "  ❄  "],
            SymbolMode::NixOS => vec![
                "          ::::.    ':::::     ::::'          ",
                "          ':::::    ':::::.  ::::'           ",
                "            :::::     '::::.:::::            ",
                "      .......:::::..... ::::::::             ",
                "     ::::::::::::::::::. ::::::    ::::.     ",
                "    ::::::::::::::::::::: :::::.  .::::'     ",
                "           .....           ::::' :::::'      ",
                "          :::::            '::' :::::'       ",
                " ........:::::               ' :::::::::::.  ",
                ":::::::::::::                 :::::::::::::  ",
                " ::::::::::: ..              :::::           ",
                "     .::::: .:::            :::::            ",
                "    .:::::  :::::          '''''    .....    ",
                "    :::::   ':::::.  ......:::::::::::::'    ",
                "     :::     ::::::. ':::::::::::::::::'     ",
                "            .:::::::: '::::::::::            ",
                "           .::::''::::.     '::::.           ",
                "          .::::'   ::::.     '::::.          ",
                "         .::::      ::::      '::::.         ",
            ],
            SymbolMode::MiddleFinger => vec!["🖕"],
        }
    }

    pub fn draw(&self, window: &Window) {
        let logo_lines = self.get_logo_lines();
        let (logo_width, logo_height) = self.get_logo_dimensions();

        // Erase old position
        for i in 0..logo_height {
            let erase_str = " ".repeat(logo_width as usize);
            window.mvaddstr(self.prev_y + i, self.prev_x, &erase_str);
        }

        // Draw new position
        window.attron(COLOR_PAIR(self.color as chtype));
        for (i, line) in logo_lines.iter().enumerate() {
            window.mvaddstr(
                self.y + i32::try_from(i).expect("logo line index too large"),
                self.x,
                line,
            );
        }
        window.attroff(COLOR_PAIR(self.color as chtype));

        window.refresh();
        napms(50);
    }

    pub fn resize(&mut self) {
        let (lines, cols) = get_term_size();
        self.max_y = lines - 1;
        self.max_x = cols - 1;

        let (logo_width, logo_height) = self.get_logo_dimensions();

        // Reset position if we are now outside bounds
        if self.x + logo_width >= self.max_x {
            self.x = (self.max_x - logo_width).max(0);
        }
        if self.y + logo_height >= self.max_y {
            self.y = (self.max_y - logo_height).max(0);
        }
    }
}
impl Default for Bouncer {
    fn default() -> Self {
        Self::new()
    }
}

/// Get terminal dimensions safely
#[must_use]
pub fn get_term_size() -> (i32, i32) {
    match term_size::dimensions() {
        Some((width, height)) => {
            // Enforce minimum size to prevent crashes
            let width = width.max(30); // Increased for NixOS logo
            let height = height.max(15);
            (height as i32, width as i32) // Return (rows, cols)
        }
        None => (24, 80), // Fallback to classic terminal size
    }
}

/// Initialize ncurses with sensible defaults
pub fn ncurses_init() -> Window {
    let window = initscr();

    // Configure terminal behavior
    window.nodelay(true);
    noecho();
    cbreak();
    curs_set(0);

    // Set up colors
    if has_colors() {
        start_color();
        use_default_colors();

        // Initialize color pairs
        for i in 0..8 {
            init_pair(i, i, -1);
        }
    }

    window.refresh();
    window
}

pub fn resize_window() {
    endwin();
    initscr();
}

/// Clean exit: restore terminal state
pub fn finish() {
    curs_set(1);
    endwin();
    std::process::exit(0);
}
