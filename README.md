# GitMind

A terminal UI for everyday git work: browse status, diffs, log and branches,
stage and commit, let an LLM draft the commit message from the staged diff,
and push to GitHub. Built with Rust, [ratatui] and [git2], with LLM access
through [rig-core]. Any OpenAI-compatible provider key works; the default endpoint is Groq.

```
cargo run --release              # splash → path prompt
cargo run --release -- ~/repo    # open a repo directly
```

## Bring your own keys

Press `,` anywhere to open settings. Values are saved to
`~/.config/gitmind/config.toml` with mode 600 and never leave your machine
except in requests to the provider you configured.

| Field         | Where to get it                                  | Env override    |
|---------------|--------------------------------------------------|-----------------|
| LLM API key   | from your LLM provider (e.g. console.groq.com/keys) | `LLM_API_KEY` |
| Model         | model id at the provider (default `openai/gpt-oss-120b`) | –       |
| GitHub token  | filled in by **Login with GitHub**, or paste a PAT (`repo` scope) | `GITHUB_TOKEN` |
| OAuth client  | client id of your GitHub OAuth App (see below)   | `GITHUB_CLIENT_ID` |

### Login with GitHub (OAuth device flow)

1. Create an OAuth App once at https://github.com/settings/developers →
   *New OAuth App*. Any name and URL; tick **Enable Device Flow**. No client
   secret is needed.
2. Put its *Client ID* in settings (`,` → OAuth client) and press `^L`, or
   press `L` from the splash or dashboard.
3. GitMind shows a one-time code and opens https://github.com/login/device.
   Approve it there; the token is stored automatically and the header shows
   your login name.

SSH remotes (`git@github.com:...`) use your running SSH agent instead of the
token. HTTPS remotes use the token, falling back to git's credential helper.

## Workflow

1. On the Status tab, `space` marks files, `v` marks all, `x` clears marks.
   `s` stages (or unstages) everything marked in one go, or just the
   highlighted file if nothing is marked. `a` / `u` stage / unstage all.
2. `c` opens the commit popup. If an LLM API key is set the model drafts a
   message from the staged diff; edit it freely, `^G` to regenerate.
3. `Enter` commits, `^P` commits and pushes.
4. `p` pushes the current branch to `origin` any time.

### More git workflow

| Key | Action |
|-----|--------|
| `R` | add `origin` or change its URL |
| `f` / `P` | fetch origin / pull (fast-forward only; diverged branches are refused) |
| `n` | new branch at HEAD, checked out |
| `D` | delete the highlighted branch (Branches tab; type its name to confirm) |
| `z` / `Z` | stash working tree (incl. untracked) / pop the latest stash |
| `Ctrl-I` | on the path prompt: `git init` a plain folder and open it |

`?` shows the full key map.

[ratatui]: https://ratatui.rs
[git2]: https://docs.rs/git2
[rig-core]: https://docs.rs/rig-core
