//! ASCII art used on the splash screen.

/// The GitHub Octocat (the classic one served by the GitHub API).
pub const OCTOCAT: &str = r#"
               MMM.           .MMM
               MMMMMMMMMMMMMMMMMMM
               MMMMMMMMMMMMMMMMMMM      ____________________________
              MMMMMMMMMMMMMMMMMMMMM    |                            |
             MMMMMMMMMMMMMMMMMMMMMMM   |  Mind your git workflow.   |
            MMMMMMMMMMMMMMMMMMMMMMMM   |_   ________________________|
            MMMM::- -:::::::- -::MMMM    |/
             MM~:~ 00~:::::~ 00~:~MM
        .. MMMMM::.00:::+:::.00::MMMMM ..
              .MM::::: ._. :::::MM.
                 MMMM;:::::;MMMM
          -MM        MMMMMMM
          ^  M+     MMMMMMMMM
              MMMMMMM MM MM MM
                   MM MM MM MM
                   MM MM MM MM
                .~~MM~MM~MM~MM~~.
             ~~~~MM:~MM~~~MM~:MM~~~~
            ~~~~~~==~==~~~==~==~~~~~~
             ~~~~~~==~==~==~==~~~~~~
                 :~==~==~==~==~~
"#;

/// "GitMind" in a figlet-style font.
pub const WORDMARK: &str = r#"
   ____ _ _   __  __ _           _ 
  / ___(_) |_|  \/  (_)_ __   __| |
 | |  _| | __| |\/| | | '_ \ / _` |
 | |_| | | |_| |  | | | | | | (_| |
  \____|_|\__|_|  |_|_|_| |_|\__,_|
"#;

/// A compact one-line mark for the dashboard header.
pub const SMALL: &str = "◉ GitMind";

/// Lines of the art, each right-padded to the same width so the block can be
/// centred as a unit (per-line centring would skew the drawing).
pub fn lines(art: &str) -> Vec<String> {
    let mut raw: Vec<&str> = art.lines().skip_while(|l| l.trim().is_empty()).collect();
    while raw.last().is_some_and(|l| l.trim().is_empty()) {
        raw.pop();
    }
    let w = raw.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    raw.into_iter().map(|l| format!("{l:<w$}")).collect()
}

pub fn width(art: &str) -> u16 {
    lines(art).first().map(|l| l.chars().count()).unwrap_or(0) as u16
}

pub fn height(art: &str) -> u16 {
    lines(art).len() as u16
}
