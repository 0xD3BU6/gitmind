use crate::app::{App, JobKind};
use crate::tui::state::{State, Tab};
use crate::ui::{layout, popup, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

fn keys(pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (k, desc) in pairs {
        spans.push(Span::styled(format!(" {k} "), theme::key()));
        spans.push(Span::styled(format!("{desc}  "), theme::dim()));
    }
    Line::from(spans)
}

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let tab = app.session.as_ref().map(|s| s.tab).unwrap_or_default();
    let line = match (app.state, tab) {
        (State::Committing, _) => keys(&[("Enter", "commit"), ("^P", "commit+push"), ("^G", "AI"), ("Esc", "cancel")]),
        (State::Prompt, _) => keys(&[("Enter", "ok"), ("Esc", "cancel")]),
        (_, Tab::Status) => keys(&[
            ("j/k", "move"),
            ("space", "mark"),
            ("s", "stage"),
            ("a/u", "all"),
            ("c", "commit"),
            ("p/f/P", "push/fetch/pull"),
            ("z/Z", "stash/pop"),
            ("R", "origin"),
            (",", "settings"),
            ("?", "help"),
            ("q", "quit"),
        ]),
        (_, Tab::Log) => keys(&[
            ("j/k", "move"),
            ("g/G", "top/bottom"),
            ("p", "push"),
            ("Tab", "next tab"),
            (",", "settings"),
            ("?", "help"),
            ("q", "quit"),
        ]),
        (_, Tab::Branches) => keys(&[
            ("j/k", "move"),
            ("Enter", "checkout"),
            ("n", "new"),
            ("D", "delete"),
            ("p/f/P", "push/fetch/pull"),
            (",", "settings"),
            (",", "settings"),
            ("?", "help"),
            ("q", "quit"),
        ]),
    };
    f.render_widget(Paragraph::new(line), Rect { height: 1, ..area });

    // background job indicator
    if let Some(job) = &app.job {
        let label = match job.kind {
            JobKind::CommitMessage => format!("{} {} thinking…", popup::spinner(app.frame), app.settings.model),
            JobKind::Push => format!("{} pushing to origin…", popup::spinner(app.frame)),
            JobKind::Fetch => format!("{} fetching origin…", popup::spinner(app.frame)),
            JobKind::Pull => format!("{} pulling…", popup::spinner(app.frame)),
        };
        let secs = job.started.elapsed().as_secs();
        let text = format!(" {label} {secs}s ");
        let w = (text.chars().count() as u16).min(area.width);
        let r = Rect { x: area.x, y: area.y + 1, width: w, height: 1 };
        f.render_widget(
            Paragraph::new(text).style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            r,
        );
    }
}

/// Transient status message drawn at the bottom-right of the screen.
pub fn toast(f: &mut Frame, app: &App) {
    let Some(t) = &app.toast else { return };
    let area = f.area();
    let text = format!(" {} {} ", if t.is_error { "✘" } else { "✔" }, t.text);
    let width = (text.chars().count() as u16).min(area.width);
    let rect = Rect {
        x: area.right().saturating_sub(width),
        y: area.bottom().saturating_sub(1),
        width,
        height: 1,
    };
    let rect = rect.intersection(layout::centered(area, area.width, area.height));
    let color = if t.is_error { theme::RED } else { theme::GREEN };
    let style = Style::default()
        .fg(ratatui::style::Color::Black)
        .bg(color)
        .add_modifier(Modifier::BOLD);
    f.render_widget(Clear, rect);
    f.render_widget(Paragraph::new(text).style(style), rect);
}
