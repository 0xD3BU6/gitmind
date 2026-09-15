//! Animated splash screen: a blinking, swaying Octocat with a typewriter
//! speech bubble and a colour-swept wordmark.

use crate::app::App;
use crate::ui::{logo, theme};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

// ---------------------------------------------------------------- helpers --

/// Write a single character into the buffer if it is inside `area`.
fn put(buf: &mut Buffer, area: Rect, x: i32, y: i32, ch: char, style: Style) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as u16, y as u16);
    if x < area.left() || x >= area.right() || y < area.top() || y >= area.bottom() {
        return;
    }
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_char(ch);
        cell.set_style(style);
    }
}

/// Draw multi-line art at (x, y) leaving spaces transparent.
fn blit(buf: &mut Buffer, area: Rect, x: i32, y: i32, lines: &[String], style_of: impl Fn(char, usize, usize) -> Style) {
    for (row, line) in lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch != ' ' {
                put(buf, area, x + col as i32, y + row as i32, ch, style_of(ch, col, row));
            }
        }
    }
}

// ----------------------------------------------------------------- octocat --

const QUOTES: [&str; 6] = [
    "Mind your git workflow.",
    "Stage. Review. Commit.",
    "Press any key to begin!",
    "git happens.",
    "Ship it with confidence.",
    "Branches are cheap.",
];

/// Rebuild the octocat for this frame: blink, sway and typewriter bubble.
fn octocat_frame(frame: u64) -> Vec<String> {
    let mut lines = logo::lines(logo::OCTOCAT);

    // --- blink: closed eyes for 3 frames every ~5s
    if frame % 48 < 3 {
        for l in lines.iter_mut() {
            *l = l.replace("00", "--");
        }
    }

    // --- typewriter bubble (row 4 holds the quote)
    let quote_idx = ((frame / 60) % QUOTES.len() as u64) as usize;
    let quote = QUOTES[quote_idx];
    let typed = ((frame % 60) * 2).min(quote.len() as u64) as usize;
    let shown = &quote[..typed];
    let cursor = if (frame / 3).is_multiple_of(2) && typed < quote.len() { "▌" } else { " " };
    let inner = format!("  {shown}{cursor}");
    let inner_w = 28;
    let body = format!("{inner:<inner_w$}");
    if let Some(l) = lines.get_mut(4) {
        let bubble_col = l.find('|').unwrap_or(0);
        let prefix: String = l.chars().take(bubble_col).collect();
        *l = format!("{prefix}|{body}|");
    }

    // --- sway tentacles (bottom rows) sideways
    const SWAY: [i32; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
    let dx = SWAY[((frame / 2) % 8) as usize];
    let n = lines.len();
    for (i, l) in lines.iter_mut().enumerate() {
        let from_bottom = n - i;
        let shift = if from_bottom <= 5 { dx } else if from_bottom <= 9 { dx / 2 } else { 0 };
        if shift > 0 {
            *l = format!("{}{}", " ".repeat(shift as usize), l);
        } else if shift < 0 {
            let s: String = l.chars().skip((-shift) as usize).collect();
            *l = s;
        }
    }
    lines
}

fn octocat_style(frame: u64) -> impl Fn(char, usize, usize) -> Style {
    let pulse = (frame / 4).is_multiple_of(2);
    move |ch, _col, _row| match ch {
        'M' => Style::default().fg(Color::Rgb(230, 237, 243)),
        '0' => Style::default()
            .fg(if pulse { theme::ACCENT } else { theme::YELLOW })
            .add_modifier(Modifier::BOLD),
        '~' | '=' => Style::default().fg(theme::PRIMARY),
        '▌' => Style::default().fg(theme::ACCENT),
        ':' | ';' | '.' | '+' | '-' | '^' => theme::dim(),
        '|' | '/' | '_' => theme::dim(),
        _ => Style::default().fg(theme::GREEN),
    }
}

// ---------------------------------------------------------------- wordmark --

const GRADIENT: [Color; 8] = [
    Color::Rgb(255, 140, 0),
    Color::Rgb(255, 170, 40),
    Color::Rgb(255, 200, 80),
    Color::Rgb(255, 230, 120),
    Color::Rgb(255, 200, 80),
    Color::Rgb(255, 170, 40),
    Color::Rgb(255, 140, 0),
    Color::Rgb(220, 110, 0),
];

fn wordmark_style(frame: u64, width: usize) -> impl Fn(char, usize, usize) -> Style {
    move |_, col, _| {
        let pos = (col as u64 + frame * 2) % (width as u64 + 8);
        let idx = (pos / 3 % GRADIENT.len() as u64) as usize;
        Style::default().fg(GRADIENT[idx]).add_modifier(Modifier::BOLD)
    }
}

// -------------------------------------------------------------------- draw --

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let frame = app.frame;

    let cat = octocat_frame(frame);
    let cat_w = i32::from(logo::width(logo::OCTOCAT)) + 2;
    let cat_h = cat.len() as i32;
    let word = logo::lines(logo::WORDMARK);
    let word_w = i32::from(logo::width(logo::WORDMARK));
    let word_h = word.len() as i32;

    let show_cat = i32::from(area.height) >= cat_h + word_h + 6 && i32::from(area.width) >= cat_w;
    let total_h = if show_cat { cat_h + word_h + 5 } else { word_h + 5 };
    let top = i32::from(area.y) + (i32::from(area.height) - total_h).max(0) / 2;
    let cx = i32::from(area.x) + i32::from(area.width) / 2;

    let mut y = top;
    if show_cat {
        blit(f.buffer_mut(), area, cx - cat_w / 2, y, &cat, octocat_style(frame));
        y += cat_h;
    }
    blit(
        f.buffer_mut(),
        area,
        cx - word_w / 2,
        y,
        &word,
        wordmark_style(frame, word_w as usize),
    );
    y += word_h;

    let tagline = Paragraph::new(Line::styled(
        "a git workflow manager for your terminal",
        theme::dim(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(tagline, row(area, y));

    let blink = (frame / 5).is_multiple_of(2);
    let hint = Line::from(vec![
        Span::styled(if blink { "▶ " } else { "  " }, Style::default().fg(theme::ACCENT)),
        Span::styled("press any key", theme::key()),
        Span::styled(" to open a repository   ", theme::dim()),
        Span::styled(",", theme::key()),
        Span::styled(" settings   ", theme::dim()),
        Span::styled("?", theme::key()),
        Span::styled(" help   ", theme::dim()),
        Span::styled("q", theme::key()),
        Span::styled(" quit", theme::dim()),
    ]);
    f.render_widget(Paragraph::new(hint).alignment(Alignment::Center), row(area, y + 2));

}

/// One full-width row of `area` at absolute row `y` (empty rect if outside).
fn row(area: Rect, y: i32) -> Rect {
    if y < i32::from(area.top()) || y >= i32::from(area.bottom()) {
        return Rect::default();
    }
    Rect {
        x: area.x,
        y: y as u16,
        width: area.width,
        height: 1,
    }
}
