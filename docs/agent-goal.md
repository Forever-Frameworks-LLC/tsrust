# Goal: working Tsrust backend application prototype

Owner-approved direction: 2026-10-26. Use this file as the continuation goal.

Build a working prototype in which I author backend logic in Tsrust's
TypeScript-style language, compile it to native code, and consume it through
generated named TypeScript API calls from React web, Electron desktop and Expo
mobile. Keep Tsrust as the backend language. Use React/Vite and shadcn for web
and desktop, Electron Forge with its Vite plugin for desktop packaging, Expo and
React Native Reusables for mobile, and TanStack Query behind a thin generated
wrapper with named calls/hooks. Electron Forge's Vite plugin is explicitly accepted.

Read AGENTS.md, docs/agent-progress.md,
docs/plans/backend-platform-pivot.md and docs/plans/backend-platform-packages.md.
Use the local development, Ponytail and Caveman guidance, except that the owner's
pivot supersedes old instructions to finish the universal UI and avoid React.
Continue from current source and accepted backend evidence. Do not restart the
compiler, finish arbitrary TypeScript/npm compatibility, or build another frontend
framework. Preserve old UI source, owner edits and historical acceptance in place.
Do not delete or archive it merely to clean up the tree. Table, custom components,
renderer/embedded parity and the old docs application are frozen, not passed.

## Finite prototype acceptance

1. Isolate the active compiler/LSP/backend path from the legacy UI runtime while
   retaining authoring compatibility and explicit legacy UI builds. Trace direct,
   embedded-source and vendored dependency paths. Record remaining coupling.
2. Build a native Tsrust SQLite-backed task-list service with list/create/update/
   delete operations, validated inputs, structured public failures and an explicit
   application-owned access policy. Expose only declared operations. Reuse existing
   HTTP, async, schema, SQL and resource contracts instead of parallel runtimes.
3. Derive one checked operation manifest and generate deterministic TypeScript
   clients, declarations and runtime codecs. Ordinary calls look like
   `api.tasks.list(args)` and `api.tasks.create(args)`, with a framework-independent
   Query Core options entry, generated React hooks layered above it, and typed
   TanStack query/mutation options. No string command names, manual
   query keys, duplicated DTOs or handwritten transport wrappers are required.
   Keep the plain client framework-independent. Settle exact spellings with
   executable TypeScript witnesses and documentation.
4. Define and test safe wire mappings, null/optional behavior, supported unions,
   integer limits and unsupported resource rejection. Validate incoming arguments
   and client responses. Preserve authorization through all transports.
   Stale generation and incompatible API versions must produce useful failures.
5. Deliver a usable React/shadcn task-list application with loading, empty/error
   states, editing/validation and keyboard-accessible controls. The native service
   can serve production assets; development uses upstream Vite HMR. Keep API
   routes distinct from SPA navigation fallback.
6. Package the same web UI with Electron and its native backend. Main/preload
   exposes only generated allowed operations to an isolated sandboxed renderer.
   Start with authenticated loopback HTTP behind preload; implement private stdio
   only if a concrete requirement or measurement justifies it. Prove readiness,
   startup failure, crash, exit, shutdown/drain and orphan cleanup. A packaged
   Windows app must work outside the repository without Rust or Node installed.
   Include optional updater setup and Tsrust-hosted feed/artifact integration
   using the selected upstream installer/updater; follow the signing, lifecycle
   and acceptance boundaries in plans/native-targets-and-secure-updates.md.
7. Build an Expo/React Native Reusables task-list client using the same API over
   HTTP. Prove it on an Android emulator/device initially. Do not claim iOS
   acceptance without macOS/simulator/device evidence. A bundled Tsrust mobile
   runtime and offline synchronization are not prototype requirements.
8. Add one bounded long-running operation with cancellation and resource cleanup.
   Distinguish cancellation from rollback or cleanup completion. Subscriptions
   are a separate follow-up unless a required prototype behavior needs them.
9. Offer Tsrust convenience commands for setup/import, dev, build/package,
   component installation and integration upgrades. Generated/imported apps are
   normal user-owned frontend projects: users can cd into them, install or upgrade
   dependencies and edit scripts/manifests/locks independently. Respect their
   toolchain/package manager; a managed toolchain is optional convenience. Tsrust
   owns initial integration, generated clients and optional migrations for changes
   affecting Tsrust integration, never unrelated dependency policy. Preview scoped
   migrations and preserve user edits. Provide configurable static mounts and a
   source-level API so one backend can serve multiple separately built web apps,
   with isolated fallbacks and routes. Verify two app mounts coexist with the API.
10. Support existing projects: configure backend/client-only projects without
    generating UI, generate/export a client into an explicit folder, pack a
    versioned standalone npm-compatible client for optional publication, and
    link/import it from an existing project brought into the workspace. External
    React, Expo or other TypeScript apps can install the exported package without
    adopting Tsrust's frontend toolchain. Preserve their package manager, lockfile,
    scripts and source; Tsrust orchestration is optional during adoption. Actual
    registry publication requires explicit authorization and credentials. Local
    pack/import acceptance must not publish anything.
11. Use researched upstream templates/packages and record exact tested compatible
    versions, licenses, provenance/integrity and resolved advisory review. Record the
    tested starter matrix and integration compatibility ranges, preserve user edits
    and prove one scoped integration migration plus rollback from Git. User-driven
    dependency additions must keep working without Tsrust adoption/approval.
12. Verify from a clean generated project and a separate existing-project client
    consumer. Show an API edit propagating through generation/type checking;
    preserve compatible older clients for rolling mobile/web updates. Record
    commands, native output, real UI/package evidence and platform gaps. Update
    documentation and generated agent guidance.

## Completion and working rules

Implement coherent sections across compiler/runtime, editor, client generation,
templates, examples and focused witnesses before batch validation. Formatting
with token preservation/idempotence, native behavior, ownership, IntelliSense and
compiler/editor agreement remain required on affected surfaces. Preserve passing
evidence; repair and rerun only affected checks.

At the backend-family milestone run appropriate full compiler/LSP regressions and
matched native versus Node-after-tsc measurements. Keep backend computation,
serialization, HTTP/IPC overhead and packaged startup measurements separate.
Native compilation is not a universal speed guarantee. Preserve original samples
and protocols; distinguish acceptance from unmeasured workloads. SQLite remains
the canonical benchmark store; publish only the allowlisted export.

The owner requested their computer back after the editor refresh. Continue with
file/terminal tools and headless verification. Do not resume interactive desktop
control unless the owner requests it.

One agent writes this tree at a time. Inspect status, preserve unrelated changes,
use local Git checkpoints before risky edits and after accepted sections, format
the entire project before source checkpoints, use one Cargo target/job and
CARGO_INCREMENTAL=0, and prune obsolete outputs before handoff. Keep routine logs
concise and disposable. No source/binary archive trees are needed.

Continue while useful work remains. Keep docs/agent-progress.md current with source
identity, exact checks/results, remaining coupling, platform gaps and the next
executable action. Do not claim completion until all required items have evidence.
Record missing prerequisites and continue independent work; never invent platform
passes or silently weaken acceptance.

No timer pushes, release tags, public releases, package publication or recurring
automations are authorized by this goal. Use remote CI only for a coherent locally
validated section with a specific platform uncertainty; record exact inputs and
reuse unchanged evidence. Releases/publication need explicit owner instructions.
