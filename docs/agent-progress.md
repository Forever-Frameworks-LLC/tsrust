# Agent continuation state

## Goal and source state

Read the complete objective at
C:/Users/skeet/.codex/attachments/ff4ed6f2-4419-4108-886b-fdbac9889081/goal-objective.md
before goal continuation. Follow agent-goal.md, plans/backend-only-reset.md and
plans/rust-setup-and-dependency-distribution.md. Goal remains incomplete and active.
One agent writes this tree. No pushes, tags, releases, publication or recurrence.
No owner editor/process control. Commit every source change, with no authorship-
based exclusions, after acceptance and whole-project formatting. Respect gitignore.

Latest accepted source milestone: 0ea3d7e3, separating optional portable source packs
and repairing reported editor diagnostics. Previous milestone 2d3a6e09 accepted
shared backend sources. ef39be55 included owner
configuration/.agents/docs changes as explicitly requested. All source changes were included in 0ea3d7e3; no authorship-based exclusions. See
milestones/portable-source-packs-and-editor-agreement.md for this section's actual
implementation, evidence and limitations. No archive/source snapshot was created.
Keep one target, CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=1. Current CLI/LSP target and installed tools are validated in the affected cases.

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
and link checks. Next prioritize the unresolved native docs stack overflow
before UI expansion; continue server-owned API and remaining distribution work as
specified below. Read exact current source/evidence; do not restart accepted gates
unless new code, failures or an unresolved uncertainty justifies it.

## Remaining required goal work

Native docs startup stack overflow (0xc00000fd) is unresolved; do not claim the
compiler/editor repair fixes it. Prioritize the actual startup repair before UI
expansion. Server API must belong to app/ and serve description/OpenAPI, allowing
independent js-client/rust-client generation anywhere. Current scaffold still imports
generated/server; do not disguise a rename as the needed architectural correction.
Ordinary binding graphs still need shared registry/Git cache migration. Finish Rust
replica execution, checked typed JSON/contextual body readers and async std:testing.
Trusted paired prebuilt install/latest-stable update, integrity/host/ownership,
interruption/locked Windows recovery and actual macOS/Linux validation remain open.
No whole-editor-panel claim is possible: current built Rust and LSP protocol evidence
is authoritative, while the owner's already running server needs their restart.
Use project-local remote-validation rules; never publish a release to test install.
