use crate::app::{App, SettingsForm};
use crate::config::TICK_RATE;
use crate::error::Result;
use crate::tui::state::{State, Tab};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;

/// Poll for a key event and apply it to `app`. Returns `true` to quit.
pub fn handle_event(app: &mut App) -> Result<bool> {
    if !event::poll(TICK_RATE)? {
        return Ok(false);
    }
    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };
    // Windows reports key release events too; only react to presses.
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }
    Ok(handle_key(app, key))
}

fn ctrl(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.code == KeyCode::Char('c') && ctrl(key) {
        return true;
    }

    if app.show_help {
        if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
            app.show_help = false;
        }
        return false;
    }

    match app.state {
        State::Splash => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('?') => app.show_help = true,
            KeyCode::Char(',') => app.open_settings(),
            KeyCode::Char('L') => app.begin_login(),
            _ => app.state = State::Inputting,
        },
        State::Inputting => match key.code {
            KeyCode::Enter => {
                let raw = app.input.trim();
                if raw.is_empty() {
                    return false;
                }
                let path = PathBuf::from(expand_tilde(raw));
                app.open(&path);
            }
            KeyCode::Esc => app.state = State::Splash,
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Char('u') if ctrl(key) => app.input.clear(),
            KeyCode::Char('s') if ctrl(key) => app.open_settings(),
            KeyCode::Char(ch) => app.input.push(ch),
            _ => {}
        },
        State::Dashboard => return dashboard_key(app, key),
        State::Committing => match key.code {
            KeyCode::Enter if ctrl(key) || key.modifiers.contains(KeyModifiers::ALT) => {
                app.commit_msg.push('\n');
            }
            KeyCode::Enter => app.commit(),
            KeyCode::Esc => {
                app.state = State::Dashboard;
                app.push_after_commit = false;
            }
            KeyCode::Backspace => {
                app.commit_msg.pop();
            }
            KeyCode::Char('u') if ctrl(key) => app.commit_msg.clear(),
            KeyCode::Char('g') | KeyCode::Char('r') if ctrl(key) => app.generate_message(),
            KeyCode::Char('p') if ctrl(key) => {
                app.push_after_commit = true;
                app.commit();
            }
            KeyCode::Char('n') if ctrl(key) => app.commit_msg.push('\n'),
            KeyCode::Char(ch) => app.commit_msg.push(ch),
            _ => {}
        },
        State::Settings => settings_key(app, key),
        State::Login => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                app.cancel_login();
            }
        }
    }
    false
}

fn dashboard_key(app: &mut App, key: KeyEvent) -> bool {
    let tab = app.session.as_ref().map(|s| s.tab).unwrap_or_default();
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Char(',') => app.open_settings(),
        KeyCode::Esc | KeyCode::Char('o') => app.close_repo(),
        KeyCode::Char('r') | KeyCode::F(5) => {
            app.refresh();
            app.notify("Refreshed");
        }

        // tabs
        KeyCode::Tab => app.next_tab(false),
        KeyCode::BackTab => app.next_tab(true),
        KeyCode::Char('1') => app.set_tab(Tab::Status),
        KeyCode::Char('2') => app.set_tab(Tab::Log),
        KeyCode::Char('3') => app.set_tab(Tab::Branches),
        KeyCode::Char('h') | KeyCode::Left => app.next_tab(true),
        KeyCode::Char('l') | KeyCode::Right => app.next_tab(false),

        // list navigation
        KeyCode::Char('j') | KeyCode::Down => app.move_selection(1),
        KeyCode::Char('k') | KeyCode::Up => app.move_selection(-1),
        KeyCode::Char('g') | KeyCode::Home => app.jump_selection(false),
        KeyCode::Char('G') | KeyCode::End => app.jump_selection(true),

        // diff scrolling
        KeyCode::Char('J') | KeyCode::PageDown => app.scroll_diff(10),
        KeyCode::Char('K') | KeyCode::PageUp => app.scroll_diff(-10),
        KeyCode::Char('d') if ctrl(key) => app.scroll_diff(10),
        KeyCode::Char('u') if ctrl(key) => app.scroll_diff(-10),

        // marking + staging
        KeyCode::Char(' ') if tab == Tab::Status => app.toggle_mark(),
        KeyCode::Char('v') if tab == Tab::Status => app.toggle_mark_all(),
        KeyCode::Char('x') if tab == Tab::Status => app.clear_marks(),
        KeyCode::Char('s') if tab == Tab::Status => app.apply_marked(),
        KeyCode::Char('a') if tab == Tab::Status => app.stage_all(),
        KeyCode::Char('u') if tab == Tab::Status => app.unstage_all(),
        KeyCode::Char('c') => app.begin_commit(),
        KeyCode::Char('p') => app.push(),
        KeyCode::Char('L') => app.begin_login(),
        KeyCode::Enter if tab == Tab::Branches => app.checkout_selected(),
        _ => {}
    }
    false
}

fn settings_key(app: &mut App, key: KeyEvent) {
    let n = SettingsForm::LABELS.len();
    match key.code {
        KeyCode::Esc => app.close_settings(false),
        KeyCode::Enter | KeyCode::Char('s') if key.code == KeyCode::Enter || ctrl(key) => {
            app.close_settings(true)
        }
        KeyCode::Tab | KeyCode::Down => app.form.focus = (app.form.focus + 1) % n,
        KeyCode::BackTab | KeyCode::Up => app.form.focus = (app.form.focus + n - 1) % n,
        KeyCode::Char('r') if ctrl(key) => app.form.reveal = !app.form.reveal,
        KeyCode::Char('l') if ctrl(key) => {
            // save what's typed (the client id in particular), then start the login
            app.close_settings(true);
            app.begin_login();
        }
        KeyCode::Char('u') if ctrl(key) => app.form.fields[app.form.focus].clear(),
        KeyCode::Backspace => {
            app.form.fields[app.form.focus].pop();
        }
        KeyCode::Char(ch) if !ctrl(key) => app.form.fields[app.form.focus].push(ch),
        _ => {}
    }
}

fn expand_tilde(path: &str) -> String {
    if let (Some(rest), Ok(home)) = (path.strip_prefix('~'), std::env::var("HOME")) {
        return format!("{home}{rest}");
    }
    path.to_string()
}
