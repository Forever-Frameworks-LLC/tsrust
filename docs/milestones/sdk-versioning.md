# Versioned SDKs across native server replicas

Accepted focused section at `89471f0c`, from local checkpoint `03f20f0`, 2026-10-27.
This is SDK/native rollout acceptance, not completion of the backend prototype.

## Behavior

The generator separates SDK package version, API compatibility major and exact
manifest identity. Owned compatibility metadata retains distinct checked schemas
and immutable release-to-wire references. Canonical/export history merges before
replacements; legacy owned output bootstraps from its package and manifest.
Same-major changes must preserve both existing callers and the new SDK's ability
to decode retained older replicas. Required result additions, removed SDK methods,
new required arguments and changed query/mutation roles reject. API-major changes
require an SDK-major change. Imported foreign baselines allow fresh client subsets.

HTTP clients choose a known input-compatible wire profile before sending exactly
one request. New arguments are not removed. Current operation identity and API
major travel as JSON Content-Type parameters, alongside artifact diagnostics;
the existing x-tsrust-contract header carries the selected profile. Existing CORS
allowlists therefore need no new header names. Modern native dispatch admits
known per-operation shapes independently of unrelated selected methods and
projects results for strict old clients. Input/result checks and authorization
remain authoritative. Compatibility errors preserve their public error code and
include the requested API major. New majors affect wire/cache identity even if
their DTOs are identical; legacy API v1 fingerprints remain usable.

## Evidence

All checks below passed with one Cargo job, incremental compilation disabled and
the shared target/cache. No performance timing, analytics import, remote CI,
publication or release was performed.

- `cargo test -p tsrust-compiler --test api_contract --offline --locked`:
  all 10 tests passed. Log `target/api-versioning-contract.log`.
- `cargo build -p tsrust-compiler --bin tsrust --offline --locked`:
  passed. Log `target/api-versioning-cli-build.log`.
- `node tools/verify-api-client.mjs`: plain named calls, strict codecs,
  TypeScript, local HTTP, rotating credentials, no retry/redirect, abort,
  generation version guards and preserved owner output passed.
  Log `target/api-versioning-client.log`.
- `node tools/verify-api-query.mjs`: existing React/TanStack behavior/types and
  new identical-schema API-major cache isolation passed.
  Log `target/api-versioning-query.log`.
- `node tools/verify-api-package.mjs`: exact local tarball allowlist/integrity,
  detached plain/React/TypeScript consumers, owner dependency additions and npm
  workspace peer resolution passed. No publication.
  Log `target/api-versioning-package.log`.
- Normal four-operation task and eleven-operation contract dispatchers passed
  virtual source checking. Log `target/api-versioning-dispatch.log`.
- `node tools/verify-api-replicas.mjs`: actual old/new native servers sharing a
  disposable SQLite database, immutable old SDK runtime reconstructed from
  `03f20f0`, new selected client subset, unrelated new operation, optional request
  and result additions, strict old-SDK projection, rejected-major/argument writes
  without effects, authorization, exact upstream request counts/no retry, and
  real headless Chromium CORS through the legacy server all passed. Both servers
  drained, temporary data/configs/sources/consumers were removed, and canonical
  generated task output was restored. Log `target/api-versioning-replicas.log`.
- `node tools/verify-native-api.mjs`: private Context, authentication, strict
  nested DTOs/tuples/unions, safe numeric mappings, retained projection, bounded
  private failures and shutdown passed. Log `target/api-versioning-native.log`.
- Changed JavaScript verifiers pass node --check. Whole-project formatter ran
  twice: zero source files and zero fingerprints changed on either pass. The
  protected owner docs source hash remained unchanged. git diff --check passed.
  Logs `target/api-versioning-format-first.log` and
  `target/api-versioning-format-second.log`.

The actual legacy task executable was preserved, not overwritten by the test.
Its SHA-256 is
`a9eba46f2376ca5c1cbbb1aa0e65fc31df8141eb19a5ebd4aadd1c5b5f47df57`.
The modern rollout executable SHA-256 is
`447324f8c41317ed4d584fb0ef5ffb4bb378872b54b63917c66b91a2a1bebd7e`.
The witness first proves that the old executable rejects the new subset's raw
whole-manifest fingerprint, then proves safe profile selection works across both
replicas. This is not two schema profiles handled by a new server.

The temporary enrichment fixture initially failed ownership checking when moving
fields from its discriminated result. A borrowed DTO projection with explicit
owned string copies repaired the fixture; no compiler ownership rule changed.
Earlier passing checks were preserved; only the affected witness was retried.

## Limits and remaining prototype work

Checked compatibility covers retained wire schemas, not arbitrary business
semantics, unknown historical servers or fleet-wide capabilities inferred from
one replica. New operations and supplied new arguments can require newer servers.
Old API majors need retained compatible handlers or explicit migrations. Schema
release references do not enforce immutable registry artifacts: publishing any
runtime-only package change still requires an appropriate new package version.
Generated packages remain private and publication is owner-controlled.

No new package dependency was added. The optional framework-independent query-core
entry, Electron/Expo targets, convenience commands/migrations, cancellation cleanup,
full backend-family regressions/measurements and remaining platform acceptance
stay required by docs/agent-goal.md. Active-editor LSP refresh remains pending
because its installed image is locked; this section does not claim that refresh.

The installed CLI was atomically refreshed after the source checkpoint, with
source/installed SHA-256
`8b397656b08a1085c3bc584e1fcf7acf9c108be621b1523fef01b09dbc0e8fe1`.
Installed-consumer task main checking and generated API stale checking pass.
