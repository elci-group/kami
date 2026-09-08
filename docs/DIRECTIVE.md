# KAMI

## Recursive Potentiality & Transformative Development Engine

**Status:** Technical Directive
**Implementation language:** Rust
**Primary paradigm:** Recursive additive mutation analysis
**Domain:** Software project development / potentiality modelling
**Core principle:** Separate project state from project potential, then recursively model credible state transitions without eroding existing strengths.

---

## 1. Executive Definition

Kami is a Rust-based development intelligence system that models the **potentiality space of a software project** independently from its current state.

Kami shall:

1. establish an evidence-backed representation of the current project state;
2. identify strengths, weaknesses, constraints, opportunities and latent capabilities;
3. construct additive mutations aligned with the project's mission;
4. preserve existing strengths unless an explicit trade-off is justified;
5. project the resulting state under realistic development returns;
6. recursively analyse the projected state;
7. terminate when additional mutation produces insufficient marginal value;
8. emit a bounded representation of the project's **visualised potential**;
9. expose that potential to deterministic execution systems such as Deliver.

Kami therefore does **not** primarily answer:

> "What should I code next?"

It answers:

> **"What could this project credibly become, and what sequence of state transitions would produce that outcome?"**

---

# 2. Fundamental Model

Kami shall maintain a strict distinction between:

```text
STATE
    What exists now.

POTENTIAL
    What could exist.

MUTATION
    A change capable of moving STATE toward POTENTIAL.

MATERIALISATION
    The realistically achievable implementation of a mutation.

PROJECTED STATE
    The state expected after materialisation.

TRAJECTORY
    A sequence of projected state transitions.

HYPERBOLE
    A deliberately expansive but internally coherent representation
    of a project's reachable potentiality space.
```

The core recursive operation is:

```text
S₀
 │
 ├── analyse
 │
 ▼
P₁
 │
 ├── constrain
 │
 ├── mutate
 │
 ├── estimate materialisation
 │
 ▼
S₁
 │
 ├── analyse
 │
 ▼
P₂
 │
 └── ...
```

Conceptually:

```text
S(n+1) = Materialise(
    Mutate(
        Analyse(S(n), Mission, Constraints)
    )
)
```

Kami must never treat the projected state as fact. It is a **modelled future state** with explicit confidence, assumptions and materialisation cost.

---

# 3. Design Doctrine

## 3.1 Additive mutation

The default mutation operator shall be additive.

Kami should prefer:

```text
A + B + C → A + B + C + D
```

over:

```text
A + B + C → A + D
```

unless removal or replacement is demonstrably necessary.

Every proposed mutation must identify:

* preserved strengths;
* enhanced strengths;
* weaknesses addressed;
* new capabilities introduced;
* dependencies introduced;
* potential regressions;
* resource requirements;
* implementation complexity;
* confidence.

---

## 3.2 Mission preservation

The project mission is an invariant.

Mutations that increase technical sophistication while reducing mission alignment must be penalised.

The optimisation objective is therefore approximately:

```text
maximize:

    mission_alignment
  + capability_gain
  + strength_amplification
  + opportunity_realisation
  + architectural_coherence

subject to:

    resource_constraints
    implementation_constraints
    compatibility_constraints
    compliance_constraints
    strength_preservation
```

---

## 3.3 Potential ≠ prediction

Kami must explicitly distinguish:

```text
possible
plausible
probable
expected
committed
```

A hyperbolic projection may contain capabilities that are not currently probable, provided that their inclusion is structurally justified.

Every projection must therefore carry confidence and assumption metadata.

---

# 4. Padagonia Integration

Kami shall use Padagonia as its ontology and constraint substrate.

Padagonia is not merely a persistence layer.

It shall represent the **potentiality space** in which Kami operates.

The ontology shall accommodate at minimum:

```text
Project
Mission
Capability
Component
Interface
Dependency
Strength
Weakness
Constraint
Resource
Mutation
PotentialState
ProjectedState
Opportunity
Risk
ComplianceRequirement
CompetitorTechnique
CLIEvent
Deliverable
Estimate
Evidence
Assumption
```

Relationships shall include:

```text
PROJECT ──HAS_MISSION──> MISSION
PROJECT ──HAS_CAPABILITY──> CAPABILITY
PROJECT ──HAS_CONSTRAINT──> CONSTRAINT
PROJECT ──EXHIBITS──> STRENGTH
PROJECT ──EXHIBITS──> WEAKNESS

MUTATION ──TRANSFORMS──> STATE
MUTATION ──PRESERVES──> STRENGTH
MUTATION ──ADDRESSES──> WEAKNESS
MUTATION ──REQUIRES──> RESOURCE

POTENTIAL_STATE ──DERIVED_FROM──> STATE
POTENTIAL_STATE ──CONTAINS──> CAPABILITY
POTENTIAL_STATE ──REQUIRES──> MUTATION

PROJECTED_STATE ──MATERIALISES──> MUTATION
PROJECTED_STATE ──BECOMES──> NEXT_STATE
```

Padagonia shall also provide:

### Budgeting

Kami must be able to reason against:

* token budget;
* inference budget;
* compute budget;
* storage budget;
* estimated implementation effort;
* project size;
* sprint duration;
* available development capacity.

### Constraints

Constraints shall be first-class ontology objects rather than prompt instructions.

This prevents recursive analysis from silently forgetting constraints introduced during earlier iterations.

---

# 5. Bound-Based Source Consolidation

LLM calls shall never receive unrestricted project context.

Kami shall implement a **bound-based source consolidation layer**.

Each inference receives a context envelope containing only evidence inside the active analytical boundary.

Example:

```text
BOUND
├── mission
├── relevant project topology
├── relevant source files
├── relevant historical evidence
├── active constraints
├── prior mutation
├── projected state
└── required analytical question
```

The consolidation system shall:

1. identify candidate evidence;
2. score relevance;
3. collapse duplicate information;
4. preserve provenance;
5. enforce token bounds;
6. prioritise high-information evidence;
7. exclude context outside the active analytical boundary.

The system should prefer:

```text
10 highly relevant files
```

over:

```text
entire repository
```

when answering a bounded question.

Source fragments shall retain provenance:

```text
source
path
symbol
line/range
commit/ref
analysis timestamp
relevance score
```

This is essential for recursive reproducibility.

---

# 6. LLM Orchestration

## Default reasoning model

The default general reasoning model shall be:

**GPT-6-Astra**

The architecture must nevertheless be provider/model agnostic.

Each inference step shall have a selectable:

```text
provider
model
context budget
reasoning mode
temperature / equivalent controls
```

where supported.

---

# 7. Interactive TUI

Kami shall provide a terminal-native interactive interface.

The TUI must allow the operator to select the inference model/provider **for each individual inference step**.

Example:

```text
┌─ KAMI / LOOP 03 ─────────────────────────────────────┐
│                                                       │
│ Step: Projected-state mutation analysis               │
│                                                       │
│ Provider       Model                    Cost           │
│                                                       │
│ > OpenAI       GPT-6-Astra              $$$            │
│   Ollama       qwen-local               LOCAL         │
│   Groq         <available model>        $              │
│   OpenRouter   <available model>        $$             │
│                                                       │
│ Context:       18,420 tokens                           │
│ Budget:        25,000 tokens                           │
│ Bound:         architecture + mission                 │
│                                                       │
│ [Enter] Run   [M] Model   [B] Bound   [V] View         │
└───────────────────────────────────────────────────────┘
```

The TUI shall support:

* provider selection;
* model selection;
* inference preview;
* context-bound inspection;
* token-budget inspection;
* estimated cost;
* rate-limit state;
* inference history;
* mutation approval;
* recursion depth;
* loop termination;
* projection comparison.

The operator must be able to override the default model without changing project configuration.

---

# 8. Ollama — Day One Requirement

Ollama compatibility is mandatory for the initial release.

The provider abstraction shall therefore support both:

```text
remote inference
local inference
```

without altering Kami's analytical pipeline.

A provider must implement a common interface resembling:

```rust
trait InferenceProvider {
    async fn models(&self) -> Result<Vec<Model>>;
    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResult>;
    async fn estimate(&self, request: &InferenceRequest) -> Result<InferenceEstimate>;
}
```

Ollama must be treated as a first-class provider rather than a fallback hack.

This allows:

```text
GPT-6-Astra → high-quality strategic inference
Ollama     → inexpensive/local iteration
```

within the same recursive trajectory.

---

# 9. InGauge Rate Management

All inference traffic shall pass through **InGauge-managed rate limiting**.

InGauge shall govern:

* request frequency;
* concurrent requests;
* provider quotas;
* token throughput;
* retry behaviour;
* backoff;
* burst limits;
* budget consumption.

Kami itself must not implement provider-specific rate-limit logic.

The relationship shall be:

```text
Kami
  ↓
Inference Scheduler
  ↓
InGauge
  ↓
Provider Adapter
  ↓
Model
```

This allows Kami to reason about analytical priority without coupling itself to transport constraints.

---

# 10. Kakashi Competitor Technique Modelling

Kami shall integrate Kakashi as its competitor-technique modelling subsystem.

Kakashi shall be used to identify:

```text
competitor capability
competitor implementation pattern
competitor interaction model
competitor workflow
competitor architectural technique
competitor optimisation
```

Kami must not merely ask:

> "Who are the competitors?"

It must ask:

> **"What techniques do other systems use to solve problems relevant to this project's mission?"**

These techniques become candidate mutations.

The resulting ontology should distinguish:

```text
COMPETITOR
    ↓
TECHNIQUE
    ↓
OBSERVED EFFECT
    ↓
TRANSFERABILITY
    ↓
ADAPTATION
    ↓
KAMI MUTATION
```

Kakashi therefore supplies **external evolutionary pressure** to Kami's internal mutation engine.

---

# 11. Vamos CLI Event-Chain Modelling

Vamos shall model proposed CLI interactions as event chains.

For CLI-oriented mutations, Kami should be capable of projecting:

```text
USER INPUT
    ↓
COMMAND PARSE
    ↓
VALIDATION
    ↓
STATE TRANSITION
    ↓
SIDE EFFECT
    ↓
OUTPUT
    ↓
NEXT EVENT
```

Vamos output shall be incorporated into potential-state modelling.

For example:

```text
Potential capability:
"interactive project mutation approval"

Vamos model:

kami
 ├── inspect
 ├── analyse
 ├── propose
 ├── compare
 ├── approve
 ├── materialise
 └── recurse
```

This allows Kami to reason about the **operational reality** of proposed capabilities rather than only their conceptual existence.

---

# 12. First-Loop Analysis Battery

The first Kami loop shall not immediately mutate the project.

It shall establish a high-resolution baseline through:

```text
Amber
Curly
Traci
Fract
Ferret
Jeenome
Wilder
```

These analyses shall be executed as independent evidence-producing passes.

Their outputs must be normalised into Padagonia.

Conceptually:

```text
                 ┌─ Amber
                 ├─ Curly
                 ├─ Traci
PROJECT ─────────┼─ Fract ──────┐
                 ├─ Ferret      │
                 ├─ Jeenome     ├──> Evidence Consolidation
                 └─ Wilder ─────┘
                                  │
                                  ▼
                             Padagonia
                                  │
                                  ▼
                              Kami S₀
```

No single analyser should be permitted to define project state.

Kami consumes their **aggregated evidence**.

Conflicting findings must be retained rather than silently averaged away.

---

# 13. Recursive Compliance Modelling

Every recursive loop shall invoke:

```text
Isopod
Lwoodz
```

for compliance modelling.

Compliance is therefore not a final gate.

It is recursive.

A mutation can introduce new compliance obligations, which must become part of the projected state.

```text
S₀
 ↓
Mutation
 ↓
Projected State
 ↓
Isopod + Lwoodz
 ↓
Compliance delta
 ↓
Updated constraints
 ↓
S₁
```

A mutation that materially increases capability but violates a hard constraint shall be rejected or explicitly marked infeasible.

Soft compliance concerns may instead become development costs or risks.

---

# 14. Bart-Based Project Sizing

Initial project sizing shall use Bart.

Bart shall establish the baseline project scale before recursive potential modelling begins.

At minimum:

```text
source LOC
file count
language distribution
storage footprint
dependency footprint
generated artefacts
historical growth
```

However, Kami shall not treat current LOC as a sufficient measure of future project size.

---

# 15. Deterministic LOC Forward Modelling

Kami shall implement deterministic estimated LOC projection through **aggregated forward modelling of storage demand**.

The model shall work from measurable quantities rather than LLM intuition.

Conceptually:

```text
Current Project
    ↓
Current storage demand
    ↓
Observed source-density ratios
    ↓
Component/capability growth assumptions
    ↓
Expected storage demand
    ↓
Estimated source expansion
    ↓
Projected LOC
```

The model should support:

```text
current LOC
current source storage
binary/storage overhead
language-specific density
historical growth rate
new component count
estimated component size
dependency growth
test growth
documentation growth
generated-code ratio
```

A projection must therefore be expressible as:

```text
EstimatedFutureLOC
    =
    CurrentLOC
    +
    Σ(ExpectedComponentLOC)
    +
    Σ(ExpectedTestLOC)
    +
    Σ(ExpectedSupportLOC)
```

with storage demand acting as an independent sanity constraint.

Kami must never manufacture precision where the available evidence does not justify it.

The output should include:

```text
estimated LOC
lower bound
upper bound
storage estimate
confidence
assumptions
```

---

# 16. Potentiality Space

Padagonia shall expose the potentiality graph to Kami.

A project can therefore be represented as:

```text
                    [Mission]
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
       [Capabilities]       [Constraints]
             │                   │
       ┌─────┴─────┐             │
       ▼           ▼             ▼
 [Strengths]   [Weaknesses]  [Limits]
       │           │             │
       └─────┬─────┴─────────────┘
             ▼
        [Mutations]
             │
       ┌─────┼─────┐
       ▼     ▼     ▼
      P₁    P₂    P₃
       │
       ▼
   [Materialisation]
       │
       ▼
      S₁
       │
       ▼
      ...
```

Kami's job is to navigate this graph.

---

# 17. Mutation Scoring

Every candidate mutation shall receive a deterministic composite score.

At minimum:

```text
mission_alignment
capability_gain
strength_amplification
weakness_reduction
opportunity_value
implementation_cost
resource_cost
constraint_penalty
compliance_cost
regression_risk
confidence
```

A conceptual score:

```text
MutationValue =
    MissionGain
  + CapabilityGain
  + StrengthGain
  + OpportunityGain
  - ImplementationCost
  - ResourceCost
  - ConstraintPenalty
  - RegressionRisk
  - ComplianceCost
```

The exact weighting shall remain configurable.

LLMs may propose and explain scores, but core accounting must remain deterministic wherever possible.

---

# 18. Recursive Loop

Each recursive iteration shall execute:

```text
1. Freeze current state.
2. Gather bounded evidence.
3. Evaluate state.
4. Identify strengths.
5. Identify weaknesses.
6. Reassert mission.
7. Load constraints from Padagonia.
8. Generate candidate mutations.
9. Apply additive-preservation filter.
10. Score mutations.
11. Select/project viable mutations.
12. Run competitor technique comparison through Kakashi.
13. Model CLI/event implications through Vamos where applicable.
14. Estimate materialisation.
15. Run Isopod/Lwoodz compliance modelling.
16. Recalculate project size through Bart where material.
17. Construct projected state.
18. Calculate marginal potential gain.
19. Persist state transition.
20. Repeat or terminate.
```

---

# 19. Termination

Kami shall terminate recursion when one or more conditions occur:

```text
marginal potential gain < configured threshold

projected mutation cost > available budget

constraint space becomes infeasible

projected state converges with current state

maximum recursion depth reached

confidence falls below minimum threshold

no mutation materially improves mission alignment
```

Termination must produce an explicit reason.

---

# 20. Hyperbolic Output

The final output is the **Hyperbolic Potential Model**.

It shall contain:

```text
Current State
        ↓
Observed Potential
        ↓
Mutation Trajectory
        ↓
Projected States
        ↓
Highest-value Future State
        ↓
Remaining Unmaterialised Potential
```

The hyperbolic model should answer:

> If this project's strengths are preserved, its weaknesses are progressively addressed, its constraints are respected, and realistic development returns are achieved, what could the project become?

This is the principal strategic artefact produced by Kami.

---

# 21. Deliver Interface

Kami shall not turn its hyperbolic output directly into implementation tasks.

Instead:

```text
KAMI
Potential State
      ↓
Potential Delta
      ↓
DELIVER
Deterministic Roadmap
      ↓
Implementation
```

Deliver receives:

```text
current state
target state
state delta
required mutations
dependencies
constraints
acceptance conditions
size estimates
priority
```

Deliver then determines the deterministic execution sequence.

This maintains architectural separation.

---

# 22. Hyperbolic Sprint

A Hyperbolic Sprint is an execution mode in which Deliver attempts to materialise a bounded segment of Kami's projected potential within a deliberately compressed development window.

Input:

```text
Current State
+
Target Potential State
+
Sprint Budget
+
Sprint Duration
```

Output:

```text
prioritised mutation sequence
dependency graph
implementation boundaries
acceptance criteria
resource allocation
```

The sprint must not redefine the target simply because the target is ambitious.

Instead, it must report:

```text
target
achievable subset
blocked subset
deferred subset
```

This preserves the distinction between **potential** and **materialisation capacity**.

---

# 23. Rust Architecture

Recommended crate structure:

```text
kami/
├── crates/
│   ├── kami-cli/
│   ├── kami-core/
│   ├── kami-ontology/
│   ├── kami-state/
│   ├── kami-potential/
│   ├── kami-mutation/
│   ├── kami-recursion/
│   ├── kami-context/
│   ├── kami-inference/
│   ├── kami-providers/
│   │   ├── openai/
│   │   ├── ollama/
│   │   └── ...
│   ├── kami-budget/
│   ├── kami-sizing/
│   ├── kami-compliance/
│   ├── kami-kakashi/
│   ├── kami-vamos/
│   ├── kami-tui/
│   └── kami-integrations/
├── Cargo.toml
└── README.md
```

Core domain types should include:

```rust
ProjectState
PotentialState
Mutation
MutationCandidate
ProjectedState
StateTransition
Constraint
Mission
Strength
Weakness
Evidence
SourceBound
InferenceRequest
InferenceResult
MaterialisationEstimate
ComplianceAssessment
ProjectSize
PotentialTrajectory
HyperbolicModel
```

---

# 24. Deterministic / Probabilistic Boundary

Kami shall maintain a hard architectural distinction.

### Deterministic systems

```text
Padagonia
InGauge
Bart
source consolidation
budget accounting
constraint enforcement
state persistence
trajectory bookkeeping
```

### Probabilistic systems

```text
LLM reasoning
mutation ideation
semantic interpretation
competitor technique interpretation
future-state synthesis
```

Probabilistic outputs must enter the deterministic layer as typed, validated objects.

An LLM must never directly mutate project state.

---

# 25. Provenance

Every significant Kami conclusion must be traceable to:

```text
source evidence
analysis subsystem
inference provider
model
inference timestamp
context bound
prompt/analysis version
assumptions
confidence
```

This is essential because recursive reasoning otherwise creates epistemic drift.

A later loop must be able to determine:

> "Why does Kami believe this capability exists?"

rather than merely inheriting an assertion from iteration N−4.

---

# 26. State Versioning

Every recursion shall create an immutable state transition record.

Example:

```text
STATE-000
    ↓
MUTATION-014
    ↓
STATE-001
    ↓
MUTATION-027
    ↓
STATE-002
```

Projected states and actual states must be distinguishable.

```text
PROJECTED_STATE
ACTUAL_STATE
```

When Deliver materialises a mutation, Kami should be able to reconcile:

```text
expected transition
vs
actual transition
```

This creates a feedback mechanism for improving future materialisation estimates.

---

# 27. Feedback Learning

Kami shall eventually maintain:

```text
predicted mutation
actual mutation
predicted cost
actual cost
predicted LOC
actual LOC
predicted storage
actual storage
predicted capability gain
observed capability gain
```

This permits calibration of the potentiality model over time.

The system should therefore become progressively better at answering:

> "When we say this project could become X, how often do we actually manage to materialise X?"

That metric should become a core quality indicator for Kami.

---

# 28. CLI Surface

Initial command surface:

```text
kami init
kami inspect
kami analyse
kami potential
kami loop
kami project
kami compare
kami trajectory
kami hyperbole
kami budget
kami constraints
kami evidence
kami state
kami reconcile
kami sprint
kami providers
kami models
kami tui
```

Primary invocation:

```text
kami loop
```

Interactive execution:

```text
kami tui
```

Potential visualisation:

```text
kami hyperbole
```

Deliver handoff:

```text
kami export --deliver
```

---

# 29. Initial Execution Contract

The first execution of Kami against a project shall follow:

```text
Bart
 ↓
Amber
 ↓
Curly
 ↓
Traci
 ↓
Fract
 ↓
Ferret
 ↓
Jeenome
 ↓
Wilder
 ↓
Evidence Consolidation
 ↓
Padagonia State Construction
 ↓
Isopod + Lwoodz
 ↓
Kakashi
 ↓
Vamos
 ↓
GPT-6-Astra / selected model
 ↓
Mutation Space
 ↓
Projected State
 ↓
Recursive Loop
```

The ordering may be parallelised where dependencies permit, but **state construction must occur only after the initial evidence battery has been consolidated**.

---

# 30. Non-Goals

Kami shall not become:

* an autonomous coding agent;
* a replacement for Deliver;
* a generic project-management tool;
* a static code-quality scanner;
* an LLM wrapper;
* an unrestricted repository summariser;
* a task generator masquerading as strategic analysis.

Its unique responsibility is:

> **Modeling and recursively expanding the credible potentiality space of a project.**

---

# 31. Definition of Done — v0.1

Kami v0.1 is complete when it can:

* initialise against a Rust/software project;
* execute the complete first-loop analysis battery;
* construct an ontology-backed project state;
* measure project size through Bart;
* establish deterministic LOC/storage projections;
* load budgets and constraints from Padagonia;
* consolidate bounded source context;
* invoke GPT-6-Astra through the provider abstraction;
* invoke at least one Ollama model locally;
* enforce inference limits through InGauge;
* present an interactive per-step model/provider selector;
* model competitor techniques through Kakashi;
* model CLI event chains through Vamos;
* execute recursive Isopod/Lwoodz compliance checks;
* generate typed mutations;
* construct projected states;
* calculate marginal potential gain;
* terminate recursion deterministically;
* generate a Hyperbolic Potential Model;
* export a deterministic mutation delta suitable for Deliver.

---

# 32. Architectural Principle

The central Kami invariant is:

```text
STATE ≠ POTENTIAL
```

The system exists to maintain that distinction.

Its loop is:

```text
        OBSERVE
           │
           ▼
        PRESERVE
           │
           ▼
         MUTATE
           │
           ▼
       MATERIALISE
           │
           ▼
         PROJECT
           │
           ▼
        RE-OBSERVE
           │
           └───────────────┐
                           ▼
                         LOOP
```

Kami therefore acts as the **potentiality engine** in the ELCI development stack.

UNI establishes condition.

Kami establishes possibility.

Deliver establishes execution.

DreamSequence establishes what actually happened.

The resulting actual state returns to Kami, where possibility is recalculated.

That creates a closed developmental system:

```text
                    ┌──────────────┐
                    │     UNI      │
                    │    STATE     │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │     KAMI     │
                    │  POTENTIAL   │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   DELIVER    │
                    │  EXECUTION   │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    ACTUAL    │
                    │    STATE     │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ DREAMSEQUENCE │
                    │  REFLECTION  │
                    └──────┬───────┘
                           │
                           └──────────► KAMI
```

**Kami's output is therefore not a roadmap. It is a model of where the project can go.**

The roadmap is what happens after that model encounters reality.

This is the important architectural boundary: **Kami should be allowed to be imaginative, but never allowed to be epistemically sloppy.** The LLM generates the hypotheses; Padagonia, Bart, InGauge, the evidence graph, and the deterministic state machinery constrain what those hypotheses are allowed to mean.
