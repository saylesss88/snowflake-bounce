use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::stdout;
use std::time::Duration;

use snowflake_bounce::Bouncer;

/// A terminal-based screensaver with bouncing snowflakes & other symbols
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Future: Add other options here like --color, --speed, etc.
}

fn main() -> std::io::Result<()> {
    // Parse CLI args (this handles --version automatically)
    let _args = Args::parse();

    // 1. SETUP
    // Enable raw mode to read keys byte-by-byte instantly
    enable_raw_mode()?;

    let mut stdout = stdout();

    // Switch to alternate screen (like vim/htop do) and hide cursor
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // 2. STATE
    let mut bouncer = Bouncer::new();
    let mut running = true;

    // 3. GAME LOOP
    while running {
        // DRAW: Render the current frame
        bouncer.draw(&mut stdout)?;

        // POLL: Wait up to 50ms for an event.
        if event::poll(Duration::from_millis(50))? {
            // Read the event ONCE
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    KeyCode::Char('c') => bouncer.cycle_color(),
                    KeyCode::Char('s') => bouncer.cycle_symbol(),
                    KeyCode::Char('f') => bouncer.set_middle_finger(),
                    KeyCode::Char('a') => bouncer.set_arch(),
                    _ => {}
                },
                Event::Resize(w, h) => {
                    bouncer.resize(w, h);
                    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                }
                _ => {}
            }
        }

        // UPDATE: Advance animation physics
        bouncer.update();
    }

    // 4. CLEANUP
    // Always restore terminal state before exiting!
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
