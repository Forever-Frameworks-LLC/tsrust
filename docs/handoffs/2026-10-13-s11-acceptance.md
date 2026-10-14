# S11 acceptance and continuation handoff

Current continuation: [compiler foundations through S12–S21](compiler-to-roadmap-continuation.md).
Use that entry for current priorities, remaining work, acceptance and resumption.
Historical checkpoint/stop statements below retain their original context.

Status: S11 implementation, family measurement, source cleanup and consumer refresh
are accepted on macOS. Stop boundary reached; do not start S12 in this run.
The compiler nesting architecture below is the next agent’s first priority.

## First priority: compiler nesting architecture

The owner explicitly requires this work before S12 or further roadmap features.
It was originally deferred to S21; that deferral is superseded. Normal program
structure should not hit the current combined 128-level function/block analysis
limit. Frame extraction repairs prevent known crashes but do not complete this work.

Replace call-stack-dependent function dependency analysis with an explicit worklist
or equivalent bounded traversal. Preserve recursive ownership inference, generic
specialization, point-sensitive effects, diagnostic coordinates and source ordering.
Audit parser, type traversal, emission and tree disposal for equivalent stack limits.
Retain clear resource diagnostics for hostile input; merely raising stack sizes or
removing guards is not acceptance. Runtime recursion and JSON depth budgets are
separate contracts. Require deep finite nesting, long ordinary function chains,
recursive type/value cases, normal worker-stack checks and compilation scaling data.

Starting evidence in SQLite: `tsrust-s11-compiler-regression.log`,
`tsrust-s11-concat-frames.log`, `tsrust-s11-control-flow-frames.log`,
`tsrust-s11-field-moves-frames.log` and `tsrust-s11-field-moves-frames3.log`.
Repairs at `60062e81` split schema dispatch, concat preparation, branch availability
joins, ordinary expression staging and nullable callback finalization into smaller
frames. Keep these witnesses and do not weaken their depths.

## Roadmap and acceptance scope

S01–S10 were already accepted. S11 implementation covers inferred reusable schemas,
scalar bounds, nested arrays, typed objects, optional/nullable/default fields,
literals/enums, ordered/discriminated unions, refinement/transformation/coercion,
record/Map/Set collections, inferred/explicit recursion, typed issue paths,
constructor/constraint messages, throwing parse, fused JSON validation,
superRefine and scoped Draft 2020-12 JSON Schema export. Extended standards
import/dialect conformance belongs to H18; no blanket Zod compatibility claim.

Checkpoints: `8176b6cb` export implementation, `60062e81` family regression repairs,
`20414343` measured baseline documentation; `fe1d75ef` verified cleanup source checkpoint.
Before and after cleanup: compiler 3,727 passed, three expected ignored; LSP 645 passed. Compiler
coverage is composed from interrupted runs plus affected repairs, with every one
of the 321 integration suites accounted for. The exact index is
`tsrust-s11-regression-index.json` in SQLite. Original failures remain preserved.

The owner requested stopping after S11 and cleanup. The full S21 goal must be
paused, not marked complete. Do not begin S12 in this agent's run.

## Measured baseline and performance limits

All 31 native-schema workloads completed native versus Node-after-tsc execution
measurements at `60062e81`: three fresh processes, eight warmups, 31 samples per
engine, 5,766 durations total. Five correctness cells per workload/engine passed.
All groups capture identical compiler source inputs. Raw records and normalized
samples are in `benchmarks/records.sqlite`; proof/review are
`tsrust-s11-measurement-proof.json` and `tsrust-s11-measurement-review.json`.

Native is slower in 28 of 31 cases. Median-of-process-medians Node/native ratios
range from 0.196 for collections (native about 5.1x slower) to 10.013 for throwing
parse (native faster). This is the first complete S11 baseline, not evidence of a
regression against prior S11 timings. References are handwritten corpus validators,
not Zod benchmarks. Some return existing JavaScript objects and use simpler errors;
native maintains typed ownership, bounded issue paths and result transport.
Export timing includes tree construction, serialization and full observation.

Existing reusable array/object/union allocation and disposal evidence remains scoped:
`tsrust-schema-reusable-run3.log`, `tsrust-schema-reusable-results4.json` and the
schema plan. It does not profile every new collection/callback path. After the
nesting priority, profile collection, recursive, pipeline and export costs before
optimizing. Preserve original measurements. Never remove safety checks or introduce
hidden copies/shared ownership to improve a chart. Refactor verification is separate
from the pre-cleanup timing identity; do not relabel old samples as newly measured.

## Approved schema contracts

- Import `{ s }` from `std:schema`; output types are inferred through every chain.
- Strict unknown-field rejection by default. `.strip()` discards extras;
  `.retain()` returns `{ value: T, extra: Map<string, JsonValue> }`.
- Owned validation input is consumed on success and failure. Explicit clone retains
  the original. Reusable immutable schemas are borrowed by validation; builders
  consume the old schema and return a new one.
- String lengths count Unicode code points. Exact integers are preserved; checked
  int-to-number conversion rejects precision loss.
- Invalid bounds throw `SchemaConstructionError`, with `limit: int | number`.
- `.default(() => value)` runs only for missing input, validates its result, permits
  readonly captures/effects and propagates factory exceptions. Null is distinct.
- `.refine()` reads input. `.transform()` owns input. `.coerce(callback)` owns raw
  input, converts first and then validates. Callback exceptions propagate normally.
- Ordered union branches commit before a consuming transform; later failure cannot
  retry another branch through a hidden input copy.
- `.safeParseJson(text)` distinguishes `kind: 'json'` from `kind: 'validation'`.
- `s.record(valueSchema)` decodes JSON string keys to Map. `s.map`/`s.set` validate
  native collections. Key/element transform collisions reject instead of losing data.
- `s.recursive(self => ...)` infers output; optional generic checks an existing named
  type. `self` stays scoped, definitions must be productive, plans uniquely owned.
- `.superRefine((value, ctx) => ctx.addIssue(...))` has readonly input, typed relative
  paths, temporary mutable context, normal exceptions and bounded issue retention.
  Callback execution continues after retention fills; invalidity is never lost.
- `.toJsonSchema()` borrows the schema and returns owned JsonValue. Unsupported
  callbacks/default factories/native Map/Set/output policies throw SchemaExportError
  with a JSON Pointer and reason; no callback executes or rule silently disappears.
  Numeric type/bounds export is approved despite native exact-conversion checks that
  JSON Schema cannot fully express. Document that boundary.

## Known issues and remaining research

Program-owned module storage cannot be consumed. `clone(message)` can return an
owned string, but a registered owned test callback calling a function that reads
module storage can still retain a transitive program borrow and reject. Diagnostic
clarity improved; the lifetime limitation remains. Suggested fix: owner-scoped test
registration/execution disposed before globals, with real lifetime provenance.
Do not erase lifetimes or add implicit clones. Workaround: local/literal storage.

The three ignored tests are an opt-in whole-source scheduling diagnostic, a
subprocess cache-publication worker invoked by its contention test, and an opt-in
frontend fanout timing probe. They are not known failing schema cases. The old
five-gate timing wording on the fanout probe is superseded by milestone scheduling.
Test-only builds report some shared
runtime entry points and partial bundle helpers as unused. Production warnings were
removed through runtime/compiler separation, without blanket suppression. Preserve
needed generated-runtime functions when addressing remaining test warnings.

Thread-pool/global mutable-state API discussion is intentionally deferred until
after async. User-authored Rust interop belongs alongside native-thread design.
The owner favors pooled execution as a possibility, but no final pool API is approved.

## Approved later contracts

Lazy owned Task values start on direct await; `.start()` starts explicitly;
`.cancel()` and `.timeout()` are approved. Results are single use. Unused lazy tasks
warn; started jobs that are never awaited/settled must error. Cancellation does not
guarantee finally runs. `std:futures` combinators accept lazy tasks and started jobs:
lazy inputs begin together when awaited; started jobs continue. Ownership and child
settlement must survive timeout, failure and parent cancellation.

Native thread handles require explicit join or detach. Module globals retain real
program ownership and explicit clone semantics. JSON globals use throwing parse
and checked tryParse, with familiar spelling documented where semantics differ.
Typed JSON imports are readonly canonical data; explicit clone enables mutation.
The globals/import naming and dependency audit must remain thorough.

## Workflow and protected owner files

Use one agent. Read AGENTS.md, development workflow, roadmap, coverage and schema
plan. Batch implementation and checks by coherent sections. Full regressions and
timing belong at meaningful milestones. Reuse
`/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`, jobs1 and serial
native tests on this host. Format the whole project before every checkpoint with
`node tools/format-project.mjs` and the verified compiler via `TSRUST`.

SQLite is canonical. Import closed records, verify byte-exact restoration, then
remove loose copies. No snapshots or archive directories. Preserve original failure
records and resume unrun suites. Prune obsolete generated outputs before handoff.

Preserve these unrelated owner files: `UI_VISION.md`,
`docs/plans/axum-hono-http-ui-contract.md`, `editable-list-app-example.md`,
`parallel-ui-foundations-plan.md`, `ui-feasibility-review.md`,
`ui-route-contracts-first-task.md`. Do not stage them with compiler work.

Cleanup evidence: `tsrust-s11-cleanup-regression-index.json` accounts for all 321
compiler integration suites and library targets, with zero unresolved failures.
The runtime split exposed two missing import boundaries; both original failures and
affected successful reruns remain in SQLite. Compiler doctests pass (zero cases).
`tsrust-s11-cleanup-lsp.log` records 645 passes. Whole-project formatting changes
zero files/fingerprints. Export, recursive, collection and fused-JSON adapters pass
40 native/compiled-Node correctness cells with zero timing samples.

Consumers built explicitly with both binary names and atomically refreshed from
`fe1d75ef`: `~/.cargo/bin/tsrust`, `~/.cargo/bin/tsrust-lsp`, and configured
`target/editor/debug/tsrust-lsp`. Copies are hash-verified in
`tsrust-s11-consumer-refresh.json`. Both editor windows restarted their servers;
the workspace's stale Tsrust errors cleared. Restarting Rust analysis also cleared
its stale syntax errors. Both windows report zero problems. The installed CLI's
recursive `tsrust check .` passes (`tsrust-s11-installed-project-check.log`).

Obsolete Cargo/test/native output trees are pruned while preserving the five
current consumer executables. The next Cargo/native build may be cold. See
`tsrust-s11-output-pruning.json`; no rebuild was run merely to refill caches.

## Copyable continuation prompt

```text
Use $caveman for concise communication. Work in
/Users/seancotter/Desktop/dev/learning/tsrust, with one agent in the tree.

Read AGENTS.md, docs/handoffs/2026-10-13-s11-acceptance.md,
docs/development-workflow.md, docs/guides/compiler-source-layout.md,
docs/standard-library-roadmap.md, docs/standard-library-coverage.md and
relevant plans. Inspect Git status first and preserve the listed owner files.
Reconcile source and SQLite evidence rather than trusting stale summaries.

Create an active goal to remove compiler call-stack-dependent nesting limits,
then continue the existing roadmap through S21 integrated release acceptance.

FIRST PRIORITY, BEFORE S12: redesign compiler function/dependency traversal and
combined function/block nesting. Ordinary programs must not hit the current
128-level combined-analysis guard. Use explicit worklists/stacks; preserve
recursive ownership inference, generic scopes, effects, diagnostic locations and
evaluation order. Audit other compiler traversals and tree disposal. Keep hostile
input resource diagnostics, and distinguish runtime recursion and JSON budgets.
Do not merely increase worker stacks, raise constants or weaken depth witnesses.
Verify deep finite structures and long call chains on normal stacks; measure
frontend scaling. The handoff identifies preserved crash evidence and repairs.

After that priority is accepted, continue S12–S21 in dependency order using the
approved contracts. Do not reopen settled semantics or count backend prototypes
as public API acceptance. Ask only about unresolved material semantics. Keep
thread-pool/global mutable-state API discussion until after async as requested.

Retain S11 original performance samples. Profile the documented collection,
recursive, pipeline and export costs before optimizing. Native performance is
not universally faster; report limitations honestly and never trade safety for
benchmark results.

Implement complete checker/LSP types, diagnostics, IntelliSense examples and
matched benchmark support with each coherent section. Batch checks after whole
sections; reserve full regressions/timing for meaningful milestones. Reuse the
documented Cargo target with CARGO_INCREMENTAL=0. Format the whole project before
checkpoints. SQLite remains canonical: verify imported closed records before
removing loose copies. No archives or snapshots. Refresh installed consumers
after verified public checkpoints and prune obsolete outputs before handoff.

Keep implementation acceptance separate from measured acceptance. Do not mark
the full goal complete while required roadmap work remains.
```
