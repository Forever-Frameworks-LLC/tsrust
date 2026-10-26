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

`tsrust dev` currently gates the entire supervisor to Unix process-control code.
Windows watcher/process-tree cleanup remains a separate concrete implementation
section. Native run/build acceptance does not imply watcher or Windows GPU-window
acceptance. Existing compiler/Table help and hosted-CI failures remain recorded;
they are not hidden by this installation work.

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

New Windows scaffolds recommend `tsrust run`; Unix scaffolds retain `tsrust dev`.
The external CLI smoke now checks the host-specific start hint and executable
suffix in addition to scaffold/check/format/native run/test/bundled JSON behavior.
Windows watcher and GPU-window acceptance remain pending separate implementation.

The final compiler reinstall completed in 1m44s. The installed-consumer smoke
passes after that rebuild, including the corrected Windows scaffold hint and
`.exe` output assertion. Whole-project formatting ran twice with zero source or
fingerprint changes; native behavior is verified on formatted sources.

Installed SHA-256 identities:

- Compiler: `d7ba18e2472ec8b7c785de74f66a9034d8753565e415f30e7d40415a495ceadb`.
- Server: `8634b56daad4815cf445278b2a151b96147a69719aa7dcf34b4a7ba299ef30fe`.
- VSIX: `b645166c4bb17fb9174ba70105ef0b2a679510e2ac94f7a1b31c8a4cdf201eda`.

These identify local development consumers built from this installation section,
not published release artifacts. The CLI includes the preceding Windows UI-path
repair as well as the new scaffold hint. No benchmark workloads changed, so this
section adds no measurements or benchmark timing run.
