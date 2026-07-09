use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_input::Input;

use crate::app::App;
use crate::i18n::Language;
use crate::models::Focus;
use crate::ui::widgets::{focused_style, title_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let t = app.t();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(4),
    ])
    .split(area);

    draw_field(
        frame,
        chunks[0],
        t.settings_output,
        &app.settings_output_input,
        app.focus == Focus::SettingsOutput,
    );
    draw_field(
        frame,
        chunks[1],
        t.settings_template,
        &app.settings_template_input,
        app.focus == Focus::SettingsTemplate,
    );
    draw_language(frame, chunks[2], app);
    draw_auto_open(frame, chunks[3], app);

    let help = vec![
        Line::from(Span::styled(
            t.settings_tips,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(t.settings_tip_template),
        Line::from(t.settings_tip_defaults),
        Line::from(t.settings_tip_file),
        Line::from(t.settings_tip_language),
        Line::from(t.settings_tip_auto_open),
        Line::from(""),
        Line::from(Span::styled(
            t.settings_keys,
            Style::default().fg(Color::Cyan),
        )),
    ];

    frame.render_widget(
        Paragraph::new(help).block(title_block(t.settings_title)),
        chunks[4],
    );
}

fn draw_language(frame: &mut Frame, area: Rect, app: &App) {
    let t = app.t();
    let focused = app.focus == Focus::SettingsLanguage;
    let border = if focused { Color::Cyan } else { Color::Gray };
    let block = title_block(t.settings_language).border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut spans = vec![Span::styled(
        format!("{}:  ", t.settings_language),
        focused_style(focused),
    )];
    for (i, lang) in Language::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        let active = app.config.language == *lang;
        let text = format!(" {} ", lang.native_label());
        let style = if active {
            Style::default()
                .fg(Color::Black)
                .bg(if focused { Color::Cyan } else { Color::Green })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(text, style));
    }
    spans.push(Span::styled(
        "   (←/→)",
        Style::default().fg(Color::DarkGray),
    ));

    frame.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn draw_auto_open(frame: &mut Frame, area: Rect, app: &App) {
    let t = app.t();
    let focused = app.focus == Focus::SettingsAutoOpen;
    let border = if focused { Color::Cyan } else { Color::Gray };
    let block = title_block(t.settings_auto_open).border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let on = app.config.auto_open;
    let spans = vec![
        Span::styled(format!("{}:  ", t.settings_auto_open), focused_style(focused)),
        pill(t.settings_on, on, focused),
        Span::raw("  "),
        pill(t.settings_off, !on, focused),
        Span::styled("   (←/→ / Enter)", Style::default().fg(Color::DarkGray)),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn pill(label: &str, active: bool, section_focused: bool) -> Span<'static> {
    let text = format!(" {label} ");
    let style = if active {
        Style::default()
            .fg(Color::Black)
            .bg(if section_focused {
                Color::Cyan
            } else {
                Color::Green
            })
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Span::styled(text, style)
}

fn draw_field(frame: &mut Frame, area: Rect, title: &str, input: &Input, focused: bool) {
    let border = if focused { Color::Cyan } else { Color::Gray };
    let block = title_block(title).border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let scroll = {
        let cursor = input.visual_cursor() as u16;
        cursor.saturating_sub(inner.width.saturating_sub(1))
    };

    frame.render_widget(
        Paragraph::new(input.value())
            .style(Style::default().fg(Color::White))
            .scroll((0, scroll)),
        inner,
    );

    if focused {
        let cursor_x = input.visual_cursor().saturating_sub(scroll as usize) as u16;
        frame.set_cursor_position((
            inner.x + cursor_x.min(inner.width.saturating_sub(1)),
            inner.y,
        ));
    }
}
