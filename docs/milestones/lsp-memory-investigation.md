# LSP memory investigation

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

Current CLI SHA-256 is
`03f689fa1ebafcd1fad8cb08c420ea837fee80c9e8e6a8bbfffefd17c28f2b39`;
current LSP SHA-256 is
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

Whole-project `node tools/format-project.mjs` passed with the current CLI after
installing locked tooling through `npm ci --ignore-scripts`: zero files formatted,
zero fingerprints changed. The initial missing-TypeScript formatter failure is
not a pass. A second whole-project formatter pass again changed zero files and
fingerprints; `tsrust format . --check --color=never` also passes. Table
acceptance and runtime performance measurements have not resumed.
