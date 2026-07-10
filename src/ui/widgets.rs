use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::models::Screen;

pub fn focused_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    }
}

pub fn title_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ))
}

pub fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let t = app.t();
    let screen = match app.screen {
        Screen::Home => t.screen_home,
        Screen::Preview => t.screen_preview,
        Screen::Queue => t.screen_queue,
        Screen::Settings => t.screen_settings,
        Screen::Help => t.screen_help,
    };

    let active = app.jobs.iter().filter(|j| j.status.is_active()).count();

    let mut spans = vec![
        Span::styled(
            " ▶ ytd ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(screen, Style::default().fg(Color::Cyan)),
        Span::raw("  │  "),
        Span::styled(
            format!("{}: {active}", t.queue_count),
            Style::default().fg(if active > 0 {
                Color::Yellow
            } else {
                Color::DarkGray
            }),
        ),
    ];

    if let Some(ver) = &app.update_available {
        spans.push(Span::raw("  │  "));
        spans.push(Span::styled(
            format!("↑ {} v{ver}  (u)", t.update_badge),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let t = app.t();
    let hints = match app.screen {
        Screen::Home => t.hint_home,
        Screen::Preview => t.hint_preview,
        Screen::Queue => t.hint_queue,
        Screen::Settings => t.hint_settings,
        Screen::Help => t.hint_help,
    };

    let msg = if app.fetching {
        format!("{} {}  │  {}", app.spinner_frame(), app.status_message, hints)
    } else {
        format!(" {}  │  {}", app.status_message, hints)
    };

    let lower = app.status_message.to_ascii_lowercase();
    let style = if lower.contains("error")
        || lower.contains("erro")
        || lower.contains("fail")
        || lower.contains("falh")
    {
        Style::default().fg(Color::Red)
    } else if app.fetching {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    frame.render_widget(
        Paragraph::new(msg).style(style).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        area,
    );
}

pub fn main_vertical_chunks(area: Rect) -> std::rc::Rc<[Rect]> {
    ratatui::layout::Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(2),
    ])
    .split(area)
}
