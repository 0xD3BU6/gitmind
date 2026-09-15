use git2::Status;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Untracked,
    Conflicted,
    Ignored,
}

impl ChangeKind {
    pub fn symbol(self) -> char {
        match self {
            ChangeKind::Added => 'A',
            ChangeKind::Modified => 'M',
            ChangeKind::Deleted => 'D',
            ChangeKind::Renamed => 'R',
            ChangeKind::TypeChange => 'T',
            ChangeKind::Untracked => '?',
            ChangeKind::Conflicted => 'U',
            ChangeKind::Ignored => '!',
        }
    }
}

/// One entry from `git status`, split into its index and worktree halves.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub index: Option<ChangeKind>,
    pub worktree: Option<ChangeKind>,
}

impl FileChange {
    pub fn from_status(path: String, s: Status) -> Self {
        let index = if s.contains(Status::INDEX_NEW) {
            Some(ChangeKind::Added)
        } else if s.contains(Status::INDEX_MODIFIED) {
            Some(ChangeKind::Modified)
        } else if s.contains(Status::INDEX_DELETED) {
            Some(ChangeKind::Deleted)
        } else if s.contains(Status::INDEX_RENAMED) {
            Some(ChangeKind::Renamed)
        } else if s.contains(Status::INDEX_TYPECHANGE) {
            Some(ChangeKind::TypeChange)
        } else {
            None
        };
        let worktree = if s.contains(Status::CONFLICTED) {
            Some(ChangeKind::Conflicted)
        } else if s.contains(Status::WT_NEW) {
            Some(ChangeKind::Untracked)
        } else if s.contains(Status::WT_MODIFIED) {
            Some(ChangeKind::Modified)
        } else if s.contains(Status::WT_DELETED) {
            Some(ChangeKind::Deleted)
        } else if s.contains(Status::WT_RENAMED) {
            Some(ChangeKind::Renamed)
        } else if s.contains(Status::WT_TYPECHANGE) {
            Some(ChangeKind::TypeChange)
        } else if s.contains(Status::IGNORED) {
            Some(ChangeKind::Ignored)
        } else {
            None
        };
        Self { path, index, worktree }
    }

    pub fn is_staged(&self) -> bool {
        self.index.is_some()
    }

    pub fn has_worktree_changes(&self) -> bool {
        self.worktree.is_some()
    }
}
