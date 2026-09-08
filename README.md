# kami

The ELCI recursive potentiality & transformative development engine. Full architectural directive: `docs/DIRECTIVE.md`.

Kami's core distinction (§2 of the directive): **STATE ≠ POTENTIAL**. It exists to model what a project could credibly become without ever treating the projection as fact. This build implements the beginning of that pipeline — the non-mutating first-loop baseline (§12) — and nothing past it.

## What's implemented

A working Rust workspace (`cargo build --workspace`, `cargo test --workspace`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo deny check`, and `cargo audit` all pass) with a real, deliberately narrow, path through the doctrine:

- **`kami-core`** — the shared domain model: `Evidence` (with provenance and confidence, §25), `ProjectSize` (§14), and the state-transition types (`ProjectState`, `StateId` formatted as `STATE-000`, `Mission`, `Strength`, `Weakness`, `Constraint`, `Capability`, §23/§26). Pure data — no I/O, no inference.
- **`kami-evidence`** — the `.kami/` state directory: an append-only JSONL evidence log, and one immutable JSON file per persisted `ProjectState` (§26 — save refuses to overwrite an existing state id).
- **`kami-elci`** — ELCI sibling adapter detection for every named system in the directive (Bart, Amber, Curly, Traci, Fract, Ferret, Jeenome, Wilder, Isopod, Lwoodz, Kakashi, Vamos, Padagonia, InGauge, Deliver, plus DreamSequence/Mesut as library-kind), reporting `AVAILABLE`/`CAPABILITY_UNAVAILABLE` per sibling rather than ever silently reimplementing one (§24, §29). Library-kind detection is a simple `Cargo.toml`-at-root check, so a sibling whose Rust workspace lives in a subdirectory (DreamSequence's is at `dreamsequence/dreamboat/`) is honestly reported as not found rather than guessed at.
- **`kami-sizing`** — deterministic project sizing (§14): storage footprint and file count via the real `bart --json` adapter (never a bare `Command::new("bart")` — always through `kami-elci`'s registry), and an exact in-process source-LOC count by recognized extension. Every figure traces to a measurement; nothing here estimates.
- **`kami-battery`** — the §12 first-loop evidence battery, for real: Amber (dependency risk/replaceability), Traci (observability-rule diagnostics), Fract (architectural entropy/module health), Ferret (structural + safety findings), and Wilder (repository-wide normalized evidence) are each invoked through `kami-elci`'s adapter registry with their working directory set to the target project, and their parsed JSON output becomes an `Evidence` record. Curly and Jeenome are named in §12 but are *not* invoked — `kami-battery::run`'s two `NotApplicable` entries explain why (Curly is a command-expansion engine with no project-analysis output; Jeenome needs a live `strace` stream, not a static path) rather than fabricating a call to either. `kami-elci::ElciAdapter::invoke_in` sets the sibling's cwd explicitly (some, like `fract index`, take no path argument and always analyse cwd).
- **`kami-cli`** (binary `kami`) — `init`, `inspect`, `status`, `state build|list|show`, each supporting `--format human|json`, covered end-to-end by `crates/kami-cli/tests/cli.rs` against the compiled binary. `state build` now runs the full sizing + battery pass and folds every resulting `Evidence` into `STATE-000`; each tool's outcome (`evidence`/`unavailable`/`not_applicable`/`invocation_failed`) is also reported directly in the command's output so nothing about the battery is hidden behind a single pass/fail.

Try it against a real project:

```bash
kami --path /path/to/project init
kami --path /path/to/project inspect
kami --path /path/to/project status
kami --path /path/to/project state build --mission "..."
kami --path /path/to/project state show --id 0
```

## What's still directive, not code

This is the first-loop baseline only. Per the directive's own doctrine (§12: "the first Kami loop shall not immediately mutate the project"), nothing below is a bug — it's the boundary this build deliberately stops at:

- **No LLM inference wired in at all** (§6, §7, §31). No provider abstraction, no GPT-6-Astra, no Ollama, no InGauge-mediated rate limiting of inference traffic — `ingauge` is detected as an ELCI sibling (§9) but not yet invoked to govern anything, since there is no inference traffic yet to govern.
- **No mutation, projection, or recursion** (§17-§22). `Mutation`, `MutationCandidate`, `ProjectedState`, `PotentialTrajectory`, and `HyperbolicModel` are not implemented as types — they'd be speculative scaffolding with nothing to populate them yet. `kami state build` produces exactly one state (`STATE-000`, `StateKind::Actual`) and then refuses to run again, by design.
- **No bound-based source consolidation layer** (§5) — there is no LLM context to bound yet.
- **No Padagonia ontology/constraint substrate integration** (§4, §16) — `padagonia` is detected as an ELCI sibling but not invoked; `.kami/`'s evidence/state storage is flat JSON/JSONL, not a Padagonia-backed graph. Constraints, missions beyond a free-text statement, strengths, and weaknesses are typed in `kami-core` but nothing yet populates them from evidence — `state build` leaves them empty. Turning battery findings (e.g. Traci's `TRC007` panic-path diagnostics, Ferret's `safety/no-panic` findings) into typed `Strength`/`Weakness` records is the natural next step and isn't done yet — it would need either a deterministic scoring rule or an LLM interpretation step, neither of which exists in this build.
- **No Kakashi/Vamos/Isopod/Lwoodz invocation** (§10, §11, §13) — detected, not yet called.
- **No deterministic LOC forward-projection model** (§15) — `ProjectSize.lower_bound_loc`/`upper_bound_loc` in the sizing output are the *measured* current LOC, not a projected future figure; `kami-sizing`'s README-level assumption text says so explicitly rather than implying a forecast that isn't there.
- **No TUI** (§7), **no Deliver export** (§21, §28's `kami export --deliver`), **no state reconciliation/feedback learning** (§26, §27), **no Hyperbolic Sprint** (§22).
- **No CI workflow** — `cargo build/test/clippy/fmt/deny/audit` all currently pass locally; nothing runs them automatically yet.

## The `.kami/` state directory

```
.kami/
├── evidence/evidence.jsonl   # append-only, one Evidence per line
└── states/STATE-000.json     # one immutable file per persisted ProjectState
```

Override which directory `kami-elci` looks in for sibling projects with `KAMI_ELCI_ROOT` (defaults to the parent of the current working directory).
