use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

pub const ACCENT: Color = Color::Rgb(255, 140, 0); // github-ish orange
pub const PRIMARY: Color = Color::Rgb(88, 166, 255); // github blue
pub const GREEN: Color = Color::Rgb(63, 185, 80);
pub const RED: Color = Color::Rgb(248, 81, 73);
pub const YELLOW: Color = Color::Rgb(210, 153, 34);
pub const PURPLE: Color = Color::Rgb(188, 140, 255);
pub const DIM: Color = Color::Rgb(110, 118, 129);
pub const BG_SEL: Color = Color::Rgb(33, 38, 45);

pub fn title() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::default().fg(DIM)
}

pub fn key() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default()
        .bg(BG_SEL)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn panel(name: &str, focused: bool) -> Block<'_> {
    let border = if focused { ACCENT } else { DIM };
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(format!(" {name} "))
        .title_style(title())
}
