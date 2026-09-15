//! GitHub OAuth *device flow*: the app shows a short code, the user enters it
//! at https://github.com/login/device, and we poll until GitHub hands back a
//! token. Needs an OAuth App client id with "Device Flow" enabled
//! (https://github.com/settings/developers). No client secret is required.

use crate::error::Result;
use serde::Deserialize;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

pub const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
pub const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
pub const USER_URL: &str = "https://api.github.com/user";
pub const SCOPE: &str = "repo";
const USER_AGENT: &str = concat!("gitmind/", env!("CARGO_PKG_VERSION"));

/// Progress reports from the background login thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginEvent {
    /// Show this code to the user and send them to `verification_uri`.
    Code {
        user_code: String,
        verification_uri: String,
        expires_in: u64,
    },
    /// Login finished: token plus the GitHub login name.
    Token { token: String, login: String },
    Error(String),
}

#[derive(Debug, Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(default = "default_expiry")]
    pub expires_in: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_expiry() -> u64 {
    900
}
fn default_interval() -> u64 {
    5
}

#[derive(Debug, Deserialize, Default)]
pub struct TokenReply {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
    #[serde(default)]
    pub interval: Option<u64>,
}

/// What to do after one poll of the token endpoint.
#[derive(Debug, PartialEq, Eq)]
pub enum Poll {
    Token(String),
    /// Keep polling, with this interval in seconds.
    Wait(u64),
    Fail(String),
}

/// Pure decision logic for a token-endpoint reply (unit-tested).
pub fn classify(reply: &TokenReply, current_interval: u64) -> Poll {
    if let Some(t) = &reply.access_token
        && !t.is_empty()
    {
        return Poll::Token(t.clone());
    }
    match reply.error.as_deref() {
        Some("authorization_pending") => Poll::Wait(current_interval),
        Some("slow_down") => Poll::Wait(reply.interval.unwrap_or(current_interval + 5)),
        Some("expired_token") => Poll::Fail("the code expired — start the login again".into()),
        Some("access_denied") => Poll::Fail("login was cancelled on GitHub".into()),
        Some(other) => Poll::Fail(format!(
            "{other}: {}",
            reply.error_description.clone().unwrap_or_default()
        )),
        None => Poll::Fail("unexpected reply from GitHub".into()),
    }
}

fn http() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(20))
        .build()?)
}

fn request_code(client: &reqwest::blocking::Client, client_id: &str) -> Result<DeviceCode> {
    let resp = client
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&[("client_id", client_id), ("scope", SCOPE)])
        .send()?;
    let status = resp.status();
    let text = resp.text()?;
    if !status.is_success() {
        return Err(format!("GitHub refused the device-code request ({status}): {text}").into());
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text)
        && let Some(err) = v.get("error").and_then(|e| e.as_str())
    {
        let desc = v.get("error_description").and_then(|d| d.as_str()).unwrap_or("");
        return Err(format!("{err}: {desc} (is Device Flow enabled on the OAuth app?)").into());
    }
    Ok(serde_json::from_str(&text)?)
}

fn poll_token(client: &reqwest::blocking::Client, client_id: &str, device_code: &str) -> Result<TokenReply> {
    let resp = client
        .post(TOKEN_URL)
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()?;
    Ok(resp.json()?)
}

fn whoami(client: &reqwest::blocking::Client, token: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct User {
        login: String,
    }
    let user: User = client
        .get(USER_URL)
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .send()?
        .error_for_status()?
        .json()?;
    Ok(user.login)
}

fn run(client_id: String, tx: &Sender<LoginEvent>) -> Result<()> {
    let client_id = client_id.trim().to_string();
    if client_id.is_empty() {
        return Err("no GitHub OAuth client id set — add one in settings (',')".into());
    }
    let client = http()?;
    let code = request_code(&client, &client_id)?;
    let _ = tx.send(LoginEvent::Code {
        user_code: code.user_code.clone(),
        verification_uri: code.verification_uri.clone(),
        expires_in: code.expires_in,
    });
    open_browser(&code.verification_uri);

    let deadline = Instant::now() + Duration::from_secs(code.expires_in);
    let mut interval = code.interval.max(1);
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_secs(interval));
        match classify(&poll_token(&client, &client_id, &code.device_code)?, interval) {
            Poll::Token(token) => {
                let login = whoami(&client, &token).unwrap_or_else(|_| "github".to_string());
                let _ = tx.send(LoginEvent::Token { token, login });
                return Ok(());
            }
            Poll::Wait(i) => interval = i.max(1),
            Poll::Fail(msg) => return Err(msg.into()),
        }
    }
    Err("the code expired — start the login again".into())
}

/// Best effort: open the verification page in the default browser.
fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let cmd = "xdg-open";
    let _ = std::process::Command::new(cmd)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Start the device flow on a background thread. Events arrive on the receiver.
pub fn spawn_device_login(client_id: String) -> Receiver<LoginEvent> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        if let Err(e) = run(client_id, &tx) {
            let _ = tx.send(LoginEvent::Error(e.to_string()));
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_replies() {
        let pending = TokenReply { error: Some("authorization_pending".into()), ..Default::default() };
        assert_eq!(classify(&pending, 5), Poll::Wait(5));
        let slow = TokenReply { error: Some("slow_down".into()), interval: Some(10), ..Default::default() };
        assert_eq!(classify(&slow, 5), Poll::Wait(10));
        let ok = TokenReply { access_token: Some("gho_x".into()), ..Default::default() };
        assert_eq!(classify(&ok, 5), Poll::Token("gho_x".into()));
        assert!(matches!(classify(&TokenReply { error: Some("expired_token".into()), ..Default::default() }, 5), Poll::Fail(_)));
        assert!(matches!(classify(&TokenReply::default(), 5), Poll::Fail(_)));
    }

    #[test]
    fn device_code_parses_with_defaults() {
        let d: DeviceCode = serde_json::from_str(
            r#"{"device_code":"d","user_code":"ABCD-1234","verification_uri":"https://github.com/login/device"}"#,
        )
        .unwrap();
        assert_eq!(d.user_code, "ABCD-1234");
        assert_eq!(d.interval, 5);
        assert_eq!(d.expires_in, 900);
    }
}
