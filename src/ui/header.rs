use crate::app::{App, Session};
use crate::tui::state::Tab;
use crate::ui::{logo, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
};

pub fn draw(f: &mut Frame, area: Rect, s: &Session, app: &App) {
    let info = &s.info;
    let mut branch_spans = vec![
        Span::styled(logo::SMALL, theme::title()),
        Span::styled("  ", theme::dim()),
        Span::styled(" ", theme::dim()),
        Span::styled(
            info.branch.clone(),
            Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD),
        ),
    ];
    if info.is_detached {
        branch_spans.push(Span::styled(" (detached)", Style::default().fg(theme::YELLOW)));
    }
    if info.ahead > 0 {
        branch_spans.push(Span::styled(
            format!(" ↑{}", info.ahead),
            Style::default().fg(theme::GREEN),
        ));
    }
    if info.behind > 0 {
        branch_spans.push(Span::styled(
            format!(" ↓{}", info.behind),
            Style::default().fg(theme::RED),
        ));
    }
    let staged = s.staged_count();
    let unstaged = s.changes.len() - staged;
    branch_spans.push(Span::styled("    ", theme::dim()));
    branch_spans.push(Span::styled(
        format!("● {staged} staged"),
        Style::default().fg(if staged > 0 { theme::GREEN } else { theme::DIM }),
    ));
    branch_spans.push(Span::styled("  ", theme::dim()));
    branch_spans.push(Span::styled(
        format!("○ {unstaged} unstaged"),
        Style::default().fg(if unstaged > 0 { theme::YELLOW } else { theme::DIM }),
    ));

    let remote = info.remote.clone().unwrap_or_else(|| "no remote".to_string());
    let line2 = Line::from(vec![
        Span::styled(" ", theme::dim()),
        Span::styled(s.path.display().to_string(), Style::default().fg(theme::PRIMARY)),
        Span::styled("   ", theme::dim()),
        Span::styled(" ", theme::dim()),
        Span::styled(remote, theme::dim()),
        Span::styled("   ", theme::dim()),
        Span::styled(info.head.clone().unwrap_or_default(), theme::dim()),
        Span::styled("   ", theme::dim()),
        Span::styled(
            if app.settings.has_ai() { format!("✦ AI {}", app.settings.model) } else { "✦ AI off (,)".to_string() },
            Style::default().fg(if app.settings.has_ai() { theme::PURPLE } else { theme::DIM }),
        ),
        Span::styled("   ", theme::dim()),
        Span::styled(
            if !app.settings.github_login.is_empty() {
                format!(" {}", app.settings.github_login)
            } else if app.settings.has_github_token() {
                " token".to_string()
            } else {
                " not logged in (L)".to_string()
            },
            Style::default().fg(if app.settings.has_github_token() { theme::GREEN } else { theme::DIM }),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::DIM));
    f.render_widget(
        Paragraph::new(vec![Line::from(branch_spans), line2]).block(block),
        area,
    );
}

pub fn tabs(f: &mut Frame, area: Rect, current: Tab) {
    let titles: Vec<Line> = Tab::ALL.iter().map(|t| Line::from(t.title())).collect();
    let tabs = Tabs::new(titles)
        .select(current.index())
        .style(theme::dim())
        .highlight_style(
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(Span::styled("│", theme::dim()));
    f.render_widget(tabs, area);
}
