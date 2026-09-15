use crate::config::MAX_DIFF_BYTES;

pub const COMMIT_PREAMBLE: &str = "\
You are an expert software engineer writing git commit messages.
Rules:
- First line: imperative mood, max 60 characters, no trailing period, \
  optionally prefixed with a conventional-commit type (feat, fix, refactor, docs, test, chore, perf, style).
- Then a blank line, then 1-4 short bullet points (starting with '- ') explaining WHAT changed and WHY.
- Describe only what the diff shows. Never invent changes.
- Output the commit message only: no code fences, no quotes, no preface, no explanation.";

/// Cut a diff down to `MAX_DIFF_BYTES` on a line boundary.
pub fn truncate_diff(diff: &str) -> String {
    if diff.len() <= MAX_DIFF_BYTES {
        return diff.to_string();
    }
    let mut cut = MAX_DIFF_BYTES;
    while cut > 0 && !diff.is_char_boundary(cut) {
        cut -= 1;
    }
    let head = &diff[..cut];
    let head = head.rsplit_once('\n').map(|(h, _)| h).unwrap_or(head);
    format!("{head}\n\n[... diff truncated, {} more bytes ...]", diff.len() - head.len())
}

pub fn commit_prompt(diff: &str, summary: &str) -> String {
    format!(
        "Write a commit message for the staged changes below.\n\n\
         Files changed:\n{summary}\n\n\
         Diff:\n```diff\n{}\n```",
        truncate_diff(diff)
    )
}
