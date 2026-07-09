mod help;
mod home;
mod preview;
mod queue;
mod settings;
mod update_modal;
pub mod widgets;

use ratatui::Frame;

use crate::app::App;
use crate::models::Screen;
use crate::ui::widgets::{draw_header, draw_status_bar, main_vertical_chunks};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = main_vertical_chunks(area);

    draw_header(frame, chunks[0], app);

    match app.screen {
        Screen::Home => home::draw(frame, chunks[1], app),
        Screen::Preview => preview::draw(frame, chunks[1], app),
        Screen::Queue => queue::draw(frame, chunks[1], app),
        Screen::Settings => settings::draw(frame, chunks[1], app),
        Screen::Help => {
            // Draw previous context-ish empty home under modal
            home::draw(frame, chunks[1], app);
            help::draw(frame, area, app);
        }
    }

    if app.screen != Screen::Help {
        draw_status_bar(frame, chunks[2], app);
    }

    // Update confirm / progress / restart overlays (Linux self-update).
    update_modal::draw(frame, area, app);
}
