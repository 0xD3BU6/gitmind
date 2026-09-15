//! Runtime constants and persisted user settings ("bring your own key").

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// How long the event loop waits for a key press before redrawing.
pub const TICK_RATE: Duration = Duration::from_millis(80);

/// Number of commits shown in the log tab.
pub const LOG_LIMIT: usize = 100;

/// Duration a toast/status message stays visible.
pub const TOAST_TTL: Duration = Duration::from_secs(4);

/// Default model id used for commit messages.
pub const DEFAULT_MODEL: &str = "openai/gpt-oss-120b";

/// Largest diff (in bytes) sent to the model; longer diffs are truncated.
pub const MAX_DIFF_BYTES: usize = 14_000;

/// Persisted settings. Secrets live only in this file (mode 600) or in the
/// environment; they are never written anywhere else.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    /// LLM provider API key. Env override: LLM_API_KEY (GROQ_API_KEY also accepted).
    pub llm_api_key: String,
    /// Model id at the provider.
    pub model: String,
    /// GitHub token used for HTTPS pushes (from OAuth login or a PAT). Env override: GITHUB_TOKEN.
    pub github_token: String,
    /// OAuth App client id used for "Login with GitHub" (device flow).
    /// Env override: GITHUB_CLIENT_ID.
    pub github_client_id: String,
    /// GitHub login name learned at OAuth time (display only).
    pub github_login: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            llm_api_key: String::new(),
            model: DEFAULT_MODEL.to_string(),
            github_token: String::new(),
            github_client_id: String::new(),
            github_login: String::new(),
        }
    }
}

impl Settings {
    /// `$GITMIND_CONFIG_DIR/config.toml`, else the platform config dir.
    pub fn path() -> Option<PathBuf> {
        if let Ok(dir) = std::env::var("GITMIND_CONFIG_DIR") {
            return Some(PathBuf::from(dir).join("config.toml"));
        }
        dirs::config_dir().map(|d| d.join("gitmind").join("config.toml"))
    }

    /// Load from disk, then let environment variables override.
    pub fn load() -> Self {
        let mut s = Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|txt| toml::from_str::<Settings>(&txt).ok())
            .unwrap_or_default();
        if let Some(k) = ["LLM_API_KEY", "GROQ_API_KEY"]
            .iter()
            .filter_map(|v| std::env::var(v).ok())
            .find(|k| !k.trim().is_empty())
        {
            s.llm_api_key = k.trim().to_string();
        }
        if let Ok(t) = std::env::var("GITHUB_TOKEN")
            && !t.trim().is_empty()
        {
            s.github_token = t.trim().to_string();
        }
        if let Ok(c) = std::env::var("GITHUB_CLIENT_ID")
            && !c.trim().is_empty()
        {
            s.github_client_id = c.trim().to_string();
        }
        if s.model.trim().is_empty() {
            s.model = DEFAULT_MODEL.to_string();
        }
        s
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::path().ok_or("cannot determine a config directory")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(path)
    }

    pub fn has_ai(&self) -> bool {
        !self.llm_api_key.trim().is_empty()
    }

    pub fn has_github_token(&self) -> bool {
        !self.github_token.trim().is_empty()
    }

    pub fn has_client_id(&self) -> bool {
        !self.github_client_id.trim().is_empty()
    }
}

/// Show only the tail of a secret, e.g. `••••••3Np`.
pub fn mask(secret: &str) -> String {
    let n = secret.chars().count();
    if n == 0 {
        return String::new();
    }
    let keep = 4.min(n);
    let tail: String = secret.chars().skip(n - keep).collect();
    format!("{}{tail}", "•".repeat(n - keep))
}
