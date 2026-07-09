use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::ui::widgets::title_block;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if !app.update_confirm && !app.update_busy && !app.update_ready_restart {
        return;
    }

    let t = app.t();
    let popup = centered_rect(64, 40, area);
    frame.render_widget(Clear, popup);

    let (title, lines) = if app.update_busy {
        (
            t.update_modal_working,
            vec![
                Line::from(Span::styled(
                    app.status_message.clone(),
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    t.update_modal_working_hint,
                    Style::default().fg(Color::DarkGray),
                )),
            ],
        )
    } else if app.update_ready_restart {
        let ver = app
            .status_message
            .clone();
        (
            t.update_modal_done,
            vec![
                Line::from(Span::styled(ver, Style::default().fg(Color::Green))),
                Line::from(""),
                Line::from(Span::styled(
                    t.update_modal_restart_hint,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    t.update_modal_quit_hint,
                    Style::default().fg(Color::DarkGray),
                )),
            ],
        )
    } else {
        // confirm
        let ver = app
            .update_available
            .as_deref()
            .unwrap_or("?");
        (
            t.update_modal_title,
            vec![
                Line::from(app.lang().msg_update_confirm_body(ver)),
                Line::from(""),
                Line::from(Span::styled(
                    t.update_modal_confirm_keys,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    t.update_modal_note,
                    Style::default().fg(Color::DarkGray),
                )),
            ],
        )
    };

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(
                title_block(title).border_style(Style::default().fg(Color::Yellow)).borders(Borders::ALL),
            ),
        popup,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ratatui::layout::Constraint::Percentage(percent_y),
        ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    ratatui::layout::Layout::horizontal([
        ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ratatui::layout::Constraint::Percentage(percent_x),
        ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vert[1])[1]
}
