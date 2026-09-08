use std::path::Path;

use anyhow::{bail, Result};
use kami_battery::PassOutcome;
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

    let mut state = ProjectState::genesis(path.display().to_string());
    state.mission = mission.map(|statement| Mission { statement });
    state.size = Some(sizing.size);
    state.evidence_ids = evidence_ids;
    state.strengths = strengths;
    state.weaknesses = weaknesses;
    store.save_state(&state)?;

    Ok(json!({ "state": state, "battery": battery_summary }))
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
