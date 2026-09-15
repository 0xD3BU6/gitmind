use crate::error::Result;
use crate::models::{FileChange, RepositoryInfo};
use git2::{Repository, StatusOptions};

fn status_options() -> StatusOptions {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);
    opts
}

/// All changed files, sorted by path.
pub fn changes(repo: &Repository) -> Result<Vec<FileChange>> {
    let mut opts = status_options();
    let statuses = repo.statuses(Some(&mut opts))?;
    let mut out: Vec<FileChange> = statuses
        .iter()
        .filter_map(|e| {
            let path = e.path().ok()?.to_string();
            Some(FileChange::from_status(path, e.status()))
        })
        .filter(|c| c.index.is_some() || c.worktree.is_some())
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Branch / remote / ahead-behind information.
pub fn repo_info(repo: &Repository) -> Result<RepositoryInfo> {
    let path = repo
        .workdir()
        .unwrap_or_else(|| repo.path())
        .to_path_buf();

    let mut info = RepositoryInfo {
        path,
        ..Default::default()
    };

    match repo.head() {
        Ok(head) => {
            info.is_detached = !head.is_branch();
            info.branch = head
                .shorthand()
                .map(str::to_string)
                .unwrap_or_else(|_| "HEAD".to_string());
            if let Some(oid) = head.target() {
                let short = oid.to_string()[..7].to_string();
                let summary = repo
                    .find_commit(oid)
                    .ok()
                    .and_then(|c| c.summary().ok().flatten().map(str::to_string))
                    .unwrap_or_default();
                info.head = Some(format!("{short} {summary}"));

                // ahead/behind relative to upstream, if any
                if let Ok(local) = repo.find_branch(&info.branch, git2::BranchType::Local)
                    && let Ok(upstream) = local.upstream()
                    && let Some(up_oid) = upstream.get().target()
                    && let Ok((ahead, behind)) = repo.graph_ahead_behind(oid, up_oid)
                {
                    info.ahead = ahead;
                    info.behind = behind;
                }
            }
        }
        Err(_) => info.branch = "(no commits yet)".to_string(),
    }

    info.remote = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().ok().map(str::to_string));

    Ok(info)
}

/// Plain-text summary used by `analyse_repo` and the tests.
pub fn repo_status(repo: &Repository) -> Result<String> {
    let info = repo_info(repo)?;
    let changed = changes(repo)?;

    let branch = if info.is_detached {
        format!("detached HEAD ({})", info.branch)
    } else {
        info.branch.clone()
    };
    let remote = match &info.remote {
        Some(url) => format!("origin ({url})"),
        None => "<no remote>".to_string(),
    };

    let mut out = String::new();
    out.push_str(&format!("✔ Branch: {branch}\n"));
    out.push_str(&format!("✔ Remote: {remote}\n"));
    if changed.is_empty() {
        out.push_str("✔ Working tree is clean\n");
    } else {
        out.push_str(&format!(
            "✔ Working tree contains changes ({} entries)\n",
            changed.len()
        ));
    }
    Ok(out)
}
