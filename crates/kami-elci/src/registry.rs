use crate::adapter::{AdapterHealth, AdapterKind, ElciAdapter, INVOKE_UNAVAILABLE};
use anyhow::{anyhow, bail};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Static description of one ELCI sibling component named in the Kami
/// directive.
pub struct ComponentDescriptor {
    pub name: &'static str,
    pub kind: AdapterKind,
    /// Directory name of the sibling project, relative to [`elci_root`].
    pub sibling_dir: &'static str,
    /// Binary name to look for on `PATH` or in the sibling's
    /// `target/{release,debug}/`, for [`AdapterKind::Binary`] components.
    pub bin_name: Option<&'static str>,
    pub role: &'static str,
}

/// Root directory that houses every ELCI sibling project. Defaults to the
/// parent of the current working directory (Kami's own siblings), and
/// can be overridden with `KAMI_ELCI_ROOT` for environments where Kami
/// isn't run from inside its own checkout.
pub fn elci_root() -> PathBuf {
    if let Ok(v) = std::env::var("KAMI_ELCI_ROOT") {
        return PathBuf::from(v);
    }
    std::env::current_dir()
        .ok()
        .and_then(|d| d.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from(".."))
}

pub fn descriptors() -> Vec<ComponentDescriptor> {
    use AdapterKind::*;
    vec![
        ComponentDescriptor {
            name: "bart",
            kind: Binary,
            sibling_dir: "bart",
            bin_name: Some("bart"),
            role: "deterministic project sizing (§14, §15)",
        },
        ComponentDescriptor {
            name: "amber",
            kind: Binary,
            sibling_dir: "amber",
            bin_name: Some("amber"),
            role: "first-loop evidence battery (§12)",
        },
        ComponentDescriptor {
            name: "curly",
            kind: Binary,
            sibling_dir: "curly",
            bin_name: Some("curly"),
            role: "first-loop evidence battery (§12)",
        },
        ComponentDescriptor {
            name: "traci",
            kind: Binary,
            sibling_dir: "traci",
            bin_name: Some("traci"),
            role: "first-loop evidence battery (§12)",
        },
        ComponentDescriptor {
            name: "fract",
            kind: Binary,
            sibling_dir: "fract",
            bin_name: Some("fract"),
            role: "first-loop evidence battery (§12)",
        },
        ComponentDescriptor {
            name: "ferret",
            kind: Binary,
            sibling_dir: "ferret",
            bin_name: Some("ferret"),
            role: "first-loop evidence battery (§12)",
        },
        ComponentDescriptor {
            name: "jeenome",
            kind: Binary,
            sibling_dir: "jeenome",
            bin_name: Some("jeenome"),
            role: "first-loop evidence battery (§12)",
        },
        ComponentDescriptor {
            name: "wilder",
            kind: Binary,
            sibling_dir: "wilder",
            bin_name: Some("wilder"),
            role: "first-loop evidence battery (§12)",
        },
        ComponentDescriptor {
            name: "isopod",
            kind: Binary,
            sibling_dir: "isopod",
            bin_name: Some("isopod"),
            role: "recursive compliance modelling (§13)",
        },
        ComponentDescriptor {
            name: "lwoodz",
            kind: Binary,
            sibling_dir: "lwoodz",
            bin_name: Some("lwoodz"),
            role: "recursive compliance modelling (§13)",
        },
        ComponentDescriptor {
            name: "kakashi",
            kind: Binary,
            sibling_dir: "kakashi",
            bin_name: Some("kakashi"),
            role: "competitor technique modelling (§10)",
        },
        ComponentDescriptor {
            name: "vamos",
            kind: Binary,
            sibling_dir: "vamos",
            bin_name: Some("vamos"),
            role: "CLI event-chain modelling (§11)",
        },
        ComponentDescriptor {
            name: "padagonia",
            kind: Binary,
            sibling_dir: "padagonia",
            bin_name: Some("padagonia"),
            role: "ontology and constraint substrate (§4)",
        },
        ComponentDescriptor {
            name: "ingauge",
            kind: Binary,
            sibling_dir: "ingauge",
            bin_name: Some("ingauge"),
            role: "inference rate/budget management (§9)",
        },
        ComponentDescriptor {
            name: "deliver",
            kind: Binary,
            sibling_dir: "deliver",
            bin_name: Some("deliver"),
            role: "deterministic execution of Kami's potential delta (§21)",
        },
        ComponentDescriptor {
            name: "dreamsequence",
            kind: Library,
            sibling_dir: "dreamsequence",
            bin_name: None,
            role: "reflection on materialised state, feeds back into Kami (§32)",
        },
        ComponentDescriptor {
            name: "mesut",
            kind: Library,
            sibling_dir: "mesut",
            bin_name: None,
            role: "agent execution substrate referenced by sibling ELCI tools",
        },
    ]
}

pub fn component_registry() -> Vec<Box<dyn ElciAdapter>> {
    descriptors()
        .into_iter()
        .map(|d| -> Box<dyn ElciAdapter> {
            Box::new(SiblingAdapter {
                root: elci_root(),
                descriptor: d,
            })
        })
        .collect()
}

/// One [`ElciAdapter`] implementation covering both binary and library
/// sibling kinds, driven entirely by [`ComponentDescriptor`].
struct SiblingAdapter {
    root: PathBuf,
    descriptor: ComponentDescriptor,
}

impl SiblingAdapter {
    fn sibling_path(&self) -> PathBuf {
        self.root.join(self.descriptor.sibling_dir)
    }

    fn resolve_binary(&self) -> Option<PathBuf> {
        let bin_name = self.descriptor.bin_name?;
        if let Ok(path) = which::which(bin_name) {
            return Some(path);
        }
        for profile in ["release", "debug"] {
            let candidate = self
                .sibling_path()
                .join("target")
                .join(profile)
                .join(bin_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }
}

impl ElciAdapter for SiblingAdapter {
    fn name(&self) -> &str {
        self.descriptor.name
    }

    fn kind(&self) -> AdapterKind {
        self.descriptor.kind
    }

    fn detect(&self) -> bool {
        match self.descriptor.kind {
            AdapterKind::Binary => self.resolve_binary().is_some(),
            AdapterKind::Library => self.sibling_path().join("Cargo.toml").is_file(),
        }
    }

    fn role(&self) -> &'static str {
        self.descriptor.role
    }

    fn version(&self) -> Option<String> {
        let path = self.resolve_binary()?;
        let output = Command::new(&path).arg("--version").output().ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        text.lines().next().map(str::trim).map(str::to_string)
    }

    fn health(&self) -> AdapterHealth {
        if !self.detect() {
            return AdapterHealth {
                available: false,
                detail: format!(
                    "{} not found: expected binary '{}' on PATH or under {}/target/{{release,debug}}, or library at {}",
                    self.descriptor.name,
                    self.descriptor.bin_name.unwrap_or("<none>"),
                    self.descriptor.sibling_dir,
                    self.sibling_path().display(),
                ),
            };
        }
        match self.descriptor.kind {
            AdapterKind::Binary => AdapterHealth {
                available: true,
                detail: format!(
                    "binary resolved: {}",
                    self.resolve_binary().unwrap().display()
                ),
            },
            AdapterKind::Library => AdapterHealth {
                available: true,
                detail: format!(
                    "library present at {} (no direct invocation in this build)",
                    self.sibling_path().display()
                ),
            },
        }
    }

    fn invoke(&self, args: &[String]) -> anyhow::Result<Value> {
        if self.descriptor.kind == AdapterKind::Library {
            bail!(
                "{INVOKE_UNAVAILABLE}: {} is a library-kind ELCI component with no CLI adapter wired up in this build",
                self.descriptor.name
            );
        }
        let path = self.resolve_binary().ok_or_else(|| {
            anyhow!(
                "{INVOKE_UNAVAILABLE}: {} binary not found",
                self.descriptor.name
            )
        })?;
        let output = Command::new(&path).args(args).output()?;
        Ok(json!({
            "adapter": self.descriptor.name,
            "binary": path.display().to_string(),
            "args": args,
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // KAMI_ELCI_ROOT is process-global state; serialize tests that touch it.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn every_descriptor_health_check_completes_without_panicking() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("KAMI_ELCI_ROOT");
        for adapter in component_registry() {
            let health = adapter.health();
            assert!(!health.detail.is_empty());
        }
    }

    #[test]
    fn a_directory_that_does_not_exist_is_capability_unavailable() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("KAMI_ELCI_ROOT", dir.path());

        let mesut = component_registry()
            .into_iter()
            .find(|a| a.name() == "mesut")
            .unwrap();
        assert!(!mesut.health().available);

        std::env::remove_var("KAMI_ELCI_ROOT");
    }

    #[test]
    fn library_kind_adapter_refuses_invoke_with_capability_unavailable() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("KAMI_ELCI_ROOT");
        let library_adapter = component_registry()
            .into_iter()
            .find(|a| a.kind() == AdapterKind::Library)
            .unwrap();
        let err = library_adapter.invoke(&[]).unwrap_err();
        assert!(err.to_string().contains(INVOKE_UNAVAILABLE));
    }

    #[test]
    fn a_sibling_directory_with_a_manifest_is_detected_as_a_present_library() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("dreamsequence")).unwrap();
        std::fs::write(
            dir.path().join("dreamsequence").join("Cargo.toml"),
            b"[package]\nname=\"dreamsequence\"",
        )
        .unwrap();
        std::env::set_var("KAMI_ELCI_ROOT", dir.path());

        let dreamsequence = component_registry()
            .into_iter()
            .find(|a| a.name() == "dreamsequence")
            .unwrap();
        assert!(dreamsequence.health().available);

        std::env::remove_var("KAMI_ELCI_ROOT");
    }
}
