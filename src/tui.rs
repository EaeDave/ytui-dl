use std::io::{self, Stdout};

use color_eyre::eyre::Result;
use crossterm::event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    terminal: TuiTerminal,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        io::stdout()
            .execute(EnterAlternateScreen)?
            .execute(EnableBracketedPaste)?
            .execute(DisableMouseCapture)?;
        self.terminal.clear()?;
        self.terminal.hide_cursor()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        io::stdout()
            .execute(LeaveAlternateScreen)?
            .execute(DisableBracketedPaste)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn terminal(&mut self) -> &mut TuiTerminal {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.exit();
    }
}
