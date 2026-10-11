# Project diagnostics milestone

Required after S08 collections acceptance and before starting S09. The user
requested project-wide checking and multiple errors per file, including files not
open in the editor. This is additional roadmap work, not S08 acceptance evidence.

## Required behavior

- Discover project `.tsr` files recursively, with the formatter's existing safe
  exclusions as the starting point. Avoid build outputs, dependencies and symlink
  cycles. Include unopened files and loaded module dependencies.
- Report multiple independent diagnostics per file. Recover at safe parser and
  analysis boundaries, suppress misleading cascades, and preserve ownership
  checking. A recovered diagnostic pass must never authorize native execution.
- Publish diagnostics for all affected project files. Recheck after edits, saves,
  file creation/deletion and dependency changes. Clear stale diagnostics, honor
  unsaved editor buffers and prevent older check results from replacing newer ones.
- Provide equivalent project checking through the CLI, with deterministic ordering
  and nonzero status when errors exist. Preserve useful single-file commands.
- Cover multiple syntax/type/ownership errors, multiple files, import failures,
  stale-result cancellation and editor-visible recovery. Refresh installed consumers
  and verify open and unopened files before accepting the milestone.

## Later configuration and scaffolding direction

The user is open to project configuration and `tsrust new <name>` scaffolding.
The current recommendation is one `tsrust.config.json` with build, check and format
sections, discovered by walking to the nearest project root. Candidate settings:
entry points, include/exclude paths, compiler parameters, binary name, targets,
indent width and quote style. Scaffolding would write standard defaults.

This configuration shape remains a design recommendation, not an implemented or
accepted interface. Split files only when actual use justifies them. Configuration
must not silently weaken ownership rules or change current project formatting.

## Current implementation checkpoint

The user selected `check` as the sole compiler checking command; `lint` is removed.
`check` accepts files and directories and defaults to the current directory. Shared
formatter/check discovery excludes `.git`, `node_modules`, `target` and symlinks,
canonicalizes and deduplicates files, and sorts diagnostics by source location.
Checking continues after errors in other files and deduplicates shared dependency
errors. The `main` entry point is optional in the language. Declaration-only and
empty files check, build and run; running without `main` exits successfully without
calling any declared functions. An existing `main` must still have no parameters
and no return value. Imported modules do not supply the entry point.

The editor command is named **Tsrust: Check File** and invokes `check`. This
checkpoint does not complete the milestone: multiple diagnostics within one file,
workspace-wide LSP scheduling/publication, stale-result handling and installed
consumer verification remain pending. Configuration/scaffolding is still future
work. Preserve the complete milestone requirements above.

Focused checkpoint validation: 92 project tests, six recursive-check/optional-entry
tests, 28 formatting tests and one editor library-diagnostic test pass. The whole
project formatter changes zero files and zero fingerprints. Optional-entry tests
execute formatted empty/declaration-only sources, preserve existing entry signature
rejections, and prove a dependency's `main` is not invoked. The editor no longer
suppresses entry diagnostics for library buffers. Closed diagnostic logs use the
`tsrust-optional-main-*` and `tsrust-project-check-*` names in SQLite.

## Statement syntax recovery

The diagnostic-only compiler API now collects syntax errors at block statement
boundaries. It preserves original byte spans, respects braces and quoted delimiter
text, handles missing semicolons, and stops after at most 64 diagnostics. Recovered
syntax trees never leave the parser; strict `check`, emission and native builds
continue to reject malformed sources. CLI standalone-file checking and standalone
LSP diagnostics consume the diagnostic list, including UTF-16 range conversion.

This is partial progress: module parsing, declaration/header and lexical recovery,
multiple semantic/ownership diagnostics, and project-wide LSP publication remain
pending. Installed consumers are still at the accepted S08 checkpoint; coordinate
CLI, language-server and extension refreshes because the extension now invokes
`check` rather than the removed `lint` command. Syntax errors currently suppress
semantic checking of the malformed file to avoid cascades. Closed focused logs
use the `tsrust-diagnostic-*` names in SQLite.

Recovery checkpoint validation: four recovery tests, seven CLI/project tests,
28 formatting tests, one normal-stack depth canary and two editor tests pass.
The depth canary exposed an unrelated compatibility regression from recursive
checking: single-file diagnostic JSON had gained a `file` field. Single-file JSON
now preserves its existing shape; directory/multiple-input results retain paths.
The whole-project formatter changes zero files and fingerprints. No timing run or
installed-consumer acceptance is claimed.

## Module syntax diagnostics checkpoint

CLI and workspace checks now collect block-statement syntax errors across loaded
modules after a strict parse fails. They use declaration sources with imports and
exports blanked without moving source offsets, including test-registration bodies.
The tolerant header graph is used only to find further diagnostics; only the
original strict check can return executable analysis. Original errors remain in
the result even if tolerant discovery cannot load another dependency.

The LSP publishes every returned error, reads unsaved dependency buffers and clears
repaired dependency buckets. A framed-protocol test verifies two diagnostics in an
unopened imported file and their removal after an unsaved repair. Its first run
failed because the fixture compared an uncanonicalized temporary-directory URI;
the corrected fixture uses canonical paths. Compiler checks cover independent
broken dependencies and original coordinates in `.test.tsr` registrations.

Validation passes: 13 compiler recovery/project checks; 11 editor optional-entry,
action and session checks; 16 project-diagnostic conversions; 12 protocol tests.
The whole-project formatter changes zero files and fingerprints. Closed logs are
stored under `tsrust-module-diagnostics-*` in SQLite. This does not yet provide
workspace discovery of unrelated unopened files, multiple semantic/ownership
errors, or recovery of every malformed declaration/header. Installed-consumer
refresh remains pending until the coordinated diagnostic update.

## Workspace discovery checkpoint

The LSP now reads workspace folders (or the legacy root URI), discovers recursive
`.tsr` inputs through the shared formatter/check discovery, and checks unopened
files. A single analysis worker handles both document jobs and workspace scans.
The pending scan is replaced after edits, opens, closes, watcher notifications and
workspace-folder changes. Revisions plus immutable buffer/disk inputs prevent stale
results from publishing. Unsaved new files under a root join the scan; open buffers
override disk contents. Discovery excludes generated directories and symlinks.

Workspace scans own diagnostics within their coverage; document jobs still refresh
navigation, actions and dependency tracking. Files outside workspace coverage retain
the existing document behavior. Closing a workspace document returns diagnostics
to its disk contents. Deleted files and removed roots clear their publications.
Scanning starts after the client's initialized notification.

Validation: 13 framed protocol tests, four project-session tests and three scanner
lifecycle tests pass. Three worker tests also pass after their initialization
helpers send the required initialized notification. An expanded protocol witness also verifies initialization
ordering, outside-workspace files and root removal. The scanner tests cover shared
roots, unsaved new files, exclusions, symlink cycles, stale buffer/disk inputs and
root removal while a scan is in flight. Whole-project formatting changes zero files
and fingerprints. Closed logs use `tsrust-workspace-scan-*` in SQLite.

Remaining diagnostic milestone work: semantic/ownership recovery, broader malformed
syntax boundaries, final combined acceptance and coordinated installed-consumer
refresh. No benchmark timing or installed-editor acceptance is claimed here.

## Statement semantic recovery checkpoint

Compiler diagnostics now replay semantic analysis from pristine input after an
invalid nonterminal statement. Valid preceding ownership changes are re-established
on every pass; partial changes from the failed statement cannot leak forward.
Invalid declarations remain in lexical scope discovery, so removing an initializer
does not accidentally expose a shadowed outer name. Follow-on unknown-name errors
that resolve to skipped declarations are suppressed, including dependent bindings.
Replay is limited to 128 attempts and 64 reported errors, with deterministic source
ordering. Recovered checked trees are discarded and never become public Analysis
or ProjectAnalysis values; strict checking/building still rejects the original input.

The CLI, standalone LSP, module diagnostics and workspace scanner use the collector.
Focused cases cover independent types, repeated transfers, overlapping access and
related locations, lexical shadows, nested callbacks, independent functions, module
coordinates, rejected builds and multiple semantic errors in unopened workspace files.
The shared rejection helper retains the original strict diagnostic assertion and now
compares the CLI's complete list against the diagnostic API.

Recovery deliberately stops at a failing return, throw or control jump, including a
failed enclosing control statement that contains one. It does not create fallthrough
paths or substitute return values. Continuing beyond these boundaries, declaration
errors and other unsupported semantic boundaries remains required milestone work.

Validation: 30 compiler tests (nine recovery, ten project diagnostics, eleven control
flow), seven editor conversion/scanner tests and the workspace protocol witness pass.
The scanner fixture now uses an atomic suffix after a parallel temporary-name collision.
The depth canary initially aborted: disassembly showed the new inline AST replacement
had increased the recursive block frame to 11,216 bytes. A nonrecursive helper reduced
the final frame to 10,592 bytes and the normal-stack canary passes. Strict checks bypass
the helper when no diagnostic skips exist. Original failing logs are retained alongside
repairs under `tsrust-semantic-diagnostics-*` in SQLite. No full milestone regression,
performance measurement or installed-consumer refresh is claimed yet.

## Independent-function recovery checkpoint

An invalid return, throw or jump no longer prevents diagnostics from independent
functions in the same file or module graph. Diagnostic replay prioritizes another
source function without reordering declarations or changing function identities.
Normal call checking still follows the original bodies and contracts. Invalid
terminal operations remain intact; no return placeholder or artificial fallthrough
is introduced. Repeated discoveries are deduplicated within the existing replay
budget. Strict executable checking does not enable diagnostic priority.

Focused validation passes 32 compiler tests (eleven recovery, ten project, eleven
control-flow) and five editor tests. Witnesses include invalid ordinary/callback
returns, jumps, try/finally, independent declarations and a failed dependency return
with an unrelated entry error. The initial location expectation was corrected to
match the existing diagnostic at the `return` keyword, without changing compiler
locations. Original red and repaired logs use `tsrust-diagnostic-boundaries-*` in
SQLite.

The requested diagnostic behavior is ready for combined milestone acceptance:
recursive CLI checking, optional main, multiple statement syntax/type/ownership
errors, independent function recovery, workspace discovery, unsaved inputs and
stale-publication handling. Recovery is deliberately bounded and may stop inside
malformed declaration/control regions; this is not a promise to enumerate every
possible error after arbitrary invalid syntax. Next run the complete compiler and
LSP regressions, repair affected failures without restarting completed groups, then
refresh CLI/LSP/extension together and verify the installed editor. Configuration
and scaffolding remain future work. S09 has not started.

## Combined regression and repair evidence

The complete workspace regression at `da0e7249` finished: 3,659 passed, four failed
and four intentionally ignored across 308 targets. The failures were three CLI
acceptance expectations and one destructuring recovery defect. Optional main made
the old missing-entry rejection obsolete; multi-error JSON required comparing the
complete diagnostic list while retaining the strict original diagnostic assertion.
The check command again explains that `--release` applies only to build/run/test.

Diagnostic replay now skips all lowered pieces of one source statement together.
It retains every lowered declaration for cascade suppression and never skips a
group containing terminal control flow. A new witness prevents internal `pattern#`
names from leaking after invalid destructuring. Equal-span diagnostics use the same
code ordering as the CLI. All 50 tests in the affected acceptance, destructuring,
recovery and control-flow batch pass, including all four original failures.

The original full run and repair batch overlapped: Cargo released its build lock
before the original tests finished, allowing the repair build to replace the shared
CLI. Original test executables retained the baseline library; later CLI invocations
used the repair patch. Treat the compiler project target onward as conservatively
mixed CLI provenance. The full run was not restarted, and no additional failures
occurred. This evidence is not a claim of a pristine single-commit full run.

Original logs and the provenance record are byte-verified in SQLite as
`tsrust-diagnostics-regressions-da0e7249.log`,
`tsrust-diagnostics-repairs-da0e7249.log`, and
`tsrust-diagnostics-cli-transition-da0e7249.json`. Whole-project formatting changed
zero files or benchmark fingerprints (`tsrust-diagnostics-repair-format.log`).
Consumer refresh and installed-editor acceptance remain pending.

The repaired compiler also passes all 19 affected LSP project-diagnostic and
workspace-scanner tests (`tsrust-diagnostics-editor-repairs.log` in SQLite).


## Workspace scan responsiveness

Installed verification at `89664eba` confirmed two independent diagnostics in an
unopened file with no main function. Removing that file cleared both diagnostics;
the already open recursive-arrays and ordered-map sources had no diagnostics.
The repository's 14 remaining diagnostics include deliberate rejection fixtures.
The refreshed compiler consumers pass a three-error/two-file main-free project
and its repaired version. Original build, CLI smoke and editor observation evidence
use `tsrust-diagnostics-*-89664eba` paths in SQLite.

The initial implementation rechecked every source after every edit. A recursive
CLI check of this repository took 44.872 seconds with the debug compiler. The LSP
now retains independent entry results, reuses unchanged successful and failed
analyses, and rechecks changed inputs and their importers. It tracks unsaved buffer
revisions, attempted missing imports, disk identities and timestamps; file creation,
deletion and root changes are discovered on every scan. Reused inputs still enter
stale-publication validation. Conflicting dependency versions observed during a
scan invalidate the combined result. Removed entries release their cached inputs.

Focused acceptance: 31 LSP tests pass across workspace inputs, project diagnostics,
scanner behavior, inconsistent-input rejection and worker protocol tests. A real
stdio LSP smoke scan over this repository reports two errors in an unopened probe,
clears them after repair and clears the removed file. Single-run observations:
debug cold scan 48.857 seconds, one-file edit 0.089 seconds, deletion 0.084 seconds;
optimized cold scan 9.544 seconds, edit 0.058 seconds, deletion 0.062 seconds.
These are editor responsiveness observations, not matched native/Node runtime
benchmarks. No generated runtime behavior changed. Use optimized language-server
builds for large workspaces. The probe sources were removed after the checks.

Evidence: `tsrust-workspace-scan-cost-89664eba.log`, `tsrust-scan-cache-red.log`,
`tsrust-scan-cache-checks.log`, `tsrust-scan-cache-final.log`,
`tsrust-scan-cache-worker.log`, `tsrust-scan-cache-protocol-cost.log`,
`tsrust-scan-cache-release-protocol-cost.log`, `tsrust-scan-cache-release-build.log`
and the executable observation script `tsrust-scan-probe.py` in SQLite.
