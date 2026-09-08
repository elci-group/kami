use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Directories never counted toward source LOC: VCS metadata, build
/// output, vendored dependencies, and Kami/sibling ELCI state
/// directories. Kept intentionally small — this is a source-density
/// count, not a repository archaeology tool.
const IGNORED_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    ".venv",
    "venv",
    "__pycache__",
    ".kami",
    ".kakashi",
    ".isopod",
    ".amber",
    ".lwoodz",
];

/// Recognized source-file extensions and the language name reported for
/// each. Anything not listed here is excluded from `source_loc` — Kami
/// must never manufacture a LOC figure for a file type it hasn't
/// deliberately decided to count (§15).
fn language_for_extension(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "rs" => "Rust",
        "py" => "Python",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "tsx" => "TypeScript",
        "jsx" => "JavaScript",
        "go" => "Go",
        "java" => "Java",
        "c" | "h" => "C",
        "cpp" | "cc" | "hpp" | "cxx" => "C++",
        "rb" => "Ruby",
        "sh" | "bash" => "Shell",
        "toml" => "TOML",
        "yaml" | "yml" => "YAML",
        "json" => "JSON",
        "md" | "markdown" => "Markdown",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "sql" => "SQL",
        _ => return None,
    })
}

#[derive(Debug, Default)]
pub struct LocCount {
    pub total: u64,
    pub file_count: u64,
    pub by_language: BTreeMap<String, u64>,
    /// Paths that couldn't be read (permission errors, non-UTF8, etc.) —
    /// their omission is why `confidence` may be < 1.0.
    pub unreadable: Vec<String>,
}

/// Deterministic, exact source-line count over `root`, restricted to
/// recognized source extensions and excluding [`IGNORED_DIRS`]. This is
/// a measurement, not a projection — see `kami-sizing::project` for the
/// (not yet implemented) forward model in §15.
pub fn count_source_loc(root: &Path) -> LocCount {
    let mut count = LocCount::default();
    walk(root, &mut count);
    count
}

fn walk(dir: &Path, count: &mut LocCount) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            count.unreadable.push(dir.display().to_string());
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => {
                count.unreadable.push(path.display().to_string());
                continue;
            }
        };
        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if IGNORED_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk(&path, count);
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let Some(language) = language_for_extension(&ext.to_ascii_lowercase()) else {
            continue;
        };
        match fs::read_to_string(&path) {
            Ok(text) => {
                let lines = text.lines().count() as u64;
                count.total += lines;
                count.file_count += 1;
                *count.by_language.entry(language.to_string()).or_insert(0) += lines;
            }
            Err(_) => count.unreadable.push(path.display().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn counts_recognized_source_files_by_language() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n// two\n").unwrap();
        fs::write(dir.path().join("README.md"), "one\ntwo\nthree\n").unwrap();
        fs::write(dir.path().join("data.bin"), [0u8, 1, 2]).unwrap();

        let count = count_source_loc(dir.path());

        assert_eq!(count.total, 5);
        assert_eq!(count.file_count, 2);
        assert_eq!(count.by_language.get("Rust"), Some(&2));
        assert_eq!(count.by_language.get("Markdown"), Some(&3));
        assert!(count.unreadable.is_empty());
    }

    #[test]
    fn ignored_directories_are_excluded() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(
            dir.path().join("target").join("generated.rs"),
            "fn x() {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("real.rs"), "fn y() {}\n").unwrap();

        let count = count_source_loc(dir.path());

        assert_eq!(count.file_count, 1);
        assert_eq!(count.total, 1);
    }
}
