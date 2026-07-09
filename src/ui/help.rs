use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::ui::widgets::title_block;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let t = app.t();
    let popup = centered_rect(70, 80, area);
    frame.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            t.help_shortcuts,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        section(t.help_nav),
        key("h", t.help_home),
        key("f", t.help_queue),
        key("s", t.help_settings),
        key("?", t.help_this),
        key("Esc", t.help_esc),
        key("q", t.help_quit),
        Line::from(""),
        section(t.help_download),
        key("Enter", t.help_enter),
        key("v / a", t.help_va),
        key("m", t.help_m),
        key("1-5", t.help_quality),
        key("p", t.help_cancel),
        key("o", t.help_open),
        Line::from(""),
        section(t.help_queue_section),
        key("j / k", t.help_jk),
        key("c", t.help_clear),
        Line::from(""),
        Line::from(Span::styled(
            t.help_footer,
            Style::default().fg(Color::DarkGray),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(title_block(t.help_title)),
        popup,
    );
}

fn section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn key(k: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {k:<10}"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(desc.to_string()),
    ])
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
