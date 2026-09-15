use crate::error::Result;
use crate::models::CommitInfo;
use git2::{Repository, Time};

pub fn format_time(t: Time) -> String {
    let secs = t.seconds() + i64::from(t.offset_minutes()) * 60;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, m) = (rem / 3600, (rem % 3600) / 60);
    // civil-from-days (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}")
}

pub fn recent_commits(repo: &Repository, limit: usize) -> Result<Vec<CommitInfo>> {
    let mut walk = repo.revwalk()?;
    if walk.push_head().is_err() {
        return Ok(Vec::new()); // unborn branch
    }
    walk.set_sorting(git2::Sort::TIME)?;

    let mut out = Vec::new();
    for oid in walk.take(limit) {
        let commit = repo.find_commit(oid?)?;
        let author = commit.author();
        out.push(CommitInfo {
            short_id: commit.id().to_string()[..7].to_string(),
            summary: commit.summary().ok().flatten().unwrap_or("").to_string(),
            body: commit.body().ok().flatten().unwrap_or("").trim().to_string(),
            author: author.name().unwrap_or("?").to_string(),
            email: author.email().unwrap_or("").to_string(),
            when: format_time(commit.time()),
        });
    }
    Ok(out)
}
