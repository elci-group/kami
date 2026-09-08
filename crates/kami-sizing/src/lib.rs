//! Deterministic project sizing (§14, §15): storage footprint via the
//! Bart ELCI adapter, source LOC via an exact in-process count. Both are
//! measurements of the *current* state — the forward LOC-projection
//! model in §15 (component/test/doc growth assumptions) is not yet
//! implemented, and this crate never fabricates one.

pub mod bart;
pub mod loc;

use std::path::Path;

use anyhow::Result;
use kami_core::ProjectSize;

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
        "lower_bound_loc/upper_bound_loc equal source_loc: this is a measurement of current LOC, not a forward projection (§15 forward growth modelling is not yet implemented)".to_string(),
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

    let size = ProjectSize {
        storage_bytes: bart.storage_bytes,
        file_count: bart.file_count,
        source_loc: counted.total,
        language_loc: counted.by_language,
        lower_bound_loc: counted.total,
        upper_bound_loc: counted.total,
        confidence,
        assumptions,
    };

    Ok(SizingResult {
        size,
        bart_raw: bart.raw,
    })
}
