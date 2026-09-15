use crate::error::Result;
use git2::{Diff, DiffFormat, DiffOptions, Repository};

fn render(diff: &Diff) -> Result<String> {
    let mut out = String::new();
    diff.print(DiffFormat::Patch, |_, _, line| {
        let origin = line.origin();
        if matches!(origin, '+' | '-' | ' ') {
            out.push(origin);
        }
        out.push_str(&String::from_utf8_lossy(line.content()));
        true
    })?;
    if out.is_empty() {
        out.push_str("(no textual changes)");
    }
    Ok(out)
}

fn head_tree(repo: &Repository) -> Option<git2::Tree<'_>> {
    repo.head().ok()?.peel_to_tree().ok()
}

/// Diff for one file. `staged` selects index-vs-HEAD, otherwise worktree-vs-index.
pub fn diff_for_file(repo: &Repository, path: &str, staged: bool) -> Result<String> {
    let mut opts = DiffOptions::new();
    opts.pathspec(path)
        .include_untracked(true)
        .show_untracked_content(true)
        .recurse_untracked_dirs(true);

    let diff = if staged {
        repo.diff_tree_to_index(head_tree(repo).as_ref(), None, Some(&mut opts))?
    } else {
        repo.diff_index_to_workdir(None, Some(&mut opts))?
    };
    render(&diff)
}

/// Full diff of the working tree against HEAD.
pub fn working_tree_diff(repo: &Repository) -> Result<String> {
    let mut opts = DiffOptions::new();
    opts.include_untracked(true).show_untracked_content(true);
    let diff = repo.diff_tree_to_workdir_with_index(head_tree(repo).as_ref(), Some(&mut opts))?;
    render(&diff)
}

/// Diff of the index against HEAD: exactly what a commit would contain.
pub fn staged_diff(repo: &Repository) -> Result<String> {
    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_index(head_tree(repo).as_ref(), None, Some(&mut opts))?;
    let text = render(&diff)?;
    if text == "(no textual changes)" {
        return Err("nothing staged to describe".into());
    }
    Ok(text)
}
