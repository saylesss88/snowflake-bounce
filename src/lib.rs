extern crate pancurses;
extern crate rand;
extern crate term_size;

use crossterm::{
    cursor, queue,
    style::{self, Color, Stylize},
    terminal,
};
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;
use std::io::{self, Write};

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
    x: u16,
    y: u16,
    dx: i32,
    dy: i32,
    color: Color,
    max_x: u16,
    max_y: u16,
    prev_x: u16,
    prev_y: u16,
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
        let (max_x, max_y) = terminal::size().unwrap_or((80, 24));

        let start_x = rng::<i32>() % (max_x - 50).max(5) + 2;
        let start_y = rng::<i32>() % (max_y - 25).max(5) + 2;

        Self {
            x: max_x / 2,
            y: max_y / 2,
            dx: 1,
            dy: 1,
            color: Color::Blue,
            max_x: max_x.saturating_sub(1),
            max_y: max_y.saturating_sub(1),
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
        let colors = [
            Color::Green,
            Color::Blue,
            Color::White,
            Color::Yellow,
            Color::Cyan,
            Color::Magenta,
            Color::Red,
        ];
        self.color = colors[rng::<usize>() % colors.len()];
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
        // 1. Calculate the NEW candidate position as signed integers (i32).
        //    We cast `self.x` (u16) to i32 so we can add negative velocity safely.
        let mut nx = self.x as i32 + self.dx;
        let mut ny = self.y as i32 + self.dy;

        // 2. Get logo size for collision checks
        let (w, h) = self.get_logo_dimensions(); // these return i32s currently

        // 3. Check X-axis collision (left or right wall)
        //    Collision with left wall (0)
        if nx <= 0 {
            nx = 0;
            self.dx = -self.dx; // Reverse direction
            self.change_color();
        }
        // Collision with right wall (max_x)
        else if nx + w >= self.max_x as i32 {
            nx = self.max_x as i32 - w;
            self.dx = -self.dx;
            self.change_color();
        }

        // 4. Check Y-axis collision (top or bottom wall)
        //    Collision with top (0)
        if ny <= 0 {
            ny = 0;
            self.dy = -self.dy;
            self.change_color();
        }
        // Collision with bottom (max_y)
        else if ny + h >= self.max_y as i32 {
            ny = self.max_y as i32 - h;
            self.dy = -self.dy;
            self.change_color();
        }

        // 5. Store the valid result back into our u16 state
        self.x = nx as u16;
        self.y = ny as u16;
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

    pub fn draw(&self, w: &mut impl Write) -> io::Result<()> {
        let (logo_width, logo_height) = self.get_logo_dimensions();
        let logo_lines = self.get_logo_lines();

        // 1. Erase the OLD position.
        //    We use the stored `prev_x` / `prev_y` to know where to overwrite with spaces.
        //    (Note: You need to keep `prev_x` and `prev_y` in your struct for this to work!)
        for i in 0..logo_height {
            queue!(
                w,
                cursor::MoveTo(self.prev_x, self.prev_y + i as u16),
                style::Print(" ".repeat(logo_width as usize))
            )?;
        }

        // 2. Draw the NEW logo at the current `x` / `y`.
        for (i, line) in logo_lines.iter().enumerate() {
            queue!(
                w,
                cursor::MoveTo(self.x, self.y + i as u16),
                style::SetForegroundColor(self.color),
                style::Print(line),
                style::ResetColor
            )?;
        }

        // 3. Flush everything to the terminal at once.
        w.flush()?;
        Ok(())
    }

    /// Update cached terminal bounds and clamp the logo position.
    ///
    /// This should be called after handling a resize event so that the
    /// bouncer uses the new terminal size for collision detection.
    pub fn resize(&mut self, w: u16, h: u16) {
        self.max_x = w.saturating_sub(1);
        self.max_y = h.saturating_sub(1);
        self.x = self.x.min(self.max_x);
        self.y = self.y.min(self.max_y);
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
