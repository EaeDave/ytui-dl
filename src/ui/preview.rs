use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::models::{MediaMode, QualityPreset};
use crate::ui::widgets::title_block;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let Some(info) = &app.preview else {
        frame.render_widget(
            Paragraph::new("Nenhum vídeo carregado. Volte ao início e busque uma URL.")
                .block(title_block("Preview")),
            area,
        );
        return;
    };

    let chunks = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Min(2),
    ])
    .split(area);

    let meta_lines = vec![
        Line::from(vec![
            Span::styled("Título:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                info.title.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Canal:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(info.uploader.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Duração: ", Style::default().fg(Color::DarkGray)),
            Span::raw(info.duration_label()),
        ]),
        Line::from(vec![
            Span::styled("ID:      ", Style::default().fg(Color::DarkGray)),
            Span::raw(info.id.clone()),
        ]),
        Line::from(vec![
            Span::styled("URL:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(info.webpage_url.clone(), Style::default().fg(Color::Blue)),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(meta_lines)
            .wrap(Wrap { trim: true })
            .block(title_block("Informações do vídeo")),
        chunks[0],
    );

    let mode_label = app.mode.label();
    let quality_label = match app.mode {
        MediaMode::Video => app.quality.label().to_string(),
        MediaMode::Audio => app.audio_format.label().to_string(),
    };

    let opts = vec![
        Line::from(vec![
            Span::styled("Modo:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                mode_label,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (m ou v/a)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                if app.mode == MediaMode::Video {
                    "Qualidade: "
                } else {
                    "Formato:   "
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                quality_label,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (1-5 / Tab)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Pasta:     ", Style::default().fg(Color::DarkGray)),
            Span::raw(app.config.output_dir.display().to_string()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter  →  adicionar à fila e baixar",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    frame.render_widget(
        Paragraph::new(opts).block(title_block("Download")),
        chunks[1],
    );

    let presets: String = QualityPreset::ALL
        .iter()
        .enumerate()
        .map(|(i, q)| format!("{}={}", i + 1, q.label()))
        .collect::<Vec<_>>()
        .join("  ");

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Atalhos de qualidade",
                Style::default().fg(Color::Magenta),
            )),
            Line::from(presets),
        ])
        .block(title_block("Referência")),
        chunks[2],
    );
}
