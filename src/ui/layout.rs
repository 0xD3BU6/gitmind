use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};

pub fn vertical_chunks(area: Rect, heights: &[Constraint]) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(heights)
        .split(area)
        .to_vec()
}

pub fn horizontal_chunks(area: Rect, widths: &[Constraint]) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(widths)
        .split(area)
        .to_vec()
}

/// A rectangle of the given size centred inside `area` (clamped to fit).
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [v] = Layout::vertical([Constraint::Length(height.min(area.height))])
        .flex(Flex::Center)
        .areas(area);
    let [h] = Layout::horizontal([Constraint::Length(width.min(area.width))])
        .flex(Flex::Center)
        .areas(v);
    h
}
