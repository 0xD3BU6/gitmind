pub mod branches;
pub mod footer;
pub mod header;
pub mod input;
pub mod layout;
pub mod log;
pub mod logo;
pub mod popup;
pub mod splash;
pub mod status;
pub mod theme;

use crate::app::App;
use crate::tui::state::{State, Tab};
use ratatui::{Frame, layout::Constraint};

/// Render the whole screen for the current app state.
pub fn draw(f: &mut Frame, app: &App) {
    // base screen
    let base = if matches!(app.state, State::Settings | State::Login | State::Prompt) { app.prev_state } else { app.state };
    match base {
        State::Splash => splash::draw(f, app),
        State::Inputting => input::draw(f, app),
        State::Dashboard | State::Committing => draw_dashboard(f, app),
        State::Settings | State::Login | State::Prompt => draw_dashboard(f, app),
    }

    // overlays
    if base == State::Committing {
        popup::commit(f, app);
    }
    if app.state == State::Settings {
        popup::settings(f, app);
    }
    if app.state == State::Login {
        popup::login(f, app);
    }
    if app.state == State::Prompt {
        popup::prompt(f, app);
    }
    if app.show_help {
        popup::help(f);
    }
    footer::toast(f, app);
}

fn draw_dashboard(f: &mut Frame, app: &App) {
    let Some(session) = &app.session else {
        input::draw(f, app);
        return;
    };
    let chunks = layout::vertical_chunks(
        f.area(),
        &[
            Constraint::Length(4), // header
            Constraint::Length(1), // tabs
            Constraint::Min(0),    // body
            Constraint::Length(2), // footer
        ],
    );

    header::draw(f, chunks[0], session, app);
    header::tabs(f, chunks[1], session.tab);

    match session.tab {
        Tab::Status => status::draw(f, chunks[2], session),
        Tab::Log => log::draw(f, chunks[2], session),
        Tab::Branches => branches::draw(f, chunks[2], session),
    }

    footer::draw(f, chunks[3], app);
}
