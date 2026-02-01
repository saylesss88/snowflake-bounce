use crossterm::{
    cursor, queue,
    style::{self, Color},
    terminal,
};
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;
use std::io::{self, Write};

// --- RNG Helper (unchanged) ---
thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}

fn rng<T>() -> T
where
    Standard: Distribution<T>,
{
    RNG.with(|rng| (*rng).borrow_mut().r#gen::<T>())
}

// --- Symbol Enums (unchanged) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolMode {
    SnowflakeSmall,
    SnowflakeLarge,
    NixOS,
    Arch,
    MiddleFinger,
}

// --- Bouncer Struct ---
pub struct Bouncer {
    x: u16,
    y: u16,
    prev_x: u16, // Added back for clearing
    prev_y: u16, // Added back for clearing
    dx: i32,
    dy: i32,
    color: Color, // Changed from i16 to crossterm::style::Color
    max_x: u16,
    max_y: u16,
    pub mode: SymbolMode,
}

impl Bouncer {
    #[must_use]
    /// # Panics
    /// Panics if the internal `try_from` conversion fails, which should be impossible
    /// due to the modulo arithmetic ensuring values are within `u16` range.
    pub fn new() -> Self {
        // Use crossterm to get size, defaulting to 80x24 if it fails
        let (cols, lines) = terminal::size().unwrap_or((80, 24));

        let max_x = cols.saturating_sub(1);
        let max_y = lines.saturating_sub(1);

        // Random start position, safely cast to i32 for math, then back to u16
        // We use slightly smaller bounds to ensure we don't start off-screen
        let start_x_pos_i32 = rng::<i32>().abs() % (i32::from(max_x) - 50).max(5) + 2;
        let start_y_i32 = rng::<i32>().abs() % (i32::from(max_y) - 25).max(5) + 2;
        // let start_x_pos_i32 = rng::<i32>() % (i32::from(max_x) - 50).max(5) + 2;
        // let start_y_i32 = rng::<i32>() % (i32::from(max_y) - 25).max(5) + 2;

        let start_x = u16::try_from(start_x_pos_i32).unwrap();
        let start_y = u16::try_from(start_y_i32).unwrap();

        Self {
            x: start_x,
            y: start_y,
            prev_x: start_x,
            prev_y: start_y,
            dx: if rng::<bool>() { 1 } else { -1 },
            dy: if rng::<bool>() { 1 } else { -1 },
            color: Color::Blue,
            max_x,
            max_y,
            mode: SymbolMode::NixOS,
        }
    }

    #[allow(clippy::match_same_arms)]
    pub const fn cycle_symbol(&mut self) {
        self.mode = match self.mode {
            SymbolMode::SnowflakeSmall => SymbolMode::SnowflakeLarge,
            SymbolMode::SnowflakeLarge => SymbolMode::NixOS,
            SymbolMode::NixOS => SymbolMode::SnowflakeSmall,
            SymbolMode::MiddleFinger => SymbolMode::SnowflakeSmall,
            SymbolMode::Arch => SymbolMode::NixOS,
        };
    }

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

    // Internal helper to pick a random color (same logic as cycle_color)
    fn change_color(&mut self) {
        self.cycle_color();
    }

    pub const fn set_middle_finger(&mut self) {
        self.mode = SymbolMode::MiddleFinger;
    }
    pub const fn set_arch(&mut self) {
        self.mode = SymbolMode::Arch;
    }

    pub fn update(&mut self) {
        // Save old position for erasing
        self.prev_x = self.x;
        self.prev_y = self.y;

        // Calculate candidate new position as signed integers
        let mut nx = i32::from(self.x) + self.dx;
        let mut ny = i32::from(self.y) + self.dy;

        let (logo_width_i32, logo_h_i32) = self.get_logo_dimensions();

        // Bounce X
        if nx <= 0 {
            nx = 0;
            self.dx = -self.dx;
            self.change_color();
        } else if nx + logo_width_i32 >= i32::from(self.max_x) {
            nx = i32::from(self.max_x) - logo_width_i32;
            self.dx = -self.dx;
            self.change_color();
        }

        // Bounce Y
        if ny <= 0 {
            ny = 0;
            self.dy = -self.dy;
            self.change_color();
        } else if ny + logo_h_i32 >= i32::from(self.max_y) {
            ny = i32::from(self.max_y) - logo_h_i32;
            self.dy = -self.dy;
            self.change_color();
        }

        self.x = u16::try_from(nx).unwrap_or(u16::MAX);
        self.y = u16::try_from(ny).unwrap_or(u16::MAX);
    }

    /// Resizes the animation area.
    ///
    /// # Panics
    /// Panics if the calculated dimensions are too large for `u16` (unlikely in normal terminals).
    pub fn resize(&mut self, w: u16, h: u16) {
        self.max_x = w.saturating_sub(1);
        self.max_y = h.saturating_sub(1);

        let (logo_width, logo_h) = self.get_logo_dimensions();

        // Clamp CURRENT position if terminal shrank
        // Cast 'x' and 'max_x' UP to the larger type (assuming logo_width is usize or i32)
        if i32::from(self.x) + logo_width >= i32::from(self.max_x) {
            self.x =
                u16::try_from(i32::from(self.max_x).saturating_sub(logo_width).max(0)).unwrap();
        }
        if i32::from(self.y) + logo_h >= i32::from(self.max_y) {
            self.y = u16::try_from(i32::from(self.max_y).saturating_sub(logo_h).max(0)).unwrap();
        }

        // Clamp PREVIOUS position safely too
        if i32::from(self.prev_x) + logo_width >= i32::from(self.max_x) {
            self.prev_x =
                u16::try_from(i32::from(self.max_x).saturating_sub(logo_width).max(0)).unwrap();
        }
        if i32::from(self.prev_y) + logo_h >= i32::from(self.max_y) {
            self.prev_y =
                u16::try_from(i32::from(self.max_y).saturating_sub(logo_h).max(0)).unwrap();
        }
    }

    // Helper: Dimensions are i32 for easy math, but small enough to fit u16
    #[allow(clippy::match_same_arms)]
    const fn get_logo_dimensions(&self) -> (i32, i32) {
        match self.mode {
            SymbolMode::SnowflakeSmall => (1, 1),
            SymbolMode::SnowflakeLarge => (5, 3),
            SymbolMode::NixOS => (46, 19),
            SymbolMode::MiddleFinger => (2, 1),
            SymbolMode::Arch => (46, 19),
        }
    }

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
            SymbolMode::Arch => vec![
                "                      ▄                       ",
                "                     ▟█▙                      ",
                "                    ▟███▙                     ",
                "                   ▟█████▙                    ",
                "                  ▟███████▙                   ",
                "                 ▂▔▀▜██████▙                  ",
                "                ▟██▅▂▝▜█████▙                 ",
                "               ▟█████████████▙                ",
                "              ▟███████████████▙               ",
                "             ▟█████████████████▙              ",
                "            ▟███████████████████▙             ",
                "           ▟█████████▛▀▀▜████████▙            ",
                "          ▟████████▛      ▜███████▙           ",
                "         ▟█████████        ████████▙          ",
                "        ▟██████████        █████▆▅▄▃▂         ",
                "       ▟██████████▛        ▜█████████▙        ",
                "      ▟██████▀▀▀              ▀▀██████▙       ",
                "     ▟███▀▘                       ▝▀███▙      ",
                "    ▟▛▀                               ▀▜▙     ",
            ],
        }
    }

    /// Draws the current state to the writer.
    ///
    /// # Errors
    /// Returns an error if writing to the output fails.
    ///
    /// # Panics
    /// Panics if the internal logic for logo dimensions fails (should be impossible).
    pub fn draw(&self, w: &mut impl Write) -> io::Result<()> {
        let (logo_width_i32, logo_height_i32) = self.get_logo_dimensions();
        let logo_lines = self.get_logo_lines();

        let logo_width = u16::try_from(logo_width_i32).unwrap();
        let logo_height = u16::try_from(logo_height_i32).unwrap();

        // 1. Erase old position safely
        let erase_str = " ".repeat(logo_width as usize);
        for i in 0..logo_height {
            // Clamp to prevent crossterm internal overflow (it does y+1 internally)
            if let Some(draw_y) = self.prev_y.checked_add(i) {
                // CRITICAL: Ensure we're within terminal bounds AND below u16::MAX - 1
                // (crossterm adds 1 internally for 1-indexed terminals)
                if draw_y < self.max_y.min(65534) {
                    queue!(
                        w,
                        cursor::MoveTo(self.prev_x.min(self.max_x.min(65534)), draw_y),
                        style::Print(&erase_str)
                    )?;
                }
            }
        }

        // 2. Draw new position safely
        for (i, line) in logo_lines.iter().enumerate() {
            if let Some(draw_y) = self.y.checked_add(u16::try_from(i).unwrap()) {
                // CRITICAL: Same bounds check
                if draw_y < self.max_y.min(65534) {
                    queue!(
                        w,
                        cursor::MoveTo(self.x.min(self.max_x.min(65534)), draw_y),
                        style::SetForegroundColor(self.color),
                        style::Print(line),
                        style::ResetColor
                    )?;
                }
            }
        }

        w.flush()?;
        Ok(())
    }
}

// Implement Default manually since Bouncer::new is not const/simple
impl Default for Bouncer {
    fn default() -> Self {
        Self::new()
    }
}
