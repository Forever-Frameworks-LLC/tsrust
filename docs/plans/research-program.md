# Research program

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Groundwork.

**Scope:** Track the required design studies and unresolved cross-cutting questions.

**Planning deliverable:** Research/design package and the four identity, competition, benchmark, and Hono feasibility documents.

**Dependencies to coordinate:** [principles-and-boundaries](principles-and-boundaries.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-51"></a>

## 51. Research Phase Deliverables

Before implementing the full language, produce a research/design package covering:
1. Language philosophy.
2. Formal ownership semantics.
3. Function argument inference.
4. Copy/clone/share semantics.
5. Shared ownership model.
6. Type system.
7. Trait/capability system.
8. Object model.
9. Closure model.
10. Async runtime.
11. Error model.
12. Collection semantics.
13. String semantics.
14. Numeric semantics.
15. Memory allocation strategy.
16. Cyclic data structures.
17. Reflection strategy.
18. Serialization strategy.
19. Rust interoperability.
20. Package management.
21. npm conversion feasibility.
22. compiler architecture.
23. debugging architecture.
24. security model.
25. incremental compilation.
26. performance targets.
27. native deployment model.
28. standard library boundaries.
For each topic provide:
Problem
Why it matters
Possible approaches
Advantages
Disadvantages
Performance impact
Developer-experience impact
Implementation complexity
Interaction with other language features
Recommended initial direction
Open questions
Prototype required? yes/no
Do not prematurely resolve questions where experimentation is necessary.

<a id="core-61"></a>

## 61. Questions That Must Remain Open During Groundwork

Do not force answers to these prematurely:
* How much ownership can realistically be inferred?
* When should inference stop and require explicit source syntax?
* What is the correct shared-ownership model?
* How should closures capture shared/mutable state?
* How should cyclic graphs work?
* Should arrays have value or shared semantics?
* Should strings be immutable?
* What should number mean?
* Should null and undefined both exist?
* Should Promise<T> exist as a source-level concept?
* Should async functions be Send by default?
* How should runtime reflection work?
* Should decorators exist?
* How should dynamic trait dispatch look?
* Should classes exist as a fundamental concept?
* Should inheritance exist?
* Should interfaces have runtime representations?
* Should exception syntax exist?
* How should cancellation work?
* How should database drivers expose borrowed data?
* Should request-scoped arenas be first-class?
* How much npm conversion is realistically possible?
* Should generated Rust remain inspectable?
* Is Rust-source generation viable long-term?
* How should native Rust dependencies be sandboxed?
* How should package versions coexist?
* How should compile times be kept competitive with Node development?
* How much JavaScript method compatibility is worth preserving?
* Which JavaScript behaviors should be intentionally rejected?
* What compiler optimizations are required to meet the performance thesis?
* What language features would make Rust lowering significantly harder?
* Which design choices could accidentally reintroduce GC-like overhead through pervasive reference counting?
These questions are part of the project, not obstacles to be hidden.

<a id="web-00"></a>

## Project Workstream: Identity, Competitive Positioning, Benchmarking, and Native Web Experience

This workstream follows the initial language-design proposal for a new backend-first, statically compiled language with TypeScript/JavaScript ergonomics, Rust-backed native compilation, strong static guarantees, inferred ownership, no tracing garbage collector, and explicit cost semantics.
The immediate goals are:
1. Select a distinctive and defensible language name.
2. Select a source-file extension.
3. Select a CLI/toolchain command.
4. Select names for the compiler, package manager, registry, and related ecosystem components if separate names are needed.
5. Research neighboring languages and transpilers to understand what already exists.
6. Clearly identify which ideas are established versus which aspects of this project should intentionally differ.
7. Establish a benchmark methodology against neighboring languages without incorporating, vendoring, or installing their source code into this project.
8. Investigate making a Hono-compatible HTTP framework/API a core built-in part of the language’s backend experience.
9. Investigate JSX/TSX-style native server-side HTML generation as a first-class feature.

<a id="web-34"></a>

## 34. Research Deliverables

Produce four documents before finalizing the language name.
Document A — Identity Matrix
For every viable name:
Name
Meaning
Pronunciation
Language collisions
CLI collisions
Package collisions
Trademark concerns
Domains
GitHub namespace
Suggested extension
Suggested CLI
Strengths
Weaknesses
Recommendation
Document B — Competitive Matrix
Compare this project against:
* RustScript
* AssemblyScript
* Perry
* Crisp
* Jux
* Rust
* Go
* Node
* Bun
* Deno
* other relevant languages discovered during research
Do not rank them globally.
Compare concrete architecture/features.
Document C — Benchmark Specification
Define workloads independently of any implementation.
Document D — Hono Compatibility Feasibility
Determine:
* API surface worth preserving
* behavior worth preserving
* behavior that should intentionally differ
* JSX feasibility
* middleware model
* native implementation architecture
* legal/licensing considerations
* migration experiments required
