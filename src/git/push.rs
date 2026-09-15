use crate::error::Result;
use crate::git::auth::{auth_hint, callbacks};
use git2::{PushOptions, Repository};
use std::cell::RefCell;

/// Push the current branch to `origin`, setting upstream if needed.
pub fn push(repo: &Repository, token: Option<&str>) -> Result<String> {
    let head = repo.head().map_err(|_| "nothing to push: no commits yet")?;
    if !head.is_branch() {
        return Err("detached HEAD — check out a branch first".into());
    }
    let branch = head.shorthand()?.to_string();
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");

    let mut remote = repo
        .find_remote("origin")
        .map_err(|_| "no 'origin' remote — press R to add one")?;
    let url = remote.url()?.to_string();
    let config = repo.config()?;

    let rejected: RefCell<Option<String>> = RefCell::new(None);
    let mut cbs = callbacks(&config, token);
    cbs.push_update_reference(|refname, status| {
        if let Some(msg) = status {
            *rejected.borrow_mut() = Some(format!("{refname}: {msg}"));
        }
        Ok(())
    });

    let push_result = {
        let mut opts = PushOptions::new();
        opts.remote_callbacks(cbs);
        remote.push(&[&refspec], Some(&mut opts))
    };
    push_result.map_err(|e| format!("push failed: {e} — {}", auth_hint(&url, token.is_some_and(|t| !t.trim().is_empty()))))?;
    drop(remote);

    if let Some(msg) = rejected.borrow().clone() {
        return Err(format!("push rejected: {msg}").into());
    }

    if let Ok(mut local) = repo.find_branch(&branch, git2::BranchType::Local)
        && local.upstream().is_err()
    {
        let _ = local.set_upstream(Some(&format!("origin/{branch}")));
    }

    Ok(format!("Pushed {branch} → origin"))
}
