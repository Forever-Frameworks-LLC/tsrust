# Continuation: compiler foundations through new roadmap features

This is the current continuation entry point. Source basis: `39aaec7d` on `main`
(object spread continuations), following `dbd03e89` (array spread) and `fa3b0d98`
(spread checking/emission). S01–S11 are accepted within their documented boundaries;
compiler nesting, the required optimization phase and S12–S21 are not complete.

At this handoff the existing goal is **paused**, not complete. Preparing these
instructions does not resume it. When the owner resumes work, continue the existing
objective through the phases below. Do not create a duplicate goal if one exists.
The historical S11-only stop instructions are superseded for resumed continuation.
Do not stop again at S11, a helper checkpoint, or a single repaired crash.

## Read first and reconcile

Inspect Git status before editing. Read `AGENTS.md`,
[development workflow](../development-workflow.md),
[compiler source layout](../guides/compiler-source-layout.md),
[roadmap](../standard-library-roadmap.md),
[coverage](../standard-library-coverage.md),
[schema plan](../plans/schema-plan.md), and
[S11 acceptance](2026-10-13-s11-acceptance.md).
Then use this handoff with the [pipeline map](../plans/compiler-nesting-pipeline.md)
and [architecture plan](../plans/compiler-nesting-architecture.md). Those plans
contain historical progress sections; later accepted source/evidence supersedes
an earlier statement that a migrated path still recurses. Their original
observations must remain intact. The pipeline's initial phase table is historical,
not a description of the current parser or every current emitter path.

Use one agent in this tree. Preserve unrelated owner changes and the separate
`codex/stdlib-path` worktree. Owner files include `UI_VISION.md`,
`docs/plans/axum-hono-http-ui-contract.md`, `editable-list-app-example.md`,
`parallel-ui-foundations-plan.md`, `ui-feasibility-review.md` and
`ui-route-contracts-first-task.md`. The owner has authorized tracking these intended
later-work files; preserve their contents and do not mix semantic changes to them
into compiler work. Inspect current status instead of assuming they are untracked.

## Current accepted architecture

The shared checker scheduler is `analysis/continuations/driver.rs`, with dedicated
function, parameter, method, adapter, deferred-call, generic, spread and constructor
frames. These own suspended expressions/bodies and resume their parents. Maintain
context identity, expected type, access mode, active loans, capture boundaries,
lexical restoration, generic scope, error cleanup and result-publication order.
`finish_builtin_result` must still enforce cloneability and capture-slot collapse
when a builtin continuation bypasses ordinary dispatch.

| Accepted source checkpoint | What can be reused |
| --- | --- |
| `79295194` parser integration | One heap scheduler across the whole grammar and separate annotation frames; public admission remains migration debt. |
| `0039df14`, `26dba7ce` editor integration | Current-text routing, cursor repair, compiler/editor agreement, typing harness and real extension-host regression. Already merged, imported and deployed; do not reintegrate Claude branches. |
| `493cf4b7` through `f025fcba` | Shared statement/emission, storage, exception and Map completion continuations. |
| `cf3caf6b`, `5ca42ea8`, `01c470ca` | Named bodies, defaults/destructuring and closure methods on the shared checker stack. |
| `349df188`, `22e393e2`, `8755de98` | Collection callbacks, reductions/sorting and generic adapter wrappers with cleanup. |
| `ae58b042`, `6e2d4fe0`, `af1078b7` | Deferred calls, generic inference/result requests and spread/receiver/optional preparation. Compatibility entries still need auditing. |
| `fa3b0d98`, `dbd03e89`, `39aaec7d` | Spread calls, array spread constructors and object spread checking/emission. |

AST/annotation cloning and disposal also have worklists. This does not establish
that checked side tables, debug rendering or every auxiliary consumer is safe.
Refer to current code and the pipeline history before repeating an old conversion.

[Object spread acceptance](object-spread-continuations.md) records the latest
section: 50 distinct driver units, 278 compiler integration tests and 11 editor
tests pass. Normal-stack discovery completes the full pipeline through 8,192
object-spread layers. At 32,768, analysis finishes but emission exceeds its
30-second observation budget. This is an unresolved scaling signal, not a source
limit or formal timing result. Two incorrect new output oracles were corrected;
the original failed run remains in SQLite.

## Phase A: finish the compiler architecture

Complete coherent paths across checking, emission, diagnostics and disposal.
The following is the known backlog, not a claim that a text search exhausts it.
Use the pipeline map to find remaining cross-phase cycles before choosing edits.

### Immediate constructor section

Set and Map constructors are confirmed recursive consumers. Baseline witnesses
complete eight layers and overflow during analysis at 32 on normal stacks.
SQLite `tsrust-nesting-constructor-audit-*` contains exact sources and LLDB traces.
Object spread from that audit is now repaired; Set and Map are not.

Start in `analysis/sets.rs::set_constructor`, `analysis/maps.rs::map_constructor`,
`emit/set_emit.rs::set_constructor_expression` and
`emit/map_emit.rs::map_constructor_expression`. Schedule source expressions on the
existing checker/emitter stacks. Preserve arity and shadow checks, declaration and
expected-type materialization, the empty scoped-Map exception, contextual literal
arrays, read-mode source access, Map value cloneability, type conversion and source
spans. Restore a moved child before propagating its error. Constructor completion
uses the ordinary expression finish; it must not inherit the source-call capture
postlude merely because another task shares a completion variant.

Retain source evaluation once and in order, null/undefined evaluation, Set string
iteration, literal-string storage, Map tuple/source iteration and requested
read/mutate wrappers. Pure emitter assembly must consume completed child fragments.
Include native constructors and non-callback builtin helpers in the surrounding
re-entry audit; repairing these two observed crashes does not close that audit.

### Remaining connected checking and type work

- Audit synchronous compatibility entries in `analysis/generic_calls.rs`,
  `generic_expression_adapters.rs`, `deferred_generic_values.rs`,
  `deferred_generic_preparation.rs` and `return_type_inference.rs`:
  `prepare_generic_call`, `prepare_named_result_hint` and
  `deferred_generic_result_hint`. A compatibility driver may remain a root entry,
  but a task already running on the shared stack must request child work instead
  of starting a nested driver.
- Audit callable-result discovery, `discover_body`, `resolved_call`, `forward_call`,
  fresh/contextual hints and remaining specialization entries. Preserve the
  ordering of callable versus data-result publication and real callee contracts.
  Diagnose semantic cycles/nonconverging specialization separately from finite
  expensive input.
- Inventory all remaining guards and representation bounds in types, aliases,
  substitution, callable resolution, environment lifetimes and generic instances.
  Known examples include discovery depth 128, callback specialization counts,
  per-declaration 128/program 1,024 generic counts and array rank representation.
  These are not presumed hardware limits. Redesign a representation or traversal
  where necessary; do not raise a constant or blindly remove a termination check.
- Audit proof/inference replay and context cloning. Generic tasks currently retain
  read-only context snapshots; spread/fresh sources and proof requests can copy
  substantial trees/state. Use dependency-aware summaries and invalidation that
  preserve callable origin, capture-instance identity, generic scope and
  point-sensitive ownership/effects. Do not cache by function/type name alone.

### Complete pipeline consumers and correctness

- Retire public parser nesting admission only when downstream checking, type
  operations, finalization, emission, diagnostic replay, project linking, editor
  requests, cloning and both success/error disposal are safe for admitted shapes.
  The parser itself is already iterative. Do not repeat parser implementation.
- Formatter nesting rescans and alias chains have quadratic behavior. Compact
  generic closers such as `x: Box<int>=1` still cause token insertion costs; a lexer
  change must cover parser, formatter and editor consumers together. Preserve
  tokens, locations, recovery and packed `>>` behavior.
- Emission still copies descendant fragments and has poor scaling for some deep
  families. Use output/fragment ownership with bounded assembly cost; profile
  before attributing every timeout to the same cause. Generated Rust need not be
  formatted or indented for the backend. Source formatting remains required.
- Repair `tsrust-lsp/tests/optional_main.rs::invalid_return_in_dependency_does_not_mask_independent_entry_error`.
  A bad imported return currently masks an independent entry diagnostic: one
  diagnostic instead of two. It reproduces on the old baseline and remains
  unresolved. Keep diagnostic replay separate from executable analysis; do not
  weaken callee contracts or the assertion to make it pass.
- Review internal linked native names leaking into diagnostics. Preserve spans
  while exposing useful source-facing type names.
- The editor still has inference/catalog duplication. Three compiler-query gaps
  remain documented in [Claude LSP handoff](claude-lsp-agreement.md): complete
  type members/signatures, expression type at a span with canonical spelling,
  and standard-module enumeration. Method agreement is one-directional today.
  Preserve existing cursor repair and typing-speed behavior. Syntax errors
  elsewhere still suppress some narrowed/open-closure completions; untyped catch
  bindings remain intentionally silent. `Result.payload` follows the existing
  common-field rule for general Result values. The owner subsequently requested
  changing the schema-facing safeParse surface; see the API follow-up below.

### Architecture acceptance and exit

Do not choose another arbitrary depth ceiling or enlarge stacks. Discover actual
behavior with geometric finite inputs on normal stacks, including mixed shapes,
long ordinary and recursive dependency graphs, captures, generics, types, effects,
errors, modules and editor consumers. Observe phase, work, time and memory. A test
budget protects the machine; a timeout is not proof that a larger input is invalid.
Any resource admission policy must follow measured costs and device capacity,
with clear diagnostics, rather than a convenient passing witness count.

Finish affected sections and then batch checks. At the completed architecture
milestone, run complete compiler/LSP regressions and formal compilation scaling
with preserved protocols, identities and individual samples. Keep semantic
rejections, unresolved failures, discovery observations and measurements separate.
All required crashes/guards and known correctness defects above must be resolved
or explicitly accounted for against the acceptance contract; no silent deferral
or claim of completion based on a few scalar chains. Preserve completed suites,
repair affected failures and resume unrun groups rather than restarting full runs.

## Owner-requested schema API follow-up (2026-10-14)

After the connected nesting work, implement the [schema result API alignment](../plans/schema-result-api-plan.md)
before new schema optimization measurements and before S12. The owner requests
Zod-like parsing rather than an unexplained payload wrapper. Target safeParse
outcomes are `{ success: true, data: Output }` and
`{ success: false, error: ValidationError }`, with checked narrowing and full
editor support. `parse` already returns the output directly and throws on invalid
input; verify and preserve that behavior. Do not flatten a user-defined payload
field or change all general Result values. Update compiler/native/editor,
examples, consumers and matched benchmarks together. This is pending, not an
implemented API. Preserve native ownership and original measurement identities.

## Phase B: required optimization of existing code

After architecture acceptance, optimize existing supported code before S12.
The owner's requirement is comparable handwritten Rust timings and allocations,
and at least comparable Node-after-tsc performance on matched workloads. Do not
replace it with a universal claim or declare it achieved without measurements.

Begin with S11 collection, recursive, pipeline and JSON Schema export costs.
The retained S11 baseline has 31 native/Node workloads, with native slower in
28. That baseline uses three fresh processes, eight warmups and 31 samples per
engine, with its original correctness cells. SQLite `tsrust-s11-measurement-proof.json` and
`tsrust-s11-measurement-review.json` define the measured scope. Reusable
allocation/disposal evidence under `tsrust-schema-reusable-*` does not profile all
later paths. Some Node references return existing objects or use simpler errors;
review semantic and allocation comparability before drawing conclusions.

Profile retained costs, prepare missing matched handwritten Rust references and
allocation observers, then optimize underlying causes without hidden clones,
shared ownership, erased lifetimes, removed safety checks or weakened oracles.
The existing runtime driver supports native/Node comparisons; it does not
implicitly provide a handwritten-Rust reference engine. Complete reference and
measurement support rather than calling generated Rust its own independent baseline.
Preserve original protocols and sample counts. Measure changed comparable
workloads and review regressions at the optimization milestone. If a material
contract mismatch prevents parity, provide concrete evidence and seek only the
remaining decision; do not silently weaken the requested target or loop forever
on unrelated cleanup.

Fix the known program-storage/test-callback lifetime issue before calling the
foundation complete: an owned registered test callback that invokes a function
reading a module global can retain a transitive program borrow and reject.
Review owner-scoped test execution/registration disposed before globals while
preserving real provenance. `clone(message)` alone does not resolve that boundary.

## Phase C: return to new features and continue through S21

Once A and B are accepted, proceed directly to source feature implementation.
Do not insert another speculative cleanup/research phase or wait for a new
instruction merely because foundations reached a checkpoint. Use the roadmap's
dependency order; S12 is the next planned feature. S17 has independent synchronous
prerequisites but is not a reason to abandon the chosen sequence without direction.

| Gate | Required continuation |
| --- | --- |
| S12 | Public native-thread spawn/join/detach, owned capture/result transfer, failure/panic handling and bounded channels. Read [thread plan](../plans/native-threads-plan.md) and [interop contracts](../interop-and-targets.md) first. |
| S13 | Parsed/checked async functions, await, unique task results, suspension-safe ownership and generated futures; [async plan](../plans/async-compiler-plan.md). |
| S14 | Tokio bootstrap, explicit start, cancellation/timeouts, combinators, timers/channels, child settlement and shutdown. |
| S15–S16 | Validated HTTP client/server, streaming/backpressure/limits/shutdown, then TLS trust/hostname/certificate/error coverage. |
| S17–S18 | Synchronous SQLite statements/rows/transactions/rollback, then separately accepted SQLx drivers/pools/cancellation/migrations. |
| S19 | Public cache integration using existing LRU/deadline/timed backend and [approved cache decisions](../plans/cache-plan.md); backend tests are not source acceptance. |
| S20 | Bounded queue ownership, backpressure, close/drain, cancellation and worker shutdown. Durable/distributed adapters remain separate scope. |
| S21 | Integrated validated service with database/cache/queue, overload and leak checks, graceful shutdown, complete editor support and reproducible distribution. |

Settled contracts must carry forward:

- Native thread handles require explicit join or detach; abandoned handles error.
  No automatic detach-on-drop, hidden payload clone or implicit shared wrapper.
- Local Rust modules and supported package wrappers belong alongside S12 before
  worker APIs harden. Automatic lazy default pooling for explicit threaded interop
  is approved; evaluate Rayon and reuse compatible pools. Direct calls stay direct.
  Exact syntax, queue/admission configuration and shutdown details remain material
  design work. The earlier blanket pool deferral does not override this approval;
  arbitrary crates/macros/traits/lifetime-heavy APIs are not thereby authorized.
- Tasks are lazy, uniquely owned and single use. Calling transfers arguments/loans
  but does not start the body. Await consumes; `.start()`, `.cancel()` and
  `.timeout()` direction is approved. Unused never-started tasks warn; abandoned
  started jobs error even through aggregates, returns and combinators. Cancellation
  requests alone do not discharge settlement obligations.
- Cancellation does not guarantee source finally execution. Owned state and child
  jobs still require exact cleanup. Combinators accept both lazy and started inputs;
  lazy children begin together when awaited, started children continue. Detailed
  race/timeout/empty/heterogeneous-result contracts still need specification.
  `std:futures` is the recommendation, not separately settled module naming.
- Cache reads borrow; explicit clone creates an independent value when supported.
  Expiration runs during operations. Further operations reject while an outstanding
  borrowed result remains used. Preserve approved ownership/cleanup; do not reopen
  those decisions because older proposal sections still say pending.

Implement each coherent source family across compiler, native execution, CLI,
LSP, diagnostics, complete `builtins.json` help/examples and matched benchmark
support. Keep implementation and measured acceptance separate. Ask only for
unresolved material semantics after completing independent authorized work.
Backend feasibility controls do not count as public API implementation.

## Commands, evidence and consumer state

Use the shared target; do not create a target per gate or repopulate pruned caches
just for handoff. Unset `RUST_MIN_STACK`; expect some test relinking/cold builds.

```sh
env -u RUST_MIN_STACK CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target \
  CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p tsrust-compiler \
  --lib --no-run
# Obtain the current probe path from Cargo output; hashes/paths can change.
env -u RUST_MIN_STACK python3 benchmarks/compiler/nesting-discovery/run.py \
  --probe "$PROBE" --output /private/tmp/NEW-discovery.json \
  --families object-spread --unbounded-parser --breadth-first \
  --start 8 --factor 4 --seconds-per-case 30 --seconds-total 120
# Operational budgets above are examples, never language limits.
python3 benchmarks/compiler/function-dependencies/run.py \
  --compiler /private/tmp/tsrust-gate100-editor-target/debug/tsrust \
  --object-spread --verify-execution --only 32 --output /private/tmp/NEW-matched.json
# --measure uses each manifest's original 2 warmups/11 samples after validation.
node benchmarks/runtime/run.mjs --execution-only --engines=native,node-tsc \
  --only=WORKLOAD_IDS
TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust \
  node tools/format-project.mjs
```

Choose fresh evidence names. `benchmarks/records.sqlite` is canonical, with
`records(path, kind, sha256)` and compressed blobs. Hyphenated filename families
are not directory prefixes for the storage CLI's `--prefix`; enumerate them with
SQLite, then read exact paths, for example:

```sh
python3 benchmarks/storage/store.py read benchmarks/records.sqlite \
  tsrust-nesting-constructor-audit-finding.json
```

Useful record families: `tsrust-nesting-constructor-audit-*`,
`tsrust-nesting-object-spread-work-*`, `tsrust-nesting-array-spread-work-*`,
`tsrust-nesting-spread-work-*`, `tsrust-nesting-generic-inference-work-*`,
`tsrust-nesting-generic-entry-work-*` and `tsrust-nesting-deferred-call-work-*`.
Earlier formal acyclic measurements are `tsrust-nesting-dependency-scaling.json`
and its proof; they do not measure the current whole architecture. Claude parser
originals are under `benchmarks/runtime/results/claude-parser-evidence/` in SQLite;
LSP originals use `claude-lsp-evidence/`, with integration under
`tsrust-lsp-agreement-integration-*`. Do not restore their removed worktrees or
reimport evidence already verified merely to revisit the reports.

At `39aaec7d`, 22 object-spread records are imported and byte-verified with
`quick_check` passing. Consumers are atomically refreshed and hash-matched:
`~/.cargo/bin/tsrust`, `~/.cargo/bin/tsrust-lsp` and
`target/editor/debug/tsrust-lsp`. The actual VS Code server restart was verified;
the owner's open `tsrust_test/src/main.tsr` has zero diagnostics. One stale app
connection initially failed to restart it; both observations are preserved.
Twelve current-section and 92 older integration executables were pruned. Shared
target size was about 2.3 GB afterward. Preserve current CLI/server/probe and
needed libraries; do not delete owner target contents based on an old size report.

When rebuilding consumers, explicitly request both packages and binary names:

```sh
env -u RUST_MIN_STACK CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target \
  CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build \
  -p tsrust-compiler -p tsrust-lsp --bin tsrust --bin tsrust-lsp
```

Before checkpoints, format the entire project and verify tokens, idempotence and
formatted-source behavior. Import closed records, read them back byte-exactly,
verify SQLite integrity, then remove only the verified loose copies. Preserve
original failures and measurements. No snapshots or archive directories. Refresh
consumers after accepted public changes with atomic sibling replacement, restart
actual configured servers, verify process change and open-document diagnostics.
Do not describe installation alone as an active-server refresh.

## Copyable continuation instruction

```text
Continue Tsrust in /Users/seancotter/Desktop/dev/learning/tsrust using $caveman.
Use one agent. Inspect Git status first. Read AGENTS.md and
docs/handoffs/compiler-to-roadmap-continuation.md, then its required references.
Reconcile Git and SQLite evidence; source basis is 39aaec7d, not the old S11
checkpoint. Claude parser and LSP branches are already integrated.

Carry the existing objective through completion, preserving its current status
until the owner resumes it. On resumption: finish compiler nesting architecture
and full-pipeline scaling acceptance, then the owner-requested schema safeParse success/data/error API alignment
and required existing-code optimization against comparable handwritten Rust timings/allocations and Node-after-tsc,
then implement S12–S21 through integrated release acceptance. Do not stop at an
individual helper checkpoint or the superseded S11-only boundary. Do not create
a duplicate goal or mark the overall goal complete while required work remains.

Start with Set/Map constructor checking and emission on the shared work stacks.
Audit all remaining connected paths, representations and resource guards; preserve
ownership, effects, generic scopes, evaluation order and diagnostic locations.
Never raise stacks/constants or select new arbitrary depth limits. Discover actual
resource behavior on normal stacks and derive any policy from evidence/device
capacity. Keep discovery timeouts separate from support limits and formal timing.

Use the handoff's complete backlog and exit criteria, including diagnostic replay,
public admission retirement, proof/context/output scaling and module-test lifetime
repair. After foundation and optimization acceptance, proceed directly to S12
source features and approved interop design, then roadmap dependencies. Preserve
settled thread/task/cache contracts; ask only about unresolved material semantics.

Implement complete compiler/native/editor/docs/examples/matched benchmark support
by coherent sections; batch affected checks. Run full regressions and original
measurement protocols at meaningful family milestones, preserving completed work
and resuming unrun groups. Reuse the documented Cargo target with incremental off.
Format the whole project before checkpoints. Keep SQLite canonical, verify exact
imports before deleting copies, preserve owner files, refresh actual consumers,
and prune obsolete generated outputs. Generated Rust does not need formatting.
```

## Status after the 2026-10-14 Claude pass

Done and committed: Set/Map constructor scheduling, array-spread operand moves,
builtin method-chain receivers on the shared stacks, diagnostic replay for
failed declared results, readable native and generic names in diagnostics,
and the schema safeParse `{ success, data } | { success, error }` API across
compiler, editor, catalog, workloads and fingerprints. Release CLI and server
are installed.

Open, in priority order:

1. Program-scoped test registry (owner-approved option 1):
   [handoff](program-scoped-test-registry.md).
2. Nested `concat` arguments still recurse through
   `concat_string_arguments` -> `expression`. Debug builds overflow between
   40 and 60 levels, so `tests/concat_depth.rs`,
   `string_concat_integer_fusion.rs` and `string_concat_padding.rs` (depth
   100) fail. The session-start commit `9ab53fb0` fails identically; this is
   not a new regression. Scheduling the arguments must keep the receiver's
   read loan active while each argument is checked; an early precheck would
   drop that loan.
3. Emission copies child fragments, which is quadratic in single-expression
   nesting depth (4.8 s at 32,768 nested Set constructors). Programs with
   many functions scale linearly.
4. Plain nested array-literal spreads re-walk type hints per layer
   (quadratic); low priority per the owner.

Console, path and value-type differential suites require the pinned Node
(`v24.11.1`) first on `PATH`; with it they pass.
