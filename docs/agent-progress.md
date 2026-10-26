# Agent continuation state

Updated: 2026-10-26. The audited dataset integration and requested cleanup are complete.
The original compiler/UI continuation goal remains incomplete.

Current owner priority is Windows integration; the latest request adds normal
launcher presentation and generated project/component agent guidance.
See [Windows installation acceptance](milestones/windows-installation.md).
Audited data integration and cleanup are complete; builds now run only for concrete
installation/feature acceptance, not to repopulate caches. Table continuation and
its pending checks remain preserved below.

## Windows development supervisor accepted

Based on local checkpoint `49b9625a`, the Windows supervisor now uses suspended
child assignment to private Job Objects, console stop handling, native recursive
notifications, exact volume/file/time stamps and canonical source identities.
The existing frontend, immutable snapshots and replacement semantics remain shared.
Sixteen focused cases pass across retained batches, including orphan isolation,
restored-write-time edits, concurrent atomic saves, frozen external imports and
documentation publication mutation/guards. Permission-only failures pass unchanged
outside the sandbox. Two real documentation separator bugs were repaired.

The normal CLI passes `tools/smoke-windows-dev.py`: Unicode/space paths, native
execution, bad-edit preservation, replacement, Ctrl+C and Ctrl+Break cleanup.
Scaffold/check/format/run/test/bundled JSON smoke also passes. Whole-project
formatting changes zero files/fingerprints; installed `format . --check` passes.
The existing Cargo-installed compiler was refreshed byte-for-byte from the verified
normal CLI; SHA-256 `9885034dd785f8eb19e959ae0125514ec913fb10c6efd3740a5b0009712f5fba`.
Original Unicode source/help bytes were preserved after correcting a transient
file-edit encoding mistake. The normal CLI uses the already built integration-test
dependency graph; no Unix-only integration test was claimed as run on Windows.

See [Windows evidence](milestones/windows-installation.md) for exact commands,
resource observations and remaining GPU/window and web prerequisite acceptance.
No test/build remains live from this section. No remote validation or release ran.
Accepted source checkpoint: `a1a6cc4e`. Pruned its four completed CLI/unit-test
executables and symbols: 930,684,416 logical bytes; observed free-space increase
930,689,024 bytes. Installed consumers, source, databases and dependency caches
remain intact. No project test/Cargo process remained before cleanup.

Next section: actual Windows GPU/window prerequisites. Hardware reports NVIDIA
GeForce RTX 4090, driver 32.0.16.1742. Reuse the existing `tsrust-ui` static example
with `--features acceptance -- --verify-window`, which presents a GPU frame and
closes its event loop. Build offline in the shared target with one job and no
incremental compilation; retain the 8 GiB private-memory / 4 GiB free-memory
bounds and investigate after 15 minutes without completion. Runtime acceptance
must exit within 30 seconds after build completion. Do not claim generated Tsrust
UI or interactive behavior solely from the Rust renderer witness.

## SQL template highlighting

The owner screenshot shows expected interpolation: backtick strings embed code
inside `${...}`, while `'id'` and `'name'` are nested quoted arguments. A real
lexical classification bug also matched `.int()` as the primitive type `int`.
The top-level primitive rule now excludes member names and call positions; type
annotation patterns retain their original rules. The exact SQL template, ordinary
quoted text, spaced method calls and stored member references have regression
assertions. The complete TextMate grammar test passes. Packaging/compilation pass,
the updated VSIX is installed, and whole-project formatting changes zero files or
fingerprints. Existing VS Code windows need reload for the new grammar.

## Duplicate Run button repair

The installed Code Runner 0.12.2 contributes `code-runner.run` to the shared
`editor/title/run` menu. Tsrust previously contributed a separate direct
`editor/title` action, producing two play buttons. Tsrust now contributes to the
shared Run menu with priority `navigation@-1` for trusted Tsrust documents.
The command palette and context-menu command remain unchanged. This applies on
all hosts; no Code Runner setting or unrelated extension was disabled.

Compilation, packaging and the focused contribution assertion pass. The rebuilt
VSIX is installed in the normal user profile. Existing VS Code windows need a
reload to re-read the updated manifest. The packaged integration suite now asserts
the shared contribution and absence of a separate direct action; the unchanged
full integration suite was not repeated for this manifest-only behavior change.

## Windows installation section

Based on `0e7a72a1`, the Windows section installs both development consumers through
Cargo and the verified VSIX in the normal VS Code user profile. Six distribution
checks, four executable-discovery checks, grammar validation and real packaged
Windows editor integration pass. Native example check/run/test pass. Full details
and remaining watcher/GPU gaps: [Windows evidence](milestones/windows-installation.md).
The final CLI reinstall and external smoke pass, including corrected scaffold hints.
Whole-project formatting reports zero changed files and zero fingerprint changes.
No remote validation or release was triggered. Original Table acceptance remains
pending; Windows integration is the owner's current priority.

## Source and current priority

- Original source: `7c956bd7`; memory checkpoint: `7b18073a`; continuation and
  owner-instruction checkpoint: `89eb642a`. The requested audited data commit
  `b35e527f3dce8242a7155dd87d27b5f32152d476` was cherry-picked as `470e8d02`.
  Owner continuation instructions merged cleanly. Untracked `.agents/` and
  `docs/agent-goal.md` remain preserved.
- Required finite scope: [continuation checklist](plans/continuation-acceptance-checklist.md).
  Proposals and explicit deferrals remain separate from required implementation.
- The owner changed priority to dataset integration and cleanup. Heavy checking
  stopped; do not restart a build, LSP or preview merely to repopulate caches.
  No push, CI dispatch, release tag or public release was performed.

## Audited data and cleanup

- `benchmarks/measurements.sqlite` is 37,199,872 bytes. SHA-256 exactly matches
  `d0d26725ac3dfc9a631766a3865f13c6c208760d08a46e6c3877ce41494ef5f3`.
- Python 3.13 read-only `public.connect()` / `public.validate()` passes with
  analytics_runs=69, analytics_workloads=621, analytics_samples=86436,
  analytics_summaries=940 and analytics_profiles=825. No benchmarks were rerun.
- Follow [PUBLIC_DATA.md](../benchmarks/storage/PUBLIC_DATA.md) and the updated
  workflow. Routine logs, source snapshots and cleanup receipts do not belong in
  analytics. The mixed legacy archive must never be published. It is absent in
  this clone; historical loose records were preserved.
- Inventory found only the project `target/`, initially 13,947,034,689 logical
  bytes. No process/user/machine CARGO_TARGET_DIR override or configured custom
  target directory was found. No matching abandoned temporary roots from the
  two interrupted checker processes were found.
- Removed `target/debug/incremental` (2,373 files; 4,330,265,868 bytes),
  `target/debug/examples` (14 files; 4,697 bytes), and 16 obsolete project test
  executable/symbol files (2,307,828,224 bytes). Total: 6,638,098,789 logical bytes.
  Observed free-space increase was 6,627,885,056 bytes; concurrent disk activity
  and allocation accounting can differ from logical lengths.
- Preserved current CLI/LSP executables and their hard-linked aliases/symbols,
  compiled dependencies, offline vendor/build caches, registry/toolchain caches,
  Node dependencies, source, owner files and the public database. No legacy
  SQLite file or sidecar was deleted. No broad temporary-directory deletion,
  git clean, hard reset, source snapshot or analytics import was used.
- The active project checker was stopped for the priority change. Final inventory
  found no project Tsrust/server, Cargo, rustc or benchmark process. The active
  editor host and unrelated Node processes were left alone. Removed test programs
  must be rebuilt from their source checkpoints when genuinely needed.

## Continuation acceptance retained

- [Memory evidence](milestones/lsp-memory-investigation.md): observed-only cached
  snapshots release unrelated old buffers; successful configured scans release
  superseded independent entries. Active immutable inputs and stale-result guards
  remain intact. Both retention witnesses and all eight workspace-scan tests pass.
- Windows local URI normalization and import-path fixes pass the encoded-URI
  witness and all 17 project-diagnostics tests. Editor consistency passes 27 tests.
  The complete completion matrix's eight failures were confined to two import-path
  rows; those rows and module inventory pass after repair. Other passing rows are
  retained; no unchanged full matrix was repeated.
- The 12-cycle memory probe passes 890 requests in 61,849 ms with 43 samples and
  zero consistency counters. Highest sampled private commit: 6,787,072 bytes.
  This does not prove the historical long-lived macOS report resolved.
- Checker agreement remains incomplete. Its first retry was stopped after another
  Unix-only fixture identity was found; the corrected retry reached receiver Time
  before the owner changed priority. Its partial progress is not a full pass.
  Current-gate output remains under `target/`; it was not imported into analytics.
- Windows installer generated paths now retain portable separators without changing
  authored-path guards. The focused path, actual versioned install/idempotence/
  owner-edit preservation and Kbd-upgrade witnesses pass. The production CLI was
  refreshed to that installer repair during the Windows installation section.
- [Table acceptance](milestones/ui-table-layout.md) remains open. `colSpan` hover
  has its type but lacks documented behavior. The old generic inheritance fixture
  illegally redeclared an inherited field. Positive cases now use legal declarations,
  with a separate duplicate-field rejection witness. No language rule changed.
- The new formatted header/checked-index unit witness initially exposed that
  duplicate-field rejection. A later test compile failed because ProjectAnalysis
  has no Debug implementation; `unwrap_err()` was repaired to `err().unwrap()`.
  The corrected witness ran and fails at `headers base.value null`; its focused
  log is `target/table-inherited-help-resumed.log`. No Table source/lock installation,
  generated-native, browser or performance acceptance has passed in this clone.
- Whole-project formatter and a separate `format . --check` pass with zero source
  or fingerprint changes. All checkpoints are local; source and tests with pending
  acceptance are explicitly recorded rather than described as complete.

## Next executable work

Windows installation and packaged editor acceptance are complete. Keep prioritizing
Windows integration: inspect native UI prerequisites and the Unix-only development
supervisor before selecting the next coherent section. Do not describe watcher,
GPU-window or web-toolchain support as accepted from console smoke alone.

When returning to Table, trace checked field documentation provenance and repair
inherited help without weakening assertions. Resume pending checker acceptance
and exact Table compiler/editor witnesses from the pause handoff. Use the shared
target, one Cargo job and CARGO_INCREMENTAL=0. Preserve completed evidence; do not
restart broad regressions or timing solely because caches were pruned.

## Windows native UI section in progress

The existing Rust static window witness builds offline in 3m53s and passes actual
frame presentation, resize sequence and event-loop closure within the 30-second
external runtime bound. Logs: `target/windows-ui-build.log` and
`target/windows-ui-window-smoke.log`. This proves the renderer window path, not
generated Tsrust application behavior or all input/clipboard/accessibility contracts.

A fresh `target/windows-ui-setup` scaffold is generated through the installed CLI
with `--ui --target native`; check and format-check pass. Its Page has reactive
Count text and an Increment button. Its offline generated native build completed runtime compilation but failed
application linking (LNK1181, missing windows.0.52.0.lib); log
`target/windows-ui-generated-build.log`. No build remains live from that run. `TSRUST_CACHE_DIR=D:/ai_dev/tsrust/target/tsrust-native`,
CARGO_BUILD_JOBS=1, CARGO_INCREMENTAL=0. The compiler-owned shared native workspace
is `runtime/build-v1/6be45785ffc2ba6b` under that cache; it owns its required native
Cargo target. This is application installation validation, not cache repopulation.
Source/public measurements and unrelated user processes remain preserved.

Next: repair preservation of Cargo native link artifacts/search paths without
repeating the completed cold runtime compilation. After successful linking, open
the generated app and verify actual
button/keyboard reactive updates and orderly close through Computer Use, under a
three-minute interactive-window bound. The Rust renderer witness alone does not
accept these generated-source interactions. Inspect/repair any Windows failure;
do not restart a live build because output observation expired.

## Launcher and generated agent guidance accepted

Source baseline a1a6cc4e. See [complete CLI evidence](milestones/cli-agent-guidance.md).
The normal arrow-key launcher and short-panel branding pass real Windows console
checks, including Ctrl+C/Esc, cursor visibility and exact input-mode restoration.
The example workspace clears inherited TERM=dumb for new terminals; existing shells
need replacement. `tsrust new` generates root AGENTS.md, and UI installation adds
an index plus checked versioned component API references through its existing
atomic transaction. Owner guides/source/lock bytes are preserved; dry runs do not
write. All 17 installer cases and four launcher helper cases pass across retained
batches, plus complete-catalog, versioned-import and real CLI guide smokes.

The final normal CLI is installed and hash-matched:
616c928d888970c69a6752ae69d2afc6a814224dae10842662d40c87eb6e63c4.
An installed-consumer check proves the exact reviewed project template is generated.
The Windows setup example now contains that guide. Whole-project formatting and
installed format-check pass with zero changes. No CLI build/test remains live.
Accepted local checkpoint: 48e96ff9. The installed CLI also backfilled the
compiler-owned target/windows-ui-setup fixture with real project/index/component
guides while preserving its existing components. The redirected integration
witness passes (help/no prompts/no writes). Four completed CLI test executables/
symbols were pruned: 1,036,318,208 logical bytes; observed free increase
1,036,324,864 bytes. Installed consumers and compiled dependencies remain intact.
Next: repair the separate Windows native import-library link/cache failure using
retained compiled runtime artifacts.

Owner-created `examples/windows-setup/wut/` is unrelated and must not be staged,
rewritten or deleted. Preserve untracked .agents/ and docs/agent-goal.md as well.

Generated native UI build 86380 is terminal (exit 1). Cargo compiled its offline
UI runtime, but final application linking fails LNK1181 for windows.0.52.0.lib.
The existing packed runtime carries rlibs/DLLs but loses Cargo native import-library
search paths. Keep shared native workspace runtime/build-v1/6be45785ffc2ba6b and
completed compile artifacts. Next Windows backend repair: retain resolved Cargo
native link artifacts through live linking and cache restoration; bump cache ABI
as needed, verify a native import-library round trip and the real generated app.
Do not rerun the cold native dependency build just to rediscover this failure.

Native linker investigation: the compiler-owned native vendor tree contains
windows_x86_64_msvc-0.52.6/lib/windows.0.52.0.lib, so the required bytes are already
in the pinned dependency bundle. Prepared.link currently passes only -L dependency,
not a native import-library search directory. Inspect the simplest shared repair:
restore the host-matching prebuilt Windows import libraries from the existing
compiler-owned FILES closure into the dependency directory and add native linking
for both fresh/cached paths. This may avoid unnecessary Cargo JSON protocols or
cache format migration because those bytes are deterministic inputs of the pinned
bundle. Verify before deciding. Link callers: main.rs, dev_cli.rs,
package_validation.rs and a runtime_build test. Preserve errors and never overwrite
an unexpected existing native artifact. Add exact byte/restoration witnesses and
run the existing generated UI build from its retained cache, then real interaction.

## Native Windows import libraries accepted

Based on adc0f5d5. Shared fallible Prepared.link restores host-matching MSVC
windows-targets imports from the pinned FILES closure, adds native search lookup,
and propagates errors through ordinary builds, dev and package validation. No
cache-format migration or cold runtime rebuild was needed. Exclusive creation
and exact length/byte checks preserve unexpected files; symlinks are rejected.

Both artifact tests pass on final source (1m52s build, 0.10s execution), covering
existing TSRLIB2 round trip, exact restored imports, idempotence and owner-byte
preservation. The normal CLI integration graph builds successfully. The retained
UI fixture links successfully with runtime=hit (14.88s native/link), log
target/windows-ui-generated-link-repair.log. Computer Use verified visible Count
1 -> 2 by click, 2 -> 3 by Space and 3 -> 4 by Return, with named accessible button
and visible keyboard focus. Close exited the owned app 0. Initial-zero, clipboard,
complete glyph/accessibility and other-platform acceptance remain unclaimed.
The first approval timed out; the owner's requested retry succeeded. No test
window remains. The final focused Cargo test is terminal, exit 0.

Whole-project formatting changed zero files/fingerprints. Installed normal CLI
and source SHA-256 match:
1f04114cd77b11d91d11133ce8a43fb49c8dbec03ca911e452ce4837a395fa78.
Installed version 0.1.0; LSP/VSIX remain previously accepted. Windows generated UI
checklist item is complete; original required continuation scope remains open.
See milestones/windows-installation.md for exact limits and evidence.

Next: checkpoint this coherent repair locally, prune its now-disposable test
executables/symbols after inventory, then resume the preserved inherited Table
help failure and component acceptance. Preserve the installed compiler, retained
native dependency cache, measurements.sqlite, owner .agents/, agent-goal.md and
examples/windows-setup/wut/. No remote CI, push, tag or release occurred.

Accepted local source checkpoint: 37a48cfb. Its four inventoried completed test
executables/symbols were removed after exact containment and process checks:
1,036,928,000 logical bytes. No selected test executable, Cargo, rustc or generated
UI process remained at cleanup. Installed tools and compiled dependencies remain.
The preserved Table witness still fails headers base.value = null. Before changing
documentation propagation, inspect member receiver spelling/resolution:
intelligence.rs resolve_token compares the raw receiver owner, expression_type
unwraps ReadOnly only for native aliases, while schema/path_intelligence.rs owns
record_alternatives. Determine whether ownership wrappers hide inherited record
members in header indices, then repair the shared lookup and retain the original
header/checked multi-file assertions. No Table source edit or rerun has occurred.

## Table inherited editor help accepted

Working from 5c2be5ac. Shared member expression lookup now unwraps
ReadOnly/Mutable/Owned around known record declarations as well as native aliases;
callable ownership wrappers remain intact. Inheritance re-expands base aliases
after imports arrive, using the existing written_type resolver. This preserves
fields and documentation through private/generic imported aliases such as
ViewProps -> ViewComponentProps -> ViewRenderProps. No language semantics changed.

The formatted multi-file unit passes header and checked indices, renamed generic
alias inheritance, ReadOnly/Mutable/Owned receivers, inherited versus local docs,
original field definition destinations, and duplicate-inherited-field rejection.
Its first cold compiler dependency rebuild took 6m21s; a new Mutable case correctly
rejected implicit ownership transfer. Explicit clone(mutable.value) repaired the
witness. The final focused unit passes in 0.07s after an 11.58s affected rebuild;
log target/table-inherited-help-resumed.log. No weakening of compiler rules.

Real ui_content protocol batch: four pass, Table fails missing colSpan docs.
After the base-alias repair, only Table was rerun and passes in 5.77s (17.89s build),
covering documented 1-1000 hover, inherited prop completion, rowSpan rejection and
zero stale/unknown-receiver/nonmember counters. Original four passing cases remain
retained, including imported inheritance and current/older content components.
Logs: target/table-editor-protocol.log and table-editor-protocol-repair.log.
Whole-project format and format . --check pass, zero files/fingerprints changed.

Next: local checkpoint and refresh installed LSP without starting an editor/server.
Then run the pending compiler ui_content Table native/installer/documentation
witnesses, retaining cache and Windows resource bounds. Actual browser Table
interaction and original finite goal remain incomplete. No full compiler/LSP or
benchmark matrix, remote CI, push, tag or release was triggered.

Accepted local editor source checkpoint: d26613f7. The current normal server hash is
88a39d6439bfb5729d896c4167f32c3e026c044066f033c197cb2ed80326cfd1.
Installed LSP refresh failed because its executable is in use; the copy did not
succeed, and the installed consumer is not claimed updated. Preserve its running
editor session. Candidate remains target/debug/tsrust-lsp.exe; install after the
existing server releases the Windows image lock. Do not start an extra LSP.

## Table native/content acceptance in progress

The compiler ui_content substring batch 19308 is terminal: five failures in 164.77s.
Three report Access denied; two documentation cases reject out-of-project relative
links introduced in installation.md. An unchanged exact Table rerun with native
permissions reaches real execution and fails Unsupported UI utility: h-10.
Logs: target/table-native-installer-docs.log and table-native-installer-permitted.log.

The pending repair adds missing scale token 10 to the existing shared spacing
list, preserving the pinned Table h-10 class. Four style layout tests pass in 0.03s
(19.45s build), including a new native/CSS-resolved 40px default and 50px custom
spacing witness, plus existing all-token/property-order coverage. Editor tokens
and docs use this same list. Installation's contributor/editor links now point to
the repository's actual GitHub documents; the project link-escape guard is unchanged.
No catalog source or immutable payload identity was rewritten.

Affected compiler batch is live in session 71922 with native permissions, shared
cache, one job, no incremental; log target/table-native-installer-docs-repaired.log.
It rebuilds the compiler with current shared UI sources and retries the five cases
(the substring also matches executable). Do not restart on quiet output. Inspect
the exact terminal result before accepting or repairing. Whole-project formatting
reports zero files and fingerprints changed. Current uncommitted files are
crates/tsrust-ui/src/style.rs, style/layout.rs, installation.md and this evidence.
After acceptance, update evidence, checkpoint locally, and continue actual browser
Table acceptance. Installed LSP refresh remains image-locked, as recorded above.

Browser prerequisites now exist on this computer: rustup target add
wasm32-unknown-unknown --toolchain 1.96.1 succeeded and installed-target inventory
confirms it. wasm-bindgen 0.2.129 matches both lockfiles. Official GitHub release
metadata (https://github.com/wasm-bindgen/wasm-bindgen/releases/tag/0.2.129) identifies
the Windows archive SHA-256 as
79e348c169d0c10c0b287647f1e753d0985353ca96800bc29be8f4261baf13e6;
local bytes match. Only the explicit regular wasm-bindgen.exe archive member was
installed, with exclusive creation, to C:/Users/skeet/.cargo/bin/wasm-bindgen.exe.
Version check passes; executable SHA-256:
7923ed91d8ece7a2986dc5de9395bff2a8e5cae483330b13d0ec97a955b0e351.
No toolchain/compiler versions were changed. The verified disposable download is
target/tool-downloads/wasm-bindgen-0.2.129-x86_64-pc-windows-msvc.tar.gz; remove after
acceptance, preserve the installed tool. Actual browser build/interactions remain
pending. Use one existing shared Cargo target, explicit WASM_BINDGEN and TSRUST,
and the current compiler after batch 71922 completes.

Latest verified batch 71922 observation: compiler rebuild completed in 7m57s;
keyboard documentation, semantic-content native/offline installation, and Table
documentation native execution all pass. current_table_exports_keep_spans_scope_
and_owned_children still fails; its captured panic is not printed until the batch
ends. The final native renderer bundle case remains running; write_stdin(71922)
confirmed it live. Re-poll this same handle, then inspect the complete log for the
first failure. Do not rerun the three successful cases or restart the final case
solely because this turn ends. No repaired Table installer/ownership acceptance
is claimed yet. The unrelated user-owned installed tsrust dev process was seen
while testing and was preserved; do not treat it as abandoned cleanup output.

## Bound Table structure repair, current acceptance

Batch 71922 is terminal: four pass, one fails, 339.67s after the 7m57s compiler
build. The native renderer bundle passes in addition to the earlier three cases.
An exact labeled rerun (88158, terminal exit 101, 21.28s) identifies the sole
failure as nested table rejects; all ownership/span/HTML assertions precede it.
Original checks were retained and their messages made specific in ui_content.rs.

Root cause: with_class_binding checked utility spelling and stored evaluated
classes, but did not validate structure after classes arrived. Editable Table
components apply semantic tags before their reactive class expression, so initial
nested-table and malformed-row guards were bypassed. Initial binding now calls
the existing validate_structure immediately after class assignment. Refresh already
validates structure before returning. No new parser, adapter policy or guard
weakening. Eight selected model tests pass, including rejected bound nesting/row
structure and exact evaluator-capture disposal; log target/table-bound-structure.log.

Only the remaining generated-native case is rebuilding/running in session 77058,
log target/table-ownership-structure-repair.log, one job, no incremental, shared
native cache and native permissions. Reuse that handle. Do not repeat the four
passing compiler cases. The compiler embeds changed UI source, so its dependent
rebuild is necessary; do not launch competing builds. Same 8 GiB private/4 GiB
free memory and 15-minute investigation bounds apply.

A durable paired interaction entry now exists at
docs/src/acceptance/table-interaction.tsrx. It imports the actual InvoiceTable,
TableStates and TableOverflow docs examples, keeps a narrow overflow container,
and adds a theme toggle. Whole-project formatting (one new fixture, zero changed
fingerprints) and the formatted entry's compiler check pass. Native/window and
browser interaction are still pending. After 77058 completes, use its current
compiler with tools/build-ui-web.mjs and installed pinned Wasm prerequisites;
keep one Cargo target. Add the new entry to formatting acceptance as appropriate.

Latest session 77058 inspection: Cargo 31052/21372 remains live, started 16:39:27
local. Compiler library finished; rustc 22216 is building the normal CLI at
opt-level=1, started 16:45:55. At inspection it used 2,665 MiB private memory and
76.48 CPU seconds, so the quiet log is not terminal. Re-poll 77058 before launching
anything. Investigate at 15 minutes from initial build start (16:54:27 local),
checking CPU progress and free memory; do not blindly restart. The interaction
entry is now registered in the existing documentation formatting fixture list.
Current uncommitted source also includes lib.rs, table_semantics_tests.rs,
ui_content.rs labeled assertions and new table-interaction.tsrx. No new source
checkpoint claims the generated-native repair accepted yet. Preserve owner files.

When the current native witness passes, refresh paired evidence and checkpoint.
For the later LSP refresh/build, selecting both compiler and LSP with their shared
ui_content no-run targets may reuse the current compiler dev-dependency graph;
avoid alternating feature graphs and unnecessary compiler rebuilds. Run only the
required exact LSP case afterward, retaining earlier passes. Installed LSP remains
image-locked; do not interrupt unrelated editor work to copy it.

## Table shared construction accepted; owner requests live docs

Session 77058 is terminal, exit 0: 11m20s required compiler/CLI rebuild, then the
exact generated-native Table witness passes in 31.55s. Nested-table rejection,
span/HTML semantics and exact evaluator-capture retention/release all pass with
original assertions. The initial-bound-class structural guard is accepted with
the eight model passes; the four earlier compiler/content/native-renderer passes
remain retained. No full compiler/LSP matrix or timing milestone is claimed.
Whole-project second formatting pass and format . --check pass with zero changes.
Current normal compiler and refreshed installed compiler SHA-256 match:
d2e248d53fe071988a621199663ee7301fc57e4654320c997f353d11d9c7e5c1.
Installed compiler was idle before refresh. The installed LSP remains separately
image-locked, not claimed refreshed.

The owner's latest priority is keeping the full docs site running in both web
and desktop while work continues. A persistent installed-compiler native dev
supervisor is live in session 45919, cwd D:/ai_dev/tsrust/docs, log
target/docs-desktop-preview.log. It watches the real docs project with --target
native --verbose, one job, no incremental, TSRUST_CACHE_DIR pointing to the retained
target/tsrust-native. First build is in progress. Do not stop this requested
preview during routine cleanup. Next: inspect its build/window, then start the
web dev supervisor on loopback port 4317 with CARGO_TARGET_DIR set to the existing
shared target and explicit installed WASM_BINDGEN. Port 4317 was free before start.
Build the initial previews sequentially; keep both active once ready. The paired
interaction entry remains useful for focused checks, but full docs previews are
now explicitly requested. Record exact live handles/PIDs and public local URL.
