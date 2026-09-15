use crate::ai;
use crate::github::{self, LoginEvent};
use crate::config::{LOG_LIMIT, Settings, TOAST_TTL};
use crate::error::Result;
use crate::git;
use crate::models::{BranchInfo, CommitInfo, FileChange, RepositoryInfo};
use crate::tui::events::handle_event;
use crate::tui::state::{PromptKind, State, Tab};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Instant;

/// Everything we know about the currently opened repository.
#[derive(Debug, Default)]
pub struct Session {
    pub path: PathBuf,
    pub info: RepositoryInfo,
    pub changes: Vec<FileChange>,
    pub commits: Vec<CommitInfo>,
    pub branches: Vec<BranchInfo>,
    pub tab: Tab,
    pub selected: [usize; 3],
    pub diff: String,
    pub diff_scroll: u16,
    pub stashes: usize,
    /// Paths marked for a bulk stage/unstage (Status tab).
    pub marked: HashSet<String>,
}

impl Session {
    pub fn is_marked(&self, path: &str) -> bool {
        self.marked.contains(path)
    }

    pub fn selected(&self) -> usize {
        self.selected[self.tab.index()]
    }

    fn list_len(&self) -> usize {
        match self.tab {
            Tab::Status => self.changes.len(),
            Tab::Log => self.commits.len(),
            Tab::Branches => self.branches.len(),
        }
    }

    pub fn selected_change(&self) -> Option<&FileChange> {
        self.changes.get(self.selected[Tab::Status.index()])
    }

    pub fn selected_commit(&self) -> Option<&CommitInfo> {
        self.commits.get(self.selected[Tab::Log.index()])
    }

    pub fn selected_branch(&self) -> Option<&BranchInfo> {
        self.branches.get(self.selected[Tab::Branches.index()])
    }

    pub fn staged_count(&self) -> usize {
        self.changes.iter().filter(|c| c.is_staged()).count()
    }

    /// One line per staged file, for the AI prompt.
    pub fn staged_summary(&self) -> String {
        self.changes
            .iter()
            .filter(|c| c.is_staged())
            .map(|c| format!("{} {}", c.index.map(|k| k.symbol()).unwrap_or(' '), c.path))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub is_error: bool,
    pub at: Instant,
}

/// What a background job is doing, so the UI can label it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
    /// Waiting for the model to propose a commit message.
    CommitMessage,
    /// Pushing to origin.
    Push,
    /// Fetching from origin.
    Fetch,
    /// Fetch + fast-forward.
    Pull,
}

/// A running background job (network work never blocks the UI thread).
pub struct Job {
    pub kind: JobKind,
    pub started: Instant,
    rx: Receiver<std::result::Result<String, String>>,
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Job").field("kind", &self.kind).finish()
    }
}

/// A GitHub device-flow login in progress.
pub struct LoginFlow {
    pub user_code: Option<String>,
    pub verification_uri: String,
    pub started: Instant,
    pub expires_in: u64,
    rx: Receiver<LoginEvent>,
}

impl std::fmt::Debug for LoginFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginFlow").field("user_code", &self.user_code).finish()
    }
}

/// The editable settings form.
#[derive(Debug, Clone, Default)]
pub struct SettingsForm {
    pub fields: [String; 4],
    pub focus: usize,
    pub reveal: bool,
}

impl SettingsForm {
    pub const LABELS: [&'static str; 4] = ["LLM API key", "Model", "GitHub token", "OAuth client"];
    pub const SECRET: [bool; 4] = [true, false, true, false];

    pub fn from_settings(s: &Settings) -> Self {
        Self {
            fields: [
                s.llm_api_key.clone(),
                s.model.clone(),
                s.github_token.clone(),
                s.github_client_id.clone(),
            ],
            focus: 0,
            reveal: false,
        }
    }

    pub fn to_settings(&self, base: &Settings) -> Settings {
        let token = self.fields[2].trim().to_string();
        Settings {
            llm_api_key: self.fields[0].trim().to_string(),
            model: self.fields[1].trim().to_string(),
            // a hand-edited token invalidates the remembered login name
            github_login: if token == base.github_token { base.github_login.clone() } else { String::new() },
            github_token: token,
            github_client_id: self.fields[3].trim().to_string(),
        }
    }
}

/// Application state shared between the event handler and the renderer.
#[derive(Debug, Default)]
pub struct App {
    pub state: State,
    /// Text currently typed into the path input.
    pub input: String,
    /// Commit message being composed.
    pub commit_msg: String,
    /// Commit immediately followed by a push.
    pub push_after_commit: bool,
    pub session: Option<Session>,
    pub toast: Option<Toast>,
    pub show_help: bool,
    /// Frame counter used for the splash animation.
    pub frame: u64,
    pub settings: Settings,
    pub form: SettingsForm,
    /// Where to return after closing settings.
    pub prev_state: State,
    pub job: Option<Job>,
    pub login: Option<LoginFlow>,
    /// Active text prompt, if any.
    pub prompt: Option<(PromptKind, String)>,
    /// Path that failed to open as a repo (offer `git init`).
    pub init_candidate: Option<PathBuf>,
}

impl App {
    /// Build an app and, if a path is given, open it right away.
    pub fn with_path(path: Option<PathBuf>) -> Self {
        let mut app = App {
            input: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            settings: Settings::load(),
            ..Default::default()
        };
        if let Some(p) = path {
            app.open(&p);
        }
        app
    }

    /// Poll for one event and update state. Returns `true` when the user wants to quit.
    pub fn tick(&mut self) -> Result<bool> {
        self.frame = self.frame.wrapping_add(1);
        if let Some(t) = &self.toast
            && t.at.elapsed() > TOAST_TTL
        {
            self.toast = None;
        }
        self.poll_job();
        self.poll_login();
        handle_event(self)
    }

    pub fn notify(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            is_error: false,
            at: Instant::now(),
        });
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            is_error: true,
            at: Instant::now(),
        });
    }

    pub fn busy(&self) -> bool {
        self.job.is_some()
    }

    // ------------------------------------------------------------ repo --

    /// Open a repository and switch to the dashboard.
    pub fn open(&mut self, path: &Path) {
        match git::open_repo(path) {
            Ok(repo) => {
                let root = repo.workdir().unwrap_or(repo.path()).to_path_buf();
                self.session = Some(Session {
                    path: root.clone(),
                    ..Default::default()
                });
                self.state = State::Dashboard;
                self.refresh();
                self.notify(format!("Opened {}", root.display()));
            }
            Err(e) => {
                let is_dir = path.is_dir();
                self.init_candidate = is_dir.then(|| path.to_path_buf());
                if is_dir {
                    self.error(format!("{e} — Ctrl-I to run git init here"));
                } else {
                    self.error(e.to_string());
                }
            }
        }
    }

    /// `git init` the last path that failed to open, then open it.
    pub fn init_here(&mut self) {
        let Some(path) = self.init_candidate.take() else {
            self.error("Enter a directory path first");
            return;
        };
        match git::init_repo(&path) {
            Ok(_) => {
                self.open(&path);
                self.notify(format!("Initialised repository at {}", path.display()));
            }
            Err(e) => self.error(e.to_string()),
        }
    }

    /// Re-read everything from disk.
    pub fn refresh(&mut self) {
        let Some(s) = self.session.as_mut() else {
            return;
        };
        let result = (|| -> Result<()> {
            let repo = git::open_repo(&s.path)?;
            s.info = git::repo_info(&repo)?;
            s.changes = git::changes(&repo)?;
            s.commits = git::recent_commits(&repo, LOG_LIMIT)?;
            s.branches = git::list_branches(&repo)?;
            let mut repo = repo;
            s.stashes = git::stash_count(&mut repo);
            Ok(())
        })();
        let present: HashSet<String> = s.changes.iter().map(|c| c.path.clone()).collect();
        s.marked.retain(|p| present.contains(p));
        for (i, len) in [s.changes.len(), s.commits.len(), s.branches.len()]
            .into_iter()
            .enumerate()
        {
            s.selected[i] = s.selected[i].min(len.saturating_sub(1));
        }
        if let Err(e) = result {
            self.error(e.to_string());
        }
        self.load_diff();
    }

    /// Load the diff for the highlighted file.
    pub fn load_diff(&mut self) {
        let Some(s) = self.session.as_mut() else {
            return;
        };
        s.diff_scroll = 0;
        let Some(change) = s.selected_change().cloned() else {
            s.diff = if s.changes.is_empty() {
                "✔ Working tree is clean — nothing to show.".to_string()
            } else {
                String::new()
            };
            return;
        };
        let staged = !change.has_worktree_changes() && change.is_staged();
        s.diff = git::open_repo(&s.path)
            .and_then(|repo| git::diff_for_file(&repo, &change.path, staged))
            .unwrap_or_else(|e| format!("error: {e}"));
    }

    pub fn move_selection(&mut self, delta: isize) {
        let Some(s) = self.session.as_mut() else {
            return;
        };
        let len = s.list_len();
        if len == 0 {
            return;
        }
        let i = s.tab.index();
        let cur = s.selected[i] as isize;
        s.selected[i] = (cur + delta).rem_euclid(len as isize) as usize;
        if s.tab == Tab::Status {
            self.load_diff();
        }
    }

    pub fn jump_selection(&mut self, to_end: bool) {
        let Some(s) = self.session.as_mut() else {
            return;
        };
        let len = s.list_len();
        let i = s.tab.index();
        s.selected[i] = if to_end { len.saturating_sub(1) } else { 0 };
        if s.tab == Tab::Status {
            self.load_diff();
        }
    }

    pub fn scroll_diff(&mut self, delta: i32) {
        if let Some(s) = self.session.as_mut() {
            s.diff_scroll = (i32::from(s.diff_scroll) + delta).max(0) as u16;
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        if let Some(s) = self.session.as_mut() {
            s.tab = tab;
        }
    }

    pub fn next_tab(&mut self, backwards: bool) {
        if let Some(s) = self.session.as_mut() {
            s.tab = if backwards { s.tab.prev() } else { s.tab.next() };
        }
    }

    // --------------------------------------------------------- staging --

    /// Stage the highlighted file if it has worktree changes, otherwise unstage it.
    pub fn toggle_stage(&mut self) {
        let Some(s) = &self.session else { return };
        let Some(change) = s.selected_change().cloned() else {
            return;
        };
        let path = s.path.clone();
        let result = git::open_repo(&path).and_then(|repo| {
            if change.has_worktree_changes() {
                git::stage_path(&repo, &change.path).map(|_| format!("Staged {}", change.path))
            } else {
                git::unstage_path(&repo, &change.path).map(|_| format!("Unstaged {}", change.path))
            }
        });
        self.finish(result);
    }

    /// Mark/unmark the highlighted file and move down one row.
    pub fn toggle_mark(&mut self) {
        let Some(s) = self.session.as_mut() else { return };
        let Some(path) = s.selected_change().map(|c| c.path.clone()) else {
            return;
        };
        if !s.marked.remove(&path) {
            s.marked.insert(path);
        }
        self.move_selection(1);
    }

    /// Mark every file, or clear all marks if everything is already marked.
    pub fn toggle_mark_all(&mut self) {
        let Some(s) = self.session.as_mut() else { return };
        if s.marked.len() == s.changes.len() {
            s.marked.clear();
        } else {
            s.marked = s.changes.iter().map(|c| c.path.clone()).collect();
        }
    }

    pub fn clear_marks(&mut self) {
        if let Some(s) = self.session.as_mut() {
            s.marked.clear();
        }
    }

    /// Stage or unstage every marked file in one go. If any marked file still
    /// has worktree changes they all get staged; otherwise they get unstaged.
    pub fn apply_marked(&mut self) {
        let Some(s) = &self.session else { return };
        let files: Vec<FileChange> = s
            .changes
            .iter()
            .filter(|c| s.marked.contains(&c.path))
            .cloned()
            .collect();
        if files.is_empty() {
            self.toggle_stage();
            return;
        }
        let staging = files.iter().any(|c| c.has_worktree_changes());
        let path = s.path.clone();
        let result = git::open_repo(&path).and_then(|repo| {
            let mut n = 0;
            for f in &files {
                if staging && f.has_worktree_changes() {
                    git::stage_path(&repo, &f.path)?;
                    n += 1;
                } else if !staging && f.is_staged() {
                    git::unstage_path(&repo, &f.path)?;
                    n += 1;
                }
            }
            Ok(format!("{} {n} file(s)", if staging { "Staged" } else { "Unstaged" }))
        });
        if result.is_ok()
            && let Some(s) = self.session.as_mut()
        {
            s.marked.clear();
        }
        self.finish(result);
    }

    pub fn stage_all(&mut self) {
        let Some(s) = &self.session else { return };
        let result = git::open_repo(&s.path)
            .and_then(|repo| git::stage_all(&repo))
            .map(|_| "Staged all changes".to_string());
        self.finish(result);
    }

    pub fn unstage_all(&mut self) {
        let Some(s) = &self.session else { return };
        let result = git::open_repo(&s.path)
            .and_then(|repo| git::unstage_all(&repo))
            .map(|_| "Unstaged all changes".to_string());
        self.finish(result);
    }

    // -------------------------------------------------------- committing --

    /// Open the commit popup; ask the model for a message if a key is configured.
    pub fn begin_commit(&mut self) {
        let staged = self.session.as_ref().map(|s| s.staged_count()).unwrap_or(0);
        if staged == 0 {
            self.error("Nothing staged — press 's' on a file or 'a' to stage all");
            return;
        }
        self.state = State::Committing;
        self.push_after_commit = false;
        if self.settings.has_ai() {
            self.generate_message();
        } else {
            self.notify("No LLM API key set — type a message, or press ',' to add a key");
        }
    }

    /// Ask the model for a commit message based on the staged diff.
    pub fn generate_message(&mut self) {
        if self.busy() {
            self.error("Still working on the previous request");
            return;
        }
        if !self.settings.has_ai() {
            self.error("No LLM API key — press ',' to open settings");
            return;
        }
        let Some(s) = &self.session else { return };
        let diff = match git::open_repo(&s.path).and_then(|repo| git::staged_diff(&repo)) {
            Ok(d) => d,
            Err(e) => {
                self.error(e.to_string());
                return;
            }
        };
        let rx = ai::spawn_commit_message(self.settings.clone(), diff, s.staged_summary());
        self.job = Some(Job {
            kind: JobKind::CommitMessage,
            started: Instant::now(),
            rx,
        });
    }

    pub fn commit(&mut self) {
        let Some(s) = &self.session else { return };
        let msg = self.commit_msg.clone();
        let result = git::open_repo(&s.path)
            .and_then(|repo| git::commit(&repo, &msg))
            .map(|id| format!("Committed {id}"));
        let ok = result.is_ok();
        if ok {
            self.commit_msg.clear();
            self.state = State::Dashboard;
        }
        self.finish(result);
        if ok && self.push_after_commit {
            self.push_after_commit = false;
            self.push();
        }
    }

    // ------------------------------------------------------------- push --

    pub fn push(&mut self) {
        self.network(JobKind::Push, |repo, token| git::push(repo, Some(token)));
    }

    pub fn fetch(&mut self) {
        self.network(JobKind::Fetch, |repo, token| git::fetch(repo, Some(token)));
    }

    pub fn pull(&mut self) {
        self.network(JobKind::Pull, |repo, token| git::pull(repo, Some(token)));
    }

    /// Run a git network operation on a background thread.
    fn network<F>(&mut self, kind: JobKind, op: F)
    where
        F: FnOnce(&git2::Repository, &str) -> Result<String> + Send + 'static,
    {
        if self.busy() {
            self.error("Still working on the previous request");
            return;
        }
        let Some(s) = &self.session else { return };
        let path = s.path.clone();
        let token = self.settings.github_token.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = git::open_repo(&path)
                .and_then(|repo| op(&repo, &token))
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
        self.job = Some(Job {
            kind,
            started: Instant::now(),
            rx,
        });
    }

    /// Collect the result of a finished background job, if any.
    fn poll_job(&mut self) {
        let Some(job) = &self.job else { return };
        let outcome = match job.rx.try_recv() {
            Ok(r) => r,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("background task died".to_string()),
        };
        let kind = job.kind;
        self.job = None;
        match (kind, outcome) {
            (JobKind::CommitMessage, Ok(msg)) => {
                self.commit_msg = msg;
                self.notify(format!("Proposal from {}", self.settings.model));
            }
            (JobKind::Push | JobKind::Fetch | JobKind::Pull, Ok(msg)) => {
                self.notify(msg);
                self.refresh();
            }
            (_, Err(e)) => self.error(e),
        }
    }

    // ----------------------------------------------------------- prompts --

    pub fn open_prompt(&mut self, kind: PromptKind) {
        if self.session.is_none() {
            return;
        }
        let prefill = match kind {
            PromptKind::SetOrigin => self
                .session
                .as_ref()
                .and_then(|s| s.info.remote.clone())
                .unwrap_or_default(),
            _ => String::new(),
        };
        self.prompt = Some((kind, prefill));
        self.prev_state = self.state;
        self.state = State::Prompt;
    }

    pub fn cancel_prompt(&mut self) {
        self.prompt = None;
        self.state = State::Dashboard;
    }

    /// Enter pressed in a prompt: run the matching action.
    pub fn submit_prompt(&mut self) {
        let Some((kind, value)) = self.prompt.take() else { return };
        self.state = State::Dashboard;
        let Some(s) = &self.session else { return };
        let path = s.path.clone();
        let value = value.trim().to_string();
        let result = match kind {
            PromptKind::NewBranch => git::open_repo(&path)
                .and_then(|r| git::create_branch(&r, &value))
                .map(|_| format!("Created and switched to {value}")),
            PromptKind::SetOrigin => git::open_repo(&path).and_then(|r| git::set_origin(&r, &value)),
            PromptKind::StashMessage => git::open_repo(&path).and_then(|mut r| git::stash_push(&mut r, &value)),
            PromptKind::DeleteBranch => {
                let target = s.selected_branch().map(|b| b.name.clone()).unwrap_or_default();
                if value != target {
                    Err(format!("type '{target}' to confirm deleting it").into())
                } else {
                    git::open_repo(&path)
                        .and_then(|r| git::delete_branch(&r, &target))
                        .map(|_| format!("Deleted branch {target}"))
                }
            }
        };
        self.finish(result);
    }

    pub fn delete_selected_branch(&mut self) {
        let Some(s) = &self.session else { return };
        match s.selected_branch() {
            None => {}
            Some(b) if b.is_head => self.error("Cannot delete the current branch"),
            Some(_) => self.open_prompt(PromptKind::DeleteBranch),
        }
    }

    pub fn stash_pop(&mut self) {
        let Some(s) = &self.session else { return };
        let result = git::open_repo(&s.path).and_then(|mut r| git::stash_pop(&mut r));
        self.finish(result);
    }

    // ------------------------------------------------------------ login --

    /// Start "Login with GitHub" (OAuth device flow).
    pub fn begin_login(&mut self) {
        if self.login.is_some() {
            return;
        }
        if !self.settings.has_client_id() {
            self.error("Set a GitHub OAuth client id first (settings ',' → OAuth client)");
            return;
        }
        let rx = github::spawn_device_login(self.settings.github_client_id.clone());
        self.login = Some(LoginFlow {
            user_code: None,
            verification_uri: "https://github.com/login/device".to_string(),
            started: Instant::now(),
            expires_in: 900,
            rx,
        });
        if self.state != State::Login {
            self.prev_state = if self.state == State::Settings { self.prev_state } else { self.state };
            self.state = State::Login;
        }
    }

    pub fn cancel_login(&mut self) {
        self.login = None; // dropping the receiver makes the thread's next send fail harmlessly
        self.state = match self.prev_state {
            State::Login | State::Settings => State::Dashboard,
            other => other,
        };
        if self.state == State::Dashboard && self.session.is_none() {
            self.state = State::Splash;
        }
    }

    fn poll_login(&mut self) {
        let Some(flow) = &self.login else { return };
        let ev = match flow.rx.try_recv() {
            Ok(ev) => ev,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => LoginEvent::Error("login thread died".into()),
        };
        match ev {
            LoginEvent::Code { user_code, verification_uri, expires_in } => {
                if let Some(flow) = self.login.as_mut() {
                    flow.user_code = Some(user_code);
                    flow.verification_uri = verification_uri;
                    flow.expires_in = expires_in;
                }
            }
            LoginEvent::Token { token, login } => {
                self.settings.github_token = token;
                self.settings.github_login = login.clone();
                match self.settings.save() {
                    Ok(_) => self.notify(format!("Logged in to GitHub as {login}")),
                    Err(e) => self.error(format!("logged in, but could not save: {e}")),
                }
                self.cancel_login();
            }
            LoginEvent::Error(e) => {
                self.error(e);
                self.cancel_login();
            }
        }
    }

    // --------------------------------------------------------- branches --

    pub fn checkout_selected(&mut self) {
        let Some(s) = &self.session else { return };
        let Some(branch) = s.selected_branch().cloned() else {
            return;
        };
        if branch.is_head {
            self.notify(format!("Already on {}", branch.name));
            return;
        }
        let result = git::open_repo(&s.path)
            .and_then(|repo| git::checkout_branch(&repo, &branch.name))
            .map(|_| format!("Switched to {}", branch.name));
        self.finish(result);
    }

    /// Show the outcome of a git operation and refresh the view.
    fn finish(&mut self, result: Result<String>) {
        match result {
            Ok(msg) => self.notify(msg),
            Err(e) => self.error(e.to_string()),
        }
        self.refresh();
    }

    /// Leave the dashboard and go back to the path prompt.
    pub fn close_repo(&mut self) {
        if let Some(s) = &self.session {
            self.input = s.path.display().to_string();
        }
        self.session = None;
        self.state = State::Inputting;
    }

    // --------------------------------------------------------- settings --

    pub fn open_settings(&mut self) {
        self.form = SettingsForm::from_settings(&self.settings);
        self.prev_state = self.state;
        self.state = State::Settings;
    }

    pub fn close_settings(&mut self, save: bool) {
        if save {
            let s = self.form.to_settings(&self.settings);
            match s.save() {
                Ok(p) => {
                    self.settings = s;
                    self.notify(format!("Saved settings to {}", p.display()));
                }
                Err(e) => {
                    self.error(format!("could not save settings: {e}"));
                    return;
                }
            }
        }
        self.state = match self.prev_state {
            State::Settings | State::Committing => State::Dashboard,
            other => other,
        };
        if self.state == State::Dashboard && self.session.is_none() {
            self.state = State::Inputting;
        }
    }
}
