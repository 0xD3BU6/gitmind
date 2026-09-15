//! Remotes, fetch and pull.

use crate::error::Result;
use crate::git::auth::{auth_hint, callbacks};
use git2::{FetchOptions, Repository, build::CheckoutBuilder};

/// Add `origin` or change its URL.
pub fn set_origin(repo: &Repository, url: &str) -> Result<String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("remote URL is empty".into());
    }
    if repo.find_remote("origin").is_ok() {
        repo.remote_set_url("origin", url)?;
        Ok(format!("origin → {url}"))
    } else {
        repo.remote("origin", url)?;
        Ok(format!("Added origin {url}"))
    }
}

/// Fetch every branch from origin. Returns the number of refs updated.
pub fn fetch(repo: &Repository, token: Option<&str>) -> Result<String> {
    let mut remote = repo
        .find_remote("origin")
        .map_err(|_| "no 'origin' remote — press R to add one")?;
    let url = remote.url()?.to_string();
    let config = repo.config()?;
    let mut opts = FetchOptions::new();
    opts.remote_callbacks(callbacks(&config, token));
    opts.prune(git2::FetchPrune::On);
    remote
        .fetch(&["+refs/heads/*:refs/remotes/origin/*"], Some(&mut opts), Some("gitmind fetch"))
        .map_err(|e| format!("fetch failed: {e} — {}", auth_hint(&url, token.is_some_and(|t| !t.trim().is_empty()))))?;
    let stats = remote.stats();
    Ok(format!(
        "Fetched origin ({} objects, {} new)",
        stats.total_objects(),
        stats.received_objects()
    ))
}

/// Fetch, then fast-forward the current branch to its upstream.
/// Refuses anything that would need a real merge.
pub fn pull(repo: &Repository, token: Option<&str>) -> Result<String> {
    fetch(repo, token)?;

    let head = repo.head().map_err(|_| "no commits yet")?;
    if !head.is_branch() {
        return Err("detached HEAD — check out a branch first".into());
    }
    let branch = head.shorthand()?.to_string();
    let upstream_name = format!("refs/remotes/origin/{branch}");
    let upstream = repo
        .find_reference(&upstream_name)
        .map_err(|_| format!("origin has no branch '{branch}' yet — push first"))?;
    let their = repo.reference_to_annotated_commit(&upstream)?;
    let (analysis, _) = repo.merge_analysis(&[&their])?;

    if analysis.is_up_to_date() {
        return Ok(format!("{branch} is up to date"));
    }
    if !analysis.is_fast_forward() {
        return Err(format!(
            "{branch} and origin/{branch} have diverged — merge or rebase with git"
        )
        .into());
    }
    let target = their.id();
    // Check the new tree out first (HEAD is still the old commit, so libgit2
    // sees the real differences), then move the branch reference.
    let obj = repo.find_object(target, None)?;
    let mut cb = CheckoutBuilder::new();
    cb.safe();
    repo.checkout_tree(&obj, Some(&mut cb))
        .map_err(|e| format!("cannot fast-forward: {e} (local changes in the way?)"))?;
    let mut local = repo.find_reference(&format!("refs/heads/{branch}"))?;
    local.set_target(target, "gitmind pull: fast-forward")?;
    repo.set_head(&format!("refs/heads/{branch}"))?;
    Ok(format!("Fast-forwarded {branch} to {}", &target.to_string()[..7]))
}
