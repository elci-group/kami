use anyhow::{anyhow, Context, Result};
use kami_elci::component_registry;
use std::path::Path;

/// Storage footprint measured by Bart's `--json` scan (§14): total bytes
/// and total file count, as Bart aggregates them at the root node.
pub struct BartSize {
    pub storage_bytes: u64,
    pub file_count: u64,
    /// Bart's raw JSON tree, kept as evidence data so a later loop can
    /// see exactly what was measured (§25 provenance).
    pub raw: serde_json::Value,
}

/// Invoke Bart through the ELCI adapter registry — never a bare
/// `Command::new("bart")` — so Bart's absence is reported the same way
/// every other sibling's absence is (§24, §29).
pub fn measure(root: &Path) -> Result<BartSize> {
    let adapters = component_registry();
    let bart = adapters
        .iter()
        .find(|a| a.name() == "bart")
        .ok_or_else(|| anyhow!("bart adapter missing from the ELCI registry"))?;

    let envelope = bart.invoke(&["--json".to_string(), root.display().to_string()])?;

    let exit_code = envelope.get("exit_code").and_then(|v| v.as_i64());
    if exit_code != Some(0) {
        let stderr = envelope
            .get("stderr")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        anyhow::bail!("bart exited with {exit_code:?}: {stderr}");
    }

    let stdout = envelope
        .get("stdout")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("bart adapter envelope missing stdout"))?;
    let raw: serde_json::Value =
        serde_json::from_str(stdout).with_context(|| "parsing bart --json output")?;

    let storage_bytes = raw
        .get("size")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("bart --json output missing top-level 'size'"))?;
    let file_count = raw
        .get("file_count")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("bart --json output missing top-level 'file_count'"))?;

    Ok(BartSize {
        storage_bytes,
        file_count,
        raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Exercises the real `bart` binary on PATH — no mock. If `bart`
    /// isn't installed in the environment running the tests, this test
    /// is skipped rather than failing, matching how `kami status`
    /// reports the same adapter as `CAPABILITY_UNAVAILABLE`.
    #[test]
    fn measures_a_real_directory_via_the_bart_binary() {
        if which::which("bart").is_err() {
            eprintln!("skipping: bart not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello world").unwrap();
        fs::write(dir.path().join("b.txt"), "more content here").unwrap();

        let result = measure(dir.path()).unwrap();

        assert_eq!(result.file_count, 2);
        assert!(result.storage_bytes > 0);
        assert!(result.raw.get("path").is_some());
    }
}
