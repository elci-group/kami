//! The `.kami/` state directory (mirrors the `.kakashi/` convention):
//! append-only evidence, and one immutable JSON file per `ProjectState`
//! (§26 — state versioning). Nothing here is source code; it is Kami's
//! own working memory for the project it is analysing.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use kami_core::{Evidence, ProjectState, StateId};

pub const STATE_DIR: &str = ".kami";

pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open (creating if necessary) the `.kami/` directory under `workdir`.
    pub fn open(workdir: impl AsRef<Path>) -> Result<Self> {
        let root = workdir.as_ref().join(STATE_DIR);
        for sub in ["evidence", "states"] {
            fs::create_dir_all(root.join(sub))
                .with_context(|| format!("creating {}/{sub}", root.display()))?;
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn evidence_log(&self) -> PathBuf {
        self.root.join("evidence").join("evidence.jsonl")
    }

    /// Evidence is append-only: nothing here offers a way to edit or
    /// delete a past entry (§25).
    pub fn record_evidence(&self, ev: &Evidence) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.evidence_log())
            .with_context(|| format!("opening {}", self.evidence_log().display()))?;
        writeln!(file, "{}", serde_json::to_string(ev)?)?;
        Ok(())
    }

    pub fn read_evidence(&self) -> Result<Vec<Evidence>> {
        read_jsonl(&self.evidence_log())
    }

    fn state_path(&self, id: StateId) -> PathBuf {
        self.root.join("states").join(format!("{id}.json"))
    }

    /// Persist a state as an immutable JSON file. Refuses to overwrite an
    /// existing state id, since the state chain (§26) must stay append-only.
    pub fn save_state(&self, state: &ProjectState) -> Result<PathBuf> {
        let path = self.state_path(state.id);
        if path.is_file() {
            anyhow::bail!(
                "refusing to overwrite existing state {} at {}",
                state.id,
                path.display()
            );
        }
        let bytes = serde_json::to_vec_pretty(state)?;
        fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }

    pub fn load_state(&self, id: StateId) -> Result<ProjectState> {
        let path = self.state_path(id);
        let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// All persisted state ids, ascending.
    pub fn list_states(&self) -> Result<Vec<StateId>> {
        let dir = self.root.join("states");
        let mut ids = Vec::new();
        for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(rest) = name
                .strip_prefix("STATE-")
                .and_then(|s| s.strip_suffix(".json"))
            {
                if let Ok(n) = rest.parse::<u32>() {
                    ids.push(StateId(n));
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    /// The id the next persisted state should use: one past the highest
    /// existing state, or `STATE-000` if none exist yet.
    pub fn next_state_id(&self) -> Result<StateId> {
        Ok(self
            .list_states()?
            .last()
            .map(|id| id.next())
            .unwrap_or(StateId::ZERO))
    }
}

fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| Ok(serde_json::from_str(l)?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kami_core::ProjectState;

    #[test]
    fn evidence_round_trips_through_the_append_only_log() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();

        let ev = Evidence::new("project-size", "kami-sizing::bart", "bart --json .");
        store.record_evidence(&ev).unwrap();
        store.record_evidence(&ev).unwrap();

        let read = store.read_evidence().unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].id, ev.id);
    }

    #[test]
    fn state_ids_increment_and_states_refuse_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();

        assert_eq!(store.next_state_id().unwrap(), StateId::ZERO);

        let s0 = ProjectState::genesis(dir.path().to_string_lossy().to_string());
        store.save_state(&s0).unwrap();
        assert_eq!(store.next_state_id().unwrap(), StateId(1));

        let err = store.save_state(&s0).unwrap_err();
        assert!(err.to_string().contains("refusing to overwrite"));

        let loaded = store.load_state(StateId::ZERO).unwrap();
        assert_eq!(loaded.id, StateId::ZERO);
    }

    #[test]
    fn list_states_ignores_unrelated_files() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        fs::write(store.root().join("states").join("notes.txt"), b"hi").unwrap();
        assert!(store.list_states().unwrap().is_empty());
    }
}
