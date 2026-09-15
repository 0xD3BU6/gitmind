use crate::error::Result;
use git2::{IndexAddOption, Repository, Signature};
use std::path::Path;

pub fn stage_path(repo: &Repository, path: &str) -> Result<()> {
    let mut index = repo.index()?;
    let workdir = repo.workdir().ok_or("bare repository")?;
    if workdir.join(path).exists() {
        index.add_path(Path::new(path))?;
    } else {
        index.remove_path(Path::new(path))?;
    }
    index.write()?;
    Ok(())
}

pub fn unstage_path(repo: &Repository, path: &str) -> Result<()> {
    match repo.head().ok().and_then(|h| h.peel(git2::ObjectType::Commit).ok()) {
        Some(head) => repo.reset_default(Some(&head), [path])?,
        None => {
            // No commits yet: unstaging just means removing from the index.
            let mut index = repo.index()?;
            index.remove_path(Path::new(path))?;
            index.write()?;
        }
    }
    Ok(())
}

pub fn stage_all(repo: &Repository) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
    index.update_all(["*"], None)?;
    index.write()?;
    Ok(())
}

pub fn unstage_all(repo: &Repository) -> Result<()> {
    match repo.head().ok().and_then(|h| h.peel(git2::ObjectType::Commit).ok()) {
        Some(head) => repo.reset_default(Some(&head), ["*"])?,
        None => {
            let mut index = repo.index()?;
            index.clear()?;
            index.write()?;
        }
    }
    Ok(())
}

/// Commit whatever is currently staged. Returns the short id of the new commit.
pub fn commit(repo: &Repository, message: &str) -> Result<String> {
    let message = message.trim();
    if message.is_empty() {
        return Err("commit message is empty".into());
    }
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    if let Some(p) = &parent
        && p.tree_id() == tree_oid
    {
        return Err("nothing staged to commit".into());
    }
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    let sig = repo
        .signature()
        .or_else(|_| Signature::now("gitmind", "gitmind@localhost"))?;
    let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(oid.to_string()[..7].to_string())
}
