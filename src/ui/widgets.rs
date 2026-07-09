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
    let screen = match app.screen {
        Screen::Home => "Início",
        Screen::Preview => "Preview",
        Screen::Queue => "Fila",
        Screen::Settings => "Config",
        Screen::Help => "Ajuda",
    };

    let active = app
        .jobs
        .iter()
        .filter(|j| j.status.is_active())
        .count();

    let line = Line::from(vec![
        Span::styled(
            " ▶ ytui-dl ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(screen, Style::default().fg(Color::Cyan)),
        Span::raw("  │  "),
        Span::styled(
            format!("fila: {active}"),
            Style::default().fg(if active > 0 {
                Color::Yellow
            } else {
                Color::DarkGray
            }),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

pub fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let hints = match app.screen {
        Screen::Home => "Tab foco  Enter buscar  v/a modo  1-5 qualidade  f fila  s config  ? ajuda  q sair",
        Screen::Preview => "Enter baixar  m modo  1-5 qualidade  Esc voltar  f fila  ? ajuda",
        Screen::Queue => "j/k navegar  p cancelar  c limpar finalizados  o pasta  Esc voltar",
        Screen::Settings => "Tab campo  Enter salvar  Esc cancelar",
        Screen::Help => "Esc / ? / q fechar",
    };

    let msg = if app.fetching {
        format!("{} {}  │  {}", app.spinner_frame(), app.status_message, hints)
    } else {
        format!(" {}  │  {}", app.status_message, hints)
    };

    let style = if app.status_message.to_ascii_lowercase().contains("erro")
        || app.status_message.to_ascii_lowercase().contains("falh")
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
