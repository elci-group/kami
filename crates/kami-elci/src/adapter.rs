use std::path::Path;

use serde_json::Value;

pub const INVOKE_UNAVAILABLE: &str = "CAPABILITY_UNAVAILABLE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    /// A sibling ELCI project that exposes a CLI binary Kami can shell
    /// out to.
    Binary,
    /// A sibling ELCI project present on disk with no CLI entry point
    /// wired up yet. Detected as present, but `invoke()` always reports
    /// `CAPABILITY_UNAVAILABLE`.
    Library,
}

#[derive(Debug, Clone)]
pub struct AdapterHealth {
    pub available: bool,
    pub detail: String,
}

/// The common adapter contract every ELCI sibling integration implements
/// (§29, §36 in the Kakashi directive; Kami mirrors the same doctrine:
/// never silently recreate a sibling's functionality, and never pretend
/// a missing sibling is present).
pub trait ElciAdapter {
    fn name(&self) -> &str;
    fn kind(&self) -> AdapterKind;
    fn detect(&self) -> bool;
    fn role(&self) -> &'static str;
    fn version(&self) -> Option<String>;
    fn health(&self) -> AdapterHealth;

    /// Invoke the adapter's underlying tool with its working directory set
    /// to `dir`. Some siblings (e.g. Fract's `index`) take no explicit
    /// path argument and always analyse the process's cwd — setting it
    /// explicitly is the only correct way to point them at a target
    /// project, so every caller goes through this rather than relying on
    /// Kami's own ambient cwd. Returns `Err` describing
    /// `CAPABILITY_UNAVAILABLE` when the tool isn't a directly invokable
    /// binary, or isn't detected at all.
    fn invoke_in(&self, dir: &Path, args: &[String]) -> anyhow::Result<Value>;

    /// Convenience for callers that don't care about the sibling's
    /// working directory (it takes an explicit path argument, or doesn't
    /// need one at all).
    fn invoke(&self, args: &[String]) -> anyhow::Result<Value> {
        self.invoke_in(Path::new("."), args)
    }
}
