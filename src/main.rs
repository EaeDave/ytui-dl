mod action;
mod app;
mod config;
mod downloader;
mod error;
mod event;
mod models;
mod tui;
mod ui;

use color_eyre::eyre::Result;
use tokio::sync::mpsc;

use crate::action::Action;
use crate::app::App;
use crate::config::Config;
use crate::event::EventHandler;
use crate::tui::Tui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::load();
    let mut app = App::new(config);
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    app.set_action_tx(action_tx.clone());

    let mut tui = Tui::new()?;
    tui.enter()?;

    // Event pump: keyboard / tick / render
    let _events = EventHandler::new(action_tx.clone(), 4.0, 30.0);

    // Initial render
    tui.terminal().draw(|frame| ui::draw(frame, &app))?;

    loop {
        let Some(action) = action_rx.recv().await else {
            break;
        };

        app.update(action)?;

        if app.should_quit {
            break;
        }

        // Redraw after every action for snappy UX (event loop is already rate-limited by ticks)
        tui.terminal().draw(|frame| ui::draw(frame, &app))?;
    }

    tui.exit()?;
    Ok(())
}
