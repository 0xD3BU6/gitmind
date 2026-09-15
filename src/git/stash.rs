use crate::error::Result;
use git2::{Repository, Signature, StashFlags};

pub fn stash_push(repo: &mut Repository, message: &str) -> Result<String> {
    let sig = repo
        .signature()
        .or_else(|_| Signature::now("gitmind", "gitmind@localhost"))?;
    let msg = if message.trim().is_empty() { "gitmind stash" } else { message.trim() };
    repo.stash_save(&sig, msg, Some(StashFlags::INCLUDE_UNTRACKED))
        .map_err(|e| format!("stash failed: {e}"))?;
    Ok(format!("Stashed: {msg}"))
}

pub fn stash_pop(repo: &mut Repository) -> Result<String> {
    let mut count = 0;
    repo.stash_foreach(|_, _, _| {
        count += 1;
        true
    })?;
    if count == 0 {
        return Err("no stash entries".into());
    }
    repo.stash_pop(0, None).map_err(|e| format!("stash pop failed: {e}"))?;
    Ok("Applied and dropped stash@{0}".to_string())
}

pub fn stash_count(repo: &mut Repository) -> usize {
    let mut n = 0;
    let _ = repo.stash_foreach(|_, _, _| {
        n += 1;
        true
    });
    n
}
