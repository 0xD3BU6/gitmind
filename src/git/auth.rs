//! Credential callbacks shared by push, fetch and pull.

use git2::{Config, Cred, CredentialType, RemoteCallbacks};
use std::cell::Cell;

/// Credentials are tried in order: SSH agent (for ssh remotes), the GitHub
/// token (for https remotes), then git's own credential helper.
pub fn callbacks<'a>(config: &'a Config, token: Option<&'a str>) -> RemoteCallbacks<'a> {
    let attempts = Cell::new(0u32);
    let token = token.map(str::trim).filter(|t| !t.is_empty());
    let mut cbs = RemoteCallbacks::new();
    cbs.credentials(move |url, username, allowed| {
        let n = attempts.get();
        attempts.set(n + 1);
        if n > 3 {
            return Err(git2::Error::from_str("authentication failed after several attempts"));
        }
        if allowed.contains(CredentialType::SSH_KEY) {
            return Cred::ssh_key_from_agent(username.unwrap_or("git"));
        }
        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
            if let (Some(t), 0) = (token, n) {
                // GitHub accepts a token as the password with any username.
                return Cred::userpass_plaintext("x-access-token", t);
            }
            return Cred::credential_helper(config, url, username);
        }
        Cred::default()
    });
    cbs
}

/// A hint to show next to an auth failure.
pub fn auth_hint(url: &str, has_token: bool) -> &'static str {
    let is_ssh = url.starts_with("git@") || url.starts_with("ssh://");
    if is_ssh {
        "check that your SSH agent has a key GitHub knows"
    } else if has_token {
        "check the GitHub token in settings (',')"
    } else {
        "log in with GitHub (L) or use an ssh remote"
    }
}
