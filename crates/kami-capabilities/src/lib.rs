//! Deterministic `Capability` discovery (§4: `PROJECT ──HAS_CAPABILITY──>
//! CAPABILITY`) from a project's real Cargo target list — `cargo
//! metadata --no-deps`, which every Cargo project already exposes, so
//! nothing here parses `Cargo.toml` by hand or guesses at structure.
//!
//! `cargo` is the ambient Rust toolchain, not an ELCI sibling, so this
//! doesn't go through `kami-elci`'s adapter registry — the same reasoning
//! `kami-sizing`'s in-process LOC walk uses for not going through it
//! either. A project with no `Cargo.toml` (not a Rust project, or not
//! this kind of Rust project) is reported as `NotApplicable`, never
//! silently skipped or guessed at.
//!
//! Only `bin` and `lib` targets become capabilities — `test`/`bench`/
//! `example` targets are internal tooling, not something the project
//! *provides*, which is what §4's `Capability` node is meant to capture.

use std::path::Path;
use std::process::Command;

use kami_core::{Capability, Evidence};
use serde_json::Value;

pub enum DiscoveryOutcome {
    Evidence(Evidence),
    NotApplicable(String),
    InvocationFailed(String),
}

pub struct Discovery {
    pub outcome: DiscoveryOutcome,
    /// Non-empty only when `outcome` is `Evidence`.
    pub capabilities: Vec<Capability>,
}

pub fn discover(root: &Path) -> Discovery {
    if !root.join("Cargo.toml").is_file() {
        return Discovery {
            outcome: DiscoveryOutcome::NotApplicable(format!(
                "no Cargo.toml at {}",
                root.display()
            )),
            capabilities: Vec::new(),
        };
    }

    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version=1"])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            return Discovery {
                outcome: DiscoveryOutcome::InvocationFailed(e.to_string()),
                capabilities: Vec::new(),
            }
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Discovery {
            outcome: DiscoveryOutcome::InvocationFailed(format!(
                "cargo metadata exited {:?}: {}",
                output.status.code(),
                stderr.lines().take(3).collect::<Vec<_>>().join(" / ")
            )),
            capabilities: Vec::new(),
        };
    }

    let data: Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(e) => {
            return Discovery {
                outcome: DiscoveryOutcome::InvocationFailed(format!(
                    "could not parse cargo metadata JSON: {e}"
                )),
                capabilities: Vec::new(),
            }
        }
    };

    let ev = Evidence::new(
        "capabilities",
        "kami-capabilities::cargo-metadata",
        format!(
            "cargo metadata --no-deps --format-version=1 (cwd={})",
            root.display()
        ),
    )
    .with_confidence(1.0)
    .with_data(data.clone());

    let capabilities = extract_capabilities(&data, &ev.id);

    Discovery {
        outcome: DiscoveryOutcome::Evidence(ev),
        capabilities,
    }
}

fn extract_capabilities(data: &Value, evidence_id: &str) -> Vec<Capability> {
    let Some(packages) = data.get("packages").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut capabilities = Vec::new();
    for pkg in packages {
        let pkg_name = pkg.get("name").and_then(Value::as_str).unwrap_or("unknown");
        let Some(targets) = pkg.get("targets").and_then(Value::as_array) else {
            continue;
        };
        for target in targets {
            let kinds: Vec<&str> = target
                .get("kind")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let Some(&kind) = kinds.iter().find(|k| **k == "bin" || **k == "lib") else {
                continue;
            };
            let target_name = target
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            capabilities.push(Capability {
                id: format!("cargo:{pkg_name}:{target_name}"),
                name: target_name.to_string(),
                description: format!("{pkg_name} provides a {kind} target: {target_name}"),
                evidence_ids: vec![evidence_id.to_string()],
            });
        }
    }
    capabilities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_project_with_no_cargo_toml_is_not_applicable() {
        let dir = tempfile::tempdir().unwrap();
        let d = discover(dir.path());
        assert!(matches!(d.outcome, DiscoveryOutcome::NotApplicable(_)));
        assert!(d.capabilities.is_empty());
    }

    #[test]
    fn a_real_cargo_project_yields_bin_and_lib_capabilities_but_not_test_targets() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"fixture\"\n\n[[bin]]\nname = \"fixture-cli\"\npath = \"src/main.rs\"\n",
        )
        .unwrap();
        std::fs::create_dir(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src").join("lib.rs"), "").unwrap();
        std::fs::write(dir.path().join("src").join("main.rs"), "fn main() {}\n").unwrap();
        std::fs::create_dir(dir.path().join("tests")).unwrap();
        std::fs::write(
            dir.path().join("tests").join("it.rs"),
            "#[test]\nfn it_works() {}\n",
        )
        .unwrap();
        let lockfile = Command::new("cargo")
            .args(["generate-lockfile"])
            .current_dir(dir.path())
            .env("CARGO_NET_OFFLINE", "true")
            .status()
            .unwrap();
        assert!(lockfile.success());

        let d = discover(dir.path());
        assert!(matches!(d.outcome, DiscoveryOutcome::Evidence(_)));

        let names: Vec<&str> = d.capabilities.iter().map(|c| c.name.as_str()).collect();
        assert!(
            names.contains(&"fixture"),
            "expected the lib target, got {names:?}"
        );
        assert!(
            names.contains(&"fixture-cli"),
            "expected the bin target, got {names:?}"
        );
        assert_eq!(
            d.capabilities.len(),
            2,
            "test targets must not become capabilities: {names:?}"
        );
        assert!(!d.capabilities[0].evidence_ids.is_empty());
    }
}
