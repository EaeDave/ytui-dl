use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::models::{MediaMode, OutputProfile, QualityPreset};
use crate::ui::widgets::title_block;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let t = app.t();
    let Some(info) = &app.preview else {
        frame.render_widget(
            Paragraph::new(t.preview_empty).block(title_block(t.screen_preview)),
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
            Span::styled(t.field_title, Style::default().fg(Color::DarkGray)),
            Span::styled(
                info.title.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(t.field_channel, Style::default().fg(Color::DarkGray)),
            Span::styled(info.uploader.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(t.field_duration, Style::default().fg(Color::DarkGray)),
            Span::raw(info.duration_label()),
        ]),
        Line::from(vec![
            Span::styled(t.field_id, Style::default().fg(Color::DarkGray)),
            Span::raw(info.id.clone()),
        ]),
        Line::from(vec![
            Span::styled(t.field_url, Style::default().fg(Color::DarkGray)),
            Span::styled(info.webpage_url.clone(), Style::default().fg(Color::Blue)),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(meta_lines)
            .wrap(Wrap { trim: true })
            .block(title_block(t.video_info_title)),
        chunks[0],
    );

    let mode_label = app.mode.label(t);
    let profile_label = app.profile.label(t);
    let quality_label = match app.mode {
        MediaMode::Video => app.quality.label(t).to_string(),
        MediaMode::Audio => {
            if app.profile == OutputProfile::WhatsApp {
                "M4A".to_string()
            } else {
                app.audio_format.label(t).to_string()
            }
        }
    };

    let quality_field = match app.mode {
        MediaMode::Video => t.field_quality,
        MediaMode::Audio => t.field_format,
    };

    let opts = vec![
        Line::from(vec![
            Span::styled(t.field_mode, Style::default().fg(Color::DarkGray)),
            Span::styled(
                mode_label,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (m / v/a)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(t.field_profile, Style::default().fg(Color::DarkGray)),
            Span::styled(
                profile_label,
                Style::default()
                    .fg(if app.profile == OutputProfile::WhatsApp {
                        Color::Cyan
                    } else {
                        Color::Green
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (w / b / p)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(quality_field, Style::default().fg(Color::DarkGray)),
            Span::styled(
                quality_label,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (1-5 / Tab)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(t.field_folder, Style::default().fg(Color::DarkGray)),
            Span::raw(app.config.output_dir.display().to_string()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            t.enter_download,
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    frame.render_widget(
        Paragraph::new(opts).block(title_block(t.download_block)),
        chunks[1],
    );

    let presets: String = QualityPreset::ALL
        .iter()
        .enumerate()
        .map(|(i, q)| format!("{}={}", i + 1, q.label(t)))
        .collect::<Vec<_>>()
        .join("  ");

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                t.quality_shortcuts,
                Style::default().fg(Color::Magenta),
            )),
            Line::from(presets),
        ])
        .block(title_block(t.reference_title)),
        chunks[2],
    );
}
