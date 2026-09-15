//! GitHub integration: OAuth device flow login.

pub mod oauth;

pub use oauth::{LoginEvent, spawn_device_login};
