use crate::error::Result;
use git2::{Cred, CredentialType, PushOptions, RemoteCallbacks, Repository};
use std::cell::{Cell, RefCell};

/// Push the current branch to `origin`, setting upstream if needed.
///
/// Credentials are tried in order: SSH agent (for ssh remotes), the GitHub
/// token (for https remotes), then git's own credential helper.
pub fn push(repo: &Repository, token: Option<&str>) -> Result<String> {
    let head = repo.head().map_err(|_| "nothing to push: no commits yet")?;
    if !head.is_branch() {
        return Err("detached HEAD — check out a branch first".into());
    }
    let branch = head.shorthand()?.to_string();
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");

    let mut remote = repo
        .find_remote("origin")
        .map_err(|_| "no 'origin' remote configured")?;
    let url = remote.url()?.to_string();
    let is_ssh = url.starts_with("git@") || url.starts_with("ssh://");

    let config = repo.config()?;
    let attempts = Cell::new(0u32);
    let token = token.map(str::trim).filter(|t| !t.is_empty()).map(str::to_string);

    let rejected: RefCell<Option<String>> = RefCell::new(None);
    let mut cbs = RemoteCallbacks::new();
    cbs.credentials(|url, username, allowed| {
        let n = attempts.get();
        attempts.set(n + 1);
        if n > 3 {
            return Err(git2::Error::from_str("authentication failed after several attempts"));
        }
        if allowed.contains(CredentialType::SSH_KEY) {
            return Cred::ssh_key_from_agent(username.unwrap_or("git"));
        }
        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
            if let (Some(t), 0) = (&token, n) {
                // GitHub accepts a PAT as the password with any username.
                return Cred::userpass_plaintext("x-access-token", t);
            }
            return Cred::credential_helper(&config, url, username);
        }
        Cred::default()
    });

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
    push_result.map_err(|e| {
        let hint = if is_ssh {
            "check that your SSH agent has a key GitHub knows"
        } else if token.is_some() {
            "check the GitHub token in settings (',')"
        } else {
            "add a GitHub token in settings (',') or use an ssh remote"
        };
        format!("push failed: {e} — {hint}")
    })?;
    drop(remote);

    if let Some(msg) = rejected.borrow().clone() {
        return Err(format!("push rejected: {msg}").into());
    }

    // make sure the branch tracks origin/<branch>
    if let Ok(mut local) = repo.find_branch(&branch, git2::BranchType::Local)
        && local.upstream().is_err()
    {
        let _ = local.set_upstream(Some(&format!("origin/{branch}")));
    }

    Ok(format!("Pushed {branch} → origin"))
}
