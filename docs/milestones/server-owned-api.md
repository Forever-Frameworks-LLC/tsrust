# Server-owned API integration

Accepted section on Windows x86_64, 2026-10-27. The full backend/SDK/distribution
objective remains open. This section removes the server's generated-client startup
prerequisite; it does not claim complete installation/update or platform acceptance.

## Result and ownership

A fresh tsrust new service --app owns operations, context and authorization in app/.
Its tsrust.config.json api settings select public operations and compatibility
baselines. app/main.tsr imports dispatch, describe and openapi from api:server and
mounts them on the ordinary native Router. The compiler derives and checks dispatch,
codecs and descriptions in memory; the server never imports a client package and
SDK generation is not required to check, emit, build, test or start it.

The scaffold creates app/, config, README, AGENTS and ignore files. It creates no
generated/, SDK package, frontend tooling or integration marker. SDK consumers choose
js-client/, rust-client/ or another destination, using --url or --description.
Explicit source-based SDK export requires --out when api.output is omitted. Optional
server.output preserves legacy explicit source export for existing applications;
it is absent from new scaffolds and ignored by normal in-memory server compilation.
No owner-generated application was migrated or overwritten.

Shared api::configuration validates the same settings in compiler and CLI. Native
source generation reuses CheckedApi::native_source_versioned, including authorization,
input/output codecs, bounded failures, descriptions and retained wire profiles.
The internal target/tsrust-api/server.tsr identity is not a source file on disk.
Raw reserved-path imports are rejected. Public operations cannot depend on dispatch,
directly or indirectly; bootstrap rejects that dependency before recursive analysis.

Wire facts and executable handlers share the checked operation graph, including
resolved dependencies. An independently preloaded conflicting source/resolution
fails api-input-changed rather than mixing contract and handler versions. Existing
unreadable nearest config fails instead of selecting a parent's API policy; successful
unsaved-provider configuration is preserved. LSP snapshots retain exact derived
source for diagnostic coordinates through SourceProvider::generated. Real inputs
remain the invalidation dependencies; no global source registry was introduced.

Application-owned reviewed manifests/history files supply deployment baselines.
The server does not read a generated client's compatibility directory. Independent
SDK exports retain their existing ownership and release-history protections.

## Acceptance

- Compiler api_project: 4/4, covering no disk output, operation/config regeneration,
  direct/indirect cycles, denied nearest config, one-read source consistency and
  rejection of preloaded/mixed versions.
- Application scaffold: 3/3, including fresh layout, target rejection before writes,
  standalone/inline legacy config behavior, independent SDK export and owner edits.
- LSP api_project: 1/1, clean diagnostics, unsaved operation/config invalidation and
  preservation of actual source bytes. Bundled facade guards/hover: 2/2.
- These 10 tests passed together on final implementation. Formatting controls were
  then completed for inline compiler fixtures; affected compiler/editor tests passed
  5/5 again. No passing broad suite was restarted for that test-only change.
- Backend compiler/LSP and workspace all-target JSON checks each have successful
  terminal build-finished and zero diagnostic messages. Final affected backend
  metadata refresh after fixture formatting also passed. Normal CLI/LSP consumer
  build and its cached refresh succeeded; tests and shipping-consumer dependency
  feature configurations were kept distinct.
- Fresh review project: generated-projects/backend-owned-api, created by the CLI.
  Check/emit passed with Rust/Cargo removed from PATH, emitted native HTTP without
  a UI runtime, and wrote no dispatcher source. Rust files still exist elsewhere
  on this host; this is PATH isolation, not an uninstalled-toolchain OS image.
- Actual native build succeeded and the project's std:testing suite passed 1/1.
- Real HTTP TypeScript and async Rust SDK acceptance passed from both live and saved
  descriptions, with structured failures and validation against the native service, metadata/custom-
  transport authentication against owned HTTP fixtures, parallel Rust requests and
  static Rust type rejection. Generated OpenAPI matches
  the same served description. No server source/client output was needed by SDKs.
- Strengthened the same HTTP test: copied only the executable to a deployment
  directory and ran it with empty PATH and absent Rust/Cargo/Tsrust cache paths.
  SDK calls passed; no source/tool/cache files appeared in its working directory.
  The original checkout remains on the host; this is relocated execution/environment
  evidence, with existing Windows system runtime prerequisites, not a fresh OS VM.
- Existing Windows dev smoke driver gained --backend-project mode. Actual service
  HTTP greeting passed; a malformed edit preserved the previous running server;
  a repaired edit rebuilt in 5334ms and changed the greeting. Original bytes were
  restored, no dispatcher file appeared, Ctrl+Break stopped the owned supervisor
  and child handles confirmed exit. This is a smoke duration, not a benchmark.
- Real LSP protocol probes passed for the review project and original UI facade:
  clean, deliberate-error rejection, repair and physical-file cursor assistance
  where applicable. No owner editor/process was controlled. Owned services stopped.

Current gate logs are target/api-server-*-current.log, backend/workspace diagnostic
JSONL, final-focused-tests, formatted-tests, native-build/native-tests, http-sdk,
deployed-http-sdk, dev-acceptance/dev and real-editor/real-facade-editor. These are
disposable command evidence, not analytics. No SQLite evidence import or full
compiler/LSP executable regression/performance matrix was performed for this section.
The broader backend API/body/testing family and its required milestone checks remain
tracked separately from these accepted focused contracts.

## Repaired validation failures

Sandbox filesystem restrictions denied SDK output replacement in scaffold tests.
The already-built executable passed those cases outside the sandbox; the final
combined run used that boundary. LSP fixture initially used an unsupported anonymous
Context type argument, then omitted a record field terminator. It now uses the same
named, terminated AppState shape as the scaffold and verifies formatter idempotence.
The context fixture was checked directly before another expensive rebuild. Neither
language syntax nor ownership/private-module guards were weakened for the fixture.

## Remaining objective

Complete Rust old/new native replica execution, typed/contextual HTTP body readers,
async std:testing, ordinary binding shared-cache migration, trusted paired prebuilt
installation/latest-stable update and actual required OS/architecture execution.
Preserve original UI source/history outside ordinary backend dependencies. Native
UI expansion stays deferred and future component work should begin with primitives.
