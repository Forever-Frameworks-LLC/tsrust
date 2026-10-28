# Backend-only reset

Owner direction, 2026-10-27; continuation explicitly authorized after goal alignment.
This supersedes React/Vite/shadcn, Electron/Forge,
Expo/RNR, TanStack wrappers, frontend package-manager selection, frontend migration
and multi-target UI orchestration requirements in earlier plans. The previous
automated goal metadata still needs replacement/resumption in the app: available
agent goal tools cannot edit the objective or resume a paused goal. This document
and agent-goal.md govern the owner's directly authorized ongoing work.

## Implementation order

1. Checkpoint the existing work, including unvalidated layout changes, after required
   whole-project formatting. Preserve owner edits and original Tsrust UI history.
   Inventory introduced UI templates, SDK adapters, dependencies, demos, tools,
   docs and registrations before deleting them. Remove active React/Electron/Expo/
   npm scaffolding and their integration tooling. Retain the backend starter,
   checked HTTP dispatcher, independent clients, API compatibility, native tests,
   static serving where useful to arbitrary external clients, and compiler/LSP.
   Original Tsrust UI stays outside the normal backend dependency/install path;
   future revival requires a separate decision. Do not delete owner-generated apps.
2. Establish one transport-neutral HTTP contract as the generation input. A live
   service can expose its description and OpenAPI; a saved description works offline.
   TypeScript and Rust generators consume the same validated contract, without
   requiring the original Tsrust source or any UI framework. Reuse the existing
   checked manifest/codecs/history; investigate OpenAPI representability rather
   than silently treating arbitrary OpenAPI as equivalent to Tsrust wire types.
   Preserve named calls, request/response validation, authentication customization,
   structured errors and safe rolling-version compatibility. No mandatory cache or
   query library. Define supported OpenAPI subset/import behavior explicitly.
3. Begin the [Rust setup and dependency distribution migration](rust-setup-and-dependency-distribution.md)
   with the dependency-path inventory. Distinguish prebuilt CLI/LSP dependencies,
   application build dependencies, binding graphs and machine prerequisites.
   Implement actionable doctor and explicit setup/resolution; ordinary builds are
   locked/offline against Cargo caches, portable source exports remain vendored.
4. Deliver installation and self-update for Windows, macOS and Linux. Reuse the
   established artifact/release mechanism; inventory supported architectures and
   installer ownership first. Explicit update checking reports the latest compatible
   stable version; explicit updating verifies trusted release metadata and artifact
   integrity, stages replacement and preserves a working installation on failure.
   Account for running/locked Windows CLI and LSP binaries, permissions, interrupted
   updates, rollback and compiler/LSP version pairing. Package-manager-owned installs
   must receive appropriate upgrade guidance instead of being overwritten. Do not
   silently install Rust during CLI updates or fetch during routine build/dev.

## Acceptance

- Fresh backend creation emits no frontend templates, Node manifests/locks or UI
  dependency requirements. Normal compiler/LSP builds exclude UI runtime coupling.
- Existing backend/API compatibility and resource-safety tests remain valid.
  Focused diagnostics and editor witnesses cover changed CLI/config boundaries.
- Fetch a description from a running API, generate TS and Rust clients outside the
  source tree, compile both and call the real backend. Repeat from the saved contract.
  Validate OpenAPI output, malformed contract rejection, errors, auth and supported
  old/new replica compatibility. No framework adapter is required.
- Execute the setup/cache/export/deployment matrix in the linked distribution plan.
- Test installation, check/update, no-update, corrupt/untrusted artifact, interrupted
  replacement, rollback and locked-LSP behavior in isolated installations. Record
  Windows/macOS/Linux and architecture coverage separately; no cross-platform claims
  from mocks alone. Deployed apps need no developer tooling.
- Run whole-project formatting before local source checkpoints. Preserve historical
  evidence; update active help/docs. No releases, pushes or remote CI solely to test.

## Decisions to settle from code and upstream documentation

- Which existing HTTP description/version endpoint and manifest can be reused?
  Which OpenAPI version/subset faithfully represents the supported wire contract?
- Rust client: minimum maintained HTTP/serialization dependencies, async/blocking
  policy, configurable transport/auth and compatibility with supported toolchains.
- Which release metadata provides authenticity beyond a checksum served beside an
  archive? Define trust and recovery before executing downloaded replacement code.
- Installer paths, supported host triples, package-manager detection, proxy/offline
  operation and permission boundaries on each OS; signing/notarization availability.
- Is periodic update notification wanted? Default to explicit checks; do not add
  background network traffic or telemetry without a product decision.

Status: backend scaffolding/cleanup foundations, shared backend sources/portable
exports and [server-owned API integration](../milestones/server-owned-api.md) are
accepted in the recorded Windows scope. The fresh server requires no generated
client or dispatcher directory; live/saved TS and async Rust SDK consumers and
native dev hot reload passed. Complete Rust native replica/auth coverage, checked
body readers/async testing, binding-cache migration and production install/update
remain open. Existing UI acceptance is historical evidence, not new product scope.
