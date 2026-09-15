use crate::app::Session;
use crate::ui::{layout, theme};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, area: Rect, s: &Session) {
    let cols = layout::horizontal_chunks(
        area,
        &[Constraint::Percentage(60), Constraint::Percentage(40)],
    );

    let title = format!("Commits ({})", s.commits.len());
    if s.commits.is_empty() {
        let p = Paragraph::new(Line::styled("  no commits yet", theme::dim()))
            .block(theme::panel(&title, true));
        f.render_widget(p, cols[0]);
    } else {
        let items: Vec<ListItem> = s
            .commits
            .iter()
            .map(|c| {
                ListItem::new(Line::from(vec![
                    Span::styled("◆ ", Style::default().fg(theme::ACCENT)),
                    Span::styled(c.short_id.clone(), Style::default().fg(theme::YELLOW)),
                    Span::raw(" "),
                    Span::raw(c.summary.clone()),
                    Span::styled(format!("  {}", c.author), theme::dim()),
                ]))
            })
            .collect();
        let list = List::new(items)
            .block(theme::panel(&title, true))
            .highlight_style(theme::selected())
            .highlight_symbol("▶");
        let mut state = ListState::default().with_selected(Some(s.selected[1]));
        f.render_stateful_widget(list, cols[0], &mut state);
    }

    let detail: Vec<Line> = match s.selected_commit() {
        Some(c) => {
            let mut v = vec![
                Line::from(vec![
                    Span::styled("commit  ", theme::dim()),
                    Span::styled(c.short_id.clone(), Style::default().fg(theme::YELLOW)),
                ]),
                Line::from(vec![
                    Span::styled("author  ", theme::dim()),
                    Span::raw(format!("{} <{}>", c.author, c.email)),
                ]),
                Line::from(vec![
                    Span::styled("date    ", theme::dim()),
                    Span::raw(c.when.clone()),
                ]),
                Line::from(""),
                Line::styled(c.summary.clone(), Style::default().add_modifier(Modifier::BOLD)),
            ];
            if !c.body.is_empty() {
                v.push(Line::from(""));
                v.extend(c.body.lines().map(|l| Line::raw(l.to_string())));
            }
            v
        }
        None => vec![Line::styled("select a commit", theme::dim())],
    };
    let p = Paragraph::new(detail)
        .block(theme::panel("Details", false))
        .wrap(Wrap { trim: false });
    f.render_widget(p, cols[1]);
}
