use std::io::{self, Stdout};

use color_eyre::eyre::{Result, WrapErr};
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
    entered: bool,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend).wrap_err("create terminal backend")?;
        Ok(Self {
            terminal,
            entered: false,
        })
    }

    pub fn enter(&mut self) -> Result<()> {
        crate::diag::breadcrumb("enable_raw_mode…");
        enable_raw_mode().wrap_err(
            "enable raw mode (on Windows, use Windows Terminal; legacy conhost can fail)",
        )?;
        crate::diag::breadcrumb("enable_raw_mode ok");

        crate::diag::breadcrumb("EnterAlternateScreen…");
        io::stdout()
            .execute(EnterAlternateScreen)
            .wrap_err("enter alternate screen")?;
        crate::diag::breadcrumb("EnterAlternateScreen ok");

        // Optional features — must not abort startup on older Windows consoles.
        let _ = io::stdout().execute(EnableBracketedPaste);
        let _ = io::stdout().execute(DisableMouseCapture);

        self.terminal.clear().wrap_err("clear terminal")?;
        let _ = self.terminal.hide_cursor();
        self.entered = true;
        crate::diag::breadcrumb("tui entered");
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if !self.entered {
            return Ok(());
        }
        self.entered = false;
        let _ = self.terminal.show_cursor();
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        let _ = io::stdout().execute(DisableBracketedPaste);
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
