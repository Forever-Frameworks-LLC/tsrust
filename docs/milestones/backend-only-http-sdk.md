# Backend-only starter and HTTP description foundation

2026-10-27, Windows x64. This accepts the scoped foundation below, not completion
of UI dependency isolation, Rust SDKs, setup, installation or self-update.

Introduced React/Electron task frontends, frontend templates, framework adapters,
their verification scripts/dependency graphs and known obsolete build outputs are
removed. Original Tsrust UI source/history and owner-generated apps are preserved.
The original UI authoring crate and embedded optional GPU dependency source graph
still require isolation from ordinary backend distribution; do not call the whole
compiler/distribution UI-free yet.

The application starter defaults to backend-only with app/{api,lib,tests} and
generated/{client,server}. It exposes only explicitly selected operations and mounts
application-authorized GET description/OpenAPI handlers. TypeScript SDKs are plain,
dependency-free ESM. Saved HTTP descriptions reproduce them without backend source;
ownership and retained-history checks remain. OpenAPI 3.1.1 describes the actual
single POST wire endpoint with JSON schemas and an x-tsrust-http extension. It is
not an arbitrary OpenAPI importer; application server URL/auth are deployment-owned.

## Executed evidence

- target/backend-only-tests-current.log: api_contract 11/11 passed. Scaffold tests
  failed with sandbox access denied; target/backend-scaffold-recheck.log records
  their normal Windows filesystem rerun, 3/3 passed. No product workaround was needed.
- Fresh CLI project generated at generated-projects/backend-http; source check
  passed and native greeting test passed 1/1. It emits no frontend folder or root
  Node workspace. target/backend-http-build.log records native server build success.
- target/backend-http-sdk.log: running native service returns matching public
  description/OpenAPI; a detached saved description generates a client that calls
  the actual service, handles whitespace/empty input and rejects wrong argument
  types. The verifier checks port ownership and stops its process after execution.
- target/backend-http-editor.log: real LSP entry/operation diagnostics clean;
  deliberate in-memory type error detected and cleared, disk source preserved.
- target/backend-plain-package.log: npm 11.17.0 offline package allowlist/integrity,
  detached plain runtime/TypeScript clients, preserved user edits/dependency
  additions and ordinary workspace imports passed. No publication or install scripts.
- Whole-project formatting reported zero edits/fingerprint changes; whitespace
  checks and affected JavaScript verifier syntax checks passed.

Initial compilation read the earlier library while SDK source was being added;
the resulting stale-library import failure is retained in backend-only-tests.log.
The subsequent current-source suite above supersedes it. Installed compiler/LSP
were not replaced. No broad compiler/LSP matrix or performance claim is made.

## Remaining sequence

Live URL description acquisition, Rust client generation, protocol/auth/version
runtime acceptance for both languages, original UI dependency isolation, explicit
doctor/setup/cache migration and cross-platform installation/update remain required.
Follow agent-goal.md and plans/backend-distribution-inventory.md. No release,
publication or remote workflow was triggered.
