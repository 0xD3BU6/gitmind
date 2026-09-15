use crate::app::App;
use crate::ui::{layout, logo, theme};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Position},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let word_h = logo::height(logo::WORDMARK);
    let width = area.width.clamp(20, 80);
    let region = layout::centered(area, width, word_h + 8);
    let chunks = layout::vertical_chunks(
        region,
        &[
            Constraint::Length(word_h),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
        ],
    );

    let word: Vec<Line> = logo::lines(logo::WORDMARK)
        .into_iter()
        .map(|l| Line::styled(l, theme::title()))
        .collect();
    f.render_widget(Paragraph::new(word).alignment(Alignment::Center), chunks[0]);

    let input_area = chunks[2];
    let input = Paragraph::new(app.input.as_str())
        .style(ratatui::style::Style::default().fg(theme::PRIMARY))
        .block(theme::panel("Repository path", true));
    f.render_widget(input, input_area);

    let x = input_area.x + 1 + app.input.chars().count() as u16;
    f.set_cursor_position(Position::new(
        x.min(input_area.right().saturating_sub(2)),
        input_area.y + 1,
    ));

    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Enter", theme::key()),
            Span::styled(" open   ", theme::dim()),
            Span::styled("Ctrl-U", theme::key()),
            Span::styled(" clear   ", theme::dim()),
            Span::styled("Ctrl-S", theme::key()),
            Span::styled(" settings   ", theme::dim()),
            Span::styled("Esc", theme::key()),
            Span::styled(" back   ", theme::dim()),
            Span::styled("Ctrl-C", theme::key()),
            Span::styled(" quit", theme::dim()),
        ]),
        Line::styled("tip: '~' expands to your home directory; any subfolder of a repo works", theme::dim()),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[4]);
}
