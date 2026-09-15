use crate::error::Result;
use git2::Repository;
use std::path::Path;

/// Open a git repository at `path`, searching parent directories as `git` does.
pub fn open_repo(path: &Path) -> Result<Repository> {
    if !path.is_dir() {
        return Err(format!("'{}' is not a directory", path.display()).into());
    }
    Repository::discover(path)
        .map_err(|e| format!("Failed to open git repo at '{}': {e}", path.display()).into())
}
