use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::size::ProjectSize;

/// A state's position in the immutable transition chain (§26): `S0`,
/// `S1`, ... Formatted as `STATE-000` per §26's worked example, and
/// serialized as that string (not a bare integer) so `--format json`
/// output matches the id shown everywhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateId(pub u32);

impl StateId {
    pub const ZERO: StateId = StateId(0);

    pub fn next(self) -> StateId {
        StateId(self.0 + 1)
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STATE-{:03}", self.0)
    }
}

impl std::str::FromStr for StateId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix("STATE-").unwrap_or(s).parse().map(StateId)
    }
}

impl Serialize for StateId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for StateId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// §26: a projected state is a model of a possible future, never fact.
/// Only `Actual` states are grounded entirely in measured evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateKind {
    Actual,
    Projected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strength {
    pub id: String,
    pub description: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weakness {
    pub id: String,
    pub description: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintKind {
    /// Violating this makes a mutation infeasible (§13, §19).
    Hard,
    /// Violating this becomes a cost/risk, not a rejection (§13).
    Soft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub description: String,
    pub kind: ConstraintKind,
    /// Where the constraint came from (a config file, a compliance
    /// subsystem, a prior mutation's side effect) — constraints are
    /// first-class ontology objects, not prompt text (§4).
    pub source: String,
    /// Traceable to the evidence that established this constraint (§25) —
    /// e.g. the Isopod/Lwoodz compliance-check evidence it came from.
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// §23/§26: one immutable node in the project's state-transition chain.
/// Kami's first loop produces exactly one of these (`STATE-000`,
/// `StateKind::Actual`) and does not yet mutate it (§12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub id: StateId,
    pub parent: Option<StateId>,
    pub kind: StateKind,
    pub project_root: String,
    pub mission: Option<Mission>,
    pub strengths: Vec<Strength>,
    pub weaknesses: Vec<Weakness>,
    pub constraints: Vec<Constraint>,
    pub capabilities: Vec<Capability>,
    pub size: Option<ProjectSize>,
    pub evidence_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl ProjectState {
    pub fn genesis(project_root: impl Into<String>) -> Self {
        Self {
            id: StateId::ZERO,
            parent: None,
            kind: StateKind::Actual,
            project_root: project_root.into(),
            mission: None,
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            constraints: Vec::new(),
            capabilities: Vec::new(),
            size: None,
            evidence_ids: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_id_formats_and_round_trips_through_json() {
        let id = StateId(7);
        assert_eq!(id.to_string(), "STATE-007");
        assert_eq!("STATE-007".parse::<StateId>().unwrap(), id);

        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"STATE-007\"");
        assert_eq!(serde_json::from_str::<StateId>(&json).unwrap(), id);
    }

    #[test]
    fn genesis_state_is_actual_and_parentless() {
        let state = ProjectState::genesis("/tmp/project");
        assert_eq!(state.id, StateId::ZERO);
        assert_eq!(state.parent, None);
        assert_eq!(state.kind, StateKind::Actual);
    }
}
