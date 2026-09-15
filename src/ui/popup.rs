use crate::app::{App, JobKind, SettingsForm};
use crate::config::mask;
use crate::tui::state::PromptKind;
use crate::ui::{layout, theme};
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

pub const SPINNER: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

pub fn spinner(frame: u64) -> &'static str {
    SPINNER[(frame % SPINNER.len() as u64) as usize]
}

pub fn commit(f: &mut Frame, app: &App) {
    let area = f.area();
    let inner_w = usize::from(area.width.min(76).saturating_sub(2)).max(1);
    let msg_rows: u16 = app
        .commit_msg
        .lines()
        .map(|l| (l.chars().count().max(1)).div_ceil(inner_w) as u16)
        .sum::<u16>()
        .max(1);
    let height = (msg_rows + 6).clamp(9, area.height.saturating_sub(2));
    let rect = layout::centered(area, area.width.min(76), height);
    f.render_widget(Clear, rect);

    let staged = app.session.as_ref().map(|s| s.staged_count()).unwrap_or(0);
    let generating = matches!(app.job.as_ref().map(|j| j.kind), Some(JobKind::CommitMessage));

    let mut body: Vec<Line> = if generating && app.commit_msg.is_empty() {
        vec![Line::from(vec![
            Span::styled(format!("{} ", spinner(app.frame)), Style::default().fg(theme::ACCENT)),
            Span::styled(
                format!("asking {} for a commit message…", app.settings.model),
                theme::dim(),
            ),
        ])]
    } else if app.commit_msg.is_empty() {
        vec![Line::styled(
            if app.settings.has_ai() {
                "type a message, or Ctrl-G to ask the model"
            } else {
                "type a message (no LLM API key set — ',' to add one)"
            },
            theme::dim(),
        )]
    } else {
        app.commit_msg
            .lines()
            .enumerate()
            .map(|(i, l)| {
                if i == 0 {
                    Line::styled(l.to_string(), Style::default().add_modifier(Modifier::BOLD))
                } else {
                    Line::raw(l.to_string())
                }
            })
            .collect()
    };
    if app.commit_msg.ends_with('\n') {
        body.push(Line::raw(""));
    }

    let inner_h = rect.height.saturating_sub(2) as usize;
    let footer_rows = 2;
    let avail = inner_h.saturating_sub(footer_rows);
    // if the message is taller than the box, show its tail (where the cursor is)
    if body.len() > avail {
        let skip = body.len() - avail;
        body.drain(..skip);
    }
    while body.len() < avail {
        body.push(Line::raw(""));
    }
    body.push(Line::raw(""));
    body.push(Line::from(vec![
        Span::styled(format!("{staged} staged  "), Style::default().fg(theme::GREEN)),
        Span::styled("Enter", theme::key()),
        Span::styled(" commit  ", theme::dim()),
        Span::styled("^P", theme::key()),
        Span::styled(" commit+push  ", theme::dim()),
        Span::styled("^G", theme::key()),
        Span::styled(" AI  ", theme::dim()),
        Span::styled("^N", theme::key()),
        Span::styled(" newline  ", theme::dim()),
        Span::styled("^U", theme::key()),
        Span::styled(" clear  ", theme::dim()),
        Span::styled("Esc", theme::key()),
        Span::styled(" cancel", theme::dim()),
    ]));

    let title = if app.settings.has_ai() {
        format!("Commit · AI: {}", app.settings.model)
    } else {
        "Commit".to_string()
    };
    let p = Paragraph::new(body)
        .block(theme::panel(&title, true))
        .wrap(Wrap { trim: false });
    f.render_widget(p, rect);

    // cursor at the end of the last line
    if !generating || !app.commit_msg.is_empty() {
        let inner_w = rect.width.saturating_sub(2) as usize;
        let last = app.commit_msg.rsplit('\n').next().unwrap_or("");
        let len = last.chars().count();
        let row = app.commit_msg.matches('\n').count() as u16;
        let (cx, cy) = if inner_w == 0 {
            (0, 0)
        } else {
            ((len % inner_w) as u16, row + (len / inner_w) as u16)
        };
        let max_y = rect.height.saturating_sub(4);
        f.set_cursor_position(Position::new(rect.x + 1 + cx, rect.y + 1 + cy.min(max_y)));
    }
}

pub fn prompt(f: &mut Frame, app: &App) {
    let Some((kind, value)) = &app.prompt else { return };
    let area = f.area();
    let rect = layout::centered(area, area.width.min(70), 6);
    f.render_widget(Clear, rect);
    let hint = match kind {
        PromptKind::NewBranch => "created at HEAD and checked out".to_string(),
        PromptKind::SetOrigin => "e.g. git@github.com:you/repo.git or https://github.com/you/repo.git".to_string(),
        PromptKind::StashMessage => "working tree + untracked files are stashed (Z pops)".to_string(),
        PromptKind::DeleteBranch => format!(
            "type the branch name to confirm: {}",
            app.session.as_ref().and_then(|s| s.selected_branch()).map(|b| b.name.as_str()).unwrap_or("")
        ),
    };
    let lines = vec![
        Line::styled(format!(" {value}"), Style::default().fg(theme::PRIMARY)),
        Line::styled(format!(" {hint}"), theme::dim()),
        Line::from(vec![
            Span::styled(" Enter", theme::key()),
            Span::styled(" ok  ", theme::dim()),
            Span::styled("^U", theme::key()),
            Span::styled(" clear  ", theme::dim()),
            Span::styled("Esc", theme::key()),
            Span::styled(" cancel", theme::dim()),
        ]),
    ];
    f.render_widget(Paragraph::new(lines).block(theme::panel(kind.title(), true)), rect);
    let x = rect.x + 2 + value.chars().count() as u16;
    f.set_cursor_position(Position::new(x.min(rect.right().saturating_sub(2)), rect.y + 1));
}

pub fn login(f: &mut Frame, app: &App) {
    let area = f.area();
    let rect = layout::centered(area, area.width.min(64), 11);
    f.render_widget(Clear, rect);
    let Some(flow) = &app.login else { return };

    let mut lines: Vec<Line> = Vec::new();
    match &flow.user_code {
        None => lines.push(Line::from(vec![
            Span::styled(format!("{} ", spinner(app.frame)), Style::default().fg(theme::ACCENT)),
            Span::styled("contacting GitHub…", theme::dim()),
        ])),
        Some(code) => {
            let left = flow.expires_in.saturating_sub(flow.started.elapsed().as_secs());
            lines.push(Line::styled("1. Open this page (your browser may already have it):", theme::dim()));
            lines.push(Line::styled(format!("   {}", flow.verification_uri), Style::default().fg(theme::PRIMARY)));
            lines.push(Line::raw(""));
            lines.push(Line::styled("2. Enter this code:", theme::dim()));
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {code} "),
                    Style::default().fg(ratatui::style::Color::Black).bg(theme::ACCENT).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", spinner(app.frame)), Style::default().fg(theme::ACCENT)),
                Span::styled(format!("waiting for approval · {left}s left", ), theme::dim()),
            ]));
        }
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::styled("Esc", theme::key()), Span::styled(" cancel", theme::dim())]));
    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Login with GitHub", true)).wrap(Wrap { trim: false }),
        rect,
    );
}

pub fn settings(f: &mut Frame, app: &App) {
    let area = f.area();
    let rect = layout::centered(area, area.width.min(76), 18);
    f.render_widget(Clear, rect);

    let form = &app.form;
    let mut lines: Vec<Line> = vec![Line::styled(
        "Bring your own keys. Stored locally (mode 600); never sent anywhere but the provider.",
        theme::dim(),
    )];
    lines.push(Line::raw(""));

    let inner_w = rect.width.saturating_sub(4) as usize;
    for (i, label) in SettingsForm::LABELS.iter().enumerate() {
        let focused = i == form.focus;
        let value = &form.fields[i];
        let shown = if SettingsForm::SECRET[i] && !form.reveal && !focused {
            mask(value)
        } else {
            value.clone()
        };
        let shown: String = shown.chars().rev().take(inner_w.saturating_sub(16)).collect::<Vec<_>>().into_iter().rev().collect();
        lines.push(Line::from(vec![
            Span::styled(if focused { "▶ " } else { "  " }, Style::default().fg(theme::ACCENT)),
            Span::styled(format!("{label:<13}"), if focused { theme::key() } else { theme::dim() }),
            Span::styled(
                if shown.is_empty() { "<empty>".to_string() } else { shown },
                if focused {
                    Style::default().fg(theme::PRIMARY)
                } else if value.is_empty() {
                    theme::dim()
                } else {
                    Style::default()
                },
            ),
        ]));
        let hint = match i {
            0 => "your LLM provider's API key  ·  env: LLM_API_KEY".to_string(),
            1 => "e.g. openai/gpt-oss-120b, openai/gpt-oss-20b, qwen/qwen3.8-27b".to_string(),
            2 => {
                if app.settings.github_login.is_empty() {
                    "filled by ^L login, or paste a PAT (repo scope)  ·  env: GITHUB_TOKEN".to_string()
                } else {
                    format!("logged in as {}  ·  ^L to log in again", app.settings.github_login)
                }
            }
            _ => "OAuth App client id with Device Flow on: github.com/settings/developers".to_string(),
        };
        lines.push(Line::styled(format!("    {hint}"), theme::dim()));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("Enter/^S", theme::key()),
        Span::styled(" save  ", theme::dim()),
        Span::styled("Tab/↑↓", theme::key()),
        Span::styled(" field  ", theme::dim()),
        Span::styled("^L", theme::key()),
        Span::styled(" login with GitHub  ", theme::dim()),
        Span::styled("^R", theme::key()),
        Span::styled(if form.reveal { " hide  " } else { " reveal  " }, theme::dim()),
        Span::styled("^U", theme::key()),
        Span::styled(" clear  ", theme::dim()),
        Span::styled("Esc", theme::key()),
        Span::styled(" cancel", theme::dim()),
    ]));

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Settings", true)).wrap(Wrap { trim: false }),
        rect,
    );

    // cursor on the focused field
    let value_len = form.fields[form.focus].chars().count().min(inner_w.saturating_sub(16));
    let y = rect.y + 3 + (form.focus as u16) * 2;
    let x = rect.x + 1 + 2 + 13 + value_len as u16;
    f.set_cursor_position(Position::new(x.min(rect.right().saturating_sub(2)), y));
}

pub fn help(f: &mut Frame) {
    let rows: &[(&str, &str)] = &[
        ("Global", ""),
        ("?", "toggle this help"),
        (",", "settings: LLM API key, model, GitHub token"),
        ("q / Ctrl-C", "quit"),
        ("", ""),
        ("Dashboard", ""),
        ("Tab / Shift-Tab, h / l, 1 2 3", "switch tab"),
        ("j / k, ↑ / ↓", "move selection"),
        ("g / G", "jump to top / bottom"),
        ("J / K, PgUp / PgDn", "scroll diff"),
        ("r / F5", "refresh from disk"),
        ("o / Esc", "open another repository"),
        ("", ""),
        ("Status tab", ""),
        ("s / space", "stage or unstage highlighted file"),
        ("a / u", "stage / unstage everything"),
        ("c", "commit (AI drafts the message if a key is set)"),
        ("p", "push current branch to origin"),
        ("space / v / x", "mark file / mark all / clear marks"),
        ("s", "stage or unstage marked files (or the highlighted one)"),
        ("L", "login with GitHub (OAuth device flow)"),
        ("", ""),
        ("Remote & sync", ""),
        ("R", "add origin or change its URL"),
        ("f / P", "fetch origin / pull (fast-forward only)"),
        ("z / Z", "stash working tree / pop latest stash"),
        ("Ctrl-I (path prompt)", "git init a plain folder"),
        ("", ""),
        ("Commit popup", ""),
        ("Enter / ^P", "commit / commit then push"),
        ("^G", "(re)generate message with the model"),
        ("", ""),
        ("Branches tab", ""),
        ("Enter", "checkout highlighted branch (safe)"),
        ("n / D", "new branch at HEAD / delete highlighted branch"),
    ];

    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, d)| {
            if d.is_empty() {
                Line::styled(k.to_string(), theme::title())
            } else {
                Line::from(vec![
                    Span::styled(format!("  {k:<30}"), theme::key()),
                    Span::raw(d.to_string()),
                ])
            }
        })
        .collect();

    let area = f.area();
    let rect = layout::centered(area, 70, (lines.len() as u16 + 2).min(area.height));
    f.render_widget(Clear, rect);
    f.render_widget(Paragraph::new(lines).block(theme::panel("Help", true)), rect);
}

/// Keep `Rect` import used on all paths.
#[allow(dead_code)]
fn _unused(_: Rect) {}
