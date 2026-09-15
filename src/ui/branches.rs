use crate::app::Session;
use crate::ui::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

pub fn draw(f: &mut Frame, area: Rect, s: &Session) {
    let title = format!("Branches ({})", s.branches.len());
    if s.branches.is_empty() {
        let p = Paragraph::new(Line::styled("  no local branches", theme::dim()))
            .block(theme::panel(&title, true));
        f.render_widget(p, area);
        return;
    }

    let name_w = s.branches.iter().map(|b| b.name.len()).max().unwrap_or(10);
    let items: Vec<ListItem> = s
        .branches
        .iter()
        .map(|b| {
            let mut spans = vec![
                if b.is_head {
                    Span::styled("* ", Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD))
                } else {
                    Span::raw("  ")
                },
                Span::styled(
                    format!("{:<w$}", b.name, w = name_w),
                    if b.is_head {
                        Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
                Span::raw("  "),
            ];
            if let Some(up) = &b.upstream {
                spans.push(Span::styled(format!("⇢ {up}"), Style::default().fg(theme::PRIMARY)));
                if b.ahead > 0 {
                    spans.push(Span::styled(format!(" ↑{}", b.ahead), Style::default().fg(theme::GREEN)));
                }
                if b.behind > 0 {
                    spans.push(Span::styled(format!(" ↓{}", b.behind), Style::default().fg(theme::RED)));
                }
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(b.last_commit.clone(), theme::dim()));
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(theme::panel(&title, true))
        .highlight_style(theme::selected())
        .highlight_symbol("▶");
    let mut state = ListState::default().with_selected(Some(s.selected[2]));
    f.render_stateful_widget(list, area, &mut state);
}
