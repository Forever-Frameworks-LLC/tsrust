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
improvements do not constitute incremental compilation: each build starts frontend checking afresh, and a
fresh source version compiles the whole generated native program.

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
