use std::{
    io::{self, Stdout},
    ops::{Deref, DerefMut},
};

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub struct TerminalInstance {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalInstance {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: setup_terminal().context("setup failed")?,
        })
    }
}

impl Deref for TerminalInstance {
    type Target = Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for TerminalInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for TerminalInstance {
    fn drop(&mut self) {
        if let Err(e) = restore_terminal(&mut self.terminal).context("restore terminal failed") {
            tracing::error!("failed to restore terminal: {}", e);
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
}

/// Restore the terminal. This is where you disable raw mode, leave the alternate screen, and show
/// the cursor.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}
