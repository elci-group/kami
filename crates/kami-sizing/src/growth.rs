//! Deterministic LOC forward projection (§15) from real `git log
//! --numstat` history — never LLM intuition, and never a guessed
//! default when there's no usable history to measure.

use std::path::Path;
use std::process::Command;

use kami_core::GrowthProjection;

use crate::loc::language_for_extension;

/// One commit's net change to recognized-source-extension files.
struct CommitDelta {
    timestamp_unix: i64,
    added: u64,
    removed: u64,
}

pub struct GrowthHistory {
    pub commit_count: usize,
    pub window_days: f64,
    pub total_added: u64,
    pub total_removed: u64,
    pub net_change: i64,
    pub loc_per_day: f64,
    /// Smallest and largest single-commit net change observed — the
    /// basis for `GrowthProjection`'s bounds, rather than an invented
    /// percentage band.
    pub min_commit_delta: i64,
    pub max_commit_delta: i64,
}

pub enum GrowthOutcome {
    History(GrowthHistory),
    /// Not a git repo, `git` isn't runnable, or fewer than two commits
    /// touch recognized source files.
    NoHistory(String),
}

/// Measure historical LOC growth from `root`'s git history.
pub fn measure_growth(root: &Path) -> GrowthOutcome {
    let output = Command::new("git")
        .args(["log", "--pretty=format:COMMIT %ct", "--numstat"])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            return GrowthOutcome::NoHistory(format!(
                "git log exited {:?}: {}",
                o.status.code(),
                String::from_utf8_lossy(&o.stderr)
                    .lines()
                    .next()
                    .unwrap_or("")
            ))
        }
        Err(e) => return GrowthOutcome::NoHistory(format!("could not run git: {e}")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let deltas = parse_commit_deltas(&stdout);

    if deltas.len() < 2 {
        return GrowthOutcome::NoHistory(format!(
            "only {} commit(s) touch recognized source files; need at least 2 to measure a rate",
            deltas.len()
        ));
    }

    let min_ts = deltas.iter().map(|d| d.timestamp_unix).min().unwrap();
    let max_ts = deltas.iter().map(|d| d.timestamp_unix).max().unwrap();
    let window_days = (max_ts - min_ts) as f64 / 86_400.0;

    if window_days <= 0.0 {
        return GrowthOutcome::NoHistory(
            "all commits share the same timestamp; growth rate undefined".to_string(),
        );
    }

    let total_added: u64 = deltas.iter().map(|d| d.added).sum();
    let total_removed: u64 = deltas.iter().map(|d| d.removed).sum();
    let net_change = total_added as i64 - total_removed as i64;
    let loc_per_day = net_change as f64 / window_days;

    let commit_net_changes: Vec<i64> = deltas
        .iter()
        .map(|d| d.added as i64 - d.removed as i64)
        .collect();
    let min_commit_delta = *commit_net_changes.iter().min().unwrap();
    let max_commit_delta = *commit_net_changes.iter().max().unwrap();

    GrowthOutcome::History(GrowthHistory {
        commit_count: deltas.len(),
        window_days,
        total_added,
        total_removed,
        net_change,
        loc_per_day,
        min_commit_delta,
        max_commit_delta,
    })
}

fn parse_commit_deltas(log: &str) -> Vec<CommitDelta> {
    let mut deltas = Vec::new();
    let mut current: Option<CommitDelta> = None;

    for line in log.lines() {
        if let Some(rest) = line.strip_prefix("COMMIT ") {
            if let Some(c) = current.take() {
                deltas.push(c);
            }
            if let Ok(ts) = rest.trim().parse::<i64>() {
                current = Some(CommitDelta {
                    timestamp_unix: ts,
                    added: 0,
                    removed: 0,
                });
            }
            continue;
        }
        let Some(commit) = current.as_mut() else {
            continue;
        };
        let mut parts = line.split('\t');
        let (Some(added), Some(removed), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        // Binary files report "-\t-\tpath" in numstat; skip those.
        let (Ok(added), Ok(removed)) = (added.parse::<u64>(), removed.parse::<u64>()) else {
            continue;
        };
        let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if language_for_extension(&ext.to_ascii_lowercase()).is_none() {
            continue;
        }
        commit.added += added;
        commit.removed += removed;
    }
    if let Some(c) = current {
        deltas.push(c);
    }
    // Commits that touched no recognized-extension file contribute a
    // zero-delta entry above; keep only commits that changed source.
    deltas.retain(|d| d.added > 0 || d.removed > 0);
    deltas
}

/// Project `current_loc` forward by `horizon_days`, using `history`'s
/// observed rate and per-commit extremes. Confidence is a function of
/// the observation window alone (`window_days / (window_days + 7.0)`,
/// capped at 0.9): a short window stays low-confidence regardless of
/// how many commits happened inside it, since commit pace over hours
/// doesn't evidence pace over weeks.
pub fn project(history: &GrowthHistory, current_loc: u64, horizon_days: f64) -> GrowthProjection {
    let commits_per_day = history.commit_count as f64 / history.window_days;
    let expected_commits = commits_per_day * horizon_days;

    let projected_delta = history.loc_per_day * horizon_days;
    let lower_delta = history.min_commit_delta as f64 * expected_commits;
    let upper_delta = history.max_commit_delta as f64 * expected_commits;

    let clamp = |loc: f64| -> u64 { loc.max(0.0).round() as u64 };
    let projected_loc = clamp(current_loc as f64 + projected_delta);
    let mut lower_bound_loc = clamp(current_loc as f64 + lower_delta);
    let mut upper_bound_loc = clamp(current_loc as f64 + upper_delta);
    if lower_bound_loc > upper_bound_loc {
        std::mem::swap(&mut lower_bound_loc, &mut upper_bound_loc);
    }

    let confidence = (history.window_days / (history.window_days + 7.0)).min(0.9);

    GrowthProjection {
        horizon_days,
        projected_loc,
        lower_bound_loc,
        upper_bound_loc,
        confidence,
        assumptions: vec![
            format!(
                "based on {} commit(s) touching recognized source files over a {:.2}-day observation window",
                history.commit_count, history.window_days
            ),
            format!("observed net rate: {:.1} LOC/day", history.loc_per_day),
            "bounds are the smallest/largest single-commit net change observed, scaled by the expected commit count over the horizon — not an invented percentage band".to_string(),
            "a short observation window yields low confidence regardless of commit count: pace over hours does not evidence pace over weeks".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    fn init_repo(dir: &Path) {
        git(dir, &["init", "-q"]);
        git(dir, &["config", "user.email", "test@example.com"]);
        git(dir, &["config", "user.name", "Test"]);
    }

    #[test]
    fn a_directory_with_no_git_history_reports_no_history() {
        let dir = tempfile::tempdir().unwrap();
        match measure_growth(dir.path()) {
            GrowthOutcome::NoHistory(_) => {}
            GrowthOutcome::History(_) => panic!("expected NoHistory for a non-git directory"),
        }
    }

    #[test]
    fn a_single_commit_reports_no_history() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        fs::write(dir.path().join("a.rs"), "fn a() {}\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-q", "-m", "one"]);

        match measure_growth(dir.path()) {
            GrowthOutcome::NoHistory(_) => {}
            GrowthOutcome::History(_) => panic!("expected NoHistory for a single commit"),
        }
    }

    #[test]
    fn two_commits_with_a_real_gap_produce_a_measurable_rate() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());

        fs::write(dir.path().join("a.rs"), "fn a() {}\n").unwrap();
        git(dir.path(), &["add", "."]);
        commit_at(dir.path(), "first", 1_000_000_000);

        fs::write(dir.path().join("a.rs"), "fn a() {}\nfn b() {}\nfn c() {}\n").unwrap();
        git(dir.path(), &["add", "."]);
        commit_at(dir.path(), "second", 1_000_000_000 + 10 * 86_400);

        let history = match measure_growth(dir.path()) {
            GrowthOutcome::History(h) => h,
            GrowthOutcome::NoHistory(reason) => panic!("expected History, got NoHistory: {reason}"),
        };

        // The first commit's own diff (against the empty tree) adds the
        // initial 1-line file; the second commit adds 2 more lines.
        assert_eq!(history.commit_count, 2);
        assert_eq!(history.total_added, 3);
        assert_eq!(history.total_removed, 0);
        assert!((history.window_days - 10.0).abs() < 0.01);
        assert!((history.loc_per_day - 0.3).abs() < 0.01);

        let projection = project(&history, 3, 30.0);
        assert_eq!(projection.horizon_days, 30.0);
        assert!(projection.projected_loc > 3);
        assert!(projection.confidence > 0.0 && projection.confidence <= 0.9);
        assert!(!projection.assumptions.is_empty());
    }

    #[test]
    fn non_source_file_changes_do_not_count_toward_the_rate() {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());

        fs::write(dir.path().join("a.rs"), "fn a() {}\n").unwrap();
        git(dir.path(), &["add", "."]);
        commit_at(dir.path(), "first", 2_000_000_000);

        // A binary-ish extension with no recognized language.
        fs::write(dir.path().join("data.bin"), [0u8, 1, 2, 3]).unwrap();
        git(dir.path(), &["add", "."]);
        commit_at(dir.path(), "second", 2_000_000_000 + 5 * 86_400);

        match measure_growth(dir.path()) {
            GrowthOutcome::NoHistory(reason) => {
                assert!(reason.contains("only 1 commit"));
            }
            GrowthOutcome::History(_) => {
                panic!("the second commit touched no recognized source file")
            }
        }
    }

    fn commit_at(dir: &Path, message: &str, unix_ts: i64) {
        let ts = format!("@{unix_ts} +0000");
        let status = Command::new("git")
            .args(["commit", "-q", "-m", message])
            .current_dir(dir)
            .env("GIT_AUTHOR_DATE", &ts)
            .env("GIT_COMMITTER_DATE", &ts)
            .status()
            .unwrap();
        assert!(status.success());
    }
}
