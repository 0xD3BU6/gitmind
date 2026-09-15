//! LLM integration (OpenAI-compatible provider via `rig-core`). Requests run on a background
//! thread and report back through a channel so the TUI never blocks.

pub mod analyzer;
pub mod client;
pub mod prompts;

pub use analyzer::parse_proposal;
pub use client::{Client, spawn_commit_message};
