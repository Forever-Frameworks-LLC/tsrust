# Dev-loop performance milestone

Status: required, next implementation priority. Owner priority set 2026-10-11.
Complete this milestone before adding more S09 APIs or starting application workflows.
S09's completed evidence stays accepted; monotonic time resumes afterward. This is
part of integrated release acceptance, not an optional optimization backlog.

## Existing acceptance

Quiet default output, verbose phase timings, persistent validated native artifacts,
private executable copies, shorter supervisor waits and reuse of unchanged source
bytes are implemented. Last-good-program retention on failed builds is required.
At the recorded baseline the watcher scanned directories and metadata. These
improvements do not constitute incremental compilation: at that baseline each build started frontend checking afresh, and a
fresh source version compiled the whole generated native program.

## Required implementation order and exit evidence

1. **Filesystem events.** Replace routine full-tree polling with event-driven source
   and config watching. Handle atomic saves, renames, additions, deletions, new
   directories, rapid edit bursts and edits during a build. Recover from dropped or
   coalesced events through reconciliation; provide a tested, explicit fallback when
   native watching is unavailable. Preserve exclusions and symlink policy. Prove
   idle projects avoid periodic full-tree scans. Verify watcher cleanup on shutdown.
2. **Retained compiler state.** Keep the project graph and reusable frontend state
   alive between dev builds. Invalidate changed modules and affected dependents,
   including changed exports, removed imports and configuration changes. Compare
   diagnostics and native behavior with clean builds after edit sequences. Prove
   unrelated modules reuse state; artifact cache hits alone do not satisfy this step.
   Bound retained memory and recover from failed checks without stale diagnostics.
3. **Separate native runtime compilation.** Compile reusable runtime support
   separately from changing application code and link the required pieces. Preserve
   feature selection, native ownership/lifetime behavior, offline builds and portable
   toolchain invalidation. Prove ordinary application edits reuse runtime objects.
   Document remaining application recompilation boundaries explicitly. Do not weaken
   borrow checking or hide shared ownership to establish the boundary.
4. **Bounded caches and installed acceptance.** Define retention/eviction for old
   artifacts and runtime variants. Preserve active executable safety and concurrent
   project isolation. Verify cold start, warm restart and fresh edits through the
   installed CLI, retaining the last working program on failed or cancelled builds.

Use focused failing tests for each stage. Cover macOS and Linux watcher backends;
record which platforms actually execute acceptance tests and retain explicit limits.
Keep callbacks, default exports, recursive views and resource cleanup represented in
clean-versus-incremental equivalence checks as relevant code paths change.

## Measurement and closure

Before optimization, capture a reproducible baseline on small and larger generated
multi-module projects. Record project shape, source/toolchain identities and sample
counts. Measure cold startup, identical warm startup, leaf edit, shared-dependency
edit, configuration edit, idle CPU, retained memory and cache growth. Report stage
costs and edit-to-program-output latency with repeated samples and distribution
summaries; do not substitute identical-source cache-hit time for fresh-edit latency.

Preserve baseline and final records in SQLite and compare matching cases. Investigate
regressions, then refresh installed consumers and run lifecycle/equivalence checks.
At the completed milestone run the relevant full compiler/editor regression scope.
Implementation and measured acceptance are separate fields; neither is complete yet.
Do not close this milestone with any of the three architectural steps merely listed
as future work. Track unexpected blockers here and in the main roadmap; request an
owner decision only if the resolution materially changes the approved contract.

## Current next action

The initial baseline is captured using `benchmarks/dev/baseline.py`: three samples
each for 6-file and 130-file projects. Median leaf edits were 522.89 ms and
556.73 ms; shared-dependency edits were 508.98 ms and 537.04 ms. Idle supervisor
CPU was 0.02 seconds per three-second window for 6 files and 0.16–0.17 seconds
for 130 files. These are diagnostic samples, not final performance acceptance.
Exact samples, source hashes, phase logs and original failed fixture attempts are
stored as `tsrust-dev-baseline-*` in SQLite. Formatted generated projects also pass
idempotence and native output checks. The original long expression hit the existing
128-depth limit; sequential additions preserve the intended graph and arithmetic.

Complete filesystem-event loss/race acceptance, installed refresh and baseline
comparison next. Continue through
retained frontend state and separate runtime compilation before resuming S09 APIs.

## Filesystem notification implementation checkpoint

Native notifications now drive cached dev snapshots. macOS uses kqueue vnode
notifications for source files/config/directories; Linux uses recursive directory
inotify subscriptions. Directory subscriptions are installed before traversal.
Changes reconcile exact source bytes and refresh subscriptions, including replacement
inodes and newly discovered directories. Native errors visibly switch to polling;
`TSRUST_DEV_WATCH=poll` forces the fallback. Queue overflow on Linux requests full
reconciliation. macOS drains notification bursts in bounded batches. Owned
close-on-exec descriptors release subscriptions when the watcher is dropped.

Seven focused supervisor/watcher tests pass on macOS. They cover no idle directory
scans, same-size edits, restored mtime, atomic saves, new directories, deletions,
config recovery, excluded directories/symlinks, root-loss fallback and event bursts.
Five project CLI checks pass with native watching; both dev lifecycle checks also
pass with forced polling. The burst test first caught repeated rescans and retains
its original failure. Records use `tsrust-dev-events-*` in SQLite.

The CLI now has a pinned Unix-only libc 0.2.183 dependency, resolved offline from
the existing registry cache. Generated user programs gain no dependency from this
watcher. Linux code is not yet executed on this macOS host. macOS per-file descriptor
limits can force polling, which is documented. Installed refresh, matching baseline
comparison and further loss/race acceptance remain pending. This checkpoint does
not close the dev-loop milestone or its three required architectural steps.

A concurrent-save witness now performs 2,000 atomic replacements across 100 files
while snapshots reconcile. Final contents match an independent clean scan and
native subscriptions remain active. This macOS witness passes; it does not stand
in for Linux overflow execution. Evidence: `tsrust-dev-event-races.log` in SQLite.

## Native watcher comparison and installed acceptance

The unchanged dev-loop-v1 harness and exact generated source hashes were rerun with
three samples per shape after `c1431c38`. For 6 files, median leaf/shared edits were
530.16/512.35 ms (baseline 522.89/508.98 ms); for 130 files, 554.31/535.10 ms
(baseline 556.73/537.04 ms). This change does not establish a fresh-edit speedup.
Idle CPU fell from 0.02 seconds to 0.00 at ps resolution for 6 files, and from
0.16–0.17 to 0.00–0.01 per three-second window for 130 files. Zero is a measurement
resolution limit, not proof of no CPU work.

Median 130-file cold/warm startup increased from 372.52/109.35 ms to
402.40/122.76 ms. Subscription setup at that checkpoint included a redundant initial refresh; review this overhead before closing watcher performance
acceptance. Three samples do not establish statistical significance. Original
baseline, individual comparison samples and derived summaries remain in SQLite as
`tsrust-dev-baseline-fixed.json`, `tsrust-dev-events-measured.json` and
`tsrust-dev-events-comparison.json`.

Four installed/workspace CLI copies were atomically refreshed and hash verified
from `c1431c38`. Installed startup, atomic-save rebuild, exact output and clean
shutdown pass without polling fallback. The LSP is unchanged and remains at its
accepted environment checkpoint. Evidence uses `tsrust-dev-events-installed*`.
Linux execution/overflow evidence, startup overhead review and the remaining
retained-state/runtime-compilation steps still prevent milestone closure.

The redundant initial subscription traversal is removed. A focused cost witness
failed with two walks and passes with one; real notifications and failed reads
still refresh subscriptions. All eight watcher/supervisor checks pass, including
concurrent saves. Evidence: `tsrust-dev-startup-*`; comparative timing follows.

The unchanged comparison protocol was rerun after `810b225e`, with identical source
hashes and three samples per shape. For 6/130 files, cold medians were 426.17/444.88
ms and warm medians 115.81/131.64 ms; these do not demonstrate a wall-clock gain
from removing the extra walk. Leaf medians were 528.56/597.42 ms; shared edits
496.51/534.24 ms. Phase timings varied too (6-file cold native/link median rose
from 231.1 to 269.2 ms). Keep all original runs; do not select only favorable cases.
Idle CPU remains 0.00–0.01 seconds per three-second window at ps resolution. The
cost witness proves the redundant walk was removed, not an end-to-end speedup.
Records use `tsrust-dev-startup-measured.json` and `tsrust-dev-startup-comparison.json`.

Four CLI consumers are refreshed/hash verified from `810b225e`; installed startup,
source edit, native notification operation and clean shutdown pass. Retained
compiler state is the next implementation step. Startup distributions remain part
of final milestone review rather than prompting repeated standalone retiming.
Linux execution/overflow validation remains explicit outstanding acceptance.

## Retained frontend state: syntax layer

`ProjectSession` now retains pre-link module syntax. Cache keys include exact source
text, module position, test-entry mode and dependency generations. Any dependency
change conservatively invalidates importers. Unrelated modules reuse syntax;
unreachable entries are evicted, and failed graph loads clear retained entries.
Linking operates on cloned syntax with fresh identities/source offsets; whole-program
ownership/type checking still runs. No checked arena or user-value lifetime is reused.

Three session tests compare fresh and retained diagnostics/emission across unchanged
builds, leaf edits, type changes, default imports, semantic errors and recovery,
module removal, recursive views, file cleanup and test registration. Two existing
default-export native tests also pass. Original missing-API failure and passing
records are stored as `tsrust-project-session-*` in SQLite.

This is a frontend layer, not completed incremental dev compilation. The CLI does not yet keep a ProjectSession alive across dev builds. Next integrate
a persistent dev build worker, preserve cancellation/last-good behavior, and prove
installed reuse. No
speedup or full retained-state acceptance is claimed from these tests alone.


## Retained frontend state: module headers

The session now also retains strict unresolved module headers keyed by exact source
identity and text. Every build resolves imports and validates exports against current
inputs; cached headers never preserve resolved dependency edges. Removed modules
are evicted, and graph failures clear both header and syntax caches.

Four session checks pass, including changed symlink resolution, missing dependencies,
recovery, unchanged-module reuse and the existing clean-build comparisons. Three
focused loader checks also pass for diamond graphs, invalid dependencies and tolerant
editor headers. Evidence is stored as `tsrust-project-headers-*` in SQLite. This
remains a library layer: persistent CLI worker integration, installed reuse and
performance acceptance are still outstanding.


## Persistent dev frontend worker

Dev now owns one frontend thread and reuses its ProjectSession between builds.
A bounded channel permits at most one queued request in addition to active work.
Cancelled requests lose their response receiver; completed stale results cannot
replace the running program. Native compilation remains a supervised child process
group, cancelled on changes or shutdown. Diagnostics are checked against the current
watcher snapshot before display. Type/ownership checking and Rust emission remain
fresh. Single-file checking keeps its existing semantics and clears module caches.

Five project CLI checks, five session checks and eight watcher/supervisor checks
pass. The lifecycle witness first
failed without reuse, then proves an unrelated module reuses syntax and two unchanged
modules reuse headers after edits. It also covers failure recovery, keeping the last
working process, and shutdown. Entry changes and switching to single-file source
match clean checking. Records use `tsrust-dev-session-*` in SQLite.

This checkpoint is not installed or measured acceptance. Frontend parsing itself is
not preempted; obsolete responses are discarded and queued work is bounded. Next
verify cancellation during builds and immutable source snapshots, then refresh
consumers and measure the completed retained-state stage. Separate native runtime
compilation, artifact eviction and Linux watcher execution remain required.

## Dev build cancellation acceptance

A blocked-backend witness exposed that the retained-worker integration had moved
backend version probing onto the supervisor thread without cancellation. Version
probing now runs in a supervised process group, using the same bounded polling and
cleanup as native compilation. The ordinary CLI and dev share backend version
validation, preserving the pinned-toolchain contract.

Two focused subprocess witnesses pass: interrupting a blocked version probe reaps
its wrapper and descendant, and editing during a blocked native build reaps the
obsolete compiler while preserving the last working program until the newest
executable succeeds. The stale candidate never runs. Five project CLI checks also
pass. Original failure and passing evidence use `tsrust-dev-cancel-*` in SQLite.
Immutable source snapshot acceptance remains next; installed refresh and performance
measurement are still pending for the retained-state stage.

## Captured dev inputs

Each frontend request now owns shared immutable watcher bytes. Entry/module checking
and diagnostic excerpts use that provider; configuration is parsed from the same
captured input. A watched source absent from a request cannot be read later merely
because it appeared on disk during compilation. The request queue remains bounded,
and unchanged source buffers are shared rather than copied for each request.

The snapshot witness changes entry and dependency contents after capture and verifies
identical emitted Rust. It also verifies that errors quote the captured erroneous
line after the disk file is corrected, and rejects a source created after capture.
Nine binary unit checks, five session checks, two cancellation checks and five CLI
checks pass; the CLI checks pass again after switching config parsing to captured
bytes. Evidence uses `tsrust-dev-snapshot-*` in SQLite, including the initial test
fixture's missing-config repair.

The existing watch boundary remains: excluded paths and dependencies outside the
project watch tree use normal filesystem reads. Resolution still checks current
filesystem identities; deleted/renamed files can invalidate a pending check, whose
result is guarded against the latest watcher input before publication. These tests
do not establish an atomic filesystem snapshot or wider dependency watching.
Installed refresh and the matching retained-state measurements are next, followed
by separate runtime compilation and bounded native artifact retention.

## Installed retained-state comparison

Four CLI consumers were atomically refreshed and hash verified from `f1431c76`.
The unchanged dev-loop-v1 harness completed three samples each for 6 and 130 files
using the installed CLI. All output oracles, native watching, syntax reuse after leaf
edits, full reuse on config-only edits and clean shutdown checks pass. Compiler,
harness and source hashes are preserved in `tsrust-dev-retained-*` SQLite records.
The LSP is unchanged at its accepted environment checkpoint.

For 6/130 files, cold-start medians were 411.71/421.72 ms, warm starts 130.40/138.75
ms, leaf edits 544.46/594.08 ms, shared edits 529.37/566.78 ms and config edits
266.88/244.36 ms. The preceding watcher checkpoint had leaf medians 528.56/597.42
ms and shared medians 496.51/534.24 ms. This does not establish a fresh-edit speedup;
retain the original samples and do not select favorable cases. Median leaf native
compilation/linking costs were 241.2/263.3 ms; frontend checks were 1.8/12.6 ms.
The supervised backend probe also costs roughly 49–52 ms in these edit samples.

Idle supervisor RSS was 9,952–10,016 KiB for 6 files and 13,824–14,960 KiB for
130 files, versus roughly 2–3 MiB with a separate short-lived frontend process.
This expected retention cost is not peak process-tree memory or a leak test. Idle
CPU remains 0.00–0.01 seconds per three-second window at ps resolution. Long edit
sequences and memory/cache bounds still require milestone acceptance.

Continue with separate reusable native runtime compilation. Preserve backend
cancellation while reviewing probe overhead in the final performance pass. The
required milestone remains open for runtime separation, cache eviction, Linux
execution evidence and final regression/performance acceptance.

## Separate runtime: initial crate boundary

The compiler can now split selected byte, file and path support into a separate Rust
library source while retaining program-specific records, unions and borrow adapters
in application source. The emitter and splitter share exact bundled module builders;
standalone `emit` remains self-contained. Only modules already selected by emission
are extracted. This operation recognizes compiler-owned source, not arbitrary Rust.

A focused native witness compiles the runtime as an rlib and links the application
at both O0 and O3. Byte views, missing-file errors and path output match their
oracles. Rust rejects returning a borrowed view after its owner dies across the
crate boundary (E0515). Application-only edits change application emission while
leaving runtime source identical. A program requiring none of these modules remains
unchanged. Evidence is `tsrust-runtime-boundary-*` in SQLite, retaining the original
missing-API witness and corrected file-error oracle.

This is the initial boundary, not completed runtime separation. Next connect reusable
runtime artifacts to build/dev and extend the boundary to ordinary console/string
support, which the dev benchmark exercises. Preserve feature selection and generic
ownership rules; application-specific monomorphization may remain application work.
Installed reuse, bounded artifact retention and measured acceptance remain required.

## Runtime artifacts in build and dev

Build/run/test and dev now prepare separated support through a shared runtime build
adapter. Runtime artifacts use exact-source/backend/platform/optimization identities
and the existing checksum-verified atomic cache publication. Each build copies the
library into its private directory before linking. Program artifact identity advances
to ABI 2 so older whole-program artifacts cannot bypass the new compilation path.
Standalone emit remains unchanged; programs without extracted support need no rlib.

A CLI witness changes application source and verifies runtime miss then hit at both
O0 and O3, exact native outputs, and distinct runtime artifacts per optimization
profile. Dev witnesses separately interrupt application and runtime compilation;
both reap compiler descendants, preserve the last working process, reject obsolete
executables and reuse the runtime on the final build. Backend-probe interruption
and five existing project CLI checks also pass. Records use `tsrust-runtime-cache-*`
in SQLite, including the initial absent-reuse failure and corrected artifact count
(the cache also retains lock files).

This supports the initial byte/file/path boundary. Console/string support is next;
the simple dev-loop workload still has no extracted runtime. Runtime artifact and
lock-file retention are not yet bounded, and installed/performance acceptance for
this stage is pending. Do not close the required milestone on these focused checks.

## Console and string runtime boundary

Ordinary console and string support now join the separate runtime library. Console
formatting, stream state and test capture share one runtime module; generated
application calls import its public Rust helpers and location guard. String support
preserves the existing standard Cow string alias and borrowed/owned behavior. Only
Rust helper visibility changes; no language API or view lifetime changes. A small
number-format tie helper remains available to application-specific support code.

Eight linked native cases pass across O0/O3, covering plain string/console code and
the existing bytes/views, file and path cases. Application edits preserve runtime
source identity. Twelve console checks and five CLI reporter checks pass, including
stdout/stderr grouping and original source coordinates. Runtime-cache reuse across
application edits and all three build-cancellation checks also pass. Evidence uses
`tsrust-console-runtime-*` in SQLite.

The ordinary dev-loop workload now selects a reusable runtime artifact. Generic
application support and program-specific ABI adapters remain in application code.
Next bound runtime/program artifacts and lock retention, verify extended edit-session
memory, refresh installed consumers, and run final regression/performance acceptance.
No speedup is claimed before the matching measurements complete.

## Bounded active cache generation

Program and runtime caches now use a v2 namespace with one persistent advisory lock
per namespace. Publication, private copying and eviction coordinate on that lock.
Each namespace retains at most 32 artifacts and 256 MiB including identity/checksum
metadata. Access refreshes recency; eviction favors the artifact just published.
Artifacts larger than the byte limit still build but are not retained. Abandoned
staging directories are removed while holding the namespace lock. Running binaries
and libraries already copied into private build directories remain unaffected.

The generation boundary deliberately leaves older per-artifact lock inodes intact:
older compiler processes can still be waiting on them. Limits apply to the active
v2 generation, not legacy cache directories. Retire obsolete managed legacy outputs
at final consumer refresh; do not claim existing legacy disk usage is already capped.

Four retention tests, eight existing cache/CLI checks and the runtime-cache witness
pass. They cover 48 successive builds, oversized artifacts, aggregate byte limits,
96 concurrent publications with verified private copies, abandoned staging cleanup,
legacy preservation, multiprocess damaged-entry repair and native reuse/invalidation.
The subprocess worker remains intentionally ignored as a direct test but is executed
by its parent contention test. An older compile-count expectation was updated to
include separate runtime compilation; original failure is retained. Evidence uses
`tsrust-cache-retention-*` in SQLite.

Extended dev-session memory, installed refresh, Linux execution and final milestone
regression/performance review remain pending.

## Installed runtime reuse and extended-session measurements

Four CLI consumers are refreshed/hash verified from `0ac179f2`. The unchanged
six-sample dev-loop-v1 protocol passes output, runtime reuse, native watching and
shutdown checks. Matching source and harness hashes are verified against both the
original baseline and retained-frontend checkpoint. Records use
`tsrust-dev-runtime-*` in SQLite.

For 6/130 files, median leaf edits are 451.22/488.77 ms (preceding retained-frontend
544.46/594.08), and shared edits 432.17/458.14 ms (529.37/566.78). Cold medians
increase to 585.73/481.57 ms (411.71/421.72); the first small-project cold sample
was 1,280.0 ms and is retained. Warm medians are 138.85/123.97 ms. Config medians
are 280.46/356.65 ms (266.88/244.36); the larger shape's individual config samples
are 356.6, 250.4 and 360.0 ms. All hit the program cache; backend probe times were
59.5, 31.0 and 61.2 ms, so probe/supervisor scheduling remains a review target.
These three-sample distributions show improved edit latency in this run, not a
universal performance guarantee. Preserve cold/config regressions in final review.

A separate 48-edit-per-shape installed witness verifies all 96 output oracles and
runtime hits. Each final cache has 32 program artifacts, one runtime artifact and
one lock per namespace. Program caches occupy 36.5/41.4 MB and runtime caches 2.6 MB.
Small-project RSS ends at 12,672 KiB; large-project RSS at 16,624 KiB. During edits
24–48 the ranges are 12,672–12,704 and 16,608–16,624 KiB, within the witness's 8 MiB
stability allowance. This bounds the observed session, not all project sizes or
allocator behavior. The exact diagnostic script is retained in SQLite as
`tsrust-dev-session-witness.py` for reproduction.

Full compiler regressions have started; their acceptance is pending. Complete
editor regressions, remaining lifecycle validation, performance review and
obsolete-output cleanup before milestone closure.

## Linux notification acceptance and regression repairs

The installed Docker Desktop runner now executes the actual watcher and source
discovery modules on Linux aarch64, offline, using Rust 1.96.1. Eight tests pass,
including concurrent atomic saves, exclusions, idle behavior, root-loss fallback
and real inotify queue overflow. The overflow witness independently observes
`IN_Q_OVERFLOW` at the runner's 16,384-event queue limit. It first reproduced
repeated rescans after a burst; bounded draining (at most sixteen 64 KiB reads per
poll) repairs that defect. Larger queues and continuous writers can leave events
for the next poll, preserving bounded supervisor work. This is watcher-module
acceptance, not a full Linux CLI or compiler regression run. Original failure and
passing evidence are retained as `tsrust-linux-*` records in SQLite.

The full macOS compiler pass exposed a private console `label` helper used by
generated `String(value)` conversions after runtime separation. It also exposed
an isolated Rust test harness that omitted the console capture dependencies, and
an older backend-argument assertion that did not account for automatic linking.
The argument, helper and harness repairs pass their affected checks.
The original full pass is preserved and will not be restarted. Only affected
groups require reruns after repair.

The quiet dev-output witness also stopped a cold build after a two-second gap
between log lines. It now waits for its expected program output with a 30-second
overall deadline, matching other lifecycle checks; the separate measurement
protocol remains responsible for latency acceptance. This repair passes its focused rerun.

Both complete passes have finished. LSP acceptance reports 530 passed, zero
failed and one intentionally ignored measurement. The original compiler pass
reports 3,160 passed, 43 failed and three ignored tests across 276 suite summaries.
The compiler failures occupy 29 integration suites; those suites and the expanded
runtime-separation witness completed with 294 passing checks and one stale reporter
assertion. That assertion now verifies grouped failure output, the thrown message,
failure counts and cleanup output, and its focused rerun passes. Combined retained
and repaired results cover all 3,203 compiler tests, with three intentional ignores.
Repair records are `tsrust-dev-regression-repairs.log` and
`tsrust-dev-reporter-assertion-repair.log` in SQLite. Installed consumer refresh and
remaining dev-loop performance/lifecycle review are still pending. The exact full-pass records are
`tsrust-dev-milestone-compiler.log` and `tsrust-dev-milestone-lsp.log` in SQLite.

Runtime separation adds no required user flags: `tsrust build`, `run`, `test` and
`dev` select, cache and link support automatically. The Rust backend's `--extern`
argument is an internal implementation detail.

## Cache contention and dev cancellation

A focused lifecycle witness reproduced `dev` hanging on Ctrl+C while another
process held the program-cache namespace lock. Dev now attempts cache lookup and
publication without waiting, for both program and runtime artifacts. A busy cache
causes a private build and skips publication if still busy. Ordinary build/run/test
cache coordination is unchanged. This can duplicate compilation during contention,
but preserves the existing isolation and eviction lock while keeping dev supervision
responsive. No user option or configuration is required.

Seventeen focused cache, retention, runtime-reuse and cancellation checks pass;
the intentional subprocess-worker ignore is still exercised by its parent test.
A strengthened witness holds both cache locks through compilation, observable
program output and successful interruption. Original failure and passing evidence
are stored as `tsrust-dev-cache-contention-*` and `tsrust-dev-both-cache-locks.log`
in SQLite. These focused checks supplement the completed full regressions.
