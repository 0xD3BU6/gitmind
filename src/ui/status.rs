use crate::app::Session;
use crate::models::ChangeKind;
use crate::ui::{layout, theme};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

fn kind_style(kind: Option<ChangeKind>) -> (char, Style) {
    match kind {
        None => ('·', theme::dim()),
        Some(k) => {
            let color = match k {
                ChangeKind::Added | ChangeKind::Untracked => theme::GREEN,
                ChangeKind::Modified | ChangeKind::TypeChange => theme::YELLOW,
                ChangeKind::Deleted => theme::RED,
                ChangeKind::Renamed => theme::PURPLE,
                ChangeKind::Conflicted => theme::RED,
                ChangeKind::Ignored => theme::DIM,
            };
            (k.symbol(), Style::default().fg(color).add_modifier(Modifier::BOLD))
        }
    }
}

pub fn draw(f: &mut Frame, area: Rect, s: &Session) {
    let cols = layout::horizontal_chunks(
        area,
        &[Constraint::Percentage(38), Constraint::Percentage(62)],
    );
    draw_files(f, cols[0], s);
    draw_diff(f, cols[1], s);
}

fn draw_files(f: &mut Frame, area: Rect, s: &Session) {
    let title = if s.marked.is_empty() {
        format!("Changes ({})", s.changes.len())
    } else {
        format!("Changes ({}) · {} marked", s.changes.len(), s.marked.len())
    };
    let block = theme::panel(&title, true);

    if s.changes.is_empty() {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::styled("  ✔ working tree clean", Style::default().fg(theme::GREEN)),
            Line::styled("  nothing to stage or commit", theme::dim()),
        ])
        .block(block);
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = s
        .changes
        .iter()
        .map(|c| {
            let (i_sym, i_style) = kind_style(c.index);
            let (w_sym, w_style) = kind_style(c.worktree);
            let dot = if c.is_staged() {
                Span::styled("● ", Style::default().fg(theme::GREEN))
            } else {
                Span::styled("○ ", theme::dim())
            };
            let mark = if s.is_marked(&c.path) {
                Span::styled("▣ ", Style::default().fg(theme::ACCENT))
            } else {
                Span::styled("▢ ", theme::dim())
            };
            ListItem::new(Line::from(vec![
                mark,
                dot,
                Span::styled(i_sym.to_string(), i_style),
                Span::styled(w_sym.to_string(), w_style),
                Span::raw(" "),
                Span::raw(c.path.clone()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected())
        .highlight_symbol("▶");
    let mut state = ListState::default().with_selected(Some(s.selected_change().map(|_| s.selected[0]).unwrap_or(0)));
    f.render_stateful_widget(list, area, &mut state);
}

fn diff_line(line: &str) -> Line<'static> {
    let style = if line.starts_with("+++") || line.starts_with("---") {
        Style::default().fg(theme::PRIMARY).add_modifier(Modifier::BOLD)
    } else if line.starts_with("diff --git") || line.starts_with("index ") {
        theme::dim()
    } else if line.starts_with("@@") {
        Style::default().fg(theme::PURPLE)
    } else if line.starts_with('+') {
        Style::default().fg(theme::GREEN)
    } else if line.starts_with('-') {
        Style::default().fg(theme::RED)
    } else {
        Style::default().fg(Color::Reset)
    };
    Line::styled(line.to_string(), style)
}

fn draw_diff(f: &mut Frame, area: Rect, s: &Session) {
    let title = match s.selected_change() {
        Some(c) if c.has_worktree_changes() => format!("Diff · {} (unstaged)", c.path),
        Some(c) => format!("Diff · {} (staged)", c.path),
        None => "Diff".to_string(),
    };
    let lines: Vec<Line> = s.diff.lines().map(diff_line).collect();
    let total = lines.len() as u16;
    let scroll = s.diff_scroll.min(total.saturating_sub(1));
    let p = Paragraph::new(lines)
        .block(theme::panel(&title, false))
        .scroll((scroll, 0));
    f.render_widget(p, area);

    // scroll indicator
    if total > area.height.saturating_sub(2) {
        let pct = (u32::from(scroll) * 100 / u32::from(total.max(1))) as u16;
        let label = format!(" {pct}% ");
        let r = Rect {
            x: area.right().saturating_sub(label.len() as u16 + 1),
            y: area.bottom().saturating_sub(1),
            width: label.len() as u16,
            height: 1,
        };
        f.render_widget(Paragraph::new(label).style(theme::dim()), r);
    }
}
