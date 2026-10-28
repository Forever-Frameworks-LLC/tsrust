# Windows installation and local integration

Owner priority, 2026-10-26: make Windows integration and installation work on
the current computer before resuming the component-library sequence. This is a
local development installation, not authorization to publish a release.

## Acceptance checklist

- [x] Install both source-built consumers in the existing Cargo bin directory and
  verify discovery from PowerShell and the editor without workspace-specific paths.
- [x] Check the real MSVC native pipeline outside the checkout: scaffold, check,
  format, native execution, tests and bundled JSON all pass with the existing CLI.
- [x] Preserve `.exe` names in Windows packages and verify exact manifest hashes.
  All six distribution tests pass, including existing preflight/dirty-tree guards.
- [x] Verify editor executable discovery, including spaces/Unicode and rejection
  of invalid explicit paths. All four discovery tests and TextMate grammar pass.
- [x] Package the VSIX with the existing formatter defaults extended in place.
  The first build exposed a duplicate configuration-defaults block; it was removed
  before the clean successful rebuild/package.
- [x] Run the packaged extension in a real Windows VS Code host, including unsaved
  diagnostics, fixes, formatting, imported assistance and native Run File tasks.
- [x] Install that verified VSIX into the user's normal VS Code installation and
  leave a runnable small project ready for use.
- [x] Complete source installation documentation, record consumer identities and
  save a formatted local checkpoint. Keep unsupported Windows features explicit.

## Implementation and scope

The existing Rustup MSVC 1.96.1 toolchain and Visual Studio 2022 installation pass
the native smoke rather than only executable-presence checks. Cargo's standard
installation mechanism is reused; no second installer or custom PATH manager is
introduced. The existing Cargo bin directory is already available to the shell.

Packaging now selects executable suffixes from the requested target, preserving
Linux/macOS names. Editor test fixtures also use host executable suffixes. The
packaged-extension runner invokes the Windows Electron CLI entry directly, keeping
paths out of cmd.exe, and defaults to the shared target with a Windows suffix.
The extension contributes its native language-specific formatter default while
preserving explicit user choices.

The Windows supervisor now shares the existing frontend/build/replacement loop,
with native Windows notifications, file stamps, console handling and Job Objects.
Native console acceptance does not imply Windows GPU-window or web prerequisite
acceptance. Existing compiler/Table help and hosted-CI failures remain separate.

## Resource and evidence policy

Use the shared `target/`, CARGO_INCREMENTAL=0 and one Cargo build job. Keep an
8 GiB private-commit build bound and at least 4 GiB available physical memory;
investigate a build without completion after 15 minutes rather than blindly retrying.
Editor validation uses an isolated small workspace and its existing request/task
timeouts. Do not activate an unbounded repository scan to test installation.

Current command logs are disposable under `target/`; no installation logs, cleanup
receipts or source snapshots are imported into analytics. Public measurements and
legacy evidence stay untouched. No push, workflow dispatch, tag or release occurs.

## Windows acceptance evidence (2026-10-26)

Installed development consumers use the existing Cargo bin directory:
`C:/Users/skeet/.cargo/bin/tsrust.exe` (compiler 0.1.0) and
`C:/Users/skeet/.cargo/bin/tsrust-lsp.exe` (server package 0.2.0).
Both resolve from PowerShell; no custom PATH manager or workspace server setting
was needed. Builds use Rust 1.96.1 MSVC, one job and disabled incremental compilation.

The packaged VSIX passed a real Windows VS Code integration run with automatic
installed-consumer discovery, exit 0. Coverage includes unsaved ownership/type/syntax
diagnostics, UTF-16 positions, multiple documents, repairs, rapid edits, restart,
close/reopen, missing-server handling, documentation hover, signatures, completion,
definitions, references, outline, imported modules and native Run File tasks.
The Result fixture now tests a real Result parameter separately from schema's
validation union, preserving immediate completion and payload assertions. Document
positions use actual editor text so Windows CRLF does not invalidate hover offsets.
The clean final run is `target/windows-vscode-integration-final.log`.

The same package is installed in the normal user profile as
`tsrust-local.tsrust@0.2.0`. The small `examples/windows-setup` project is open in
a new VS Code window. Its native greeting and assertion-based test pass. Existing
user settings were preserved. An installed server was observed at 46,112,768 bytes
private memory before opening the small workspace; this is a bounded installation
observation, not a long-session memory acceptance claim.

The initial Windows scaffold repair recommended `tsrust run`; the accepted
supervisor now lets all supported hosts recommend `tsrust dev`.
The external CLI smoke now checks the host-specific start hint and executable
suffix in addition to scaffold/check/format/native run/test/bundled JSON behavior.
Windows watcher acceptance is recorded below. GPU-window acceptance remains pending.

The final compiler reinstall completed in 1m44s. The installed-consumer smoke
passes after that rebuild, including the corrected Windows scaffold hint and
`.exe` output assertion. Whole-project formatting ran twice with zero source or
fingerprint changes; native behavior is verified on formatted sources.

Initial installation SHA-256 identities (superseded compiler below):

- Compiler: `d7ba18e2472ec8b7c785de74f66a9034d8753565e415f30e7d40415a495ceadb`.
- Server: `8634b56daad4815cf445278b2a151b96147a69719aa7dcf34b4a7ba299ef30fe`.
- VSIX: `b645166c4bb17fb9174ba70105ef0b2a679510e2ac94f7a1b31c8a4cdf201eda`.

These identify local development consumers built from this installation section,
not published release artifacts. The CLI includes the preceding Windows UI-path
repair as well as the new scaffold hint. No benchmark workloads changed, so this
section adds no measurements or benchmark timing run.

## Windows development supervisor acceptance

Based on local checkpoint `17f6f315`. This ports the existing supervisor rather
than creating a second development loop. Windows consumers start suspended, join
a private kill-on-close Job Object, then resume through their owning primary
thread. Ctrl+Break requests bounded shutdown; forced job termination covers
remaining descendants, including orphans. Console Ctrl+C/Ctrl+Break set the
supervisor stop flag. Unicode recursive notifications request reconciliation;
Windows volume/file IDs, length, creation/write/change times preserve exact
snapshot invalidation even when a same-size edit restores its write timestamp.
Only existing windows-sys 0.61.2 bindings are enabled as a direct Windows dependency.

Acceptance passes in retained batches: platform compilation; sixteen focused
frontend/watcher/web-server/process/publication cases; formatted scaffold smoke;
and native development with failed-edit preservation, replacement and shutdown.
`tools/smoke-windows-dev.py` owns its hidden console and disposable workspace.
No GPU/window or web prerequisite acceptance is implied by these checks.

Platform contracts consulted during the port:

- [AssignProcessToJobObject](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject): nested jobs and inherited descendant membership.
- [FindFirstChangeNotificationW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstchangenotificationw): recursive change notification and explicit rearming; root removal requires separate validation.
- Rust's `spawn_with_attributes` remains experimental, so this implementation uses
  stable Command creation flags and a suspended thread. Job assignment occurs
  before the child executes. No nightly Rust feature or custom command-line
  quoting implementation is introduced. Unix signal/group behavior is unchanged
  but has not been executed on Windows.

Final Windows supervisor evidence:

- First focused batch: 11 passed. Two atomic-save and one localhost socket case
  were denied by sandbox policy and all pass unchanged with native permissions.
  Preserve those exact reruns; no assertions or workload counts were weakened.
- The documentation watcher exposed generated backslashes at an authored-path
  guard. Generated joins now reuse the existing portable-path helper after
  validating authored segments. The exact watcher passes (0.03s).
- The newly enabled publication witness then exposed native separator spelling
  in serialized source-page comparisons. Portable identity comparison repairs it;
  content mutation/addition/deletion and traversal/backslash guards pass (19.65s).
- The supervisor canonicalizes its root to match Windows extended-path source IDs.
  `tools/smoke-windows-dev.py target/debug/tsrust.exe` passes with a path containing
  spaces and Unicode, live native execution, failed-edit preservation, successful
  replacement, Ctrl+Break cleanup, restart and real Ctrl+C shutdown. Its hidden
  consoles and temporary workspace are closed and removed.
- `node tools/smoke-toolchain.mjs target/debug/tsrust.exe` passes scaffold, check,
  format, native run, project tests and bundled JSON. Whole-project formatting
  reports zero source/fingerprint changes, and installed `format . --check` passes.
- The normal CLI was built through a no-run integration target to reuse the
  already compiled test dependency graph. No Unix-only integration test was
  reported as executed on Windows. A transient file-edit encoding mistake was
  repaired from original source bytes before the final build. Original Unicode
  help bytes are verified; UI labels remain byte-preserved in the source diff.
- The existing Cargo-installed compiler was refreshed from that verified normal
  CLI, without changing its PATH or Cargo package registration. Target and installed
  SHA-256 are identical:
  `9885034dd785f8eb19e959ae0125514ec913fb10c6efd3740a5b0009712f5fba`.
  Installed compiler reports 0.1.0; server remains the previously accepted 0.2.0.
- Current concise logs: `target/windows-dev-focused.log`,
  `target/windows-dev-doc-watch.log`, `target/windows-dev-doc-publication.log`,
  `target/windows-dev-cli-build-final.log`, `target/windows-dev-native-smoke.log`
  and `target/windows-dev-scaffold-smoke.log`. They are not analytics records.

The initial 15m22s focused build crossed its investigation threshold while
progressing through library and CLI object generation. At investigation, memory
was about 2.5 GiB with over 12 GiB free; the same live build completed. Subsequent
CLI-only rebuilds took about two minutes. No duplicate full pass, benchmark timing,
push, hosted workflow, tag or release occurred. Windows GPU/window and web toolchain
validation remain distinct next sections; Unix execution remains unverified here.

Post-checkpoint cleanup (`a1a6cc4e`): four completed-gate unit/integration test
executables and symbols removed after exact-path inventory and process checks.
Logical bytes removed: 930,684,416; observed free-space increase: 930,689,024.
Compiler/LSP consumers, warm dependencies, public measurements and owner files
remain preserved. Test programs are reproducible from the source checkpoint.

Next Windows native-renderer section is bounded separately: RTX 4090 with driver
32.0.16.1742, existing auto-closing `static --verify-window` witness, shared target,
one Cargo job, incremental disabled, 8 GiB private / 4 GiB free memory bounds,
15-minute build investigation and 30-second post-build runtime limit.

Windows native renderer foundation passes: offline `cargo build --locked -p
 tsrust-ui --features acceptance --example static` (3m53s), followed by the existing
`static.exe --verify-window` with a 30-second external timeout. It presents frames
through its resize sequence and exits normally, printing `GPU frame presented;
event loop closed`. No claim of complete glyph, input or generated-source parity
is inferred from this narrow renderer witness. A generated native UI project now
checks/formats successfully; its offline application backend is accepted below.

## Generated Windows UI and native import libraries

Based on adc0f5d5, the generated app initially failed LNK1181 for
windows.0.52.0.lib after its complete offline runtime compile. The pinned compiler
bundle already contains that library. The shared runtime linker now restores
host-matching MSVC imports beside dependency rlibs and adds their native search
path. Ordinary builds, dev and package validation share this fallible operation.
Unexpected existing files and symlinks are rejected without replacement. Other
hosts and runtimes without windows-targets are unchanged.

The existing TSRLIB2 format and native dependency identity remain valid. The
focused artifact witness proves exact import bytes after pack/unpack, repeat
restoration, and preservation of unexpected owner bytes. The two artifact tests
pass; log: target/windows-native-import-tests.log. The current compiler also builds
through the normal integration dependency graph; no Unix integration execution is
claimed. No cold runtime compilation was repeated.

The current normal compiler successfully builds target/windows-ui-setup using
TSRUST_CACHE_DIR=D:/ai_dev/tsrust/target/tsrust-native, one Cargo job and incremental
compilation disabled. target/windows-ui-generated-link-repair.log records
runtime=hit and successful linking in 14.88 seconds. Its generated source had
already passed check and stable formatting. The retained native workspace is
runtime/build-v1/6be45785ffc2ba6b; compiled dependency caches remain preserved.

Computer Use inspected the actual generated Tsrust UI window. The first observed
counter value was 1, so no initial-zero assertion is claimed. One mouse click
changed 1 to 2, focused Space changed 2 to 3, and Return changed 3 to 4. Captures
showed legible Count text, the Increment button and its keyboard focus ring.
Accessibility exposed the named button and focused element. Clicking the observed
Close control exited the owned process 0. The initial Computer Use approval timed
out; the owner requested another attempt, then access succeeded. No claims about
clipboard, all glyphs, complete accessibility semantics or other platforms follow
from this counter witness.

Whole-project formatting reports zero changed files and fingerprints. The verified
normal CLI refreshed the existing Cargo-installed compiler; source and installed
SHA-256 match: 1f04114cd77b11d91d11133ce8a43fb49c8dbec03ca911e452ce4837a395fa78.
Installed version remains 0.1.0. No release, tag, push or hosted CI was used.
