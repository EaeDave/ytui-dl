use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use ratatui::Frame;
use tui_input::Input;

use crate::app::App;
use crate::models::{Focus, QualityPreset};
use crate::ui::widgets::{focused_style, title_block};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(7),
        Constraint::Min(3),
    ])
    .split(area);

    draw_url(frame, chunks[0], app);
    draw_options(frame, chunks[1], app);
    draw_hint_panel(frame, chunks[2], app);
}

fn draw_url(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::UrlInput;
    let border = if focused { Color::Cyan } else { Color::Gray };
    let block = title_block("URL do YouTube").border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let scroll = input_scroll(&app.url_input, inner.width);
    let text = Paragraph::new(app.url_input.value())
        .style(Style::default().fg(Color::White))
        .scroll((0, scroll));
    frame.render_widget(text, inner);

    if focused {
        let cursor_x = app
            .url_input
            .visual_cursor()
            .saturating_sub(scroll as usize) as u16;
        frame.set_cursor_position((
            inner.x + cursor_x.min(inner.width.saturating_sub(1)),
            inner.y,
        ));
    }
}

fn draw_options(frame: &mut Frame, area: Rect, app: &App) {
    let block = title_block("Opções");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    let mode_focus = app.focus == Focus::Mode;
    let quality_focus = app.focus == Focus::Quality;
    let audio_focus = app.focus == Focus::AudioFormat;
    let confirm_focus = app.focus == Focus::Confirm;

    let mode_line = Line::from(vec![
        Span::styled("Modo:      ", focused_style(mode_focus)),
        pill("Vídeo", app.mode == crate::models::MediaMode::Video, mode_focus),
        Span::raw("  "),
        pill("Áudio", app.mode == crate::models::MediaMode::Audio, mode_focus),
        Span::styled("   (v/a ou ←/→)", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(mode_line), rows[0]);

    let mut quality_spans = vec![Span::styled(
        "Qualidade: ",
        focused_style(quality_focus),
    )];
    for (i, q) in QualityPreset::ALL.iter().enumerate() {
        if i > 0 {
            quality_spans.push(Span::raw(" "));
        }
        quality_spans.push(pill(q.label(), app.quality == *q, quality_focus));
    }
    quality_spans.push(Span::styled(
        "   (1-5)",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(Line::from(quality_spans)), rows[1]);

    let mut audio_spans = vec![Span::styled(
        "Áudio:     ",
        focused_style(audio_focus),
    )];
    for (i, f) in crate::models::AudioFormat::ALL.iter().enumerate() {
        if i > 0 {
            audio_spans.push(Span::raw(" "));
        }
        audio_spans.push(pill(f.label(), app.audio_format == *f, audio_focus));
    }
    frame.render_widget(Paragraph::new(Line::from(audio_spans)), rows[2]);

    let out = format!("Saída:     {}", app.config.output_dir.display());
    let confirm = if confirm_focus {
        Span::styled(
            "  ▶  Enter para buscar / baixar",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            "  Enter para buscar informações do vídeo",
            Style::default().fg(Color::DarkGray),
        )
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(out, Style::default().fg(Color::Gray)),
            confirm,
        ])),
        rows[3],
    );
}

fn draw_hint_panel(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![
        Line::from(Span::styled(
            "Como usar",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("1. Cole a URL (Ctrl+Shift+V / paste do terminal)"),
        Line::from("2. Escolha Vídeo ou Áudio e a qualidade"),
        Line::from("3. Enter → preview → Enter de novo para baixar"),
        Line::from(""),
    ];

    if let Some(warn) = &app.tools_warning {
        lines.push(Line::from(Span::styled(
            format!("⚠ {warn}"),
            Style::default().fg(Color::Yellow),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "✓ yt-dlp detectado",
            Style::default().fg(Color::Green),
        )));
        if app.tools.as_ref().is_some_and(|t| t.has_ffmpeg()) {
            lines.push(Line::from(Span::styled(
                "✓ ffmpeg detectado",
                Style::default().fg(Color::Green),
            )));
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(title_block("Guia rápido")),
        area,
    );
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
        Style::default().fg(Color::DarkGray).bg(Color::Black)
    };
    Span::styled(text, style)
}

fn input_scroll(input: &Input, width: u16) -> u16 {
    let cursor = input.visual_cursor() as u16;
    cursor.saturating_sub(width.saturating_sub(1))
}

/// Clear helper kept for modal overlays.
#[allow(dead_code)]
pub fn clear(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
}
