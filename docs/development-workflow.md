# Development and acceptance workflow

Use the [compiler source layout](guides/compiler-source-layout.md) when adding or
relocating compiler modules.

## Implementation-first development

Implementation is the priority. Add focused regression witnesses while implementing
new behavior, including ownership, diagnostics, output and disposal where relevant.
The owner's 2026-10-12 direction is to defer running checks and tests until the
whole coherent section is implemented across compiler, native behavior, editor,
examples and benchmark support. Run an earlier check only when a specific
uncertainty blocks implementation. Do not repeatedly validate partially completed
pieces or run full compiler/LSP suites during routine edits.

A feature's focused acceptance includes its compiler and LSP behavior, consistent
signatures, completion, hover, parameter help, diagnostics, verified executable
examples and documentation. Its benchmark support must also be ready: matched
TypeScript sources, independent result oracles, result observers, native ABI and
driver support, source fingerprints, guards and registrations. Run representative untimed
checks with the completed section. A future benchmark
command must execute current supported workloads without first implementing or
repairing support accumulated over dozens of gates.

## Batched development checks

Finish a coherent section before running its affected suites. For the current
schema work, ordered unions and discriminated unions form one section, including
checker, IntelliSense, examples and matched workload support. Write regression
witnesses during implementation without running each one immediately. Preserve
completed evidence and rerun only affected checks after repairs. An individual gate
or source checkpoint does not trigger testing. Full regressions and timing remain
reserved for meaningful feature-family milestones.

When backend witnesses can share a native compilation, compile them together in isolated modules with their original assertions and run them in one executable. Keep case identity in generated source so failures remain attributable. Preserve evaluation-order, ownership, failure and output assertions; batching must not reduce coverage.

## Formatting acceptance

Run `node tools/format-project.mjs` before every source checkpoint, not only after
editing formatter code. This formats every `.tsr` file in the project, excluding
`.git`, `node_modules`, `target` and symlinks. Set `TSRUST` to the current verified
compiler when using the shared warm target. The ordinary `tsrust format` command
provides the same recursive formatting behavior for user projects.

The checkpoint command holds pre-format benchmark sources in memory and refreshes
manifest fingerprints only when the pinned source matches those bytes or the
committed baseline and all code tokens remain unchanged. It preserves literal
contents, template chunks and operators. Source fingerprints identify exact inputs
in immutable benchmark evidence; formatting never changes operation counts, sample
counts, oracles or measurement status. Register new workload hashes after formatting.
Semantic source changes still require their ordinary implementation acceptance.

Catalog excerpt and benchmark scenario checks compare code tokens or canonical
formatting so layout changes do not create false failures. Keep their native output
assertions and mutation rejection controls. Store Node-only references as `.ts`,
not `.tsr`; intentionally malformed source can remain in inline rejection witnesses.
Do not hide formatting failures or silently bypass the normal compiler checks.

Every added or modified Tsrust acceptance fixture must also exercise formatting.
Format successful source in memory, require a stable second pass, then check or
execute the formatted source with the original assertions and expected results.
Use tests/support::format_round_trip for shared compiler fixtures. The shared
native execute helper and generic adapter native helper apply this automatically.
Project fixtures with imports must use the project CLI path after formatting.

The formatter corpus check covers every lexable .tsr fixture for token preservation
and idempotence, and rechecks standalone sources that originally pass analysis.
Deliberately malformed lexical fixtures remain rejection witnesses. Keep native
and module checks for semantic behavior; token preservation alone does not prove
execution parity. Do not rewrite unrelated owner fixtures as a verification side effect.

## Native assertion-based fixtures

Use the built-in std:testing library for new executable behavior checks wherever possible. Assert observed values directly instead of printing them solely for an external stdout comparison. Run through the compiler's project/module path so assertion imports and generated native execution are exercised. Keep stdout checks when output formatting itself is the behavior under test, and retain independent Node reference evidence for parity.

When a required assertion or matcher is missing, implement it with focused failing tests, ownership and failure checks, IntelliSense documentation and the normal acceptance workflow. Do not replace the missing operation with logging or weaken the test to avoid adding it. Prefer borrowing assertions for existing owned or borrowed values; do not introduce implicit clones to make a matcher fit.

## Meaningful feature-family milestones

Run complete regressions and accumulated performance comparisons after meaningful
feature progress, not on a fixed gate-count schedule. Define a concrete family scope
and finish its compiler, LSP and benchmark support before the heavy run. For example,
complete the scoped Node built-in array methods and all their editor behavior before
running the complete compiler/LSP regression, array comparisons and optimization
review. An individual method, a small repair, or five more numbered gates does not
by itself justify this work. An explicit user request can require a broader check.

The former five-gate schedule is superseded. Accepted checkpoints81,86,91 and96
remain historical evidence. The old checkpoint101 queue records pending work; it
is not an instruction to launch or complete a heavy pass now. Array-literal spread
has focused acceptance while the broader array milestone remains open.

At a completed family milestone, run the complete compiler and LSP regressions and
matched benchmarks, review correctness and comparable performance signals, then
optimize. Preserve passed suites and original command evidence. Repair/rerun only
affected checks and resume unrun suites rather than restarting the full pass. Keep
runs bounded. If the user changes priorities, stop the heavy continuation and retain
completed evidence. Track focused implementation acceptance and deferred milestone
measurements separately; never describe unmeasured workloads as measured.

SQLite remains the canonical analytics store. Keep exact original records, source
and toolchain identities, protocols and sample counts. Markdown reports are optional
exports. Do not retime unaffected workloads for presentation or replay history
solely because a new feature landed.

## Analytics purpose and storage

At completed feature-family milestones, use correctness checks and matched performance measurements to review the completed foundation before optimization or further dependencies. Routine checkpoint timing should compare generated native execution against Node after TypeScript compilation. The broader runtime matrix remains available for optional comparison; preserve its existing measurements as context. A universal runtime win is not an acceptance requirement.

Keep SQLite as the persistent analytics source of truth: exact original records, source and toolchain identities, workload semantics, individual samples, normalized allocation counters and derived queryable summaries. Loose files may serve active checks, but closed records should be imported and verified before their disposable copies are removed. Preserve byte-exact restoration for historical evidence consumers. Do not rerun measurements merely to change their presentation.

A checkpoint should produce a concise review of correctness failures, native/compiled-Node throughput ratios and changes against comparable earlier measurements. Flag slower workloads and material regressions for investigation; distinguish measurement noise and changed workload costs from compiler regressions before refactoring. Compare matching semantic epochs, operation counts and measurement scopes. Do not merge samples from separate runs or describe source acceptance as timing evidence.

Use `node benchmarks/runtime/run.mjs --execution-only --engines=native,node-tsc --only=<workload IDs>` for routine checkpoint timing. The selected engines are recorded explicitly; omitting `--engines` retains the broader eight-engine matrix. All existing correctness cases and per-engine sample counts remain unchanged.

The future official documentation site can read derived data from this database for charts and tables. Site implementation and continuous Markdown table generation are not part of compiler gate acceptance.

## Built-in IntelliSense documentation

Every new method or property must have complete documentation in `crates/tsrust-lsp/src/builtins.json` before its gate closes. Apply this requirement to gates already in progress. The catalog supplies hover, completion documentation and signature help.

Each hover description must include:

1. A plain-language explanation of observable behavior.
2. Argument meanings, accepted types and ranges, optional defaults, and relevant errors. State when there are no arguments.
3. The return value and whether the receiver changes.
4. At least one concise Tsrust example and its expected result. Add an edge-case example when it clarifies behavior.
5. Relevant native differences, including UTF-8 byte indexing, Unicode semantics, integer limits or rounding.
6. A short ownership note after the behavioral explanation.

Do not substitute generic implementation descriptions for behavior. Follow current compiler semantics rather than copying JavaScript documentation. Keep catalog signatures, parameter labels and descriptions consistent with compiler arity/type rules and actual editor responses.

Keep executable example source and expected stdout in each entry's `examples` array where applicable. Check examples with the current compiler and execute generated native code when output matters. Reuse focused acceptance fixtures for boundary/error examples when practical. Include the changed catalog entry identities, checked examples, native outputs and affected editor protocol checks in gate evidence. Prose review is still required: a schema or a passing example alone cannot prove all semantic claims.

Preserve existing editor fixes and unrelated changes. If those files change during a regression, retain the original run and supplement it with affected checks on the current editor source.

IntelliSense documentation must focus on purpose, observable behavior, arguments, defaults, errors and useful examples. Keep runtime reference versions, test baselines, benchmark/checkpoint details and implementation provenance in developer evidence, not hover or parameter help. Describe relevant limitations directly without a runtime comparison narrative.

## Editor agreement with the compiler

The editor is how a person or an agent learns what the language accepts. An
empty completion list or a missing hover reads as "this is not valid". Treat
every place where the editor is silent, or offers unrelated names, while the
compiler has an answer as a correctness defect.

A language feature is not accepted until these `tsrust-lsp` suites pass. They
are affected suites for any change to the checker's member rules, the built-in
catalog, the standard library, completion, hover or request routing.

- `completion_matrix`: one row per receiver kind and syntactic position. Each
  row runs against the real server typed without a pause, typed at 80 ms per
  key, settled, and settled with a syntax error elsewhere. The answers must be
  equal. A new receiver kind, position or `std:` module needs a row; the suite
  fails without one.
- `checker_agreement`: for each receiver type, completion offers exactly the
  fields the compiler accepts, shows the compiler's type for each, and offers
  no method the compiler rejects. Add a receiver type with each new built-in
  or record-like type. An exception needs a written reason and fails when it
  no longer applies.
- `editor_consistency`: completion is not empty where hover reports a type;
  every documented member is reachable; typing a project document leaves the
  server counters `staleAnswers`, `emptyKnownReceiver` and
  `nonMemberAfterSeparator` at zero.

Rules for editor code:

- Derive members and types from the compiler: its checked facts, its native
  type declarations, or a check of a repaired document. Do not add a table of
  what a type offers. Where the compiler has no query yet, add the agreement
  row first and request the query.
- A request is answered from an analysis of the current text. A document with
  imports waits for its project analysis; it is not answered from the
  single-document index.
- After a member separator, offer members or nothing. Never fall back to
  names of the enclosing scope.
- When a receiver has a known type and no member, hover says why and
  completion offers the action that narrows it.
- Before you infer a language rule from editor output, compile a witness.

## Compiler recursion and stack usage

Keep shared recursive dispatch small. Built-in analysis, branch joins and assignment selection should own their temporary state in dedicated helpers so ordinary calls and blocks do not retain unrelated state on the stack. Preserve the existing source-depth diagnostics and normal test-worker stack; increasing the worker stack is not an acceptance fix. When a depth witness aborts, identify its failing phase and inspect actual compiled frames before changing the implementation. Resume the failed checks once the narrow witness passes.

## Refreshing executable consumers

Refresh installed CLI and configured editor servers after accepted public API checkpoints when the owner is using those APIs; do not defer all consumer refreshes until a full milestone. Inspect workspace overrides as well as default installed paths, restart the active server, and verify open-document diagnostics.

Finish consumer refreshes before starting checks that execute those binaries. When selecting Cargo binaries explicitly, name both `--bin tsrust` and `--bin tsrust-lsp` alongside their packages; listing two packages with only `--bin tsrust` leaves the LSP executable stale. Verify both installed consumers independently. When replacing an existing debug/editor executable, copy the new binary to a temporary sibling file and atomically rename it into place; never overwrite the live executable inode. This keeps already running processes on their original image and new invocations on the refreshed image. After refreshing an installed language-server binary, restart the active editor language server with `Tsrust: Restart Language Server` (or click the Tsrust status item). Atomic replacement leaves existing processes on the old executable. Verify diagnostics in the already open editor documents, in addition to checks launched with the refreshed binary. Git preserves source checkpoints, not ignored executables. Rebuild consumers from the relevant commit when restoration is needed; do not create binary archives. If a refresh race interrupts checks, keep the completed evidence and rerun the affected groups after refresh finishes.


## Git checkpoints

Use local Git commits for source, tests, documentation and benchmark-support checkpoints.
Inspect status before starting a gate, preserve unrelated changes, and commit accepted
work with its evidence references. A clean checkpoint before a risky change replaces file snapshots. Never reset or
discard another agent's changes. Do not create archive directories or snapshots;
the owner deleted the historical archives.

Generated executables, run artifacts and the canonical SQLite analytics store are
ignored by Git. Rebuild binaries from the relevant source commit when needed. Preserve analytics evidence in SQLite, not duplicate loose trees; a source commit
is not its backup. Import and verify closed records before removing their loose copies. Do not create archive
directories or snapshots.


## Generated-file retention

SQLite is the durable home for closed benchmark records and diagnostic evidence.
Import original bytes, verify hashes and database integrity, then remove loose copies.
When a path already names different immutable bytes, preserve the new version under
an explicit content-addressed SQLite record path; never overwrite historical data.
Do not create filesystem archives, source snapshots or duplicate evidence trees.
Keep concise logs for the current gate only, and preserve the five current executable
consumers during cache cleanup.

Reuse the documented single Cargo target with CARGO_INCREMENTAL=0. At handoffs,
inspect workspace and scratch target growth and remove obsolete test/native executables,
intermediate build trees and expired cache variants. Do not keep a target tree per gate.
A cleanup may make the next build cold; report that honestly and do not start a rebuild
merely to repopulate caches after cleanup. Historical measurements remain in SQLite.

## Intentional rejection fixtures in the editor

Use a reasoned `// @tsrust ignore - ...` directive immediately before an
intentionally invalid statement in a checked-in rejection fixture. This keeps
workspace checking useful without suppressing compilation failures. Rejection
acceptance must still check strict analysis and emission on the original source;
do not remove the invalid code or rely on the suppressible CLI check result.

The closure/ownership fixture cleanup at b59d1055 removes 11 expected diagnostics
from 10 files. Recursive workspace check reports no diagnostics; emission rejects
all 10 fixtures and the complete closure source corpus retains strict rejection
assertions. SQLite evidence uses `tsrust-editor-*-b59d1055`, including the initial
check output and the corpus adjustment's original failed run. Editor warnings are
separate from these intentional errors and are not hidden by this cleanup.
