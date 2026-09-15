use crate::models::Proposal;

/// Turn a raw model reply into a title + body, stripping fences and quotes.
pub fn parse_proposal(raw: &str) -> Proposal {
    let mut text = raw.trim();
    // strip ``` fences
    if let Some(rest) = text.strip_prefix("```") {
        let rest = rest.split_once('\n').map(|(_, r)| r).unwrap_or(rest);
        text = rest.strip_suffix("```").unwrap_or(rest).trim();
    }
    // drop a leading "Commit message:" style preface
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("commit message:") {
        text = text[15..].trim();
    }
    let mut lines = text.lines().map(str::trim_end);
    let title = lines
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(|c| c == '"' || c == '`' || c == '\'')
        .trim_end_matches('.')
        .to_string();
    let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    Proposal { title, body }
}

impl Proposal {
    /// Full commit message text.
    pub fn to_message(&self) -> String {
        if self.body.is_empty() {
            self.title.clone()
        } else {
            format!("{}\n\n{}", self.title, self.body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fences_and_quotes() {
        let p = parse_proposal("```\n\"feat: add thing.\"\n\n- one\n- two\n```");
        assert_eq!(p.title, "feat: add thing");
        assert_eq!(p.body, "- one\n- two");
        assert_eq!(p.to_message(), "feat: add thing\n\n- one\n- two");
    }

    #[test]
    fn title_only() {
        let p = parse_proposal("Commit message: fix crash on empty repo");
        assert_eq!(p.title, "fix crash on empty repo");
        assert!(p.body.is_empty());
    }
}
