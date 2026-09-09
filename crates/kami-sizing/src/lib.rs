//! Deterministic project sizing (§14, §15): storage footprint via the
//! Bart ELCI adapter, source LOC via an exact in-process count — both
//! measurements of the *current* state — plus a forward LOC projection
//! (§15) derived from real `git log --numstat` history, never LLM
//! intuition. A project with no usable git history gets `growth: None`,
//! never a guessed default.

pub mod bart;
pub mod growth;
pub mod loc;

use std::path::Path;

use anyhow::Result;
use kami_core::ProjectSize;

/// How far ahead `size_project`'s growth projection looks by default.
pub const DEFAULT_GROWTH_HORIZON_DAYS: f64 = 30.0;

pub struct SizingResult {
    pub size: ProjectSize,
    /// Bart's raw `--json` tree, for callers that want to record it as
    /// evidence (§25).
    pub bart_raw: serde_json::Value,
}

/// Measure `root`'s current size. Fails if the Bart adapter is
/// unavailable or errors — Kami never falls back to a guessed storage
/// figure (§24).
pub fn size_project(root: &Path) -> Result<SizingResult> {
    let bart = bart::measure(root)?;
    let counted = loc::count_source_loc(root);

    let mut assumptions = vec![
        "source_loc counts only recognized source extensions (see kami-sizing::loc::language_for_extension)".to_string(),
        "lower_bound_loc/upper_bound_loc equal source_loc: this is a measurement of current LOC, not the forward projection — see the separate `growth` field for that (§15)".to_string(),
    ];
    let confidence = if counted.unreadable.is_empty() {
        1.0
    } else {
        assumptions.push(format!(
            "{} path(s) were unreadable and excluded from the LOC count",
            counted.unreadable.len()
        ));
        0.9
    };

    let growth = match growth::measure_growth(root) {
        growth::GrowthOutcome::History(history) => Some(growth::project(
            &history,
            counted.total,
            DEFAULT_GROWTH_HORIZON_DAYS,
        )),
        growth::GrowthOutcome::NoHistory(reason) => {
            assumptions.push(format!("no growth projection: {reason}"));
            None
        }
    };

    let size = ProjectSize {
        storage_bytes: bart.storage_bytes,
        file_count: bart.file_count,
        source_loc: counted.total,
        language_loc: counted.by_language,
        lower_bound_loc: counted.total,
        upper_bound_loc: counted.total,
        confidence,
        assumptions,
        growth,
    };

    Ok(SizingResult {
        size,
        bart_raw: bart.raw,
    })
}
