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

// Thread-local pseudo-random number generator used by the animation.
//
// A small, cheap RNG is created per thread and reused for all random
// choices such as starting position, velocity, and color.
thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}

/// Generate a random value using the thread-local RNG.
///
/// This uses the `Standard` distribution for the requested type `T`.
fn rng<T>() -> T
where
    Standard: Distribution<T>,
{
    RNG.with(|rng| (*rng).borrow_mut().r#gen::<T>())
}

/// Symbol selection for the animated “logo” drawn in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolMode {
    /// Single-character Unicode snowflake.
    SnowflakeSmall,
    /// Larger 3-line snowflake.
    SnowflakeLarge,
    /// Multi-line ASCII-art NixOS logo.
    NixOS,
    /// Single-character Unicode middle finger.
    MiddleFinger,
}

/// Animated logo that moves and bounces within the terminal.
///
/// The `Bouncer` tracks position, velocity, current color, and which
/// symbol/logo to render, and exposes methods to update physics,
/// redraw, and respond to resizes and key events.
pub struct Bouncer {
    x: i32,
    y: i32,
    prev_x: i32,
    prev_y: i32,
    dx: i32,
    dy: i32,
    color: i16,
    max_x: i32,
    max_y: i32,
    /// Currently selected display mode.
    pub mode: SymbolMode,
}

impl Bouncer {
    /// Create a new `Bouncer` with random starting position and direction.
    ///
    /// The starting coordinates are chosen to keep the initial logo away
    /// from the edges of the terminal, and the default symbol mode is
    /// the NixOS logo.
    #[must_use]
    pub fn new() -> Self {
        let (max_y, max_x) = get_term_size();

        let start_x = rng::<i32>() % (max_x - 50).max(5) + 2;
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
            mode: SymbolMode::NixOS,
        }
    }

    /// Cycle through the available symbol modes.
    ///
    /// Pressing `'s'` in the UI calls this to step between small snowflake,
    /// large snowflake, and NixOS logo. The middle-finger mode always
    /// cycles back to the small snowflake.
    pub const fn cycle_symbol(&mut self) {
        self.mode = match self.mode {
            SymbolMode::SnowflakeSmall => SymbolMode::SnowflakeLarge,
            SymbolMode::SnowflakeLarge => SymbolMode::NixOS,
            SymbolMode::NixOS => SymbolMode::SnowflakeSmall,
            SymbolMode::MiddleFinger => SymbolMode::SnowflakeSmall,
        };
    }

    /// Randomize the current color used to draw the logo.
    pub fn cycle_color(&mut self) {
        self.change_color();
    }

    /// Internal helper to pick a new random color.
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

    /// Advance the animation by one step.
    ///
    /// Updates position based on velocity, bounces off the terminal
    /// boundaries, and randomizes color on each bounce.
    pub fn update(&mut self) {
        self.prev_x = self.x;
        self.prev_y = self.y;

        self.x += self.dx;
        self.y += self.dy;

        let (logo_width, logo_height) = self.get_logo_dimensions();

        if self.x <= 0 {
            self.x = 0;
            self.dx = -self.dx;
            self.change_color();
        } else if self.x + logo_width >= self.max_x {
            self.x = self.max_x - logo_width;
            self.dx = -self.dx;
            self.change_color();
        }

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

    /// Switch the current symbol to the middle-finger glyph.
    ///
    /// Pressing `'f'` in the UI calls this.
    pub fn set_middle_finger(&mut self) {
        self.mode = SymbolMode::MiddleFinger;
    }

    /// Logical width and height of the current logo in characters.
    ///
    /// These dimensions are used for collision detection and erasing.
    const fn get_logo_dimensions(&self) -> (i32, i32) {
        match self.mode {
            SymbolMode::SnowflakeSmall => (1, 1),
            SymbolMode::SnowflakeLarge => (5, 3),
            SymbolMode::NixOS => (46, 19),
            SymbolMode::MiddleFinger => (2, 1),
        }
    }

    /// Text lines for the currently selected logo.
    ///
    /// Each string in the returned vector represents one row to be drawn.
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

    /// Erase the previous logo position and draw it at the new coordinates.
    ///
    /// This uses ncurses to clear the previous rectangle, apply the
    /// current color pair, print each logo line, and refresh the window.
    pub fn draw(&self, window: &Window) {
        let logo_lines = self.get_logo_lines();
        let (logo_width, logo_height) = self.get_logo_dimensions();

        // Clear the old area.
        for i in 0..logo_height {
            let erase_str = " ".repeat(logo_width as usize);
            window.mvaddstr(self.prev_y + i, self.prev_x, &erase_str);
        }

        // Draw the logo at the new location with the active color pair.
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

    /// Update cached terminal bounds and clamp the logo position.
    ///
    /// This should be called after handling a resize event so that the
    /// bouncer uses the new terminal size for collision detection.
    pub fn resize(&mut self) {
        let (lines, cols) = get_term_size();
        self.max_y = lines - 1;
        self.max_x = cols - 1;

        let (logo_width, logo_height) = self.get_logo_dimensions();

        if self.x + logo_width >= self.max_x {
            self.x = (self.max_x - logo_width).max(0);
        }
        if self.y + logo_height >= self.max_y {
            self.y = (self.max_y - logo_height).max(0);
        }
    }
}

impl Default for Bouncer {
    /// Construct a default `Bouncer` using `Bouncer::new()`.
    fn default() -> Self {
        Self::new()
    }
}

/// Get terminal dimensions as `(rows, cols)` with sane minimums.
///
/// Returns a fallback of `(24, 80)` if the size cannot be detected.
#[must_use]
pub fn get_term_size() -> (i32, i32) {
    match term_size::dimensions() {
        Some((width, height)) => {
            let width = width.max(30);
            let height = height.max(15);
            (height as i32, width as i32)
        }
        None => (24, 80),
    }
}

/// Initialize ncurses and return the main window.
///
/// Configures non-blocking input, disables echo, hides the cursor,
/// and sets up basic color pairs using the default terminal background.
pub fn ncurses_init() -> Window {
    let window = initscr();

    window.nodelay(true);
    noecho();
    cbreak();
    curs_set(0);

    if has_colors() {
        start_color();
        use_default_colors();

        for i in 0..8 {
            init_pair(i, i, -1);
        }
    }

    window.refresh();
    window
}

/// Reinitialize the ncurses window after a terminal resize.
pub fn resize_window() {
    endwin();
    initscr();
}

/// Restore the terminal to a usable state and terminate the process.
///
/// This shows the cursor again, ends ncurses mode, and exits with
/// status code 0.
pub fn finish() {
    curs_set(1);
    endwin();
    std::process::exit(0);
}
