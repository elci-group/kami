//! Padagonia ontology/graph export for Kami's project states (§4, §16).
//!
//! This is additive, not a replacement: `.kami/states/STATE-000.json`
//! (`kami-evidence`) stays the authoritative, already-tested source of
//! truth. This crate writes the same `ProjectState` into a real
//! Padagonia graph store (`.kami/graph.pad`) as a second, queryable
//! representation — the directive's "potentiality graph" (§16) — using
//! Padagonia as a library dependency, exactly the pattern the
//! `ferret-padagonia` sibling crate uses, rather than shelling out to
//! `padagonia`'s CLI (which has no "add a node" subcommand — it's built
//! around synthetic-graph benchmarking and file inspection, not
//! programmatic construction).
//!
//! Node/relation names here follow §4 of the directive as closely as
//! Padagonia's plain string labels allow: `PROJECT ──HAS_MISSION──>
//! MISSION`, `PROJECT ──EXHIBITS──> STRENGTH`/`WEAKNESS`, `PROJECT
//! ──HAS_CONSTRAINT──> CONSTRAINT`, `PROJECT ──HAS_CAPABILITY──>
//! CAPABILITY`. Padagonia's own `domain.rs`/`software_ontology.rs`
//! ontologies are for different consumers (Dreamsequence's control
//! plane, and structural-transformation boundaries respectively) and
//! don't cover Kami's Project/Mission/Strength/Weakness/Constraint
//! vocabulary, so this crate defines Kami's own label set rather than
//! reusing either.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use kami_core::ProjectState;
use padagonia::{NodeId, Provenance, Scalar, Store};

pub const NODE_PROJECT_STATE: &str = "ProjectState";
pub const NODE_MISSION: &str = "Mission";
pub const NODE_STRENGTH: &str = "Strength";
pub const NODE_WEAKNESS: &str = "Weakness";
pub const NODE_CONSTRAINT: &str = "Constraint";
pub const NODE_CAPABILITY: &str = "Capability";

pub const REL_HAS_MISSION: &str = "has_mission";
pub const REL_EXHIBITS: &str = "exhibits";
pub const REL_HAS_CONSTRAINT: &str = "has_constraint";
pub const REL_HAS_CAPABILITY: &str = "has_capability";

fn provenance(evidence: Vec<String>) -> Provenance {
    // agent/model record that no LLM produced this node — every field
    // here is a direct, deterministic transcription of `ProjectState`
    // (§24's deterministic/probabilistic boundary).
    Provenance::new("kami", "deterministic", 1.0, 0.0, unix_seconds(), evidence)
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Write `state` into `store` as a `ProjectState` node plus its mission,
/// strengths, weaknesses, constraints, and capabilities, each linked by
/// the §4 relationship it names. Returns the `ProjectState` node's id.
pub fn write_state(store: &mut Store, state: &ProjectState) -> NodeId {
    let project_props = vec![
        ("id", Scalar::String(state.id.to_string())),
        ("kind", Scalar::String(format!("{:?}", state.kind))),
        ("project_root", Scalar::String(state.project_root.clone())),
    ];
    let project_node = store.add_node(
        NODE_PROJECT_STATE,
        project_props,
        None,
        provenance(state.evidence_ids.clone()),
    );

    if let Some(mission) = &state.mission {
        let mission_node = store.add_node(
            NODE_MISSION,
            vec![("statement", Scalar::String(mission.statement.clone()))],
            None,
            provenance(Vec::new()),
        );
        store.add_edge(
            project_node,
            mission_node,
            REL_HAS_MISSION,
            Vec::new(),
            None,
            provenance(Vec::new()),
        );
    }

    for s in &state.strengths {
        let node = store.add_node(
            NODE_STRENGTH,
            vec![
                ("id", Scalar::String(s.id.clone())),
                ("description", Scalar::String(s.description.clone())),
            ],
            None,
            provenance(s.evidence_ids.clone()),
        );
        store.add_edge(
            project_node,
            node,
            REL_EXHIBITS,
            Vec::new(),
            None,
            provenance(Vec::new()),
        );
    }

    for w in &state.weaknesses {
        let node = store.add_node(
            NODE_WEAKNESS,
            vec![
                ("id", Scalar::String(w.id.clone())),
                ("description", Scalar::String(w.description.clone())),
            ],
            None,
            provenance(w.evidence_ids.clone()),
        );
        store.add_edge(
            project_node,
            node,
            REL_EXHIBITS,
            Vec::new(),
            None,
            provenance(Vec::new()),
        );
    }

    for c in &state.constraints {
        let node = store.add_node(
            NODE_CONSTRAINT,
            vec![
                ("id", Scalar::String(c.id.clone())),
                ("description", Scalar::String(c.description.clone())),
                ("kind", Scalar::String(format!("{:?}", c.kind))),
                ("source", Scalar::String(c.source.clone())),
            ],
            None,
            provenance(Vec::new()),
        );
        store.add_edge(
            project_node,
            node,
            REL_HAS_CONSTRAINT,
            Vec::new(),
            None,
            provenance(Vec::new()),
        );
    }

    for cap in &state.capabilities {
        let node = store.add_node(
            NODE_CAPABILITY,
            vec![
                ("id", Scalar::String(cap.id.clone())),
                ("name", Scalar::String(cap.name.clone())),
                ("description", Scalar::String(cap.description.clone())),
            ],
            None,
            provenance(Vec::new()),
        );
        store.add_edge(
            project_node,
            node,
            REL_HAS_CAPABILITY,
            Vec::new(),
            None,
            provenance(Vec::new()),
        );
    }

    project_node
}

/// Build a fresh graph containing just `state` and save it to `path`
/// (overwriting any existing file — the graph is a derived export, not
/// an accumulating store, in this phase).
pub fn export_state(path: &Path, state: &ProjectState) -> Result<()> {
    let mut store = Store::new();
    write_state(&mut store, state);
    store.save(path)?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct GraphSummary {
    pub node_count: usize,
    pub edge_count: usize,
    pub nodes_by_label: BTreeMap<String, usize>,
    pub edges_by_label: BTreeMap<String, usize>,
}

/// Load the graph at `path` and summarize it by node/edge label —
/// enough to prove the export round-trips through Padagonia's real
/// on-disk format without pulling the whole graph back into `kami-cli`.
pub fn summarize(path: &Path) -> Result<GraphSummary> {
    let store = Store::load(path)?;
    let mut summary = GraphSummary::default();
    let (node_count, edge_count, _facts, _distinct_node_labels, _distinct_edge_labels) =
        store.stats();
    summary.node_count = node_count;
    summary.edge_count = edge_count;

    for (label_id, ids) in store.node_label_index() {
        let name = store
            .string_table()
            .resolve(label_id.0)
            .unwrap_or("?")
            .to_string();
        summary.nodes_by_label.insert(name, ids.len());
    }
    for (relation_id, ids) in store.edge_label_index() {
        let name = store
            .string_table()
            .resolve(relation_id.0)
            .unwrap_or("?")
            .to_string();
        summary.edges_by_label.insert(name, ids.len());
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kami_core::{Mission, ProjectState, Strength, Weakness};

    fn sample_state() -> ProjectState {
        let mut state = ProjectState::genesis("/tmp/example");
        state.mission = Some(Mission {
            statement: "Ship reliable software".to_string(),
        });
        state.strengths.push(Strength {
            id: "fract:architecture-health".to_string(),
            description: "All modules healthy".to_string(),
            evidence_ids: vec!["ev-1".to_string()],
        });
        state.weaknesses.push(Weakness {
            id: "amber:known-cves".to_string(),
            description: "1 dependency has a known CVE".to_string(),
            evidence_ids: vec!["ev-2".to_string()],
        });
        state
    }

    #[test]
    fn export_then_summarize_round_trips_node_and_edge_counts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("graph.pad");
        let state = sample_state();

        export_state(&path, &state).unwrap();
        let summary = summarize(&path).unwrap();

        // 1 ProjectState + 1 Mission + 1 Strength + 1 Weakness.
        assert_eq!(summary.node_count, 4);
        // has_mission + exhibits(strength) + exhibits(weakness).
        assert_eq!(summary.edge_count, 3);

        assert_eq!(summary.nodes_by_label.get(NODE_PROJECT_STATE), Some(&1));
        assert_eq!(summary.nodes_by_label.get(NODE_MISSION), Some(&1));
        assert_eq!(summary.nodes_by_label.get(NODE_STRENGTH), Some(&1));
        assert_eq!(summary.nodes_by_label.get(NODE_WEAKNESS), Some(&1));
        assert_eq!(summary.edges_by_label.get(REL_HAS_MISSION), Some(&1));
        assert_eq!(summary.edges_by_label.get(REL_EXHIBITS), Some(&2));
    }

    #[test]
    fn a_state_with_no_mission_or_findings_exports_just_the_project_node() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("graph.pad");
        let state = ProjectState::genesis("/tmp/empty");

        export_state(&path, &state).unwrap();
        let summary = summarize(&path).unwrap();

        assert_eq!(summary.node_count, 1);
        assert_eq!(summary.edge_count, 0);
    }

    #[test]
    fn node_properties_round_trip_through_the_real_on_disk_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("graph.pad");
        export_state(&path, &sample_state()).unwrap();

        let store = Store::load(&path).unwrap();
        let project_node = store
            .nodes()
            .values()
            .find(|n| store.string_table().resolve(n.label.0) == Some(NODE_PROJECT_STATE))
            .unwrap();
        let value = project_node
            .properties
            .iter()
            .find(|(k, _)| store.string_table().resolve(k.0) == Some("project_root"))
            .map(|(_, v)| v);
        assert_eq!(value, Some(&Scalar::String("/tmp/example".to_string())));
    }
}
