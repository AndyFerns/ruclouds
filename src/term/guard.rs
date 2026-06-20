use anyhow::Result;
use crossterm::{
    cursor,
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

/// RAII guard that sets up and tears down the terminal state.
///
/// On construction: enable raw mode → enter alternate screen → hide cursor.
/// On drop:         disable raw mode → leave alternate screen → show cursor → flush.
pub struct TermGuard;

impl TermGuard {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        Ok(TermGuard)
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        teardown();
    }
}

/// Perform terminal teardown unconditionally (safe to call multiple times).
///
/// Order: disable raw mode → leave alternate screen → show cursor → flush stdout.
pub fn teardown() {
    let _ = terminal::disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, cursor::Show);
    let _ = stdout.flush();
}
