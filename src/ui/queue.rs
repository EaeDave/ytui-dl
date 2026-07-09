use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::models::JobStatus;
use crate::ui::widgets::title_block;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(4)]).split(area);

    if app.jobs.is_empty() {
        frame.render_widget(
            Paragraph::new("Fila vazia. Busque um vídeo na tela inicial e confirme o download.")
                .style(Style::default().fg(Color::DarkGray))
                .block(title_block("Fila de downloads")),
            chunks[0],
        );
    } else {
        let items: Vec<ListItem> = app
            .jobs
            .iter()
            .enumerate()
            .map(|(i, job)| {
                let selected = i == app.queue_selected;
                let status_color = match job.status {
                    JobStatus::Done => Color::Green,
                    JobStatus::Failed | JobStatus::Cancelled => Color::Red,
                    JobStatus::Downloading => Color::Yellow,
                    JobStatus::Queued => Color::Cyan,
                };

                let marker = if selected { "▸ " } else { "  " };
                let title = truncate(job.display_title(), 48);
                let mode = job.mode.label();
                let line = Line::from(vec![
                    Span::raw(marker),
                    Span::styled(
                        format!("[{}] ", job.status.label()),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(
                        title,
                        if selected {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ),
                    Span::styled(
                        format!("  ({mode})"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect();

        frame.render_widget(
            List::new(items).block(title_block("Fila de downloads")),
            chunks[0],
        );
    }

    // Detail / progress for selected job
    if let Some(job) = app.jobs.get(app.queue_selected) {
        let ratio = (job.progress / 100.0).clamp(0.0, 1.0);
        let label = match job.status {
            JobStatus::Downloading => {
                let speed = job.speed.as_deref().unwrap_or("—");
                let eta = job.eta.as_deref().unwrap_or("—");
                format!(
                    "{:.1}%  ·  {speed}  ·  ETA {eta}",
                    job.progress
                )
            }
            JobStatus::Done => "Concluído".into(),
            JobStatus::Failed => job
                .error
                .clone()
                .unwrap_or_else(|| "Falhou".into()),
            JobStatus::Cancelled => "Cancelado".into(),
            other => other.label().into(),
        };

        let gauge_color = match job.status {
            JobStatus::Done => Color::Green,
            JobStatus::Failed | JobStatus::Cancelled => Color::Red,
            _ => Color::Cyan,
        };

        let gauge = Gauge::default()
            .block(title_block("Progresso do item selecionado"))
            .gauge_style(
                Style::default()
                    .fg(gauge_color)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(ratio)
            .label(label);

        frame.render_widget(gauge, chunks[1]);
    } else {
        frame.render_widget(
            Paragraph::new("").block(title_block("Progresso")),
            chunks[1],
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}…")
    }
}
