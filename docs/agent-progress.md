# Agent continuation state

## Goal and source state

Read the complete objective at
C:/Users/skeet/.codex/attachments/ff4ed6f2-4419-4108-886b-fdbac9889081/goal-objective.md
before goal continuation. Follow agent-goal.md, plans/backend-only-reset.md and
plans/rust-setup-and-dependency-distribution.md. Goal remains incomplete and active.
One agent writes this tree. No pushes, tags, releases, publication or recurrence.
No owner editor/process control. Commit every source change, with no authorship-
based exclusions, after acceptance and whole-project formatting. Respect gitignore.

Latest accepted source milestone: e61fed53, compiler-owned API dispatch and independent
SDK generation; see milestones/server-owned-api.md. cc8a0024 accepted the docs startup
profile repair; 0ea3d7e3 accepted portable source packs/editor repair; 2d3a6e09 accepted
shared backend sources. ef39be55 included owner configuration/.agents/docs edits.
All source changes were included at their checkpoints; no authorship exclusions.
Keep one target, CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=1. Current installed consumers
are the accepted API pair recorded below. Full backend/distribution goal is open.

## Accepted current section

Compiler build.rs/build-dependencies/vendor include_bytes payload removed. Ordinary
backend builds use shared Cargo sources; explicit setup --portable creates/validates
an optional exact Cargo vendor pack. Source export is toolchain-free after explicit
preparation, with vendors, licenses, lock/toolchain/config/prerequisite metadata.
TSRLIB3 retains pinned Windows imports; TSRLIB2 remains readable. Backend dev reports
runtime preparation. Retained UI code remains intact; generated runtime feature
rewriting recognizes both spaced and compact predicates. Bindings are not migrated.

The supplied 43 Rust diagnostics exposed an integration test's incomplete copied
CLI root and partial-fixture/renderer-disabled warnings. Tests now use real CLI
entrypoints and standard assertion APIs, feature consumers match helpers, and
ownership/parsed metadata remain retained under narrow explained expectations.
No global diagnostic suppression. New stdlib HTTP coverage preserves single body
consumption, response status, static serving, TLS/configuration and query behavior.

The remaining Tsrust module-native at stdlib/ui/api.tsr is repaired through shared
project::bundled_entry context in FileSourceProvider and LSP snapshots. Only the
canonical compiler-development UI facade path AND exact bundled bytes receive
logical std:ui context. Copies, modified buffers and imported user modules retain
normal private-binding checks. The guard is unchanged. LSP entry aliases preserve
physical cursor assistance and snapshot invalidation. Real protocol clean/modified-
private-error/repair/Node-hover passes without changing owner disk bytes.

Accepted: packaging 6/6, edited public links 1/1; native bundle/import/artifacts 3/3;
pack setup/tamper/failure cleanup 2/2; toolchain setup 2/2; shared cache/deployment 2/2;
portable relocated offline export and metadata 1/1 (65.93s); UI unit 106/106; HTTP
runtime 20/20; exact env backend Rust tests 2/2; date 3/3; cache 12/12; nonduplicated
HTTP static signatures/ownership 1/1; actual CLI UI content 1/1 (97.16s); current CLI
JSON/runtime/artifact 4/4; bundled original/copy/modified facade 2/2 and real LSP probe.
Full backend compiler/LSP all-target check on final shared helper source passed:
zero warnings/errors, terminal successful JSON build-finished, 1m56s. Full workspace
initial/final earlier passed zero warnings/errors, 106 UI tests; latest refresh below.

Keep repaired failures explicit: first portable copy hit old 40s deadline, now 120s
and complete parity passed. Added env assertion wrongly included argv[0]; fixed to
skip executable and exact runtime/tests passed under standard rustc --test in an
owned temporary wrapper, removed after execution. New JSON fixture corrected to
existing strictEqual before next run. Passed gates were preserved. No full compiler/
LSP regression or matched benchmark matrix was restarted; no analytics import.

## Final validation complete

31756 is terminal. Both final diagnostic configurations succeeded, zero warnings/
errors; both streams were parsed and require successful build-finished records.
Direct CLI check of original stdlib/ui/api.tsr passed. Current CLI created
 generated-projects/backend-portable-sources --app. Initial SDK verifier invocation
preceded its required build and could not spawn the absent executable; no backend
behavior failed. 79277 then built the fresh backend successfully. Resumed 93724
passed actual native HTTP/TS/Rust SDK acceptance (41.27s) and real project LSP
clean/error/repair. The owned server/LSP were stopped by their verifiers. Its runtime
workspace has no vendor directory or private Cargo home. No owned service is live.

All current sessions are terminal, including 31756/79277/93724/42871/95836/56242.
Do not poll/restart them. Logs: target/backend-editor-rust-diagnostics-accepted-current.jsonl
and matching check log; editor-rust-diagnostics-accepted-current.jsonl and matching
check log; bundled-facade-compiler-current.log; portable-backend-scaffold-current.log;
portable-backend-build-current.log; portable-backend-http-sdk-current.log;
portable-backend-editor-current.log. The milestone contains exact other accepted
counts/failures. No matched performance or analytics import was performed.

## Installed consumers and cleanup accepted

After the source checkpoint, both installed development consumers were staged in
C:/Users/skeet/.cargo/bin with expected-old/new hash guards and rollback preservation.
No owner process was stopped. CLI SHA256:
1AE19C92087663BE504646F1ABBA219842666BA18263A8D087469C25D3F0A9AC;
LSP SHA256: 53A93B85E008A4C0795709ECEEF311C34723D33E4D9894BCED265A2343DD8F15.
Installed CLI --version/check of the original facade passed; installed LSP real
clean/modified-private-error/repair/hover protocol passed
(target/installed-bundled-facade-editor-current.log). Final CLI size 40812544 bytes,
LSP 26595840 bytes. This local development refresh is not production trusted update.

Owner LSP PID 30920 was left running and may still use the old image. Preserved
image C:/Users/skeet/.cargo/bin/tsrust-lsp-running-eacf50ae86934295a698c4ef91b43db4.exe
has expected SHA256 E5966160755F0EF19B5E245348150DD0B9640413238902E7EB8595F15600D41B.
Also preserve the earlier tsrust-lsp-running-0a62d1de9bc945a680da390dec72ad54.exe,
SHA256 046c8e50e1d867988cb72ecbcc1f6d28b66f36cb8a1a078ba8d43b7a27f596a9,
until mapped process ownership/exit is proven. Do not delete either merely because
the canonical new file has another hash. The owner was told to use their own
Tsrust: Restart Language Server command (tsrust.restartServer) to adopt the repair.
Do not control their editor or stop/restart their process for cleanup.

Whole-project formatting before source checkpoint: 0 files changed, 0 fingerprints,
clean diff check. Obsolete owned target/debug/build/tsrust-compiler-* payload dirs
removed: 10. Old executable/symbol files removed: 8 (executable bytes 2012523520),
including the stale compiler unit executable with the earlier env test assertion.
All deletion targets were checked absolute paths within target, rejected reparse
traversal and skipped active executable owners. Current prepared source pack, warm
Cargo target, review projects and owner mapped images were preserved. No live
owned server or tool process remains. The acceptance milestone records failures/
repairs separately; passing gates were not restarted.

This documentation-only checkpoint is prepared after required whole-project format
and link checks. The docs startup profile repair below is now accepted. Continue server-owned API
and remaining distribution work as
specified below. Read exact current source/evidence; do not restart accepted gates
unless new code, failures or an unresolved uncertainty justifies it.

## Remaining required goal work

Native docs startup profile repair is accepted below; UI expansion remains frozen.
Server-owned API integration is accepted below: app/ and in-memory api:server supply
HTTP descriptions/OpenAPI; independent js-client/rust-client outputs need no server
source. Keep source/input snapshot agreement while completing the remaining work.
Ordinary binding graphs still need shared registry/Git cache migration. Finish Rust
replica execution, checked typed JSON/contextual body readers and async std:testing.
Trusted paired prebuilt install/latest-stable update, integrity/host/ownership,
interruption/locked Windows recovery and actual macOS/Linux validation remain open.
No whole-editor-panel claim is possible: current built Rust and LSP protocol evidence
is authoritative, while the owner's already running server needs their restart.
Use project-local remote-validation rules; never publish a release to test install.

## Docs startup profile accepted — 2026-10-27

Previous goal continuation was progress: implemented the shared native UI application
profile, added its regression, verified both all-target diagnostic configurations,
rebuilt the CLI, executed formatted backend assertions and actual docs dev, and
inspected a rendered native preview. See milestones/native-docs-development-profile.md
for root-cause evidence, commands, repaired experiment failures and limitations.

Both CLI and dev select UI application development opt-level=1, matching existing
support; backend stays 0, release 3, overflow checks remain enabled. Cache identity
includes optimization. The default stack reserve remains unchanged. Controlled
preview rendered and navigated, closed with exit 0. Current CLI docs dev built in
217391ms, launched PID 36060; its window was inventoried. Temporary-app capture
approval timed out, so no workaround/input was attempted. Ctrl+C stopped our session;
window inventory confirmed both owned previews closed. No owner process was stopped.

Profile unit 1/1 passed. Backend compiler/LSP and workspace all-target checks both
success, zero warnings/errors. Current CLI build succeeded. Formatted-source backend
smoke passed outside sandbox after its native temp archive rename was denied inside.
Final whole-project format: 0 files, 0 fingerprints; clean diff check. Installed LSP
exact-facade clean/error/repair/hover probe passed again in the diagnostic follow-up.
No whole-editor-panel claim. No full regression/performance matrix or analytics import.
Sessions 18880,42516,66465,84107,86997,94962 are terminal; do not poll/restart them.

Source checkpoint cc8a0024 includes all changes. Guarded installed CLI refresh passed
--version and exact original facade check; current installed SHA256 is
90D6BB34FD06398096F0A61ADA139AC3B91C6781C3198712A79E15F8543829BF.
LSP remains the already verified 53A93B85... version; owner mapped images preserved.
Pruned 346247105 bytes of disposable diagnostics and 110564134 bytes of obsolete
crashing docs cache, plus unused old CLI rollback copy after live-owner/hash checks.
Keep current docs cache c332a85856722893, runtime caches and prepared source pack.

The subsequent server-owned API section below completed that architectural step;
its accepted source and installed-consumer state supersede the old generated/server
prerequisite. Keep legacy explicit exports only for existing-project compatibility.

## Server-owned API section accepted — e61fed53 and installed pair

This continuation made concrete progress: closed nearest-config policy fallthrough
and source/fact consistency gaps, completed compiler/editor/native HTTP acceptance,
and exercised actual dev hot reload. See milestones/server-owned-api.md for exact
implementation, controls, repaired fixture/sandbox failures and practical limits.
Full objective remains open; no goal completion or whole-editor-panel claim.

Fresh CLI review project: generated-projects/backend-owned-api. Server owns app/
operations/context/auth and imports api:server. No generated/client or server tree,
SDK package or integration marker is created. Explicit legacy server export remains
available for existing projects. Independent TS/Rust SDK export never changes server.
Bootstrap carries the exact checked operation graph into final lowering; conflicting
preloaded input fails api-input-changed. Denied nearest config fails closed. LSP
retains derived source without creating physical files or disk invalidation inputs.

Accepted current source: focused compiler/scaffold/editor/facade 10/10; formatted
fixture follow-up 5/5. Backend and workspace all-target JSON streams each successful,
zero diagnostics; final affected backend refresh passed. Normal CLI/LSP build and
cached normal refresh passed. Whole-project format remains 0 files/fingerprints.
Actual project native build and std:testing 1/1 passed; check/emit passed without
Rust/Cargo on PATH, with no UI runtime in emitted code. Normal and relocated
executable HTTP SDK acceptance passed, live/saved TS and async Rust clients, with
no source/tool/cache files in deployment cwd. Host still has original tools/source;
do not claim an uninstalled-toolchain OS image or all-platform deployment.

Windows native dev passed HTTP, invalid-edit preservation, repaired hot reload
(5334ms), original source restoration and owned supervisor/child cleanup. Real
project and UI-facade LSP clean/error/repair probes passed. No owned service is live.
Sessions 14676,84958,40951,99629,68416,9410,47871,10841,49851,23961 are terminal.
Do not restart or poll them. Current logs are target/api-server-*-current.*; retain
concise evidence, prune emitted diagnostic sources and superseded test consumers.

Current normal target hashes:
CLI 6726391A5BCCA08216D8A16AADD997191C1B7AC624B3EDC44FF90FCE4B4402EC;
LSP 3E641BFF4E10862CC62A145F28140ED16990CA058B2D243A00E81F390D04553E.
Both installed consumers now match these target hashes. Installed CLI version and
review-project check passed; installed project LSP clean/error/repair/dispatch-hover
and original-facade clean/error/repair/hover passed. Source checkpoint e61fed53 includes
all accepted files. Post-install cleanup removed 1048397368 bytes of superseded
consumers/diagnostics. Authoritative CIM inventory found zero live compiler/LSP
processes; four old mapped-image/rollback backups were then hash-checked and removed.
No owner process was stopped. Current variants, caches, review apps and source pack
remain. Whole-editor-panel adoption is not inferred from installed protocol checks.

Next executable action: complete remaining HTTP SDK/body/testing work. Inspect
plans/http-request-body-readers.md and std:testing async registration, preserving the
accepted server-owned API baseline and ready native/Node fixtures. Actual Rust old/
new native replica coverage and auth must remain explicit. Then ordinary binding
shared-cache migration and production paired install/check/update with required
Windows/macOS/Linux execution. Full family regressions/measurement remain tracked;
do not restart passed focused gates without changed inputs or a specific concern.
No full executable regression/performance matrix, analytics import or remote release
was run. Keep those broader family/platform requirements open as the goal requires.

Post-install evidence: CLI/project check and both installed LSP probes passed;
project probe now asserts hover on api:server dispatch as well as diagnostics.
New/changed relative documentation links since 58d07d79 passed 8/8. A broader
whole-roadmap scan encountered the unchanged historical link
../benchmarks/runtime/diagnostics/gate100-call-spread-implementation.json, absent
from disk. It was not fabricated/restored or counted as healthy; historical-link
maintenance remains separate from current API link acceptance. Current Node
verification scripts pass syntax checks. No source-code or runtime failure remains
in this accepted section. Keep the full remaining goal open.
