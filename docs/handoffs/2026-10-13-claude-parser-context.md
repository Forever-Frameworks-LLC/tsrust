# Tsrust: parser and type-syntax architecture handoff

You own the parser/type-syntax portion of an ongoing compiler nesting migration. Implement it, validate it, and return reviewable commits plus evidence. Codex continues checker/ownership analysis and emission in the original checkout. Do not take over S12–S21, runtime optimization, or the full project goal.

## Checkout and coordination

Original checkout: `/Users/seancotter/Desktop/dev/learning/tsrust`.
Start from committed parser checkpoint:
`2f96cc92de3b3ed6f879ab108339a2f9585a9b62`.

Create your own worktree, not another writer in the original checkout:

```sh
git -C /Users/seancotter/Desktop/dev/learning/tsrust worktree add -b codex/claude-parser-continuations /Users/seancotter/Desktop/dev/learning/tsrust-claude-parser 2f96cc92de3b3ed6f879ab108339a2f9585a9b62
```

If that branch/path already exists, inspect it and reuse the intended checkout; do not reset or delete it. Inspect Git status first and preserve unrelated changes. Do not copy the original checkout's dirty files. They belong to Codex's current analysis/emission migration. Your parser baseline is already committed at the SHA above.

One agent writes each checkout. Do not modify the original checkout, installed compiler/editor consumers, or shared acceptance documents there. Return commits for integration; do not merge into Codex's branch yourself. Do not use extra agents unless the owner explicitly authorizes them.

Own parser implementation in `crates/tsrust-compiler/src/syntax/` and type-syntax parsing in `crates/tsrust-compiler/src/types/syntax.rs`, together with directly required parser helpers and parser-focused tests. Keep existing AST and annotation interfaces stable where possible. Codex owns `analysis/`, `emit/`, generic proof/specialization and integration acceptance. If a shared representation/interface change is necessary, describe the concrete change and coordinate it before editing shared contracts. These ownership boundaries are coordination boundaries, not a reason to leave a parser dependency unsolved.

Use a dedicated handoff document such as `docs/handoffs/claude-parser-continuations.md` for your findings rather than editing the shared pipeline plan. If another component blocks full-source execution, provide the exact reproducer and phase evidence to Codex and continue independent parser work.

## Read first

Read `AGENTS.md` and:

- `docs/development-workflow.md`
- `docs/guides/compiler-source-layout.md`
- `docs/plans/compiler-nesting-pipeline.md`
- `docs/handoffs/2026-10-13-s11-acceptance.md`
- `docs/plans/standard-library-roadmap.md`
- `docs/guides/language/standard-library-coverage.md`
- `docs/plans/schema-plan.md`

S01–S11 are accepted within documented boundaries. Do not reopen their contracts. The owner has authorized tracking previously protected later-work files, but preserve their contents and avoid unrelated edits.

## Objective and non-negotiable direction

Remove call-stack-dependent parser and annotation nesting limitations by implementing connected explicit continuations/worklists. Do not increase stacks or depth constants, add shape-specific rejection guards, or declare the last passing sample a support ceiling.

The owner specifically rejects small guard-oriented iteration loops. Work across the whole connected parser/type/default/body/recovery cycle. A helper is not stack-safe when it invokes another helper that starts a new recursive traversal. Include lexer/template interaction, annotation conversion/validation, error recovery and partial-tree disposal in the audit when they are reachable from parser work.

Preserve grammar, precedence, associativity, source spans, token provenance, generic/alias scopes, pending declaration state, closure identities, contextual parsing, evaluation order and diagnostic recovery. Preserve semantic rejection of genuinely invalid or cyclic constructs; distinguish that from arbitrary depth rejection. No language limits should be inferred from test sizes or external test timeouts. Measure resource growth before proposing an operational policy.

## Baseline: what already exists, what remains

`syntax/expression_work.rs` has expression work frames for precedence, unary completion, grouping/comma expressions, postfix access, calls and arguments, indexing, arrays, records, templates and conditionals. It also queues arrow closure bodies and simple nested blocks, return/throw/expression statements. Recovery frames retain parser state and work-stack extents; unwinding restores pending declarations and scope state.

This is incomplete. Control statements, declarations, named/anonymous function bodies, parameters/defaults, constructors and type annotations still have compatibility paths that can re-enter parsing. Audit the actual code; this list is a starting map, not exhaustive coverage.

The parser still contains prototype nesting guards, including 128-level checks in expression/block work and `types/syntax.rs`, plus postfix/depth validation paths. These guards are migration debt, not accepted product limits. Retire them as the connected architecture and consumers become safe; do not leave them as the final design. Do not remove them blindly while recursive cycles remain.

Clone/drop worklists already exist in syntax and annotation storage. Reuse them, but verify error/unwind paths rather than assuming their presence proves disposal safety. Existing shallow grammar/recovery/parity tests must continue to pass.

On the baseline, full-source tests still expose checker/emitter stack overflows. Codex is fixing these separately. In newer uncommitted Codex work, arrays, records, templates, field/index checking, aggregate emission and named/temporary storage operands now use continuations. Three emission-isolated tests pass at 4,096 layers; arrays/comma-contexts/temporary-index integration suites pass 26 tests. Those changes are NOT in your baseline. Do not reproduce or cherry-pick them into your parser work unless coordinated.

Newer full-source discovery passes array samples 8/16/32/64 and mixed call-record-array-access samples 8/16; larger samples still encounter the old parser guard. These are observations, not support ceilings. Broader Codex regressions are still running and have exposed nullable callback-record view failures; Codex owns their investigation. No compiler architecture acceptance or new formal scaling acceptance exists yet.

## Working loop

1. Map the connected parser execution graph from entry through syntax, types, defaults, bodies, recovery and disposal. Identify recursive strongly connected paths and state each suspended parent must retain. Use this to design the continuation model for the whole owned subsystem before selecting implementation sections.
2. Implement a coherent connected section. A child parse must return a request to the owning scheduler, not synchronously enter a second driver. Retain parent construction state, token position, pending declarations and scope/recovery restoration explicitly. Add focused witnesses during implementation, but avoid rebuilding after each small edit.
3. Once the section is coherent, run affected grammar, recovery, diagnostic-location and formatting tests. An earlier interface check is appropriate only when a concrete uncertainty blocks implementation. Preserve original failures. Repair and rerun affected checks; do not restart already-passed unrelated matrices.
4. Discover behavior with geometrically growing valid mixed inputs on ordinary stacks: expressions inside defaults inside closures and control-flow bodies; annotations interleaved with generic parameters/defaults; nested templates, records, arrays, calls and indexes; malformed deep inputs and recovery. Record phase completion, crash/diagnostic/timeout classification, time, memory and output size. A test timeout is an unfinished observation, not a language limit. Isolate parsing when downstream analysis is blocked, while retaining end-to-end reproducers for integration.
5. Audit all remaining recursive re-entry and obsolete depth guards in the owned subsystem. Continue implementing connected missing paths. Do not stop because one shape passes, one guard moves, or one fixed depth succeeds. Observe formatter, diagnostics and LSP-facing parse consumers where parser changes affect them; report cross-owner failures with evidence.
6. Format the whole project before each source checkpoint, preserve evidence, commit coherent work, and update your handoff. Distinguish source checkpoints from accepted architecture and implementation checks from performance measurements. Repeat until the owned parser objective is implemented and verified, or a specific external interface dependency needs coordination. Continue independent work while that dependency is pending.

## Build, formatting and evidence discipline

Use the documented single Cargo target with incremental compilation disabled:

```sh
export CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
```

Coordinate build/validation windows with Codex through the owner before using the shared target. Another checkout's build replaces artifacts there; never assume the compiler at a shared path matches your sources. Record source commit/diff and executable hashes. Do not prune shared artifacts while Codex uses them. Avoid simultaneous timing or memory measurements. If working on another machine, record its separate environment; do not compare measurements as though machines matched.

Run Rust tests serially with `-- --test-threads=1`. Keep `RUST_MIN_STACK` unset and do not introduce custom larger stacks. Batch affected checks after coherent sections, not whole compiler/LSP/benchmark matrices after every edit.

Before every source checkpoint, use a verified compiler and run:

```sh
TSRUST=/absolute/path/to/verified/tsrust node tools/format-project.mjs
```

The usual executable is `/private/tmp/tsrust-gate100-editor-target/debug/tsrust`, but verify ownership/source identity before using it. Preserve token equality, formatter idempotence and formatted-source behavior. No source snapshots or archive directories; use Git checkpoints.

`benchmarks/compiler/nesting-discovery/run.py` and the ignored `nesting_probe::observe_pipeline_stages` unit test provide initial discovery machinery. Inspect them before use. They report parse/analysis/emission/disposal separately. They are not formal timing acceptance. Existing matched compiler timing protocols retain two warmups and eleven measured samples; do not silently reduce them.

SQLite is canonical. The original checkout's `benchmarks/records.sqlite` is ignored and will not appear automatically in your worktree. Do not concurrently write or replace that database. Retain concise current-run logs, exact source hashes and closed evidence records for Codex to import through its single evidence writer. Codex must verify byte-exact restoration and SQLite integrity before removing verified loose copies. Return evidence paths and hashes in your handoff. No invented measurements, deleted failures or rewritten historical records.

## Deliverable

Return:

- Branch and ordered commit SHAs, with a clean explanation of final parser architecture.
- Exact tested source/executable identities and commands, passed checks, original failures and remaining gaps.
- Geometric discovery observations with operational budgets distinguished from language behavior.
- Any required interface changes or cross-owner reproducers, including diagnostic locations and stage classification.
- Closed evidence paths/hashes for SQLite import and your concise handoff document.

Do not mark the overall Tsrust goal, compiler nesting work, optimization or S12–S21 complete. Your acceptance claim must match the parser scope actually proven. Codex performs integration and broader release acceptance after reviewing and merging your commits.
