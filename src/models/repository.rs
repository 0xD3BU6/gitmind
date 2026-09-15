use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct RepositoryInfo {
    pub path: PathBuf,
    pub branch: String,
    pub remote: Option<String>,
    pub head: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub is_detached: bool,
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub short_id: String,
    pub summary: String,
    pub body: String,
    pub author: String,
    pub email: String,
    pub when: String,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub last_commit: String,
}
