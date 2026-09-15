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
| GitHub token  | https://github.com/settings/tokens, `repo` scope | `GITHUB_TOKEN`  |

SSH remotes (`git@github.com:...`) use your running SSH agent instead of the
token. HTTPS remotes use the token, falling back to git's credential helper.

## Workflow

1. `s` / `a` stage files on the Status tab.
2. `c` opens the commit popup. If an LLM API key is set the model drafts a
   message from the staged diff; edit it freely, `^G` to regenerate.
3. `Enter` commits, `^P` commits and pushes.
4. `p` pushes the current branch to `origin` any time.

`?` shows the full key map.

[ratatui]: https://ratatui.rs
[git2]: https://docs.rs/git2
[rig-core]: https://docs.rs/rig-core
