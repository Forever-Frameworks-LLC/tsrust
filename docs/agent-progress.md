# Agent continuation state

## Direction and checkpoints

Follow agent-goal.md, plans/backend-only-reset.md and
plans/rust-setup-and-dependency-distribution.md. Backend-only direction supersedes
frontend integration work. The complete objective remains open; do not mark it done.
No releases, pushes, tags, package publication or automated recurrence is authorized.
One agent writes this tree. The owner explicitly requested committing the entire
working tree, including .gitignore, docs/src/main.tsr and .agents/. Preserve their
semantics; do not keep excluding them from checkpoints. Never stop owner processes.

Accepted checkpoints: 8904fb8a organized 333 loose documentation files with no new
broken links; 514cb10a added doctor and explicit pinned-toolchain setup. The editor section is accepted in 0c5270b9; see milestones/docs-project-indexing.md. Source checkpoints use
node tools/format-project.mjs with the current verified TSRUST; maintain one Cargo
target, CARGO_INCREMENTAL=0 and one build job. No full matrix is justified after each
routine edit. Keep analytics in SQLite; do not import current-gate logs or snapshots.

## Current editor acceptance

The installed LSP stalled beyond 120s on docs while the compiler entry checked in
22117ms. Workspace-disabled reproduction ruled out scanning alone. Local timings
showed compiler checking finished near 17s and checked-fact application blocked
publication after declaration indexing. The new index avoids applying unrelated
named-record facts to every loader, reuses completed structural expansions and
preserves expanded JSON property help. Reverted a line-table experiment after it
failed to fix the stall. Baseline confirmed three fixture/JSON failures already
existed; Windows virtual URIs, obsolete whitespace expectation and JSON help are
now corrected without weakening production URI checks.

198 affected cases pass: project intelligence 178, interfaces 3, schema editing 15,
text imports 2. Actual local LSP docs clean/error/repair and backend-async entry/API
clean/error/repair pass, disk files untouched. The first docs publication is still
about 75 seconds for 1668 modules; this is a diagnostic observation, not a benchmark.
No whole-editor Problems claim: owner reports about 40, and no VS Code MCP tool is
exposed/configured here. The 16 authoring-only UI build warnings and native docs
0xc00000fd stack overflow remain open. No diagnostic suppression or indexing quota
was added. The verifier's default publication deadline remains 15s; docs explicitly
uses 120s. Opt-in TSRUST_LSP_TRACE_TIMINGS emits only local stage observations.

All owned sessions are terminal, including setup 48141; LSP builds 4699/6663;
regression sessions 49121/63164/73386/64824/45957/4597; docs probes
19476/20831/39988/85098/14845/49425/13764/84850/7355. Do not poll or restart them.
The owner's installed LSP process is unrelated and must remain running. No owned
server or UI window is live. Logs: target/lsp-project-help-current.log,
target/docs-editor-visible-facts-current.log, target/backend-editor-current.log.

## Installed tools

Validated CLI is installed at C:/Users/skeet/.cargo/bin/tsrust.exe, SHA256
626C0008849B3A075B8D3501460C693D2214549B9F6C9346FEC9064F610B6DEF.
Installed doctor --json passed. Its setup unit 1/1 and CLI integration 2/2 passed;
actual doctor --native --json found Rust/Cargo 1.96.1 and compiled/linked/executed a
minimal host program without network. Setup process fixtures prove explicit consent,
exact minimal-profile/no-self-update arguments, success and failure handling. No
live toolchain installation, global defaults/configuration change or network setup
was performed. Application dependency readiness is explicitly unchecked.

The validated LSP is now installed at C:/Users/skeet/.cargo/bin/tsrust-lsp.exe,
SHA256 E5966160755F0EF19B5E245348150DD0B9640413238902E7EB8595F15600D41B.
The installed executable passed the backend-async clean/error/repair probe as well
(target/installed-backend-editor-current.log). Windows allowed hash-guarded staging
and rename/replacement without stopping the owner's running LSP or editor. The old
mapped image remains at
C:/Users/skeet/.cargo/bin/tsrust-lsp-running-0a62d1de9bc945a680da390dec72ad54.exe,
SHA256 046c8e50e1d867988cb72ecbcc1f6d28b66f36cb8a1a078ba8d43b7a27f596a9.
Delete that exact old binary only after confirming its process has exited and its
hash is unchanged; do not stop the process for cleanup. The live owner session has
NOT automatically adopted the repair: a user-initiated LSP restart/reopen will load
the canonical new binary. No workspace/editor settings were changed. This local
consumer refresh does not implement trusted production self-update or prove zero
Problems in the owner's current editor.

## Distribution work next

Immediate full-tree checkpoint requested by owner: backend_sources.rs now contains
small metadata materialization, namespace validation and hashed Cargo config/env
provenance. Build/dev executable cache keys call its application_identity helper.
Runtime preparation and setup are not yet connected to the new metadata; several
helpers are consequently unused until that integration is finished. No source-cache
acceptance tests or compiler build have run for these partial edits. This checkpoint
records unfinished work explicitly; it is not an accepted implementation milestone.


Prepared untracked crates/tsrust-compiler/backend-dependencies/{Cargo.toml,Cargo.lock,
src/lib.rs}. Cargo pruned the seeded original lock offline from 531 to 266 packages;
all retained versions, sources and checksums match the original closure. Offline
--locked --all-features metadata succeeds and contains no wgpu, winit, vello, parley
or taffy. This profile is NOT YET CONNECTED to setup or native builds. The owner requested
an immediate full-tree checkpoint, so these in-progress sources are included;
do not claim runtime integration or acceptance is complete.
Disposable metadata is reduced to target/backend-dependency-metadata-current.json.

Next implement a coherent shared-source family: explicit dependency preparation,
normal locked/offline Cargo builds against the real shared Cargo home, config/env
provenance in artifact cache keys, a separate legacy UI path, and removal of mandatory
CLI vendor embedding. Preserve portable offline exports (explicit prepared vendor
packs can be reused without invoking Rust during export), exact binding locks,
notices, relocated rebuilds, artifact packing and Windows import-library restoration.
Normal check/emit/build/dev must never silently install or fetch. The last ordinary
CLI integration build took 10m31s, so stop rebuilding the same giant payload for
small CLI edits. Inventory is plans/backend-distribution-inventory.md. After that,
implement trusted paired CLI/LSP installation and latest-stable update with Windows
locked-binary handling, ownership and failure recovery; no publication as a test loop.

## Other required backend work retained

The server should own its API in app/, expose its description/OpenAPI, and let users
independently generate js-client/ or rust-client/ anywhere. Replace the public
generated/server prerequisite with a checked server API facility; compiler-internal
lowering is acceptable. Current scaffold still uses the old generated server tree.
Do not present moving/renaming that tree as the architectural correction.

Async SDK section de81e66f is accepted; see milestones/async-http-sdk.md and
milestones/backend-only-http-sdk.md. Fresh review app is generated-projects/backend-async.
Actual Rust old/new replica execution remains pending; TypeScript replica coverage
is prior evidence. HTTP I/O remains async throughout. Preserve original UI source,
owner review apps and historical evidence; no archive directories or snapshots.
Future UI work starts with a finite set of reusable primitives before larger widgets.

After distribution/server-boundary work, return to async std:testing registration
(no unawaited callbacks or nested blocking runtimes), migrate pre-library executable
assertions to normal tests, and implement checked HTTP JSON body readers. Text/form/
multipart/owned streaming exist; contextual body transfer and response use need
ownership repair. XML needs an upstream parser-backed plan. See
plans/http-request-body-readers.md. Preserve meaningful compiler rejection cases,
actual process-boundary fixtures and benchmark drivers.
