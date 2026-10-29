# LSP memory investigation

Continuation update, 2026-10-26: the owner reprioritized audited benchmark data
integration and generated-output cleanup. Checker agreement was stopped at receiver
`Time`; it is not a full pass. Dataset validation and cleanup results are recorded
in [the current progress file](../agent-progress.md). No cache-repopulation build,
normal editor LSP or preview was restarted after cleanup.

Source baseline: `7c956bd7`; Windows investigation begun 2026-10-26. No normal
editor, preview or whole-workspace scan has been started by this investigation.

## Initial observations

The historical macOS report remains unresolved. This clone has no `target/`,
installed Tsrust consumers or `benchmarks/records.sqlite`. Historical Markdown
references are retained evidence descriptions, not locally verified database bytes.
Do not delete loose records until canonical storage is available and verified.

Windows reports 33,478,492 KiB total physical memory and 16,935,908 KiB available
at initial inspection. Node is v24.19.0. The pinned Rust 1.96.1 toolchain was absent;
rustup installed it during toolchain inspection. Stable 1.98.1 also exists but is
not selected for acceptance. Visual Studio 2022 Community is present; linker
availability remains to be demonstrated. Existing editor-owned Cargo metadata
processes hold the global package-cache lock; do not terminate unrelated processes.

## Bounds and stop conditions

Only one Cargo build owner uses `target/`, `CARGO_BUILD_JOBS=1` and
`CARGO_INCREMENTAL=0`. Stop the initial prerequisite build after 15 minutes without
completion, at 8 GiB private committed bytes across its process tree, or below
4 GiB available physical memory. A blocked package-cache wait is not a memory test.

Before launching a reproducer, use one standalone LSP process and a small synthetic
workspace. Limit it to 120 seconds, 1 GiB private committed bytes, 512 MiB working
set growth or less than 4 GiB available physical memory. Record working set and
private committed bytes; Windows private commit is not equivalent to observed
swap residency. Do not claim compressed/swapped footprint or macOS safety from
these counters. Record process count, edits, outstanding requests and completion.
Repeated edit/open/close, interrupted scan and dependency invalidation are required.
No full repository scan is authorized until these bounded probes support it.

## Source trace

`main.rs` uses one worker, one-slot job/result channels and one latest pending
analysis per open document. Workspace interruption is cooperative between entries;
an individual project check is synchronous. Deferred requests remain live until
completion or their three-second answer deadline.

`workspace_scan::Cache` retains one entry per discovered source. Each entry retains
a `WorkspaceSnapshot`, including every open buffer at scan time, even unrelated
buffers. Dependency freshness only observes accessed sources. Reusing an entry can
therefore keep unrelated obsolete buffer text alive. Entries covered by a configured
graph can remain in the cache after being superseded. These are concrete retention
hypotheses, not yet proof of the owner's full reported growth.

Foreground `ProjectSession` retains indices and actions per open project entry.
Closing an entry removes its index/actions; unrelated invalidated entries remain
until replacement/close. Trace these consumers before changing snapshot semantics.
Preserve absent-path/config invalidation, version observations and coordinate text.

## Acceptance status

Implementation is in progress. `WorkspaceSnapshot::retained` keeps observed open
buffers/revisions for cached scan entries and foreground indices/actions. Active
jobs still own all inputs. No cache-size cap or reduced checking mode is added.
A weak-reference witness checks release of unrelated old text while preserving
dependency close/reopen invalidation. Successful scans now remove independent
entries superseded by configured graphs; interrupted scans preserve completed work.
A focused formatted-project witness checks failed-to-successful graph retention.

Windows file URI conversion previously rejected ordinary drive URIs and encoded
canonical backslashes as URI authority text. This blocked a real local protocol
probe. The prerequisite repair handles drive and verbatim-disk identities and
keeps remote authorities rejected. A Windows-only round-trip witness and the real
session helper use the repaired conversion.

`tools/lsp-memory-probe.mjs` is a one-process, bounded Windows protocol diagnostic.
It opens 24 small imported entries, checks formatting stability and imported hover,
changes a separate 512 KiB comment buffer between entries, closes documents and
records working set/private commit/available physical memory. Its JSON is current
gate evidence under `target/`, not a benchmark or a historical footprint claim.
The initial probe passed 76 requests in 3,365 ms with no stderr and normal exit.
Its two samples reported 22,327,296/26,038,272 working-set bytes and
1,056,768/4,874,240 private committed bytes. This is a startup diagnostic, not
long-lived acceptance. The expanded probe repeats 12 open/edit/invalidate/close
cycles, with idle intervals and per-cycle samples. It supplements deterministic ownership and interruption
witnesses; it cannot alone prove long-lived macOS memory safety.

The expanded probe passed 890 requests in 61,849 ms with 43 samples, no stderr,
normal exit and zero stale/member consistency counters. Highest sampled working
set was 19,210,240 bytes; highest sampled private commit was 6,787,072 bytes.
Final sampled private commit was 2,899,968 bytes. Every sample stayed below the
recorded limits. Current evidence is `target/lsp-memory-probe.json`; initial
evidence is `target/lsp-memory-probe-initial.json`. The expanded probe overlapped
the affected agreement-test run. These are diagnostic observations, not isolated
performance measurements or an observed process peak.

The expanded probe's CLI SHA-256 is
`03f689fa1ebafcd1fad8cb08c420ea837fee80c9e8e6a8bbfffefd17c28f2b39`;
its LSP SHA-256 is
`9683938dd85c731eb1c7d95c68e9d55df24ad149d91222787fe8d27d8325e264`.
These are local consumers, not installed/active-editor refresh claims.

Focused successful commands include:

```text
cargo build --locked -p tsrust-lsp --bin tsrust-lsp -p tsrust-compiler --bin tsrust
cargo test --locked -p tsrust-lsp --lib retained_analysis_releases_unrelated_revisions_without_losing_dependency_guards -- --nocapture
target/debug/deps/tsrust_lsp-f2f2305c6f5ddb90.exe workspace_scan::retention_tests::repaired_configured_graph_releases_superseded_independent_entries --exact --nocapture
target/debug/deps/project_diagnostics-dcf29c2d78f38454.exe windows_local_file_uris_roundtrip_drive_and_canonical_paths --exact --nocapture
target/debug/deps/workspace_scan-088191edfb9b71a5.exe --test-threads=1
node tools/lsp-memory-probe.mjs
```

Build/test environment is `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=1`,
`CARGO_TARGET_DIR=D:/ai_dev/tsrust/target`; selected Rust is pinned 1.96.1.

`node --check tools/lsp-memory-probe.mjs` and `git diff --check` pass. The first
build failed for missing offline vendor sources; the locked fetch then passed.
The retry built the compiler library but exposed a Windows CLI compile error:
`docs_cli::matches_snapshot` referenced Unix-only `dev_watch::Inputs`. Its helper
and dedicated watcher test now share the Unix boundary. The subsequent CLI/LSP
build passed in 2m 28s; CLI `--version` reports `tsrust 0.1.0`.
An observed compiler
private-commit sample reached 2,362,220,544 bytes, below the 8 GiB build limit;
available physical memory stayed above 4 GiB at every sample. These are samples,
not a measured peak.

The cached-input weak-reference witness, superseded-entry witness and Windows
drive/canonical URI witness pass. All eight workspace scan tests pass, including
unsaved files, missing-import creation, cache reuse, interrupted scans and stale
publication guards. Initial scan failures exposed Windows normal/verbatim prefix
differences for new files. Canonicalizing the existing ancestor fixes that identity
without losing lexical path observations. An intermediate helper incorrectly joined
an empty suffix onto existing file paths; that failed run is retained, and the
helper now returns existing canonical paths directly.

The first editor-suite invocation stopped at two checker-agreement failures caused
by Unix-only virtual fixture URIs. Those fixtures now use host-absolute temporary
identities; assertions remain unchanged. The three required agreement suites are
running with `--no-fail-fast`, single test thread, and exact output in
`target/lsp-memory-editor-checks-current.log`; the earlier output remains in
`target/lsp-memory-editor-checks.log`.

A third Unix-only identity was subsequently found in the callable-field helper.
The CPU-active checker-agreement process was stopped before spending more time on
that known-invalid fixture. Its abnormal exit is retained in the current log.
The helper now uses the same host-absolute virtual URI path, with unchanged checked
contract comparisons. The no-fail-fast invocation continues the completion and
consistency suites. Rerun only checker_agreement with visible receiver progress;
the interrupted test is not accepted evidence.

The complete completion matrix finished in 575.82 seconds. Eight failures were
limited to the standard/project import-path rows in all four typing modes; the
remaining rows are retained. The module-inventory check also missed std:testing.
Two Windows URI assumptions in `position_completion.rs` caused those failures:
the compiler-confirmed module probe used a Unix virtual URI, and sibling discovery
stripped the URI prefix without decoding it. Both now use shared canonical local
path handling. An encoded-space/Unicode/percent directory witness passes. Rerunning
only import-path rows plus module inventory passes all three tests in 7.31 seconds;
the full affected project-diagnostics suite passes 17 tests in 0.81 seconds.
Exact repair evidence is `target/lsp-import-path-acceptance.log`.

Editor consistency passes all 27 tests in 39.66 seconds on the original complete
run. Checker agreement is the only required memory-section editor acceptance still
running; its repaired invocation exposes receiver progress in
`target/lsp-checker-agreement-final.log`. No complete regression or runtime timing
pass is implied by these focused suites.

Whole-project `node tools/format-project.mjs` passed with the current CLI after
installing locked tooling through `npm ci --ignore-scripts`: zero files formatted,
zero fingerprints changed. The initial missing-TypeScript formatter failure is
not a pass. A second whole-project formatter pass again changed zero files and
fingerprints; `tsrust format . --check --color=never` also passes. Table
acceptance and runtime performance measurements have not resumed.

## macOS bounded probe — 2026-10-28

The existing probe now supports macOS counters without changing the Windows
counter path or its limits. It uses one isolated LSP and the same 24-entry,
512 KiB unrelated-buffer revision, dependency invalidation, formatting and
close/reopen workload. macOS sampling uses `footprint` JSON for physical footprint,
kernel peak and swapped/compressed bytes, `ps` for RSS and `memory_pressure -Q`
for system free percentage. Missing/invalid counters fail the probe. Samples are
process-specific; the system pressure counter is explicitly system-wide.

On this 8 GiB Mac, stop conditions are physical footprint >=1 GiB, growth >=512 MiB,
swapped/compressed memory >=256 MiB, or system free memory below 20%. The normal
probe retains its 120-second deadline. `--soak` repeats the same workload for at
least 30 minutes, with a 31-minute hard stop. These limits guard the diagnostic;
they do not cap the server or replace an investigation of growth.

The initial macOS run passed all 890 requests in 43,638 ms with 34 samples,
zero stderr and normal exit. First/last physical footprint was
2,326,984/15,794,656 bytes; the reported process peak was 15,794,656 bytes.
Maximum sampled swapped/compressed memory was 3,457,024 bytes (final 3,194,880).
RSS ranged from 8,880,128 to 26,705,920 bytes. System free percentage stayed
35–41%. These are diagnostic observations, not a benchmark or long-session proof.
Tested LSP SHA-256:
`5e5de5b448885901110c6f43cab3855899ad4615da906f244be13808ea70ecf4`.
The probe now records the tested binary hash in subsequent output automatically.
Current disposable output is `target/lsp-memory-probe.json`; the next run replaces
it after completion. This summary preserves the short-run result without adding
routine logs to SQLite. The subsequent sustained macOS result is recorded below.

## Sustained macOS bounded workload — 2026-10-28

The same binary (`5e5de5b448885901110c6f43cab3855899ad4615da906f244be13808ea70ecf4`)
completed `node tools/lsp-memory-probe.mjs --soak` normally in 1,801,653 ms.
All 38,926 requests passed across 526 open/edit/invalidate/close cycles, including
format idempotence, changed imported hover and zero stale/member consistency
counters. There were 1,427 memory samples, no stderr and no reported failure.

- Physical footprint: 2,359,752 bytes initially, 16,826,872 finally; reported
  process peak 16,826,872 bytes.
- Near 10 minutes: 16,794,080 bytes; near 20 minutes: 14,123,512; near 30 minutes:
  16,826,872. Observed usage dropped between cycles; no sustained high growth was
  reproduced within this workload.
- Maximum sampled swapped/compressed memory: 15,581,184 bytes; final 3,768,320.
- RSS: 8,798,208–27,656,192 bytes; final 24,854,528.
- System-wide free-memory percentage: 26–56%, above the 20% stop condition.

The workload reused 24 imported entries and one 512 KiB unrelated revision buffer.
It is not a full repository scan, a representative large workspace, a multi-hour
acceptance claim or a throughput benchmark. The original report is not dismissed
using RSS alone: physical footprint and swapped/compressed counters were measured.
Public guidance now states the fixes and bounded evidence with these limits.
Disposable output: `target/lsp-memory-probe.json` and
`/tmp/tsrust-macos-memory-soak.log`; no routine data was imported into SQLite.
