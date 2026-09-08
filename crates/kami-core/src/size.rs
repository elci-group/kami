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
}
