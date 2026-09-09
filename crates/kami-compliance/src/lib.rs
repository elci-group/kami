//! Recursive compliance modelling (§13): Isopod and Lwoodz, invoked
//! through `kami-elci`'s adapter registry with the target project as
//! their working directory — never a bare `Command::new(...)`, mirroring
//! `kami-battery`'s convention. Kept as a separate pass from the §12
//! first-loop battery because the directive treats compliance modelling
//! as its own recursive step (§13, §18 step 15), not part of the
//! evidence battery.

use std::path::Path;

use kami_core::Evidence;
use kami_elci::{component_registry, ElciAdapter};
use serde_json::Value;

pub enum PassOutcome {
    Evidence(Evidence),
    Unavailable(String),
    InvocationFailed(String),
}

pub struct PassResult {
    pub tool: &'static str,
    pub outcome: PassOutcome,
}

/// Run Isopod's and Lwoodz's one-shot compliance checks against `root`.
pub fn run(root: &Path) -> Vec<PassResult> {
    let adapters = component_registry();
    vec![
        invoke_pass(
            &adapters,
            root,
            "isopod",
            &["check".into(), "--json".into()],
        ),
        invoke_pass(
            &adapters,
            root,
            "lwoodz",
            &["--json".into(), "check".into()],
        ),
    ]
}

fn invoke_pass(
    adapters: &[Box<dyn ElciAdapter>],
    root: &Path,
    tool: &'static str,
    args: &[String],
) -> PassResult {
    let Some(adapter) = adapters.iter().find(|a| a.name() == tool) else {
        return PassResult {
            tool,
            outcome: PassOutcome::Unavailable(format!(
                "{tool} adapter missing from the ELCI registry"
            )),
        };
    };

    if !adapter.detect() {
        return PassResult {
            tool,
            outcome: PassOutcome::Unavailable(adapter.health().detail),
        };
    }

    let envelope = match adapter.invoke_in(root, args) {
        Ok(v) => v,
        Err(e) => {
            return PassResult {
                tool,
                outcome: PassOutcome::InvocationFailed(e.to_string()),
            }
        }
    };

    let stdout = envelope
        .get("stdout")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match serde_json::from_str::<Value>(stdout) {
        Ok(data) => {
            let exit_code = envelope.get("exit_code").and_then(Value::as_i64);
            // Both tools may exit non-zero on a genuine compliance FAIL
            // (isopod) — that's real signal, not an invocation error, so
            // exit code is recorded in provenance but never gates success.
            let ev = Evidence::new(
                "compliance",
                format!("kami-compliance::{tool}"),
                format!(
                    "{tool} {} (cwd={}); exit_code={exit_code:?}",
                    args.join(" "),
                    root.display()
                ),
            )
            .with_confidence(1.0)
            .with_data(data);
            PassResult {
                tool,
                outcome: PassOutcome::Evidence(ev),
            }
        }
        Err(e) => {
            let stderr = envelope
                .get("stderr")
                .and_then(Value::as_str)
                .unwrap_or_default();
            PassResult {
                tool,
                outcome: PassOutcome::InvocationFailed(format!(
                    "could not parse {tool}'s stdout as JSON: {e}; stderr: {}",
                    stderr.lines().take(3).collect::<Vec<_>>().join(" / ")
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_both_named_compliance_tools_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let results = run(dir.path());
        let tools: Vec<&str> = results.iter().map(|r| r.tool).collect();
        assert_eq!(tools, vec!["isopod", "lwoodz"]);
    }

    /// Exercises the real binaries on PATH — no mocks, matching
    /// `kami-battery`'s test convention. Each tool is skipped
    /// individually if it isn't installed.
    #[test]
    fn real_tools_that_are_installed_produce_parsed_evidence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        for r in run(dir.path()) {
            if which::which(r.tool).is_err() {
                eprintln!("skipping {}: not on PATH", r.tool);
                continue;
            }
            match r.outcome {
                PassOutcome::Evidence(ev) => {
                    assert_eq!(ev.subject, "compliance");
                    assert_eq!(ev.source, format!("kami-compliance::{}", r.tool));
                    assert!(!ev.data.is_null());
                }
                PassOutcome::InvocationFailed(msg) => panic!("{} invocation failed: {msg}", r.tool),
                PassOutcome::Unavailable(detail) => {
                    panic!("{} reported unavailable: {detail}", r.tool)
                }
            }
        }
    }
}
