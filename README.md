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
- **`kami-compliance`** — recursive compliance modelling (§13), a separate pass from the §12 battery per the directive's own pipeline ordering: Isopod's `check --json` (governance-control status) and Lwoodz's `--json check` (dependency license compatibility) are invoked through the same `kami-elci` adapter pattern. Isopod may exit non-zero on a genuine control failure — that's real signal, recorded in provenance, and never treated as an invocation error.
- **`kami-findings`** — deterministic extraction of typed `Strength`/`Weakness`/`Constraint` records from battery (§12) and compliance (§13) evidence (§18 steps 3-5), with no LLM in the loop (§24's deterministic/probabilistic boundary — this is a pure function from JSON to typed records). Each extractor is built and tested against a real captured example of that tool's own schema, not a guessed one: Amber's `metadata.cve_count` (not its `score.classification` opinion) drives a dependency-CVE weakness or a clean-dependencies strength; Fract's top-level `summary.{warning,critical,score}` rollup (not per-finding severity, which reads `"info"` even on a fully healthy scan) drives an architecture-health strength or weakness; Traci and Ferret findings are grouped by rule id, keeping only their real elevated severities (`critical`/`error`, `major`/`critical` respectively); Wilder's `HIGH`/`CRITICAL` findings become weaknesses, but Wilder never yields a strength — its own `limitations` field discloses partial coverage, so an empty `findings` array is treated as "nothing measured," not "nothing wrong." Isopod's `FAIL`-status controls each become a `Soft` `Constraint` (a compliance gap is a cost/risk, not an outright blocker — §13); Lwoodz's licensing obligation is always recorded as a `Hard` `Constraint` (it applies whether or not it's currently satisfied), with an actual incompatibility additionally surfaced as a `Weakness`.
- **`kami-capabilities`** — deterministic `Capability` discovery (§4) from `cargo metadata --no-deps --format-version=1` — every Cargo project already exposes this, so nothing here parses `Cargo.toml` by hand. `cargo` is the ambient toolchain, not an ELCI sibling, so this doesn't go through `kami-elci`'s registry (the same reasoning `kami-sizing`'s LOC walk uses). Only `bin`/`lib` targets become capabilities; `test`/`bench`/`example` targets are internal tooling, not something the project *provides*. A project with no `Cargo.toml` is reported `NotApplicable`, never guessed at — and a path-dependency workspace (like this one) moved to a different location on disk will correctly report `InvocationFailed` rather than fabricate a result, since `cargo metadata` itself can't resolve the sibling paths from the new location.
- **`kami-padagonia`** — a real Padagonia graph export of each `ProjectState` (§4, §16), additive to (never replacing) `kami-evidence`'s flat JSON, which stays authoritative. Padagonia is used as a *library* dependency (`padagonia = { path = "../../../padagonia" }`, the same pattern the sibling `ferret-padagonia` crate uses), not shelled out to — its CLI has no "add a node" subcommand; it's built around synthetic-graph benchmarking and file inspection. `write_state` creates a `ProjectState` node plus `Mission`/`Strength`/`Weakness`/`Constraint`/`Capability` nodes, each linked by the exact relationship §4 names (`has_mission`, `exhibits`, `has_constraint`, `has_capability`) with real `Provenance` (`agent: "kami"`, `model: "deterministic"` — recording that no LLM touched it). Padagonia's own `domain.rs` (Dreamsequence's control-plane ontology) and `software_ontology.rs` (structural-transformation boundaries) are for different consumers and don't cover Kami's Project/Mission/Strength/Weakness vocabulary, so this crate defines Kami's own node/relation labels rather than reusing either. `export_state`/`summarize` round-trip through Padagonia's real on-disk MessagePack format (`.kami/graph.pad`), tested against the actual file, not an in-memory stand-in.
- **`kami-cli`** (binary `kami`) — `init`, `inspect`, `status`, `state build|list|show`, `graph`, each supporting `--format human|json`, covered end-to-end by `crates/kami-cli/tests/cli.rs` against the compiled binary. `state build` runs the full sizing + battery + compliance + capability-discovery passes, extracts typed findings from every evidence-producing tool, folds all of it (`Evidence`, `Strength`, `Weakness`, `Constraint`, `Capability`) into `STATE-000`, and additionally exports the state to `.kami/graph.pad` — a failure there is reported in the output (`padagonia.exported: false` + an error), never propagated as a command failure, since the flat JSON state is already saved by that point. `kami graph` summarizes the exported graph's node/edge counts by label. Each pass's outcome (`evidence`/`unavailable`/`not_applicable`/`invocation_failed`) is also reported directly in `state build`'s output so nothing is hidden behind a single pass/fail.

Try it against a real project:

```bash
kami --path /path/to/project init
kami --path /path/to/project inspect
kami --path /path/to/project status
kami --path /path/to/project state build --mission "..."
kami --path /path/to/project state show --id 0
kami --path /path/to/project graph
```

## What's still directive, not code

This is the first-loop baseline only. Per the directive's own doctrine (§12: "the first Kami loop shall not immediately mutate the project"), nothing below is a bug — it's the boundary this build deliberately stops at:

- **No LLM inference wired in at all** (§6, §7, §31). No provider abstraction, no GPT-6-Astra, no Ollama, no InGauge-mediated rate limiting of inference traffic — `ingauge` is detected as an ELCI sibling (§9) but not yet invoked to govern anything, since there is no inference traffic yet to govern.
- **No mutation, projection, or recursion** (§17-§22). `Mutation`, `MutationCandidate`, `ProjectedState`, `PotentialTrajectory`, and `HyperbolicModel` are not implemented as types — they'd be speculative scaffolding with nothing to populate them yet. `kami state build` produces exactly one state (`STATE-000`, `StateKind::Actual`) and then refuses to run again, by design.
- **No bound-based source consolidation layer** (§5) — there is no LLM context to bound yet.
- **Padagonia graph export is additive-only, not a real constraint substrate yet** (§4, §16) — `.kami/graph.pad` mirrors a persisted `ProjectState`, but nothing reads constraints back out of it to feed a later loop, there's no cross-state graph (each export is a fresh single-state graph, not an accumulating one), and Padagonia's vector search / HNSW index / query engine aren't used at all.
- **No mutation scoring** (§17) even though strengths/weaknesses/constraints now exist to score against — there is no `MutationValue` accounting yet because there are no candidate mutations yet (§17-§22 stay unimplemented, see above).
- **No Kakashi/Vamos invocation** (§10, §11) — detected, not yet called. (Isopod/Lwoodz *are* invoked now, via `kami-compliance` — see above.)
- **No deterministic LOC forward-projection model** (§15) — `ProjectSize.lower_bound_loc`/`upper_bound_loc` in the sizing output are the *measured* current LOC, not a projected future figure; `kami-sizing`'s README-level assumption text says so explicitly rather than implying a forecast that isn't there.
- **No TUI** (§7), **no Deliver export** (§21, §28's `kami export --deliver`), **no state reconciliation/feedback learning** (§26, §27), **no Hyperbolic Sprint** (§22).
- **No CI workflow** — `cargo build/test/clippy/fmt/deny/audit` all currently pass locally; nothing runs them automatically yet.

## The `.kami/` state directory

```
.kami/
├── evidence/evidence.jsonl   # append-only, one Evidence per line
├── states/STATE-000.json     # one immutable file per persisted ProjectState
└── graph.pad                 # Padagonia export of the same state (additive, §4/§16)
```

Override which directory `kami-elci` looks in for sibling projects with `KAMI_ELCI_ROOT` (defaults to the parent of the current working directory).
