use gitmind::ai::parse_proposal;
use gitmind::ai::prompts::{commit_prompt, truncate_diff};
use gitmind::config::MAX_DIFF_BYTES;

#[test]
fn prompt_includes_summary_and_diff() {
    let p = commit_prompt("+fn main() {}", "A src/main.rs");
    assert!(p.contains("A src/main.rs"));
    assert!(p.contains("+fn main() {}"));
}

#[test]
fn long_diffs_are_truncated_on_a_line_boundary() {
    let line = "+".repeat(99) + "\n";
    let big = line.repeat(MAX_DIFF_BYTES / 100 + 50);
    let t = truncate_diff(&big);
    assert!(t.len() < big.len());
    assert!(t.contains("diff truncated"));
    assert!(t.lines().next().unwrap().len() == 99);
}

#[test]
fn proposal_message_layout() {
    let p = parse_proposal("feat(ui): add settings popup\n\n- store keys locally\n- mask secrets");
    assert_eq!(p.title, "feat(ui): add settings popup");
    assert_eq!(p.to_message(), "feat(ui): add settings popup\n\n- store keys locally\n- mask secrets");
}
