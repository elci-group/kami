//! Shared domain types for Kami (§23): the strict STATE / POTENTIAL
//! distinction (§2), evidence provenance (§25), and deterministic
//! project sizing (§14, §15). Pure data — no I/O, no inference, no
//! sibling-tool invocation. See `kami-evidence` for persistence and
//! `kami-elci` for sibling ELCI adapters.

pub mod evidence;
pub mod size;
pub mod state;

pub use evidence::Evidence;
pub use size::{GrowthProjection, ProjectSize};
pub use state::{
    Capability, Constraint, ConstraintKind, Mission, ProjectState, StateId, StateKind, Strength,
    Weakness,
};
