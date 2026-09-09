use std::path::Path;

use anyhow::{bail, Result};
use kami_battery::PassOutcome;
use kami_compliance::PassOutcome as CompliancePassOutcome;
use kami_core::{Evidence, Mission, ProjectState, StateId};
use kami_elci::component_registry;
use kami_evidence::Store;
use serde_json::json;

fn sizing_evidence(path: &Path) -> Result<(Evidence, kami_sizing::SizingResult)> {
    let sizing = kami_sizing::size_project(path)?;
    let ev = Evidence::new(
        "project-size",
        "kami-sizing",
        format!("bart --json {}; in-process source LOC walk", path.display()),
    )
    .with_confidence(sizing.size.confidence)
    .with_data(json!({ "size": sizing.size, "bart_raw": sizing.bart_raw }));
    Ok((ev, sizing))
}

pub fn init(path: &Path) -> Result<serde_json::Value> {
    let store = Store::open(path)?;
    Ok(json!({ "state_dir": store.root().display().to_string() }))
}

pub fn inspect(path: &Path) -> Result<serde_json::Value> {
    let store = Store::open(path)?;
    let (ev, sizing) = sizing_evidence(path)?;
    store.record_evidence(&ev)?;
    Ok(json!({ "evidence_id": ev.id, "size": sizing.size }))
}

pub fn status(path: &Path) -> Result<serde_json::Value> {
    let _ = path;
    let adapters = component_registry();
    let rows: Vec<_> = adapters
        .iter()
        .map(|a| {
            let health = a.health();
            json!({
                "name": a.name(),
                "kind": format!("{:?}", a.kind()),
                "role": a.role(),
                "available": health.available,
                "detail": health.detail,
                "version": a.version(),
            })
        })
        .collect();
    Ok(json!({ "adapters": rows }))
}

pub fn state_build(path: &Path, mission: Option<String>) -> Result<serde_json::Value> {
    let store = Store::open(path)?;
    if !store.list_states()?.is_empty() {
        bail!(
            "a project state already exists under {}/.kami/states; recursive mutation (§17-22) \
             is not implemented in this build, so `kami state build` only produces the \
             non-mutating STATE-000 baseline (§12). Run `kami state show --id 0` to see it.",
            path.display()
        );
    }

    let (ev, sizing) = sizing_evidence(path)?;
    store.record_evidence(&ev)?;

    let mut evidence_ids = vec![ev.id.clone()];
    let mut strengths = Vec::new();
    let mut weaknesses = Vec::new();
    let mut battery_summary = Vec::new();
    for pass in kami_battery::run(path) {
        let tool = pass.tool;
        let (outcome_label, detail, evidence_id) = match pass.outcome {
            PassOutcome::Evidence(ev) => {
                let id = ev.id.clone();
                let found = kami_findings::extract(tool, &id, &ev.data);
                strengths.extend(found.strengths);
                weaknesses.extend(found.weaknesses);
                store.record_evidence(&ev)?;
                evidence_ids.push(id.clone());
                ("evidence", None, Some(id))
            }
            PassOutcome::Unavailable(detail) => ("unavailable", Some(detail), None),
            PassOutcome::NotApplicable(detail) => ("not_applicable", Some(detail), None),
            PassOutcome::InvocationFailed(detail) => ("invocation_failed", Some(detail), None),
        };
        battery_summary.push(json!({
            "tool": tool,
            "outcome": outcome_label,
            "detail": detail,
            "evidence_id": evidence_id,
        }));
    }

    // Recursive compliance modelling (§13) — a separate pass from the
    // §12 battery above, per the directive's own pipeline ordering.
    let mut compliance_summary = Vec::new();
    let mut constraints = Vec::new();
    for pass in kami_compliance::run(path) {
        let tool = pass.tool;
        let (outcome_label, detail, evidence_id) = match pass.outcome {
            CompliancePassOutcome::Evidence(ev) => {
                let id = ev.id.clone();
                let found = kami_findings::extract(tool, &id, &ev.data);
                strengths.extend(found.strengths);
                weaknesses.extend(found.weaknesses);
                constraints.extend(found.constraints);
                store.record_evidence(&ev)?;
                evidence_ids.push(id.clone());
                ("evidence", None, Some(id))
            }
            CompliancePassOutcome::Unavailable(detail) => ("unavailable", Some(detail), None),
            CompliancePassOutcome::InvocationFailed(detail) => {
                ("invocation_failed", Some(detail), None)
            }
        };
        compliance_summary.push(json!({
            "tool": tool,
            "outcome": outcome_label,
            "detail": detail,
            "evidence_id": evidence_id,
        }));
    }

    let mut state = ProjectState::genesis(path.display().to_string());
    state.mission = mission.map(|statement| Mission { statement });
    state.size = Some(sizing.size);
    state.evidence_ids = evidence_ids;
    state.strengths = strengths;
    state.weaknesses = weaknesses;
    state.constraints = constraints;
    store.save_state(&state)?;

    // Additive (§4, §16): the flat JSON state above is already saved and
    // is the authoritative record. A Padagonia graph export failing
    // (e.g. a bad path) must not undo or block that — it's reported,
    // not propagated as a command failure.
    let graph_path = graph_path(&store);
    let padagonia = match kami_padagonia::export_state(&graph_path, &state) {
        Ok(()) => json!({ "exported": true, "path": graph_path.display().to_string() }),
        Err(e) => json!({ "exported": false, "error": e.to_string() }),
    };

    Ok(json!({
        "state": state,
        "battery": battery_summary,
        "compliance": compliance_summary,
        "padagonia": padagonia,
    }))
}

fn graph_path(store: &Store) -> std::path::PathBuf {
    store.root().join("graph.pad")
}

pub fn graph(path: &Path) -> Result<serde_json::Value> {
    let store = Store::open(path)?;
    let graph_path = graph_path(&store);
    if !graph_path.is_file() {
        bail!(
            "no Padagonia graph at {}; run `kami state build` first",
            graph_path.display()
        );
    }
    let summary = kami_padagonia::summarize(&graph_path)?;
    Ok(json!({
        "path": graph_path.display().to_string(),
        "node_count": summary.node_count,
        "edge_count": summary.edge_count,
        "nodes_by_label": summary.nodes_by_label,
        "edges_by_label": summary.edges_by_label,
    }))
}

pub fn state_list(path: &Path) -> Result<serde_json::Value> {
    let store = Store::open(path)?;
    let ids: Vec<String> = store
        .list_states()?
        .iter()
        .map(ToString::to_string)
        .collect();
    Ok(json!({ "states": ids }))
}

pub fn state_show(path: &Path, id: u32) -> Result<serde_json::Value> {
    let store = Store::open(path)?;
    let state = store.load_state(StateId(id))?;
    Ok(json!({ "state": state }))
}
