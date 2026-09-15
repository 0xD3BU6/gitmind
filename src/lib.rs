pub mod ai;
pub mod app;
pub mod config;
pub mod error;
pub mod git;
pub mod github;
pub mod models;
pub mod tui;
pub mod ui;

pub use app::App;
pub use git::analyse_repo;
pub use tui::Tui;
pub use ui::draw;
