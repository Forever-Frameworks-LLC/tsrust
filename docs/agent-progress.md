# Tsrust handoff

This is the single handoff note (owner direction, 2026-10-30): keep it current
and do not create separate handoff files. Older handoffs and the long
`agent-progress.md` history were deleted; they remain in Git history.

## Start here

Repository: `/Users/seancotter/Desktop/dev/learning/tsrust`.
Branch: `codex/optimize-native-hot-paths` (pushed; full CI dispatched per milestone).
Read `AGENTS.md`, `docs/development-workflow.md` and `docs/agent-goal.md`, then
this note. Inspect current source and `git status` before acting.

## Current state — 2026-10-30 (optimization loop; read first)

Active owner direction: optimize toward "making Rust as easy to write as
TypeScript": JS/TS-familiar API, Rust semantics and accuracy underneath, generated
code at hand-written-Rust parity. The docs restructuring in the next section is
still open but is not the current loop's focus.

**Git:** branch `codex/optimize-native-hot-paths`; local checkpoints remain
unpushed. Latest accepted optimization: `28a3409c` (typed JSON pipeline
restoration), following `1d3eb962` (single replacement sizing acceptance). Earlier
accepted work: `260d1781`/`cea19c4c` Map planning, `5083586a` schema restoration,
`610c2bc0` scalar-union arrays, `a74da73f` Rust parser defaults, `a2f94dee`
plain-schema decoding, and `1e4d2547` complete milestone/test-support repair.
The rejected small-string experiment remains on `codex/small-string-experiment`;
do not merge it. See `git log` for exact latest evidence checkpoint.

**Validation:**
- GitHub CI dispatched today ran only the default subset, because
  `gh workflow run CI --ref <branch>` without `-f full_regressions=true` skips the
  complete `cargo test --workspace` step. Do not call such a run "full".
  The repository is public, so Actions minutes are free, but the owner now
  prefers local runs (other projects stopped to free memory).
- Full local regression (one test binary at a time, memory-guarded) was stopped
  at the owner's request after 430 of 491 test files (all 399 compiler files
  plus 31 of 92 editor files; editor files after `completion_matrix`
  alphabetically were not run, nor the `--lib` unit tests at the end).
  Results: one real failure, fixed in `4bf84f84` (`primitive_storage` expected
  the removed 1 GiB inline-record limit). Environment-only failures when test
  binaries run outside cargo: `native_cache` (passes under `cargo test`),
  `native_dependency_bundle` and `shared_sources` (need `CARGO_TARGET_DIR`).
  `shared_sources::portable_export_remains_toolchain_free_and_rebuilds_after_relocation_offline`
  still fails locally with `CARGO_TARGET_DIR` set: "Portable sources are not
  prepared. Run `tsrust setup --portable --yes`". Likely machine setup (CI
  prepares portable sources); confirm before treating it as a defect.
- Validation resumed locally on `3a625efe`: run all 77 current LSP integration
  targets alphabetically after `completion_matrix`, then both crates' `--lib`
  targets. The earlier 31/92 and 61-remaining counts do not match the current
  file inventory, so target names determine coverage. Each Cargo invocation runs
  alone with `--test-threads=3`, `CARGO_INCREMENTAL=0`, one build job and
  `CARGO_TARGET_DIR` set to the repository target. A process-group watchdog polls
  RSS every 0.5 seconds and kills the group above 5,000,000 KiB or 1,800 seconds.
  Disposable current-run script/results: `/tmp/tsrust-validation-watchdog.py` and
  `/tmp/tsrust-validation-results.json`. All 77 resumed LSP integration targets
  passed: 798 tests, zero failures, three existing ignored tests. The catalog
  target passed in 757.76 seconds; peak RSS across the resumed targets was
  1,164,768 KiB (including the initial dependency rebuild). Compiler `--lib`
  passed 1,292 tests with eight existing ignores; LSP `--lib` passed 20 tests.
  Total resumed validation: 2,110 passed, zero failures, 11 existing ignores,
  no watchdog stops. Overall peak RSS was 1,611,840 KiB. Compiler library build
  reported the existing unused `decode_plain` warning in `stdlib/json/typed.rs`.
  This requested validation stage is complete; the accepted optimization and
  remaining work are recorded below.
  Remote validation is not needed for this local optimization stage.

**How to run tests safely (8 GB Mac):** a test once grew to 30 GB when a compiler
limit was removed (non-terminating instantiation). Always run test binaries
under a watchdog that kills the whole process group past a memory/time budget,
e.g. a script that starts the binary with `set -m`, sums RSS for its process
group with `ps -A -o pgid=,rss=` every 0.5 s, and `kill -9 -$pid` past ~5 GB or
30 min. Use `--test-threads=3`. Run each binary from its crate directory
(`crates/tsrust-compiler` or `crates/tsrust-lsp`) or under `cargo test`, and set
`CARGO_TARGET_DIR=$PWD/target`, or several tests fail for environmental reasons.
Compiler and editor crates share some test names (for example `api_project`):
pick binaries per crate, not by name alone. Use bash (zsh does not split
unquoted variables and mangles `$var:path`).

**How to measure:** time only on a quiet machine (the runner refuses above load
4). A/B by checking out the baseline commit, running
`node benchmarks/runtime/run.mjs --only=<id> --engines=native,node-tsc` per
workload, then the change; compare median sample durations with the Node.js
control (a control far from 1.0 means the machine moved). Add
`rust-hand` where `benchmarks/runtime/parity/<id>.rs` exists. Afterwards run
`npm run store:ingest` in `benchmarks/runtime` (adds runs to
`benchmarks/measurements.sqlite`); raw result files are disposable after that.
`npm run checkpoint` prints latest native/Node ratios. For allocations,
`node benchmarks/runtime/profile-native.mjs --only=<id>`; for per-statement
counts, wrap a copy of the generated program with a counting global allocator.
`sample <pid> 3` works (Developer Mode is enabled).

**Accepted Map snapshot optimization — 2026-10-30:**
- Baseline: validation checkpoint `6b8f62b1`. A three-second CPU sample of the
  depth-80 nesting probe identified repeated `contains_scoped_map_storage`
  walks beneath Map epoch snapshot collection. The six snapshot sites now use
  one helper that classifies each distinct type once per snapshot. Its cache
  cannot outlive record-layout changes; snapshot ordering remains `BTreeMap`.
- The new native witness exposed stale optional-selection union/payload metadata
  after Map entry narrowing. Retargeting now aligns the selected payload before
  selecting callback dispatch, fixing a generated `TsrCallbackError` versus
  `TsrStorageError` mismatch. Present/missing entries and capture release pass.
  Initial development failures were a helper `HashMap`/`BTreeMap` return-type
  mismatch and an invalid nullable assertion; both were corrected before acceptance.
- Final affected validation: **347 passed, zero failures** across compiler
  `scoped_record_composition` (204), `optional_chaining`,
  `borrowed_optional_fallback`, `exception_return_moves`, `maps`,
  `exception_handling`; LSP `maps`, `exception_handling`; compiler library
  `map_` filter (59). One binary at a time, three test threads, one Cargo job,
  incremental disabled, shared target, 5,000,000 KiB/1,800-second watchdog.
  Peak process-group RSS: 2,088,960 KiB. The earlier 203 passing scoped-record
  cases were retained, then that affected suite was rerun for the metadata fix.
- A/B protocol: existing `nesting-discovery/run.py` scoped-map-catch generator;
  fresh compiler library-probe processes on the ordinary test worker stack;
  two warmups and 11 samples per depth, load average at most 4 before each.
  Native check/emit medians (baseline → candidate): depth 32 **460.521 →
  180.750 ms (2.55×)**; 64 **3,189.947 → 818.285 ms (3.90×)**;
  80 **6,067.294 → 1,402.972 ms (4.32×)**. Node/TypeScript strict no-emit
  controls: **155.646 → 157.441**, **161.296 → 160.239**,
  **161.154 → 159.901 ms**, all within 1.2%. These are frontend costs with
  different language contracts, not runtime throughput measurements.
- `npm run store:ingest -- /tmp/tsrust-map-baseline-timings.json
  /tmp/tsrust-map-candidate-timings.json` added all 12 history entries to
  `benchmarks/measurements.sqlite`; integrity check returned `ok` and all six
  workload/engine rows contain both 11-sample runs. Engine names are
  `native-check-emit` and `node-tsc-check` to avoid conflating runtime data.
- Whole-project formatting with the validated CLI changed zero files, catalog
  snippets or benchmark fingerprints. The native witness uses the shared
  formatting round-trip helper, including idempotence and formatted execution.
- Next: profile the remaining cost of scoped Maps across catch regions before
  another optimization. Repeated subtree traversal and binding scans still
  exist; this checkpoint does **not** claim to eliminate cubic scaling.
  Existing input generator and `/tmp/tsrust-time-map.py` can reproduce the
  measurement. The temporary scripts/profile are disposable, not new handoffs.
  Cleanup removed 577 older duplicate test executables (8,673,106,648 bytes),
  4,490 completed-build intermediate objects (3,980,355,832 bytes), and 16 old
  native program cache entries (79,347,359 bytes). The newest executable per
  test source, current CLI/LSP binaries, latest native program, root locks and
  shared runtime/Cargo dependencies remain. Target size is 20 GB; no rebuild
  was started to refill caches. Ingested raw timing files were deleted.
  No push, CI dispatch, merge, release or dependency change was made.

**Accepted callable-boundary Map epochs — 2026-10-30:**
- Baseline commit `260d1781`, clean tree at start. A fresh depth-128 CPU sample
  identified repeated Map epoch planning beneath `finish_block`, including
  snapshot filtering/hashing and nested body replay. Native Map epoch planning
  now runs at callable boundaries or the existing post-check parameter-carrier
  replay. Child-block carrier joins and ownership checking remain unchanged.
  The depth-16 native witness also executes inside a nested callable, checking
  its separate lexical boundary and present/missing entries after catches.
- Affected validation: **347 passed, zero failures**, using the same nine-target
  selection and watchdog as the preceding checkpoint. Peak process-group RSS:
  2,007,600 KiB; no watchdog stops. Compiler library `map_` contributed 59 tests.
  Whole-project formatting with the validated CLI changed zero files, snippets
  or fingerprints. The expanded witness verifies formatting round trips and
  generated native execution through the existing helper.
- A/B protocol unchanged: two warmups, 11 samples per depth/engine, fresh probe
  processes, load at most 4 before each sample. Timing initially refused while
  Spotlight indexed build outputs; measurements began only after load fell.
  Native check/emit medians (baseline to candidate): depth 32 **170.383 to
  65.181 ms (2.61×)**; 64 **784.422 to 170.969 ms (4.59×)**; 80 **1,375.769
  to 250.132 ms (5.50×)**; 128 **4,631.181 to 563.738 ms (8.22×)**.
  Node/TypeScript controls: **153.102 to 154.681**, **156.083 to 157.235**,
  **160.085 to 159.522**, **168.284 to 170.127 ms**, all within 1.1%.
  Fourfold depth now costs 8.65× instead of 27.18× in this probe; this measures
  the removed repeated epoch walk, not a formal bound on all compiler passes.
- `npm run store:ingest` added 16 run entries to `benchmarks/measurements.sqlite`.
  Integrity returned `ok`; all eight workload/engine rows contain both new
  11-sample runs. Engine names distinguish frontend costs from runtime data.
  Ingested raw timing files were deleted after verification.
- Cleanup removed 129 completed-build objects (1,118,442,264 bytes). No obsolete
  test or native executables remained. No push, CI dispatch, merge, release,
  dependency change, generated unsafe or limit change was made.
- Next: profile schema trial restoration allocations, ranked target 2. Map
  compile-time optimization is accepted; further compiler changes need a new
  profile and measured benefit rather than more speculative pass restructuring.

**Accepted typed schema restoration adapters — 2026-10-30:**
- Baseline `cea19c4c`. Fresh allocation profiling found 96,000 allocations for
  2,000 collection-workload iterations (48 each). `sequence`, `pair`, `inject`
  and `remap_input` boxed each intermediate restoration. They now retain typed
  restoration state; generated function and heterogeneous branch boundaries
  still erase types. No public signatures, ownership rules or limits changed.
- Initial native collection tests found an opaque-return lifetime leak in the
  candidate: `pair` retained the validator type and its temporary mutable path
  borrow. `PairRestoration<L,R>` now stores only actual rollback state. The
  new library witness composes all adapters, reuses callback-local state, then
  rejects a readonly refinement and verifies original payload pointers/values.
- Final affected validation: **209 passed, zero failures**: all 11 compiler
  schema integration targets (100), LSP schema editing (15), compiler library
  `stdlib::schema::` (94). One binary at a time, three test threads, one Cargo
  job, shared target, incremental disabled; 5,000,000 KiB/1,800-second watchdog.
  Peak RSS 2,016,624 KiB, no watchdog stops. Only the existing `decode_plain`
  unused warning appeared. Native fixtures retain formatting round trips.
- Candidate allocation profile: **86,000 allocations / 2,000 iterations = 43
  each**, down 10.4%; zero reallocations, correct checksum and zero live bytes
  after drop. The allocation profiler is diagnostic, separate from timing.
- A/B keeps the original protocol: eight warmups, 31 samples in each of three
  processes (93 samples per workload/engine), system allocator, quiet load <4.
  Initial baseline/candidate maximum load was 3.483/3.864. Native medians in ms:
  collections **2.502834 to 2.253125 (1.1108×)**, recursive 3.001375 to 2.958833
  (1.0144×), owned_array 0.411333 to 0.409334 (1.0049×), json_object 0.838459
  to 0.809334 (1.0360×), json_union 0.326541 to 0.331375 (0.9854×), pipeline
  0.678959 to 0.696208 (0.9752×). Collections' Node control improved 1.05%.
- Apparent small regressions were investigated, not discarded. Focused repeat
  recovered pipeline to 0.674875 ms (1.0061× baseline; Node within 0.1%).
  json_object repeated at 0.829792 ms; its Node control was 7.3% slower, so no
  precise speedup is claimed there. json_union repeated at 0.332334 ms, prompting
  a source checkpoint and fresh rebuild of `cea19c4c`, then the candidate.
  Paired json_union medians were **0.328250 / 0.322959 ms**, with Node
  **0.367583 / 0.371667 ms**; maximum loads 3.847/3.779. Its emitted source
  outside the unused trial module and all **286,144 executable text bytes**
  were byte-identical between commits; there are no external trial-module
  references in that workload. The apparent slowdown was measurement variation.
- `npm run store:ingest` preserved all **34** initial/repeat history entries,
  including slower samples. SQLite integrity is `ok`; each ingested entry has
  93 samples. Raw timing/profile files were deleted after verification.
  Whole-project formatting changed zero files, snippets or fingerprints.
- Cleanup removed 130 completed-build objects (1,127,130,016 bytes) and 22 old
  native runtime program-cache entries (2,293,088,437 bytes). Latest executables,
  root locks and shared Cargo/runtime build dependencies remain. Both current
  debug and release compilers were built from the accepted candidate source.
  No push, CI dispatch, merge, release, dependency change or generated unsafe.
- Next: profile mixed-type small arrays (`type_aliases`), ranked target 3.
  Further schema allocation work needs a new profile and measured benefit.

**Accepted inline scalar-union arrays — 2026-10-30:**
- Baseline `5083586a`, clean tree. Previous goal turn made progress by committing
  the accepted typed schema restoration optimization.
- Fresh `type_aliases` allocation profile: 12,289 allocations and nine
  reallocations for 2,048 iterations, with correct output/drop accounting.
  The workload's two-element `int|string` array uses Vec despite the existing
  safe four-slot Array representation used for homogeneous scalar arrays.
- Candidate adds immutable scalar-member metadata alongside UnionId's existing
  Copy property, including snapshot/reinterning paths. Rank-zero arrays of
  canonical scalar unions use the existing Array representation. Record,
  collection, callable, recursive and deeper array layouts keep indirection.
  Conservative aggregate-size accounting includes the scalar union's tag.
  No new runtime container, dependency, language limit or unsafe code.
- New witnesses cover snapshot identity and recursive exclusion, plus native
  clone independence, crossing the four-slot growth boundary, moving arrays
  through functions, shrinking/mutation, nested-array storage, enum and literal
  unions. Existing fixtures cover array methods, equality and ownership.
- Baseline runtime run `results/2026-10-31T03-16-17-317Z.json`: original eight
  warmups, 31 samples across three processes (93 per workload/engine); maximum
  load 2.361. Native/Node medians: type_aliases 0.147041/0.039000 ms;
  union_array_values 0.557500/0.294208 ms. Those raw profile/timing files were deleted after the ingestion recorded below.
- First witness used unsupported enum-union includes(); changed it to supported
  iteration. That exposed a real emitter defect: data-union text registrations
  referenced Enum types without emitting their TsrJoinText adapters when an
  unrelated array used join(). The emitter now registers both numeric and
  string enums through their existing scalar text behavior. The native witness
  covers both enum kinds; the enum acceptance suite joins affected validation.
- Validation completed via `/tmp/tsrust-union-array-validation.py`: 12 compiler
  integration targets (union arrays, aliases, nullable/data unions, recursive
  storage/schema, primitive storage, call/array spread), three LSP targets
  (type aliases, generic aliases, switch statements), then both full --lib
  suites because checked union identity is shared across the compiler.
  One binary at a time, three threads, one Cargo job, shared target,
  CARGO_INCREMENTAL=0, 5,000,000 KiB/1,800-second process-group watchdog.
  All 12 compiler integration targets passed (102 tests), all three LSP targets
  passed (7), compiler --lib passed 1,294 with eight existing ignores, and LSP
  --lib passed 20: **1,423 passed, zero failures, eight existing ignores**.
  Peak RSS 1,910,736 KiB, no watchdog stops. The library build reported only the
  existing unused decode_plain warning.
- Candidate profile: type_aliases **10,241 allocations** (2,048 fewer), nine
  reallocations. Initial timing: union_array_values **0.557500 → 0.510917 ms
  (1.091×)**, Node 0.294208 → 0.287833 ms. type_aliases **0.147041 →
  0.156500 ms**, Node 0.039000 → 0.041291 ms; normalized speed 0.995×.
  Candidate max load 2.359. This initial pair was insufficient for acceptance;
  the final paired comparison below determined the decision.
- Added a matched twenty-element union_array_heap control to cover spilling,
  heap iteration and array methods. It shares the existing protocol, independent
  arithmetic oracle and guarded sources. Both registration/oracle tests and
  untimed native/Node verification pass. No new runtime machinery.
- Final paired comparison rebuilt release compilers from commits: parent
  `393bceea` contains exactly the `5083586a` compiler plus the new benchmark;
  candidate `a1fd6a0d` contains the validated storage change and enum text fix.
  All three workloads used the original eight warmups, 31 samples × three
  processes (93 samples per engine), system allocator and exact-result oracles.
  Maximum measured loads were 1.976 / 3.500; the runner waited while candidate
  post-build load exceeded four. Watchdog peak was 1,862,784 KiB, no stops.
- Native baseline → candidate medians: union_array_values **0.597458 →
  0.488333 ms (1.223×)**; union_array_heap **1.239000 → 1.204375 ms
  (1.029×)**; type_aliases **0.155750 → 0.147333 ms (1.057×)**.
  Node controls respectively: **0.287000 → 0.290500**, **0.812541 →
  0.777583**, **0.041000 → 0.038750 ms**. Control-normalized speeds are
  1.238× / 0.984× / 0.999×. Keep the small-union improvement (also 1.091×
  in the initial pair); do not claim a reliable speedup for type_aliases or
  heap arrays. The latter's normalized 1.6% variation is within the control
  movement; neither repeat native median regressed. Allocation reduction is
  measured independently, and all affected correctness checks pass.
- `npm run store:ingest` added all **20** timing entries, including initial
  slower samples. SQLite integrity returned `ok`; each entry has 93 samples.
  Raw results/profiles were deleted after checking stored timestamps/counts.
  Whole-project formatting passed before both checkpoints: token-preserving
  formatting of the new benchmark, then zero changes on subsequent runs.
  The new benchmark passed native/Node exact-result verification and both
  registration/oracle tests. Compiler witnesses include formatted execution.
- Cleanup removed **204** completed-build objects (**1,168,179,056 bytes**).
  Each native program-cache root already retains only its latest executable;
  shared Cargo/runtime dependencies remain. Debug and release compilers both
  reflect the accepted candidate. No push, CI dispatch, merge or release.
- Next: profile single-pass schema JSON decoding, ranked target 4. Establish
  a fresh allocation/CPU profile before changing subtree cursor behavior.


**Accepted schema JSON parser-default repair — 2026-10-31:**
- Previous goal turn made progress: accepted scalar-union storage and persisted
  measured results in `610c2bc0`. Tree was clean before this section.
- Baseline `610c2bc0`: native allocation profiles report 26,000 allocations
  for native_schema_json_fused and 86,000 for native_schema_collections across
  2,000 iterations (13/43 per iteration), no reallocations or retained leaks.
- Fresh baseline timing used release/native programs built before source edits,
  three processes × 31 samples, eight warmups, Node-after-tsc controls, maximum
  measured load 3.706 (runner waited for load below four). Native/Node medians:
  fused 0.876209/0.673375 ms; ordinary JSON object 0.816417/0.671375 ms;
  collections 2.243541/0.813458 ms. Raw result:
  `results/2026-10-31T03-54-42-594Z.json`, pending paired timing/ingestion.
- Three-second CPU sample of baseline generated fused workload collected 2,132
  samples. Source field BTreeMap construction, RawValue scans, validation and
  allocation/free paths are visible. Existing field-index cursors reparse each
  subtree; callbacks require complete syntax validation, last-duplicate-wins
  resolution and schema field order. Any streaming replacement must preserve
  those behaviors rather than validating callbacks in source-field order.
- Found a concrete default-limit defect: schema safeParseJson still imposed
  16 MiB / depth 64 / one-million-entry/node caps, contrary to owner direction
  and ordinary JSON.parse's Rust defaults. Fix in progress: reuse the existing
  syntax validator with unbounded count budgets and Serde's recursion guard;
  avoid repeating UTF-8 validation for an existing str. Explicit bounded JSON
  entry points keep their existing behavior. This is a correctness repair,
  not a claim that the ranked single-pass decoder is implemented.
- Added native witnesses above the old byte/entry/node/depth caps, malformed
  input before callbacks, recursion agreement with serde_json, duplicate
  invalid content, and exact numbers. Updated safeParseJson IntelliSense.
  Existing issue/path retention budgets remain unchanged.
- Affected validation complete via `/tmp/tsrust-schema-defaults-validation.py`:
  JSON library tests 62, compiler schema_decode 7 / collections 4 / recursive 10
  / unions 13, LSP schema_editing 15: **111 passed, zero failures/ignores**.
  Shared target, incremental off, one job/target at a time, three test threads,
  5,000,000 KiB / 1,800-second watchdog. Peak group RSS 2,006,640 KiB; no stops.
  Catalog examples and native fixtures passed formatted execution.
- First candidate built/formatted successfully. Native baseline → candidate /
  repeat medians: fused 0.876209 → 0.860208 / 0.857625 ms; ordinary object
  0.816417 → 0.827459 / 0.796292 ms; collections 2.243541 → 2.286083 /
  2.284541 ms. Node baseline → candidate / repeat: fused 0.673375 →
  0.676667 / 0.665583; object 0.671375 → 0.712459 / 0.664208;
  collections 0.813458 → 0.806083 / 0.813666 ms. Max loads 3.706 / 3.610 /
  3.637. The repeated collections ~1.8% slowdown is not dismissed as noise.
- All 18 run entries were ingested (93 samples each). This correctness repair
  is checkpointed provisionally, not accepted as a performance improvement.
  Next: specialize the existing validation visitor at compile time so unlimited
  parsing does not maintain count/depth budgets. Recheck explicit bounded
  behavior and plain decoding, then repeat affected timings before acceptance.
  Single-pass schema decoding remains outstanding after this repair.
- Provisional source checkpoint is `11dc24cf`; its 1.8% collections slowdown
  remains unaccepted. Specialization checkpoint `4cf6c13c` uses one const-generic
  visitor with BOUNDED=true for explicit-limit APIs and false for schema's
  default path. False removes node/entry/depth counter maintenance and checks;
  syntax, float detection, exact integers and Serde recursion remain intact.
  Specialization validation passed via `/tmp/tsrust-schema-defaults-specialization.py`:
  62 JSON library tests and seven schema_decode tests, zero failures/ignores.
  Sequential watchdog peak 1,985,056 KiB; no stops. Other 42 already-passed
  schema/editor tests remain valid; no emitter/API changes since that complete
  111-test run. Specialized release build and whole-project formatting passed.
- Specialized timings: fused **0.839375 ms**, ordinary object **0.791625 ms**,
  collections **2.280334 ms**; Node controls 0.679834 / 0.719750 / 0.807166 ms.
  Maximum load 3.376, watchdog peak 1,671,168 KiB, 93 samples per row.
  Fused improves 2.2% over 11dc24cf (4.4% over 610c2bc0); collections remains
  about 1.6% behind the original baseline, despite being marginally faster than
  11dc24cf. All six specialized entries ingested, including this residual signal.
- Final rebuilt A/B: original `610c2bc0` versus specialized `4cf6c13c`.
  Fused **0.864458 → 0.832166 ms (1.039×)**; collections **2.311375 →
  2.281542 ms (1.013×)**. Node controls: fused 0.683500 → 0.708000 ms;
  collections 0.813042 → 0.810500 ms. Original baseline collections itself
  moved from 2.243541 to 2.311375 ms across rebuilds (3.0% variation), while
  specialized repeats stayed at 2.280334 / 2.281542 ms. The earlier residual
  slowdown did not reproduce in the fresh pair. Accept the parser-default fix
  and the specialization; claim only a small fused-decoding speed improvement.
- Unchanged ordinary JSON object control: **0.804375 → 0.824542 ms**; Node
  **0.672208 → 0.731834 ms**. Its native measurements ranged 0.791625–0.827459
  across runs, and this workload does not call safeParseJson. Control movement
  limits precise speed claims; it is not treated as an optimization result.
  Fresh measured loads max 3.715 / 3.298; watchdog peak 1,942,112 KiB, no stops.
- `npm run store:ingest` retained all **36** timing entries across six runs,
  including every slower observation. SQLite integrity `ok`; each entry has
  93 samples. Raw timings/allocation profile deleted after verification.
  Whole-project formatting passed with zero changes. Cleanup removed 110
  completed-build objects (**1,112,962,864 bytes**); native program caches
  already retain only their latest executable. Current debug/release compilers
  reflect the accepted specialization; requested branch restored, no remote work.
- Next: implement direct DeserializeSeed decoding for plain schema outputs,
  retaining the current decoder for unsupported plans and detailed failure
  reporting. Fast decoding must execute no callbacks; only accept after full
  input consumption and exact integer-token checks. Missing/unknown fields,
  constraints, duplicate replacement, invalid Unicode and nested invalid values
  must retain existing semantics through fallback. Profile larger nested cases
  too; this repair does not complete ranked target 4.



**Accepted plain-schema direct decoding — 2026-10-31:**
- Previous goal turn completed the parser-default repair in `a74da73f`; this
  turn began with a clean tree on the requested branch. Benchmark-only
  checkpoint `0a2443b8`
  adds a four-group/sixteen-integer nested JSON control and matching TypeScript
  implementation, independent arithmetic oracle, routing, guards and report
  category. All 36 schema benchmark tests pass. Whole-project formatting proved
  the new fixture's token preservation and idempotence before checkpointing.
- Fresh accepted-code profiles: fused 26,000 and collections 86,000 allocations
  across 2,000 iterations, zero reallocations or retained leaks. The nested
  profiling attempt exposed that profile-native.mjs always rebuilds Cargo: its
  internal 120-second build timed out on uncommitted candidate source before
  recording any profile or timing. Session 93445 is terminal. Its confirmed
  orphan rustc (PID 68916) exited before the termination request (no such
  process); no overlapping work is permitted.
  Retried correctly from `0a2443b8`, with an explicit release build under the
  external watchdog before invoking the profiler/runner. Baseline completed;
  requested branch restored. Nested profile: 26,000 allocations, no reallocations
  or retained leaks for 2,000 iterations. Timing file:
  `results/2026-10-31T04-59-04-347Z.json` (93 samples per workload/engine).
  Native/Node medians (ms): fused 0.825708/0.670500; nested
  6.566417/1.823125; ordinary object 0.816333/0.685959; collections
  2.290833/0.807750. Maximum load 3.772.
  Peak group RSS 1,458,992 KiB; no watchdog stops.
- Candidate emits callback-free DeserializeSeed visitors for plain scalar,
  array and object schemas. Typed output is built directly, without per-object
  BTreeMap indexes or repeated RawValue subtree scans. Fields overwrite their
  slots on duplicates; unknown/missing fields and unsupported plans fall back.
  No callback executes during the speculative attempt. Full consumption and
  exact integer-token checking precede accepting output. Pipelines, choices,
  recursive/retained outputs and detailed failures retain the existing decoder.
- New witnesses cover nested arrays, constraints, escaped duplicate keys,
  invalid earlier/final duplicates, malformed ignored content, large exact
  integers, source-specific diagnostics and callback order. Runtime helper tests
  cover trailing data, overflow, precision and negative zero. Generated output
  is checked for the direct visitor and absence of unsafe. No public API added.
- Candidate checkpoint `680ff0d0` plus the fixes below is accepted. Inspection of Serde's
  implementation found that IgnoredAny does not enforce its ordinary recursion
  guard. The candidate now uses CheckedIgnored, reusing our syntax visitor with
  default guard behavior, for unknown fields and unsupported runtime choices.
  Added unit/native witnesses for ignored excessive nesting and Unicode.
- Initial validation: all **64 JSON library tests passed**. schema_decode had
  seven passes and one failure: the new no-unsafe text assertion found existing
  JSON Lines unit-test FFI copied into emitted source (under cfg(test)), plus a
  comment containing the word unsafe. This was not direct-decoder unsafe code.
  JSON runtime emission now excludes that trailing test-only module; compiler
  unit tests remain intact. The witness checks actual emitted unsafe block/FFI
  spellings, allowing comments. No performance acceptance claim yet.
- Resumed validation passed: schema_decode 8, JSONL owned 6/borrowed 5/writer 5,
  collections 4, pipelines 18, recursive 10, unions 13, LSP schema_editing 15.
  Including the completed 64 JSON library tests: **148 passed, zero failures**.
  One target at a time under the 5 GB/30-minute watchdog, three test threads,
  shared target and incremental compilation disabled. Peak RSS 1,712,736 KiB;
  no watchdog stops. No passing suite was restarted.
- Release build and whole-project formatting passed; zero files, snippets or
  fingerprints changed. Exact-result checks passed for all four workloads.
  Allocation profiles over 2,000 iterations: fused **26,000 → 22,000**
  (13 → 11 each); nested **26,000 → 10,000** (13 → 5 each); collections
  unchanged at 86,000 (43 each), all with zero reallocations.
- Paired A/B against `0a2443b8`: eight warmups, 31 samples × three processes,
  93 samples per workload/engine, system allocator. Runner waited for post-build
  load below four; candidate maximum measured load 3.984. Watchdog peak
  1,545,856 KiB; no stops. Native baseline → candidate medians (ms):
  fused **0.825708 → 0.710917 (1.161×)**; nested **6.566417 → 2.103542
  (3.122×)**; ordinary object 0.816333 → 0.791583; collections
  2.290833 → 2.266125. Node controls respectively: 0.670500 → 0.739417,
  1.823125 → 1.845167, 0.685959 → 0.693417, 0.807750 → 0.828208.
  Fused Node moved 10.3% slower, so the exact fused speedup is less precise;
  its movement cannot explain the native gain. Nested Node moved only 1.2%.
  No native median regressed; no optimization claim for unchanged controls.
- `npm run store:ingest` added all **16** timing history entries. Verified
  SQLite integrity (`ok`) and all timestamps/sample counts (93 each), then
  deleted disposable raw timings/profiles. Cleanup removed 122 completed-build
  objects (**1,123,992,192 bytes**); both native program-cache roots already
  keep only their latest executable. Shared dependencies remain.
- Plain scalar/array/object decoding is accepted; advanced schemas still use
  the existing decoder. Next profile remaining advanced-schema cost before
  expanding this optimization. The later owner decision keeps Rust-default
  string ownership; see the accepted sizing pass below. No push, CI dispatch,
  merge or release.


**Current milestone validation — 2026-10-31 (complete):**
- Full local inventory reconciled: all 399 compiler integration files, all 91
  LSP integration files, and both complete library targets: **492 targets,
  5,321 passed, 19 ignored, zero unresolved failures, zero watchdog stops**.
  Compiler library: 1,297 passed/eight ignored; LSP library: 20 passed.
  Nine previously validated integration targets contributed 84 passes exactly
  once. Rerun targets replaced earlier results; partial failures were not
  double-counted. Peak process-group RSS was 1,888,848 KiB. Every target ran
  sequentially with three test threads, one build job, shared `target`,
  `CARGO_INCREMENTAL=0`, and a 5,000,000 KiB/1,800-second watchdog. No CI claim.
- Portable-export test initially lacked `TSRUST_SOURCE_PACK_DIR`. Its explicit
  local prepared source pack repaired the environment; the failed test passed
  on filtered retry, preserving five original passes. No product change.
- Retained UI ExampleCard diagnostics intermittently consumed an earlier empty
  publication. Isolated retry passed. Fixed the shared test wait to match URI
  and tracked document version, retaining unversioned closed-file publications.
  Added a deterministic queued stale/current-publication regression. All six
  worker tests and every suite using the affected wait passed; full UI: 31/31.
  This is test-support correctness work, with no runtime speed claim or timing
  store entry. Generated runtime implementation is unchanged since `a2f94dee`.
- Long gates passed: checker/editor agreement 515.18 s; completion matrix
  499.49 s; intelligence/native backend catalog 644.42 s; compiler library
  including build 325.86 s; LSP library including build 5.19 s.
- All runners are terminal. Disposable ledgers/logs remain under
  `/tmp/tsrust-schema-milestone-*`; final runner session 61290 exited zero.
  Whole-project formatting with current debug CLI passed: zero changed files,
  zero documentation snippets, zero fingerprint updates. Diff whitespace clean.
- Fresh three-second collections CPU sample before this milestone included
  substantial transform/trial/allocator work; field-index visitor work was
  smaller. Do not expand the advanced-schema decoder without an isolated
  representative profile. No new performance claim from that diagnostic sample.
- Owner resolved string storage: Rust defaults. Borrow slices where lifetimes
  permit; copy independently owned strings. No reference-counted backing or
  automatic copy/share size threshold. Known byte lengths may inform capacity;
  any extra sizing pass must prove an end-to-end benefit in fresh paired runs.
- Next: profile existing string workloads, starting with replacement first/all,
  plain/token/empty-pattern cases and slice/concat controls. Runtime concat,
  padding, addition and borrowed-to-owned append/reserve already allocate known
  byte capacity; repeat delegates to Rust after checked sizing. Token replacement
  starts at source byte capacity and may reallocate on expansion. Profile before
  changing it. Existing replacement histories contain 93 samples per engine;
  use fresh baseline/current comparisons, not historical times for acceptance.

**Accepted single token-replacement sizing — 2026-10-31:**
- Source checkpoint `ab83377b` is accepted after repeated parent/candidate timing
  against `1e4d2547`. `.replace()` computes checked byte capacity using the same
  token parser that writes the result. `.replaceAll()` retains one-pass growth;
  plain replacements retain Rust's standard operations. Independent owned
  strings still copy; no shared backing storage, unsafe code or new size limits.
- Profile first: over 2,000 iterations, first/token reallocations fell 1,012 ->
  12, first/empty-pattern 3,012 -> 12; allocations remain 2,003. Slice and concat
  controls unchanged. Three allocation samples each checked independent output
  checksums and zero live bytes after drop. Allocation profile peak 315,392 KiB.
- All 14 affected tests passed (string_construction, generic_default_builtins,
  string_concat_padding, string_substring), including native debug/release output,
  Unicode/empty/unknown tokens, ownership, operand evaluation and no generated
  unsafe. Sequential 5,000,000 KiB/1,800 s watchdog, three threads; peak
  1,602,912 KiB, no stops. Product behavior and IntelliSense unchanged.
- Fresh repeat A/B, 8 warmups and 31 samples x three processes per workload and
  engine (93 samples); quiet maxima 2.238 parent/2.292 candidate, limit four:

  | Workload | Native parent -> accepted ms | Node parent -> accepted ms |
  | --- | --- | --- |
  | first/plain | .067458 -> .069083 | .075292 -> .076584 |
  | first/tokens | .113916 -> .115042 | .175542 -> .173291 |
  | first/empty pattern | .158125 -> .110292 (**1.434x**) | .222584 -> .220042 |
  | all/plain | .075459 -> .074291 | .191500 -> .190292 |
  | all/tokens | .257250 -> .251833 | .617792 -> .619875 |
  | all/empty pattern | .391875 -> .379292 | 1.129583 -> 1.149291 |

  Target improvement is 30.25%, with Node control -1.14%. Other native medians
  range -3.21% to +2.41%; their process medians overlap and do not establish a
  repeatable regression. No speed claim for unchanged controls or first/tokens.
  Earlier single-only trial independently showed first/empty 22.39% faster.
- Rejected full pre-sizing: all/tokens .235500 -> .265375 ms (+12.69%), versus
  Node +5.59%, despite fewer reallocations. The extra scan was not worthwhile.
  Retained only the measured single-replacement improvement. Initial broad-run
  control drift prompted the fresh comparison above rather than early acceptance.
- All five runs (baseline, rejected full scan, initial single-only candidate,
  repeated parent and accepted candidate) ingested: 60 minimal workload/engine
  histories, 93 samples each; verified in SQLite with integrity check. Removed
  all five disposable timing files and three allocation profiles, 47 obsolete
  object files (345,268,704 bytes), and six older native build caches.
  Whole-project formatter: zero files/snippets/fingerprints changed.
- Repeat runner session 52120 exited zero; requested branch restored. Release
  rebuilds took 210.060/209.754 s, peaks 1,401,808/1,531,520 KiB; timed passes
  9.334/9.847 s. No active job; cleanup complete. This acceptance checkpoint
  closes ranked target five. Full milestone plus affected string suites are green.

**Accepted typed JSON pipeline restoration — 2026-10-31:**
- Baseline `1d3eb962`. Allocation/CPU profiles identified JSON pipeline leaf
  adapters boxing already typed `RestoreJson` state before callbacks. The main
  pipeline adapter appeared in 391/2,093 sampled stacks including allocator work.
  Uninstrumented 20M-iteration pipeline/recursive checksums verified 140,000,007
  and 80,000,007. Recursive sample also showed dynamic JSON/BTreeMap costs.
- JSON leaves now retain concrete restoration through existing `Trial::remap_input`
  and `impl FnOnce + use<'a>`. Refinements and supplied non-branching wrappers
  preserve that type; choices, recursion, presence alternatives, consuming
  stages and composite adapters retain their existing boxing. No new runtime
  machinery, unsafe, arbitrary limits, API or IntelliSense behavior change.
- New chained-refinement witness verifies exact rollback before a consuming
  fallback and successful reuse. All **70 affected tests passed**: pipelines 19,
  collections 4, recursive 10, unions 13, diagnostic/default options 11,
  super-refine 5, decode 8. Three threads, sequential targets, shared Cargo target,
  incremental disabled; 5,000,000 KiB/1,800 s watchdog. Peak 1,498,528 KiB,
  no stops. A nonexistent queue entry was corrected without rerunning passes.
- Three allocation samples over 2,000 iterations, checked checksums/no retained
  bytes: pipeline **24,000 -> 20,000** (12 -> 10 each), recursive **86,000 ->
  80,000** (43 -> 40 each); union 10,001, discriminator 30,001, collections
  86,000 unchanged. Zero reallocations throughout.
- Fresh quiet A/B against parent, 8 warmups and 31 samples x three processes,
  93 samples per workload/engine; max load 3.552 baseline/3.741 candidate:

  | Workload | Native parent -> accepted ms | Node parent -> accepted ms |
  | --- | --- | --- |
  | pipeline | .687542 -> .636250 (**1.081x**) | .341041 -> .362625 |
  | recursive | 2.944292 -> 2.840458 (**1.037x**) | 1.346416 -> 1.338250 |
  | collections | 2.318583 -> 2.285125 | .803125 -> .819959 |
  | discriminator | .980667 -> .965583 | .532209 -> .542417 |
  | union | .321000 -> .325959 | .362125 -> .366167 |

  Pipeline native improved 7.46% while its Node control slowed 6.33%; recursive
  improved 3.53% with Node -0.61%. Union's +1.54% tracks Node +1.12%, within
  noise; no material control regression. No speed claim for unchanged controls.
- All 20 entries ingested and verified (93 samples), SQLite integrity clean.
  Whole-project formatter: zero files/snippets/fingerprints changed; diff clean.
  Deleted two raw timings, two allocation profiles and 53 obsolete object files
  (351,826,064 bytes). Native caches already retain one latest build each.
  Release build 209.432 s, peak 1,510,944 KiB; profile 74.746 s; timing including
  preparation/quiet wait 178.313 s. Sessions 69994/42876/16299 are terminal.
- Next: reprofile remaining composite pipeline restoration boundaries before
  expanding opaque returns. Do not rerun the complete milestone for this routine
  accepted gate. Keep original rollback/consuming-callback/lifetime witnesses.

**Lazy sequence rollback storage — rejected, 2026-10-31:**
- CPU sampling accepted `28a3409c`: collections/recursive 20M iterations each,
  checksums 160,000,007/80,000,007; 53.765 s, peak 32,608 KiB. Sequence buffer
  allocation/free appeared in consuming collection paths.
- Tried starting `Trial::sequence` restoration Vec empty, reserving the original
  input length only on the first reversible element. This avoided unused rollback
  storage after first-element commitment, preserving Rust ownership and capacity
  on prepared paths. 70 affected integration tests + 95 schema library tests
  passed under sequential 5 GB/30 min watchdog, three threads. Peak test RSS
  1,630,176 KiB. Candidate and added witness have now been removed.
- Allocation profile: collections 43→41/iteration, recursive 40→39; pipeline
  10 unchanged; discriminator 30,001 and union 10,001/2,000 unchanged. No reallocs.
- First quiet five-workload run (93 samples/engine, max load 3.390) against
  accepted parent measurement 2026-10-31T09:00:23.547Z: collections
  2.285125→2.190500 ms (-4.14%, Node -2.29%); recursive -.49% (Node -2.48%);
  discriminator -.32%, union -1.07%; pipeline .636250→.653917 ms (+2.78%,
  Node -6.72%). Candidate timestamp 2026-10-31T09:23:22.784Z.
- Fresh rebuilt parent/candidate pair confirmed pipeline regression: pipeline
  .633334→.651750 ms (+2.91%) while Node .345458→.342333 (-.90%). Collections
  2.307125→2.265334 (-1.81%), Node .830458→.819250 (-1.35%). Same 93 samples,
  max loads 3.684/3.984, timestamps 2026-10-31T09:29:12.559Z and
  2026-10-31T09:34:40.161Z. Fewer allocations did not justify slower pipeline.
- Rejected under the no-regression rule. Source restored exactly to `28a3409c`.
  All runners terminal zero: 34359, 53107, 22746. `npm run store:ingest` added
  all 18 run entries from these three timing runs; verified all 93-sample entries
  and SQLite integrity. Removed three raw results and allocation profile. Whole
  formatter changed zero files/snippets/fingerprints. Pruned 69 obsolete object
  files (762,892,448 bytes).
- Build executables currently reflect the rejected candidate; rebuild from current
  source before any further compiler/native measurement. Do not mistake binaries
  for accepted source. Next optimization requires a fresh profile; investigate
  composite pipeline/collection costs without this lazy sequence allocation.

**Concrete JSON composite restorations — candidate in progress:**
- Accepted source rebuilt after the rejected sequence experiment: release build
  session 44268 terminal zero, 207.976 s, peak 1,490,816 KiB. Collection/recursive
  native + Node correctness verification passed, session 47163, 31.143 s,
  peak 887,184 KiB. Release binary now represents accepted source `4d1a99d5`.
- Fresh CPU samples on those accepted executables: 20M iterations each,
  checksums collections 160,000,007 / recursive 80,000,007. Session 84629
  terminal zero, 54.776 s, peak 32,704 KiB. Log
  `/tmp/tsrust-composite-typed-cpu.log`; samples
  `/tmp/tsrust-native_schema_collections.sample` and recursive equivalent.
  Recursive sample explicitly shows `Trial::boxed` allocation within sequence
  callbacks; collection profile retains allocation/free costs in sequence paths.
- Candidate extends the accepted concrete JSON leaf restoration return type to
  JSON array and record adapters. Removes their final `.boxed()` only for these
  inputs, retaining boxes at choices, recursive references, objects and owned
  collection inputs. Leaves sequence capacity unchanged. Files:
  `emit/schema/{pipeline_trials,pipeline_array_trials,collection_trials}.rs`.
  Added `json_record_refinement_restores_entries_before_fallback` witness in
  schema_pipelines, complementing existing nested array/recursive rollback tests.
- Initial affected runner 89353 ended 101 in first suite: 16 passed, four
  generated-code failures (`E0700`), including nested array and record rollback.
  No watchdog stop. Cause: `sequence` returned an opaque closure capturing the
  validator type, which contains temporary path/issue borrows. Removing the outer
  Box exposed that overcapture. Remaining six suites were not started.
- Root-cause repair follows existing `PairRestoration`: runtime `trial.rs` now
  returns `SequenceRestoration<R>(Vec<R>)` implementing `Restore<Vec<I>,Vec<O>>`.
  Keeps exactly the prior eager capacity and rollback behavior, but excludes the
  finished validator's type from restoration state. No unsafe or lifetime extension.
- Repaired affected runner session 14345 finished zero: all 71 tests passed,
  peak 1,688,544 KiB, no watchdog stops. Evidence:
  `/tmp/tsrust-composite-typed-tests-repair.py`, log
  `/tmp/tsrust-composite-typed-tests-repair.log`. Seven suites, sequential
  5 GB/30 min watchdog, three threads, shared target, incremental disabled.
  Compiler `--lib stdlib::schema` is LIVE in session 18517 under watchdog, log
  `/tmp/tsrust-composite-typed-lib.log`; poll same handle, do not restart.
  Candidate not accepted or timed. After library tests pass rebuild release
  and run five-workload allocation profile + quiet 93-sample native/Node A/B
  against accepted source. Retain only measured gains without regressions,
  ingest, format whole project, commit and prune.

**Performance page refresh — owner requested, 2026-10-31:**
- Updated `website/tools/performance-runs.json` with accepted scalar-union,
  plain-schema decoder, string-sizing and typed JSON pipeline runs. Excludes
  rejected sequence-storage experiment and the current unmeasured candidate.
- Generator now selects the latest curated run per workload instead of counting
  duplicates; checks sample counts and displays each row's measurement date.
  Removes unsupported causal claims from the summary. Snapshot: 932 distinct
  workloads, native faster in 613, median mean-throughput ratio 1.52×.
- `npm run build` in website passed; generated performance route exists and
  all 932 workload IDs are unique. Local source/build updated; no deployment.

**Owner benchmark direction — 2026-10-31 (supersedes routine Node reruns):**
- Owner requested hand-written standard Rust comparisons alongside all public
  workloads and reusable Node/Rust baselines instead of rerunning them for every
  optimization. Preserve equivalent behavior, ownership, checks and outputs;
  report actual winners, including Tsrust overhead. Refresh reference baselines
  on workload/protocol/reference-source/toolchain/hardware changes, or when a
  noisy close result requires a fresh control. Native parent/candidate A/B stays.
- Existing `rust-hand` runner and two measured parity sources reused. Public
  performance generator now shows hand-written Rust throughput, native/Rust ratio,
  independent dates and explicit missing coverage (2/932). Baselines pinned by
  timestamp and parity-source SHA-256; sample counts checked. Website build passed.
  No new Node or Rust timing was run for this page update; no deployment.
- **Remaining owner request: implement and validate hand-written Rust coverage
  for the other 930 displayed workloads**, by coherent families with independent
  result oracles. Start current schema workloads, then remaining families. Do not
  claim all comparisons exist. Current schema candidate remains unaccepted until
  library validation and native A/B; use stored Node controls, not an automatic
  new Node timing run. Keep source/protocol provenance for reusable baselines.

**Owner rules learned this session:** match Rust's defaults for limits (no count
caps; depth bound only where rustc has `recursion_limit = 128`); generated code
must not contain `unsafe` (tests assert it); one handoff note only (this file);
fix defects when found; keep only the latest artifacts; ask the owner only for
merges or design decisions.

**Measured today (vs Node.js after tsc, M3):** single-character ASCII table and
direct `text.at(i)` (`string_at_characters` 30×, now 3.7× Node);
schema small-collection duplicate scan and identity restorations (1.26×);
`String(int)` via `to_string` (1.04–1.18×); `String(record.field)` moves a dead
record's field (`interfaces` 1.15×). Rejected with numbers: safe inline
small-string type (chains up to 47% slower, re-validating UTF-8 without
`unsafe`); `String(number)` whole-number path (up to 7% slower). Details in the
optimization log below.

**Ranked next targets:**
1. Scoped Maps across catch regions: completed the measured snapshot and
   callable-boundary epoch optimizations above. Depth 80 fell from 6.067 s to
   1.403 s in the first A/B, then from a fresh 1.376 s to 0.250 s in the second.
   Depth 128 is now 0.564 s versus 4.631 s on the preceding commit. Keep these
   regressions; proceed to schema trials before further compiler optimization.
2. Schema restoration adapters: completed the measured typed-restoration
   optimization above. Collection workload is 1.11× faster with 43 rather than
   48 allocations per iteration; retain the rollback/lifetime regressions.
   Move to mixed-type arrays before further schema allocation optimization.
3. Scalar-union arrays: completed safe inline storage above. Small-array
   scenarios measured 1.09–1.22× faster; type_aliases allocations fell by one
   per iteration without a reliable timing gain. Heap control also measured.
   Keep recursive/nested storage and enum text witnesses; move to JSON decoding.
4. Plain-schema `DeserializeSeed` decoding accepted above: nested 3.122×,
   fused 1.161× faster, fewer allocations. Advanced schemas retain the existing
   decoder; profile remaining costs before extending the fast path.
5. Completed the measured string sizing pass above: single empty-pattern token
   replacement 1.434x faster; replaceAll pre-sizing rejected. Rust-default
   ownership retained: borrow where lifetimes permit, copy independent owned
   strings. No reference-counted sharing or arbitrary size threshold.

**Next continuation:** the ranked passes and typed JSON pipeline restoration
have measured outcomes, but do not claim universal hand-written-Rust parity.
Advanced composite pipelines, choices and collections still box restoration
state. Reprofile those boundaries before extending the accepted typed approach.
Initial required validation and the full later milestone are complete; only run
affected suites for the next coherent change. Active candidate/session is recorded
above; follow that state before starting another job.

## Most recent owner correction — highest immediate priority

The owner said:

> im basically just seeing a lot of example how to documents appearing instead of
> actual documentation of the std lib and how the language works - it would be
> better to have an interactive guide of how tos than to just have a bunch of
> articles hogging up the search results

The current site over-fragments language topics into many small walkthroughs,
especially callable/generic features. Do not continue producing those articles.
Their verified facts/examples are reusable evidence, not an approved final docs
architecture. The initial `docs/plans/public-documentation-site.md` already proposed
broad language-reference topics; implementation drifted from that intent.

The requested direction is:

- Make language semantics and standard-library API reference the primary docs.
- Consolidate how-tos into an interactive guided learning experience; do not index
  every tutorial step as another competing reference article.
- Keep examples within reference entries where they explain an operation, without
  making examples the whole documentation model.
- Plan the revised structure in Markdown, then implement it incrementally. Preserve
  useful audited contracts and migrate links/anchors safely. No empty placeholders.
- Every page still needs factual and presentation review. Reorganization does not
  establish correctness, and a successful build does not establish a completed audit.

No revised navigation, consolidated reference, interactive guide or API-reference
generator has been implemented. Codex stopped after inspecting options. The
information-architecture skill was read, but no `.design` artifacts were created.

Useful implementation facts: Astro 7.4.2, Starlight 0.42.7; site configuration is
`website/astro.config.mjs`, content is `website/src/content/docs/`. Installed
Starlight schema supports frontmatter `pagefind: false`. Reuse its navigation,
styles and accessibility instead of adding another framework. Do not hide useful
reference content before its replacement exists. Preserve old URLs through the
migration, then remove superseded public source copies.

`crates/tsrust-lsp/src/builtins.json` has 103 groups / 894 entries, including
signatures, parameter documentation, behavior, ownership and examples. It is a
useful canonical source for a synchronized stdlib reference. Some group keys are
internal names; do not expose them as public types without mapping them. The
existing library overview lists 23 public module names, but detailed API reference
coverage is still incomplete. `std:path` (7 exports) and `std:env` (3 entries) were
only inspected as possible starting points; no generation work was done.

## Full goal and remaining work

Finish the finite backend-only language/CLI/docs/platform polish from PR #2. Do
not expand into native UI or arbitrary TypeScript compatibility. The original fan
stop was explicitly lifted: the owner is using a different computer.

- Async testing registration and HTTP body/SDK work have substantial accepted
  evidence in `docs/agent-goal.md` and the progress record; preserve that evidence.
- Shared dependency caches and paired CLI/LSP installation/update have implemented
  workflows and focused macOS/Linux/Windows evidence. Remaining distribution
  checks must be reconciled, not assumed complete.
- Windows and Linux installer/locked-binary checks passed previously. The newer
  isolated editor UTF-16 fixture and fresh toolchain smoke/acquisition paths still
  need their outstanding platform executions. Exact release-build commands and
  actual published acquisition remain unproven where marked pending.
- GitHub Pages was selected and configured, but a successful production deployment
  and public URLs remain pending. Local preview/build is not deployment evidence.
- Complete the finite remaining docs inventory in the migration/audit plans, using
  the corrected reference/learning architecture. Old pending/unsupported claims
  often contradict newer compiler behavior; check source before copying them.
- Retain completed family correctness/performance records. Do not restart full
  matrices or timing protocols after routine documentation changes.

A reported LSP retention issue was repaired and a bounded 30-minute macOS stress
check passed. The old blanket warning was removed. This is not proof of every
possible long-session behavior. HTTP performance evidence and original SQLite
records already exist; no new measurements were taken during recent docs edits.

The last retained nested-function guide still contains unaudited owned/scoped
selection, single-use call scopes, generic replacement and mutable factory arrays.
Its source-flow paragraph was investigated, but is not a mandate for another page.
Other language/library/operational inventory remains pending too.

## GitHub and release constraints

Original PR #2 was bootstrapped with owner approval. Last recorded draft delivery
PR is https://github.com/Forever-Frameworks-LLC/tsrust/pull/4, base `stag`, remote
branch `codex/backend-polish-staging`. Last recorded remote head was
`61ee67a44152437848a53d0fea6499e7ffec56ec`; its tree matched local `1b375f06`.
Later local work, including the recent docs commits, has not been pushed.
These are last-recorded values, not a fresh remote-state check. Revalidate before
any push. Do not force-push or trigger duplicate CI runs.

Owner-approved policy: branch-to-stag PRs run tests and require one review, with
owner review bypass; stag-to-main promotion compiles macOS/Linux/Windows artifacts,
then auto-merges/publishes if successful. New tags may be created by repository
writers; existing tags/releases must remain immutable. Prefer caching, minimal
permissions and dependable artifact reuse. Avoid unnecessary GitHub Actions usage.
No concrete new release version was selected; do not invent one or publish a new
release as part of this handoff.

## Validation and cleanup state

Recent accepted docs evidence is indexed in
`docs/plans/public-documentation-content-audit.md` and
`docs/plans/public-documentation-migration.md`. Their content-audit acceptance does
not mean the owner approved the fragmented article architecture.

The unfinished `language/callable-control-flow.md` article was deleted; its sidebar
addition and source-guide edits were reverted. No unfinished implementation remains.
Before deletion its two intentional rejection examples returned `arity`, and two
native examples printed `4 1` / `7` and `7 1`. Focused compiler flow checks passed
15 tests and editor diagnostics passed 4. Disposable logs, if still present:
`/tmp/tsrust-flow-doc-contracts.log` and `/tmp/tsrust-flow-doc-editor.log`. These are
optional current-run evidence, not durable analytics or required deliverables.

Cleanup rebuilt the restored 66-page site successfully (`npm --prefix website run
build`), removing the deleted article from generated output. The last full internal
link/anchor pass on that same committed site also passed. The agent-started docs
preview on port 4322 was stopped, its browser tab closed and viewport override reset.
No known compiler/test job is running. Restart preview if needed with
`npm --prefix website run dev -- --host 127.0.0.1 --port 4322`.

198 disposable native program cache entries (274 MiB) were pruned from
`target/tsrust-native/v2`; root lock files, shared native runtime cache and Cargo
target were preserved. Native programs rebuild when needed. Source, owner edits,
retained UI, fixtures and historical evidence were not removed.

## Working rules

One agent in this tree at a time. Inspect status first. Use the existing Cargo
target, `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, and focused section-level checks.
Do not touch the owner's VS Code instance. Format the entire project before each
source checkpoint with `TSRUST="$PWD/target/debug/tsrust" node tools/format-project.mjs`.
Keep native behavior, diagnostics, IntelliSense and examples consistent when code
changes. No new heavy suite for prose alone.

No source snapshots/archive directories. SQLite remains canonical analytics;
never publish ignored `benchmarks/records.sqlite`. Public measurements must use
`benchmarks/storage/public.py` and its provenance allowlist. Preserve original
records and distinguish unmeasured work from measured work. Keep the handoff and
finite acceptance checklist current; do not claim completion from partial evidence.

## Claude progress — 2026-10-28

Architecture recorded in `docs/plans/documentation-architecture.md` (reference
first, one interactive Learn track per journey, chapter merge table and rules).

- Standard-library and built-in reference: `website/tools/generate-api-reference.mjs`
  renders all 103 catalog groups (minus frozen `std:ui` and the internal testing
  runtime) into 28 generated pages before every `dev`/`build`. Output is ignored
  by Git. The docs workflow now also triggers on `builtins.json` changes.
- First language chapter merged: `language/optional-values/` replaces
  borrowed-optional-fallback, optional-chaining, nullish-coalescing and
  nullish-assignment. Old routes redirect to chapter anchors. All ten examples ran
  with the current debug compiler with matching output; five rejection claims
  were checked with `tsrust check`.
- Link checker now validates redirect pages against their destination anchor
  (unit test added). Site build: 91 pages + redirects; link/anchor check passed.

Not done: remaining nine chapter merges, the first Learn track, folding library
guides and backend how-tos into tracks, and the CI/installation/platform/deployment
reconciliation listed above. Nothing was pushed; no GitHub run was triggered.

## Distribution, CI and deployment reconciliation — 2026-10-28 (rows updated 2026-10-29)

Checked against workflow source and read-only `gh` queries; no run was triggered.

| Requirement | Current state | Evidence |
| --- | --- | --- |
| Release artifacts for macOS, Linux, Windows | All four targets built and smoke-tested; `v0.1.0-rc.1` published as a prerelease (updated 2026-10-29). | Release run 40861023720, Promote run 40864132322 |
| Windows paired install, locked binaries, shared caches | Passed on hosted Windows. An earlier failure (40724567590, 90382caa) was fixed by 9b658857. | Runs 40731485167 (9b658857), 40748893795 (86435b88) |
| Async testing on three platforms | Passed. | Run 40672007920 (c5b3d4a9) |
| PR CI on the staging branch | Linux compiler/LSP, VS Code extension and docs build passed for PR head 61ee67a4. | Runs 40761578381, 40761578389 |
| macOS | Local focused acceptance only (see `docs/agent-goal.md`). | Local evidence |
| GitHub Pages | Deployed at https://forever-frameworks-llc.github.io/tsrust/ (updated 2026-10-29). | Documentation run 40864208241 |
| Install/self-update from a published release | rc.1 archive installed by hand on macOS arm64 (version, doctor, native run). Managed install and self-update need the first stable release: pending `v0.1.0`. | See the release-candidate status section |
| Windows editor UTF-16 fixture | Passed on Linux and Windows (updated 2026-10-29). | Editor acceptance run 40816862762 |

Remote branch `codex/backend-polish-staging` (PR #4, draft, base `stag`) is still at
61ee67a4. Local history is not a descendant of it (the PR branch was rebuilt
from the same trees), so compare trees before any push. All local commits since
then, including this documentation restructure, are unpushed.

## Release candidate status — 2026-10-29

- Editor acceptance passed on Linux and Windows (CI run 40816862762), including
  the surrogate-pair UTF-16 diagnostic fixture.
- The five package-export tests pass locally once `tsrust setup --portable
  --offline --yes` has prepared source packs.
- Owner chose a pre-release. Version is `0.1.0-rc.1`. The toolchain installer
  now accepts `MAJOR.MINOR.PATCH-rc.N` (ordered before the stable version);
  update discovery still considers only stable releases, so self-update from a
  published release still needs a stable release to verify.
- PR #4 (`codex/backend-polish-staging` → `stag`) was merged by the owner with
  the review bypass (squash commit cf4e729c). Authors cannot approve their own
  PRs; the bypass is the owner's path for agent-opened PRs.
- PR #5 (`stag` → `main`) built all four targets and the extension (Release run
  40861023720). Promote (run 40864132322) merged it and published the
  `v0.1.0-rc.1` prerelease with four archives, `tsrust-0.2.0.vsix` and
  `SHA256SUMS`. The Documentation workflow (run 40864208241) deployed
  https://forever-frameworks-llc.github.io/tsrust/.
- Release smoke test on macOS arm64: the archive matched `SHA256SUMS`;
  `tsrust --version` printed `0.1.0-rc.1`, `tsrust-lsp --version` printed
  `0.2.0`; `doctor` and `doctor --native` passed; `tsrust run` executed a native
  program correctly. `tsrust toolchain install --root` refused as designed: "No
  published stable official toolchain is available yet".
- Pages smoke test: the home page, Learn, Language, globals guide, generated
  globals and `std:sql` reference returned 200; the old
  `reference/builtins/globals/` path redirects to `reference/globals/functions/`.
- Still open: managed install and self-update need the first stable release.
  Linux, Windows and Intel macOS archives were built and checked in CI, not
  installed by hand. This handoff commit is local only and reaches `stag` with
  the next PR.

## 0.1.0 milestone — 2026-10-29

Owner direction: no rc.2; finish the work, then publish `0.1.0`; run the
performance milestone and put the measurements in the docs.

- **Legacy UI out of the release CLI.** `tsrust ui`, `new --ui`, `dev --target
  web` and the UI menu are behind the `legacy-ui` Cargo feature (off in release
  builds). The bundled component catalog and `shadcn-MIT.txt` are not in the
  archive. Compiler `std:ui` support and the `tsrust-ui` crate remain, frozen.
  UI CLI tests run only with `--features legacy-ui`.
- **Docs inventory finished.** New chapters `language/operators` and
  `language/errors`; numbers, enums and recursive records in `language/types`;
  `for...in` and labels in `language/control-flow`; diagnostic suppression in
  `tools/cli`. 24 retained guides merged or deleted; research and developer
  guides labeled. See `docs/plans/documentation-architecture.md`.
- **Defects fixed on the way** (each with a regression test): formatter spacing
  for `c ? (): T => a : b`; misleading diagnostics for non-boolean `||=` on an
  array element and for rebinding a borrowed-view `let`; `setup --portable`
  failing on a damaged cached pack (now rebuilt and replaced); benchmark
  harness hiding prepared source packs behind its isolated Cargo home; three
  benchmark policies rejecting formatting-only source changes (now use
  registered token-identical proofs).
- **Performance milestone.** All 930 workloads measured native vs Node after
  tsc (3 × 31 samples, 175,620 samples), 880 profiled. Native faster on 613
  (median 1.52×). Median native change vs earlier measurements 1.00×; no
  regression beyond noise. Evidence: `benchmarks/storage/release-0.1.0-analytics/`.
  The public page `project/performance` is generated from
  `benchmarks/measurements.sqlite` at docs build time; publish a different run by
  editing `website/tools/performance-runs.json`.
- **Optimization targets** (native slower than Node): JSON/schema decoding,
  record/object copies, Unicode case conversion, `String.at`, recursive array
  copies, enum and interface records.
- **Known pre-existing issues, not fixed:** `store.py verify` rejects old
  absolute-path log records already in `records.sqlite`; about 55 benchmark
  harness unit tests are stale (hard-coded `target/editor/debug/tsrust`, frozen
  manifest-prefix hashes broken by fingerprint refreshes). Neither affects
  measurement.

## v0.1.0 published — 2026-10-29

- PR #6 (owner bypass merge) into `stag`; PR #7 `stag` → `main` built all four
  targets and the extension (Release run 40930352738); Promote (run
  40934384351) merged it and published `v0.1.0` as a regular release; the
  Documentation workflow redeployed Pages (performance page live).
- Archive check (macOS arm64): checksum OK; attributions hold only the Tsrust
  license and dependency notices (no UI notice); help has no `tsrust ui`;
  `tsrust 0.1.0` / `tsrust-lsp 0.2.0`; `doctor` OK.
- Managed install against the published release: `toolchain install --root`
  selected 0.1.0 and its LSP; `check` reports no newer release; `update` changes
  nothing; `rollback` refuses (no previous version); `doctor` reports a managed,
  intact pair. A console project runs and tests, and a backend project checks.
- Still unverified: updating across two real releases (needs a second release),
  and hand installation on Linux, Windows and Intel macOS (CI smoke only).


Baseline: the 0.1.0 run in `benchmarks/measurements.sqlite`. Every "after" is a
quiet-machine run (runner load guard, limit = half the logical CPUs).

| Change | Result | Kept |
| --- | --- | --- |
| Runner waits for load < CPUs/2 before each timed process; records max load | Desktop apps alone keep load near 3–4 on this M3; a limit of 2 never starts | Yes |
| `String(stringEnum)` borrows the static member text; whole numbers below 2^53 format as integers | `enum_values` 6.65M → 17.3M ops/s (2.6×; 0.14× → 0.35× of Node); other number workloads within noise | Yes (cc2dc382) |
| Left-nested string `+` appends to the fresh left temporary (exact `reserve_exact` growth) | `string_add_chained_*` 23.2M → 22.8M, `string_concatenation`/`interfaces` unchanged: allocation, not intermediate copying, dominates short strings; V8 uses ropes | No (reverted) |
| Dead `const` array receivers of `concat()`/`toReversed()` are moved (single reference, same statement list, no closures) | concat-zero workloads 1.94–3.01× (record 0.43× → 1.04× of Node); moved `toReversed` 1.27–1.57×; others within noise | Yes |
| Receiver moves extended to `slice()` (plain `Vec` only) and `toSorted()`; closures only block locals they capture | Tested (75 suites); no benchmark workload has a dead `slice()`/`toSorted()` receiver, so unmeasured | Yes |
| String search: first-byte scan for haystacks ≤ 64 bytes (`indexOf`, `lastIndexOf`, `includes`) | `indexOf` 2.45–2.72×, `lastIndexOf` 1.05–1.74×, `includes` 1.26–1.32×, JSON string projection 12.3× (0.33× → 4.1× of Node). A plain byte-window scan was tried first: 0.43–0.63× on `includes`/default `lastIndexOf` (slice compare at every position), so it was replaced | Yes |
| mimalloc as global allocator (owner approved trying it; then paused) | Schema pipeline 1.85× with mimalloc, `record_mutation` 0.91×. Profiles showed time inside macOS `free()`, but this Mac has 8 GB RAM with ~7.4 GB swap in use and ~3 GB compressed, which inflates allocator cost. Not adopted: a dependency in every program needs evidence from a clean machine. Dependency changes reverted; the harness keeps an experimental `--allocator-rlib` option | No |
| Dynamic `JSON.parse` re-scans integer tokens only when serde produced a float (only an integer too large for i64/u64 arrives that way) | JSON workloads up to 1.32× (`string_reads`, `refinement`, `typeof`, `scalars`); 54 JSON/schema workloads median 1.00×; `native_json_raw` 0.84× re-measured alone at 0.99× and does not use the parser | Yes |
| Typed JSON decode: validation pass records floats; integer re-scan only after one | 54 JSON/schema workloads median 1.04× (typed JSONC 1.18×, roundtrip 1.15×, object strip 1.15×); lowest 0.97× (noise). The validation pass itself stays: it keeps malformed input from reaching user callbacks | Yes |
| `JSON.parse` of a string skips UTF-8 re-validation (`parse_valid_utf8`); JSONC and byte inputs keep the check | vs the previous run: 26 JSON workloads median 1.01×, `typeof` 1.29×, scalars 1.18×; lowest 0.98× (noise). Inputs are tiny, so larger documents gain more | Yes |
| `JSON.parse`/`tryParse`/`std:json` `parse` without limits = plain serde_json (owner direction: TypeScript-familiar API, Rust accuracy). No default limits (HTTP `bodyLimit` and serde's 128-level nesting remain); over-range integers still reject; Rust number formatting kept (a JavaScript-style formatter was tried and reverted as a JS quirk). Also fixed a CLI panic when a diagnostic points past the user file | Speed roughly neutral: median 0.97× vs the previous run, 1.05× vs 0.1.0; drops of 0.82–0.84× are in workloads that do not call `JSON.parse`, so run-to-run noise here is about ±15%. Generated/hand-written for `int_bounds` 0.39× → 0.43×. First version regressed `typeof` to 0.33× by allocating in the float check; fixed | Yes |
| Parity engine `rust-hand` (hand-written Rust in `benchmarks/runtime/parity/`) | `native_schema_int_bounds`: generated 0.34× → 0.40× of hand-written. Per iteration: generated 2 allocations vs 1; the rest is parse overhead (UTF-8 re-check, budgeted seed, typed decode parses twice) and issue conversion | Tooling |
| `dynamic_unicode_uppercase` investigated | Node reaches ~3 ns per 20-char Unicode `toUpperCase`, i.e. it reuses the loop-invariant result; native does the real conversion (~75 ns, std ASCII fast path already). Benchmark-design artifact, not a conversion gap | n/a |

Measurement caveat: every local number, including the 0.1.0 baseline, comes from a memory-starved Mac (8 GB RAM, heavy swap). Code-level wins (fewer copies, faster search) reduce work on any machine, but magnitudes need confirming on a clean runner before publishing or adopting dependencies.
| Typed `JSON.parse<T>`/`tryParse<T>` without limits: one serde decode plus the integer-token scan (`typed::decode_plain`), no validation pass. Explicit limits, JSONC and JSONL keep the budgeted path; schema decoding is unchanged | Median duration vs the previous run: `native_json_typed` 1.38×, `native_json_literal_union` 1.35×; Node.js controls 1.01× (comparable conditions). Native ÷ Node.js: typed 0.71× → 0.96×, literal union 0.64× → 0.85×. Load ≤ 4 throughout | Yes |
| Schema rejection path (`native_schema_int_bounds`): pooled the intermediate `Issues` buffer to drop one allocation and free per rejection | No wall-clock change at 40M iterations (3.23–3.58 s vs 3.19–3.37 s): the thread-local pool costs about what the `free` did. A profile shows ~85% of the loop is the rejection path (issue buffer, `error.issues` array, record conversion); the array allocation is required by the API. Generated 79–85 ns/iteration vs hand-written 34 ns. Not adopted | No |
| `text + String(n)` (int, number or numeric enum) writes the digits into the concatenation buffer instead of formatting a temporary string; template interpolations of int and whole numbers append digits directly (`tsr_push_int`, `tsr_push_whole_number`; other numbers keep their existing spelling) | 18 affected workloads, stashed before vs after, back to back, load ≤ 3.9: median 1.07×, none slower; `enum_values` 1.62× (0.33× → 0.54× of Node.js), `array_from_map_string` 1.54× (0.51× → 0.79×), `interfaces` 1.25×, `function_expressions` 1.23× (0.73× → 0.92×). Node.js controls 0.87–1.06× | Yes |
| String-addition chains build one buffer (`6f474114`); int/number/boolean chains emit one running value (`d3f85d41`) | 27 workloads, `3790b8e0` vs `423a8dbd` back to back, load ≤ 4: median 1.04×, none slower; `interfaces` 1.84× (0.21× → 0.39× of Node.js), `map_iteration` 1.76× (0.30× → 0.52×), `string_iteration` 1.41×, `map_for_each` 1.28×, `object_values_entries` 1.23×. Arithmetic and boolean chains 1.00–1.07× (LLVM already folded the nested form; the flat form exists for depth, not speed). Node.js controls 0.95–1.13× | Yes |
| `String(x)` of a dead `const` string local moves it instead of cloning (`92600e26`; receiver-move rule) | 8 record workloads, `7d7b97d6` vs `92600e26`, load ≤ 3.9: `type_aliases` 1.13× (0.19× → 0.23× of Node.js); the other seven do not use the pattern and stay within noise (0.94–1.01×, while Node.js controls drifted 0.88–0.96×, so the second half ran on a slightly slower machine) | Yes |
| In-place `text += value` for uncaptured locals (`4855dbee`, `tsr_string_append`: borrowed → one exact allocation, owned → `push_str`) | Long accumulation (20,000 appends of 3 bytes, ×20): 0.02 s vs 0.20 s CPU for the copying form (10×, and the gap grows with length: linear vs quadratic). Short single appends on the 14 `+=` workloads, `28f66cdb` vs `4855dbee`: 0.88–1.02× (`compound_string_add_ascii` 0.94×, `declaration_lists` 0.94×), within noise or a few percent from amortized capacity retained in output arrays, the same as hand-written `push_str` | Yes (kept for the algorithmic win; parity with Rust `push_str`) |
| `text += a + b` (and longer right-side chains) appends each operand in place after one reserve (`500c09a2`, `tsr_string_reserve`) | 15 workloads, `d6fae277` vs `500c09a2`, load ≤ 4: `labeled_control` 2.37× (0.30× → 0.71× of Node.js), `for_pattern_initializers` 1.19× (0.19× → 0.25×), `in_operator` 1.09×; others 0.94–1.04× (the `compound_string_add_*` single appends are unaffected by this change). Node.js controls 0.91–1.08× | Yes |
| Appended `name.slice(...)` of a local borrows its range (`0800c571`, `tsr_slice_str`); new hand-written parity program `parity/declaration_lists.rs` | `1c652680` vs `0800c571`, load ≤ 3.9: `declaration_lists` 1.24× (0.18× → 0.22× of Node.js; generated/hand-written 0.38× → 0.46×); five other slice workloads 0.98–1.03× (no appended slices). Hand-written Rust itself is 0.53× of Node.js on this workload: V8 shares strings, so `slice()`/`repeat(1)` are free there. Remaining generated overhead: the `words` array allocated only to be destructured, `repeat(1)` and `second.slice()` copies of borrowed elements | Yes |
| Map/Set keys hashed once with a per-process random key; bucket tables use that hash directly (`27bb63d4`) | 11 workloads, `a3959610` vs `27bb63d4`, load ≤ 4: `map_values` 1.23× (0.39× → 0.48× of Node.js), `set_values` 1.14× (0.40× → 0.49×), `set_for_each` 1.13×, `map_for_each` 1.11×, `native_schema_collections` 1.06×; `map_iteration` 1.12× matched by its Node.js control (noise); unrelated controls 0.98–1.04× | Yes |
| Maps and Sets with at most 8 slots scan their slots instead of hashing; the 9th slot builds the index once (`f43235a6`) | 13 collection workloads, `647bd09f` vs `f43235a6`, load ≤ 3.9: 1.17–1.60×, none slower. `set_algebra` 1.60× (0.54× → 0.88× of Node.js), `map_values` 1.57× (0.45× → 0.73×), `set_for_each` 1.47×, `set_relations` 1.44× (0.73× → 1.03×), `set_entries` 1.40× (0.99× → 1.43×), `map_for_each` 1.35×, `array_from_collections` 1.27×, `map_iteration` 1.22×, `map_group_by` 1.21×, `map_snapshots` 1.17×, `native_schema_collections` 1.17×. Node.js controls 0.93–1.11× | Yes |
| `String(boolean)` borrows "true"/"false"; `Object.hasOwn` recognizes canonical index keys without formatting the index (`9ca1e25b`) | 10 workloads, `3aad90d0` vs `9ca1e25b`, load ≤ 4: `object_has_own` 3.16× (0.31× → 1.02× of Node.js), `array_splice_boolean` 2.13× (0.59× → 1.32×), `map_values` 1.20× (0.74× → 0.87×); others 0.99–1.03×. `array_splice_int` first read 0.86× but re-timed alone at 0.98× (noise; it does not use either path) | Yes |
| One-character ASCII strings borrow a static 128-byte table (`charAt`, `split("")`, `for…of` text, spreads, `Set(text)`); `text.at(i)` selects one character instead of `text.split("").at(i)` (`c35e14da`) | 8 workloads, `85912b00` vs `c35e14da`, load ≤ 4: `string_at_characters` 30.1× (0.13× → 3.71× of Node.js), `array_from_string_characters` 2.12× (0.31× → 0.65×), `string_iteration` 1.71× (0.25× → 0.44×), `string_split_empty_delimiter` 1.13×; controls 0.98–1.01×, Node.js controls 0.98–1.02× | Yes |
| Small-string optimization: runtime `String` as `Borrowed`/`Owned`/`Inline(≤15 bytes)` (24 bytes, like `Cow`), inline clones/slices/padding/`+=`, chains and templates through a sink (branch `codex/small-string-experiment`, `53c111b2`…) | 20 workloads vs `85912b00`: `string_at_characters` 2.03×, `array_from_string_characters` 1.57×, splits 1.03–1.20×, but `string_add_chained_*` 0.53–0.56×, `map_iteration` 0.75×, `enum_values` 0.81×, `interfaces` 0.85×, `string_iteration`/`string_concatenation` 0.87–0.88×. Generated code may not contain `unsafe`, so inline text is re-validated as UTF-8 whenever it is read as `&str`; standalone, the chain is 0.66 ms vs 0.39 ms and an unchecked variant still only reached ~0.55×. The character wins were taken by the static-table change above instead | No (branch kept) |
| Schema collections: duplicate keys of ≤8 entries compared pairwise without a HashMap; leaves whose int/number/boolean output equals the input restore through the capture-free identity closure (`283f61cb`) | 12 schema/JSON workloads, `c35e14da` vs `283f61cb`: `native_schema_collections` 1.26× (0.25× → 0.33× of Node.js; 59 → 48 allocations per iteration, `map.safeParse` of one entry 12 → 9). Others 0.90–1.03× with Node.js controls 0.92–1.05×; `native_json_containers`/`native_schema_json_array` re-timed alone at 1.01×/1.00×. Remaining schema allocations are structural: each trial layer (sequence, pair, remap, inject) boxes a `dyn FnOnce` restoration capturing the previous one, and entry/value Vecs are rebuilt between layers; generic restorations are the next fix | Yes |
| `String(int)` formats with `i64::to_string` instead of building a `TsrConsoleValue` and calling `label()` (`2f29f4e3`) | 8 workloads vs `85912b00`: `array_splice_int` 1.18× (0.56× → 0.66× of Node.js), `type_aliases` 1.15×, `labeled_control` 1.10×, `declaration_lists` 1.09× (gen/hand 0.46 → ~0.50), `interfaces` 1.04×, `enum_values` 1.00×; `string_concatenation` 1.07× and `map_iteration` 1.03× had Node.js controls at 0.81×/0.93× so are not conclusive | Yes |
| `String(number)` tries whole-number digits before the number formatter (tried as `cce967c1`, dropped) | 6 workloads vs `8afa1e58`: `array_pop_number` 0.93×, `array_shift_number`/`nullable_number_null` 0.96×, others 0.98–1.01× (Node.js controls 1.00–1.03×). Whole numbers allocate once either way and mixed inputs (0.5, NaN, ±Infinity) pay for the extra check | No |
| `String(record.field)` moves the field out of a `const` record local that is read only there (`52fef7cb`) | 4 workloads vs `2f29f4e3`: `interfaces` 1.15× (0.41× → 0.47× of Node.js; it now allocates what hand-written Rust would), `type_aliases` 1.01×, `declaration_lists` 1.00×, `labeled_control` 0.99× (Node.js controls 0.93–1.00×) | Yes |

Allocation profile (2026-10-30, `profile-native.mjs`): `native_schema_recursive`
47 allocations per iteration, `native_schema_export` 25, `native_schema_pipeline`
13, `interfaces` 8, `native_schema_string` 3, `enum_values` 2. Schema paths
build a dynamic `JSON.parse` tree and then copy it into typed output; fusing
`schema.safeParse(JSON.parse(text))` into one typed decode must keep the throw
for malformed JSON. Records now allocate close to hand-written Rust; the rest of
the gap to Node.js is short strings, which V8 keeps inline. A safe
small-string representation was tried on 2026-10-30 and rejected (see the
optimization log): without `unsafe`, reading inline text re-validates UTF-8 and
concatenation-heavy workloads lost up to 47%. Remaining short-string wins should
come from borrowing (static tables, slices of stable receivers) and avoiding
intermediate strings, as the one-character table did.

Schema text decoding (2026-10-30): `schema.safeParseJson(text)` measured about
1.25× slower than `schema.safeParse(JSON.parse(text))` (300,000 nested-object
decodes: 0.27 s vs 0.22 s CPU), so fusing the latter into the former would be a
regression. `source::document` validates the whole document, then each cursor
re-parses its subtree with serde when a field is read. Skipping the integer
re-scan without floats changed nothing measurable. The fix is a single-pass
schema `DeserializeSeed` decoder (as typed `JSON.parse<T>` already uses), which
replaces the cursor emission in `stdlib/schema`.

Next targets: add more `parity/` programs; add workloads with dead `slice()`/`toSorted()` receivers; `with()`/`toSpliced()` moves (need element assignment across array wrappers); JSON/schema decoding (group median 0.78×); record construction (records group 0.51×); `String.at`.

## Measurement store — 2026-10-30

Owner direction: canonical records are not kept. `benchmarks/measurements.sqlite`
is one table, `measurements(workload, engine, history)`, with a JSON history of
`{at, ms, ops, samples}` per run (commit `chore(benchmarks): one row per measured
workload…`). After timing, run `npm run store:ingest` in `benchmarks/runtime`;
`npm run checkpoint` prints the latest native/Node.js ratios. The ignored
`benchmarks/records.sqlite` (596 MB) was deleted, and the old analytics/export
scripts were removed. Raw files in `benchmarks/runtime/results` are disposable
once ingested; the 128 existing files (881 MB) were ingested and deleted on
2026-10-30 at the owner's request. Regenerating `BENCHMARKS.md` (`npm run
report`) now only sees result files written after that.

## Nesting limits removed — 2026-10-30

Owner direction (2026-10-29): remove limits like the 128-level nesting limit.

- `d3f85d41`: chains of `+ - * / % && ||` are flat in the checker (shared type
  hints during descent, linear time) and in emitted Rust (one buffer or one
  running value); generated programs allow `unused_parens`.
- Discovery (`benchmarks/compiler/nesting-discovery/run.py --unbounded-parser
  --breadth-first --start 256 --factor 4`, 60 s/case) over all 80 families found
  no crash. With the parser bound lifted, families complete to 4,096–1,048,576
  levels before the case budget. Evidence JSON is session scratch, not imported.
- rustc's own stack was the remaining ceiling (SIGBUS near 1,000 nested
  blocks). Tsrust now runs rustc/cargo with `RUST_MIN_STACK=1073741824` unless
  the user set it (virtual reservation); 16,384 nested blocks then compile.
  Exported packages document the variable instead of setting Cargo `[env]`,
  which would also change thread stacks under `cargo run`.
- `NestingPolicy::CONSUMERS` is unbounded: expression, block, conditional,
  match, template, type annotation, JSX and destructuring nesting have no limit.
- The CLI and the LSP analysis worker run the compiler on a
  `COMPILER_STACK_BYTES` (1 GiB, virtual) thread, so passes that still recurse
  (for example `in`, checked by synchronous `object_has_own`) reach tens of
  thousands of levels. Tests use `support::check_deep`/`accept_deep`.
- Owner direction (2026-10-30): "match the rust defaults, if rust doesnt have
  limits then we shouldnt either". rustc has no count limit on
  monomorphizations, type parameters, type arguments or record nesting; its
  only default is `recursion_limit = 128`, which bounds polymorphic
  instantiation depth and nested-type trait solving. Removed: callback origin
  specialization counts, per-target callback instance counts, generic
  specialization counts (128/declaration, 1,024/program), closure environment
  complexity, direct record layout depth, explicit type argument and type
  parameter counts. Inline record and callable storage now fail only past
  Rust's maximum object size (`isize::MAX`). Generic instantiation instead
  fails when an argument nests deeper than 128 (`INSTANTIATION_DEPTH_LIMIT`),
  which is how polymorphic recursion (`f<T>` calling `f<Box<T>>`) still gets a
  diagnostic, as in rustc. Callback specializations get the same bound:
  closures inherit the instantiation depth of the body that creates them
  (`specialization_depths`/`closure_depths`), so wrapping the callback in a
  new closure on each recursive call fails at depth 128 instead of growing
  without end (it reached ~30 GB once before the guard existed). Kept,
  matching `recursion_limit`: array rank 128, callable result discovery
  depth 128. Commit `0be17d66`.
- Measured follow-up: `scoped-map-catch` check time is cubic in depth (debug
  compiler: 32 → 0.42 s, 64 → 2.9 s, 80 → 5.5 s) while plain `catch-regions`
  and `cleanup-regions` stay at 0.02 s, so the cost is callback-holding Map
  storage across catch regions in analysis (`map_exception_storage_types`,
  `scoped_map_carriers`, `map_storage_edges` are the candidates). Profiling is
  profiled (Developer Mode enabled 2026-10-30): `finish_block` →
  `record_sequence_carriers` runs every carrier pass over the block's whole
  subtree each time any block finishes, and `scoped_map_entry_epochs` then
  filters every function binding through `contains_scoped_map_storage` for
  each statement (`native_inputs`). Work is blocks × statements × bindings.
  A scalar fast path changed nothing (the bindings are Map/record types). The
  fix is incremental: process each statement's epochs once instead of
  re-walking nested blocks from every enclosing block.
- Known failures before 0.1.0 (also in full run 40909244877, frozen UI):
  `retained_ui_catalog_entries_have_current_compiler_witnesses` and
  `retained_ui_catalog_examples_match_native_output` fail because the
  `UiComputed`/`UiSignal` catalog entries have no witness arm and no in-text
  example. Left as is under the backend-only reset; fix only if UI revives.
- Follow-up (performance, not limits): `scoped-map-catch`, `-cleanup`,
  `-catch-cleanup` time out at 256 levels and `scoped-map-mixed` at 1,024 in
  ownership analysis; `builtin-string-chains`, `captured-local-functions`,
  `field-*-cleanup`, `fresh-generic-adapters` time out at 4,096.
