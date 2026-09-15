use crate::error::Result;
use crate::models::BranchInfo;
use git2::{BranchType, Repository, build::CheckoutBuilder};

pub fn list_branches(repo: &Repository) -> Result<Vec<BranchInfo>> {
    let mut out = Vec::new();
    for entry in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = entry?;
        let name = branch.name()?.unwrap_or("?").to_string();
        let oid = branch.get().target();
        let upstream = branch
            .upstream()
            .ok()
            .and_then(|u| u.name().ok().flatten().map(str::to_string));
        let (ahead, behind) = match (oid, branch.upstream().ok().and_then(|u| u.get().target())) {
            (Some(l), Some(u)) => repo.graph_ahead_behind(l, u).unwrap_or((0, 0)),
            _ => (0, 0),
        };
        let last_commit = oid
            .and_then(|o| repo.find_commit(o).ok())
            .map(|c| {
                format!(
                    "{} {}",
                    &c.id().to_string()[..7],
                    c.summary().ok().flatten().unwrap_or("")
                )
            })
            .unwrap_or_default();
        out.push(BranchInfo {
            is_head: branch.is_head(),
            name,
            upstream,
            ahead,
            behind,
            last_commit,
        });
    }
    out.sort_by(|a, b| b.is_head.cmp(&a.is_head).then(a.name.cmp(&b.name)));
    Ok(out)
}

/// Safe checkout: refuses to clobber local modifications.
pub fn checkout_branch(repo: &Repository, name: &str) -> Result<()> {
    let refname = format!("refs/heads/{name}");
    let obj = repo.revparse_single(&refname)?;
    let mut cb = CheckoutBuilder::new();
    cb.safe();
    repo.checkout_tree(&obj, Some(&mut cb))
        .map_err(|e| format!("checkout failed: {e}"))?;
    repo.set_head(&refname)?;
    Ok(())
}

/// Create `name` at HEAD and switch to it.
pub fn create_branch(repo: &Repository, name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() || name.contains(char::is_whitespace) || name.contains("..") {
        return Err("invalid branch name".into());
    }
    let head = repo
        .head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|_| "no commits yet — commit first")?;
    repo.branch(name, &head, false)
        .map_err(|e| format!("cannot create branch: {e}"))?;
    checkout_branch(repo, name)
}

/// Delete a local branch (never the checked-out one).
pub fn delete_branch(repo: &Repository, name: &str) -> Result<()> {
    let mut b = repo.find_branch(name, BranchType::Local)?;
    if b.is_head() {
        return Err("cannot delete the current branch".into());
    }
    b.delete().map_err(|e| format!("cannot delete branch: {e}"))?;
    Ok(())
}
