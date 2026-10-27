# Production application baseline

Owner direction, 2026-10-27: stop extending hand-built prototypes, clean up, and
build the actual production implementation from the CLI outward. This changes
implementation order, not the required backend/web/desktop/mobile, SDK, lifecycle,
security, migration, editor or measurement scope in docs/agent-goal.md.

## Reset boundary

Keep accepted compiler/LSP/backend, checked operation manifest, codecs, compatible
SDK versioning, plain client, Query Core/React adapters and static mount support.
Preserve historical evidence and the frozen UI in place. Keep the task applications
as integration fixtures; they are not the default product or a substitute for CLI
generation acceptance. Preserve owner edits and ordinary project/package ownership.
Do not archive source, rewrite Git history or discard accepted work as cleanup.

Remove disposable failed test consumers and stop finished owned previews. Revert
unverified experimental fixes instead of checkpointing them as production changes.
Keep concise failure evidence when it identifies actual remaining behavior. The
attempted Forge dev witness reproduced an empty renderer/navigation failure;
packaged acceptance does not prove dev mode. The attempted exact-origin navigation
workaround did not fix it and was removed. Diagnose again in the real generated
starter, with early renderer/preload/Vite diagnostics and bounded child cleanup.

## Default application

Generate a small landing page with a name field and a typed "greet server" call.
No task database, pagination, token-entry screen or demo-specific DTOs are required
for the default. Keep editable lib/api.ts/provider setup, desktop TitleBar/config,
upstream components, normal manifests/scripts/locks and documented integration
ownership. Do not hide an app's authentication policy in the SDK or make a demo
credential form a universal bootstrap requirement. A public greeting operation
must be explicitly declared public and must not imply other operations are public.

Use the owner's app/, web/, native/ and mobile/ layout for newly created application
projects, adding an explicitly configured generated client package. Existing
projects can retain their own paths and tools. Select targets explicitly; missing
choices must not silently create legacy renderer code. Backend/client-only use
must not pull in a UI toolchain. Retain legacy scaffolding only through its explicit
legacy option, with accurate help and docs. Final command spellings are implemented
and tested together; this plan does not claim new flags already exist.

## Implementation order and acceptance

1. Implement the project configuration and actual CLI creation path for a minimal
   backend plus web greeting app. Reuse current SDK generation and native service
   contracts. Validate choices before writes, never overwrite an existing project,
   and report partial failures accurately. Dependency installation is an explicit
   observable step; preserve ordinary package-manager commands and owner edits.
2. Generate a fresh project through the built CLI. Inspect paths, selected targets,
   public/auth policy, generated package imports, versions and editable integration
   files. Run check/format/test, frontend types/build and editor diagnostics. Start
   visible dev mode and prove greeting, loading/errors, backend and frontend edits,
   clean restart/shutdown and observable diagnostics. A source copy is insufficient.
3. Add desktop to that same generation/configuration path using the admitted Forge
   Vite integration. Resolve the dev-loading failure before calling the target
   accepted. Prove editable full-width header, native caption safe areas, named-only
   sandbox bridge, startup/crash/drain/orphan handling and external packaging. Run
   a newly generated app visibly in dev mode while the owner can interact.
4. Add Expo/RNR through the same boundary, with a real Android device/emulator
   witness, normal dependency ownership and SDK imports. Do not substitute Expo web
   for native Android acceptance or claim iOS without its platform evidence.
5. Complete existing-project import/client export, configured multi-web serving,
   optional orchestration, component installation and scoped integration migration
   preview/rollback. Exercise real CLI choices and preserved user customizations.
6. Complete optional upstream signed update setup/feed integration and bounded
   long-operation cancellation/cleanup. Keep signing/platform/hardware decisions
   in native-targets-and-secure-updates.md. Finish backend-family regressions and
   matched performance review before production-readiness claims.

For every major template, CLI or SDK generator change, create a fresh CLI project,
inspect the output and keep visible dev mode running during template verification.
Fix affected type, runtime and editor problems as they appear. Use LSP protocol/
diagnostic checks without controlling the owner's editor; record locked/stale
installed editor images and unavailable platform evidence honestly. Keep generated
agent guidance accurate about real commands and supported behavior.

Each accepted vertical slice gets its own local source checkpoint after whole-
project formatting and affected checks. Do not add unrelated dependency upgrades,
new runtime frameworks or speculative abstractions. No release/publication or
remote CI is authorized by this reset.
