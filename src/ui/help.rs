use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::ui::widgets::title_block;

pub fn draw(frame: &mut Frame, area: Rect) {
    // Centered modal
    let popup = centered_rect(70, 80, area);
    frame.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            "Atalhos",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        section("Navegação"),
        key("h", "Tela inicial"),
        key("f", "Fila de downloads"),
        key("s", "Configurações"),
        key("?", "Esta ajuda"),
        key("Esc", "Voltar / fechar"),
        key("q", "Sair (Q força saída com download ativo)"),
        Line::from(""),
        section("Download"),
        key("Enter", "Buscar metadata / confirmar download"),
        key("v / a", "Modo vídeo / áudio"),
        key("m", "Alternar modo (preview)"),
        key("1-5", "Presets de qualidade (Melhor…Pior)"),
        key("p", "Cancelar download ativo"),
        key("o", "Abrir pasta de saída"),
        Line::from(""),
        section("Fila"),
        key("j / k", "Navegar itens"),
        key("c", "Limpar itens finalizados"),
        Line::from(""),
        Line::from(Span::styled(
            "ytui  ·  powered by yt-dlp  ·  Ratatui",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(title_block("Ajuda")),
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
