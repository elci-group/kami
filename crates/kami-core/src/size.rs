use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Deterministic project sizing (§14, §15). Every field here must trace
/// back to a measurement (Bart's storage scan, a source-line count) —
/// never to LLM intuition. `lower_bound_loc`/`upper_bound_loc` exist so
/// callers can distinguish "we counted this" from "we projected this."
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectSize {
    pub storage_bytes: u64,
    pub file_count: u64,
    pub source_loc: u64,
    /// Source LOC broken down by language, keyed by a human-readable
    /// language name (e.g. "Rust", "Markdown").
    pub language_loc: BTreeMap<String, u64>,
    pub lower_bound_loc: u64,
    pub upper_bound_loc: u64,
    /// 0.0-1.0 confidence in the LOC estimate. 1.0 for an exhaustive
    /// count over a fully-readable tree; lower when files were skipped
    /// (permission errors, binary detection false negatives, etc.).
    pub confidence: f64,
    pub assumptions: Vec<String>,
    /// The §15 deterministic forward projection, derived from real git
    /// history — `None` when there's no usable history to project from
    /// (e.g. not a git repo, or fewer than two commits touching
    /// recognized source files), never a guessed default.
    #[serde(default)]
    pub growth: Option<GrowthProjection>,
}

/// A deterministic LOC forward projection (§15), built from real
/// historical `git log --numstat` deltas — never LLM intuition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthProjection {
    pub horizon_days: f64,
    pub projected_loc: u64,
    /// Derived from the smallest/largest single-commit net change
    /// observed in the history window, not an invented percentage band.
    pub lower_bound_loc: u64,
    pub upper_bound_loc: u64,
    /// 0.0-1.0, a function of how many days of history back the
    /// observation window — a one-day window (this session's own
    /// history, for instance) yields a low confidence regardless of
    /// commit count, since pace over hours doesn't evidence pace over
    /// weeks. See `kami-sizing::growth` for the exact formula.
    pub confidence: f64,
    pub assumptions: Vec<String>,
}
