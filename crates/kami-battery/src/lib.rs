//! The first-loop evidence battery (§12): independent analysis passes
//! whose output is folded into evidence, never averaged together or let
//! to define state on their own. Every pass runs through
//! `kami-elci`'s adapter registry with the target project as its working
//! directory — never a bare `Command::new(...)` — so a missing sibling
//! is reported the same way everywhere else in Kami.
//!
//! Two of the seven tools named in §12 are deliberately **not** invoked
//! here, and [`run`] says so rather than fabricating a call: Curly is a
//! command-expansion engine with no project-analysis output to fold into
//! evidence, and Jeenome analyses a live `strace` stream, not a static
//! repository path.

use std::path::Path;

use kami_core::Evidence;
use kami_elci::{component_registry, AdapterKind, ElciAdapter};
use serde_json::Value;

pub enum PassOutcome {
    /// The tool ran and produced evidence Kami can use.
    Evidence(Evidence),
    /// The adapter isn't detected on this machine.
    Unavailable(String),
    /// The tool is a real ELCI sibling but its output doesn't fit a
    /// static-repository evidence pass (§12) — explained, not fabricated.
    NotApplicable(String),
    /// The tool ran but its output couldn't be turned into evidence
    /// (non-JSON output, a crash, etc.).
    InvocationFailed(String),
}

pub struct PassResult {
    pub tool: &'static str,
    pub outcome: PassOutcome,
}

impl PassResult {
    pub fn evidence(&self) -> Option<&Evidence> {
        match &self.outcome {
            PassOutcome::Evidence(ev) => Some(ev),
            _ => None,
        }
    }
}

const NOT_APPLICABLE: &[(&str, &str)] = &[
    (
        "curly",
        "schema-aware command-expansion engine — no project-analysis output to fold into \
         evidence; its `apply` subcommand patches a target's CLI rather than scanning it",
    ),
    (
        "jeenome",
        "streams strace into behavioural genomics — needs a live execution trace, not a \
         static repository path, so it has nothing to say about a project that isn't running",
    ),
];

/// Run every §12 first-loop battery tool against `root` and return one
/// [`PassResult`] per tool (five real invocations, two documented
/// non-applicable entries — never silently fewer or more).
pub fn run(root: &Path) -> Vec<PassResult> {
    let adapters = component_registry();

    let mut results = vec![
        invoke_pass(
            &adapters,
            root,
            "amber",
            &["--format".into(), "json".into()],
        ),
        invoke_pass(
            &adapters,
            root,
            "traci",
            &["check".into(), "--format".into(), "json".into()],
        ),
        invoke_pass(
            &adapters,
            root,
            "fract",
            &["index".into(), "--format".into(), "json".into()],
        ),
        invoke_pass(&adapters, root, "ferret", &["--json".into()]),
        invoke_pass(
            &adapters,
            root,
            "wilder",
            &["analyse".into(), "--format".into(), "json".into()],
        ),
    ];

    for &(tool, reason) in NOT_APPLICABLE {
        results.push(PassResult {
            tool,
            outcome: PassOutcome::NotApplicable(reason.to_string()),
        });
    }

    results
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

    if adapter.kind() == AdapterKind::Library || !adapter.detect() {
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
            let ev = Evidence::new(
                "first-loop-battery",
                format!("kami-battery::{tool}"),
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

    /// A minimal but real Cargo project with one real dependency, so
    /// tools that report on dependency usage (Amber in particular exits
    /// with no report at all on a zero-dependency crate — a real
    /// behaviour of the tool, not a bug here) have something to analyse.
    /// `anyhow` is already in this workspace's dependency graph, so
    /// resolving it offline never touches the network.
    fn fixture_project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nanyhow = \"1\"\n",
        )
        .unwrap();
        std::fs::create_dir(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("src").join("main.rs"),
            "fn main() { let _ = anyhow::anyhow!(\"fixture\"); }\n",
        )
        .unwrap();
        let status = std::process::Command::new("cargo")
            .args(["generate-lockfile"])
            .current_dir(dir.path())
            .env("CARGO_NET_OFFLINE", "true")
            .status()
            .unwrap();
        assert!(
            status.success(),
            "cargo generate-lockfile failed for the fixture project"
        );
        dir
    }

    #[test]
    fn covers_exactly_the_seven_named_battery_tools_in_order() {
        let dir = fixture_project();
        let results = run(dir.path());
        let tools: Vec<&str> = results.iter().map(|r| r.tool).collect();
        assert_eq!(
            tools,
            vec!["amber", "traci", "fract", "ferret", "wilder", "curly", "jeenome"]
        );
    }

    #[test]
    fn curly_and_jeenome_are_explained_as_not_applicable_never_fabricated() {
        let dir = fixture_project();
        let results = run(dir.path());
        for tool in ["curly", "jeenome"] {
            let r = results.iter().find(|r| r.tool == tool).unwrap();
            assert!(
                matches!(r.outcome, PassOutcome::NotApplicable(_)),
                "{tool} should be NotApplicable"
            );
        }
    }

    /// Exercises the real binaries on PATH — no mocks. Each tool is
    /// skipped individually (rather than failing the whole test) if it
    /// isn't installed in the environment running the tests, matching
    /// how `kami status` reports the same adapter as
    /// `CAPABILITY_UNAVAILABLE`.
    #[test]
    fn real_tools_that_are_installed_produce_parsed_evidence() {
        let dir = fixture_project();
        let results = run(dir.path());

        for r in &results {
            if matches!(r.outcome, PassOutcome::NotApplicable(_)) {
                continue;
            }
            if which::which(r.tool).is_err() {
                eprintln!("skipping {}: not on PATH", r.tool);
                continue;
            }
            match &r.outcome {
                PassOutcome::Evidence(ev) => {
                    assert_eq!(ev.subject, "first-loop-battery");
                    assert_eq!(ev.source, format!("kami-battery::{}", r.tool));
                    assert!(
                        !ev.data.is_null(),
                        "{} evidence data should not be null",
                        r.tool
                    );
                }
                PassOutcome::InvocationFailed(msg) => {
                    panic!("{} is on PATH but its invocation failed: {msg}", r.tool)
                }
                PassOutcome::Unavailable(detail) => {
                    panic!(
                        "{} is on PATH but was reported unavailable: {detail}",
                        r.tool
                    )
                }
                PassOutcome::NotApplicable(_) => unreachable!(),
            }
        }
    }
}
