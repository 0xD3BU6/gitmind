pub mod auth;
pub mod branch;
pub mod commit;
pub mod diff;
pub mod log;
pub mod push;
pub mod remote;
pub mod repository;
pub mod stash;
pub mod status;

pub use branch::{checkout_branch, create_branch, delete_branch, list_branches};
pub use commit::{commit, stage_all, stage_path, unstage_all, unstage_path};
pub use diff::{diff_for_file, staged_diff, working_tree_diff};
pub use log::recent_commits;
pub use push::push;
pub use remote::{fetch, pull, set_origin};
pub use repository::{init_repo, open_repo};
pub use stash::{stash_count, stash_pop, stash_push};
pub use status::{changes, repo_info, repo_status};

use crate::error::Result;
use std::path::Path;

/// Open the repository at `path` and return a human readable summary.
pub fn analyse_repo(path: &Path) -> Result<String> {
    let repo = open_repo(path)?;
    let mut out = String::from("✔ Git repository detected\n");
    out.push_str(&repo_status(&repo)?);
    Ok(out)
}
