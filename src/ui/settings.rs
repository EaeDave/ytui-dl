use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_input::Input;

use crate::app::App;
use crate::models::Focus;
use crate::ui::widgets::title_block;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(4),
    ])
    .split(area);

    draw_field(
        frame,
        chunks[0],
        "Pasta de saída",
        &app.settings_output_input,
        app.focus == Focus::SettingsOutput,
    );
    draw_field(
        frame,
        chunks[1],
        "Template do nome do arquivo (yt-dlp)",
        &app.settings_template_input,
        app.focus == Focus::SettingsTemplate,
    );

    let help = vec![
        Line::from(Span::styled(
            "Dicas",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("• Template usa placeholders do yt-dlp: %(title)s %(id)s %(ext)s …"),
        Line::from("• Os defaults de modo/qualidade atuais também são salvos ao pressionar Enter"),
        Line::from("• Arquivo: ~/.config/ytui-dl/config.toml"),
        Line::from(""),
        Line::from(Span::styled(
            "Enter = salvar   Esc = cancelar   Tab = trocar campo",
            Style::default().fg(Color::Cyan),
        )),
    ];

    frame.render_widget(
        Paragraph::new(help).block(title_block("Configurações")),
        chunks[2],
    );
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
