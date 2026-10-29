# Goal: Tsrust language, CLI, documentation and platform polish

## Active owner direction — 2026-10-29 (supersedes the ordering below)

The owner approved this plan and asked for continuous autonomous work on it:

1. **Ship stable 0.1.0.** PR #6 (`codex/language-cli-docs-polish` → `stag`)
   needs the owner's bypass merge (agents cannot approve their own PRs). Then
   open `stag` → `main`; Release and Promote publish `v0.1.0` and deploy Pages.
   Smoke-test a release archive, then `tsrust toolchain install --root`,
   `toolchain check`, `update` and `rollback` against the published stable
   release. Record results in the handoff.
2. **Optimize the generated Rust** (owner, 2026-10-29): make emitted code as close
   as possible to what a careful person would write by hand. Parity with
   hand-written Rust is the stretch target ("excellent"), not a requirement.
   Prefer removing work in generated code (copies, clones, temporaries, checks
   that can be proven unnecessary) over adding dependencies. Do not adopt a
   dependency on the strength of measurements from this memory-starved Mac
   (8 GB RAM, heavy swap); confirm on a clean machine first. Start with the
   cases where native is slower than Node after tsc
   (see `project/performance` and `benchmarks/storage/release-0.1.0-analytics/`):
   JSON and schema decoding, record copies, Unicode case conversion,
   `String.at`, recursive array copies, enum and interface records. Change
   compiler output only with regression tests; measure before and after.
3. **Fill standard-library gaps** against Node and Rust std, most-used backend
   APIs first, each with IntelliSense documentation, examples and benchmarks.
4. **Rust packages with generated bindings and full IntelliSense.** Generate the
   adapter from the crate's public API: rustdoc JSON on the pinned toolchain
   (RUSTC_BOOTSTRAP) with a source-parsing fallback. Bind plain functions,
   structs, enums, methods, `String`/`Vec`/`Option`/`Result` and simple borrows;
   report skipped items (generics, traits, lifetime-bound returns, closures,
   async, macro-only APIs) and emit an editable stub for them. Doc comments
   become hover text. Raise design questions with the owner before large commitments.

Measurement rules: time benchmarks only on a quiet machine. The runner must
check load before each chunk; never build, test or run CI-like work locally
while timing. Run heavy regression suites on GitHub (dispatch
`full_regressions`), not locally during measurements. Publish results through
the SQLite dataset and the generated performance page.


Owner handoff, 2026-10-28: Codex continuation stopped for transfer to Claude;
completion is not claimed. Read [the current handoff](handoffs/2026-10-28-claude-handoff.md).
The owner's latest docs direction prioritizes language/stdlib reference and a
consolidated interactive guide instead of many indexed how-to articles.


Owner-approved replacement direction, 2026-10-27. The owner authorized continuation
of this goal after aligning it with backend-only scope. This supersedes the earlier
React/Electron/Expo application prototype objective. Follow
[the backend-only reset](plans/backend-only-reset.md) and
[Rust setup and dependency distribution](plans/rust-setup-and-dependency-distribution.md).
Historical application-platform plans and acceptance remain evidence, not active UI
requirements. No React, Electron, Expo, TanStack or frontend package-manager
orchestration work is required. Users bring their own UI and call HTTP APIs.

## Continuation mandate — 2026-10-28

The owner requested autonomous continuation after PR #2 CI succeeds, completing
language, CLI and documentation polish with Windows support explicitly required.
CI run 40663277431 passed all four checks on commit 5b09ec80. The owner later
authorized the bootstrap merge: PR #2 merged as caf5284b with the identical tree.
The staged pipeline is authorized to publish successful stag-to-main promotions;
bootstrap itself must not publish a release.
Work continues on codex/language-cli-docs-polish from that tested source.

The finite completion checklist below carries forward the backend-only handoff.
Audit current evidence before changing accepted behavior. Do not interpret polish
as arbitrary TypeScript compatibility or revival of deferred UI work.

- [x] Accept pending async std:testing registration: compiler/native ownership,
  factory outputs, shared multi-file state, real HTTP execution, failures/teardown,
  project cache reuse and all three focused LSP tests. Ordinary PR CI did not run
  the testing_async integration targets. Hosted run 40672007920 on c5b3d4a9
  subsequently passed all 11 compiler/native cases, 3 LSP cases and all-target
  diagnostics on Linux, macOS and Windows.
- [x] Complete request-body JSON decoding and contextual body streaming with
  limits, one-consumption behavior, error handling, docs and editor agreement.
  Focused native/compiler/catalog controls, HTTP receiver agreement and both
  typing-speed rows pass; native/Node untimed readers each produce 5376.
  Backend all-target diagnostics are clean. Timings remain a milestone task.
- [x] Close remaining generated TypeScript/Rust SDK authorization and compatible/
  incompatible replica execution requirements against a real native service.
  The native SDK replica witness passes on macOS arm64 with old/new generated
  clients, protected metadata, denied operations, missing-operation/API-major
  compatibility errors and no mutation effects for rejected calls. Prior saved/
  live description and typed-model evidence remains in server-owned-api.md.
- [x] Finish ordinary binding dependency migration while retaining exact offline
  portable exports and shared-cache integrity. Focused macOS acceptance passes:
  package transactions 7/7, shared/portable sources 4/4, schema controls, schema 2
  editor help and backend all-target diagnostics. Platform execution remains part
  of the distribution milestone.
- [ ] Complete paired CLI/LSP install, doctor, setup, update checks, replacement,
  rollback and recovery. Execute Windows native compiler/test/dev/editor and
  installation/update paths, including locked binaries; separately verify macOS
  and Linux. Add Windows hosted validation and non-publishing package validation.
  2026-10-28 reconciliation: Windows hosted toolchain acceptance passed in runs
  40731485167 (9b658857) and 40748893795 (86435b88): toolchain_installation,
  shared_sources and package_management, including locked processes. Remaining:
  install and self-update from a published release (none exists or is authorized).
- [ ] Reconcile supported language behavior, CLI help, guides and runnable examples;
  verify behavioral examples and complete required IntelliSense documentation.
- [ ] Migrate public documentation to Astro Starlight following
  [the public documentation plan](plans/public-documentation-site.md). Prioritize
  the plan, digestible expandable sections, reviewed content migration, removal of
  superseded public Markdown, repository link migration and an automatically
  deployed free static host. Preserve internal/historical evidence and retained UI
  consumers. The owner selected GitHub Pages after reviewing its limits.
- [ ] Complete remaining feature-family regressions and matched performance
  evidence, review regressions, and reconcile the checklist with authoritative
  results. Unexecuted platform checks and legacy lint debt stay explicit.

The owner clarified that this is a different computer; the earlier fan stop does
not apply here. Resume bounded local validation with one Cargo job, incremental
compilation disabled and resource checks. Use hosted jobs for platform evidence.
No recurring automation is required.

## Current owner clarifications

The server is the source of truth: users write and serve their API in app/.
Publish the checked HTTP description and OpenAPI from that running service, then
produce independent js-client/ or rust-client/ SDKs at a caller-selected location.
A normal server must not require users to generate, import or maintain a public
generated/server tree. Compiler-internal serialization/dispatch lowering may be
an implementation detail; it must not make client generation a server startup
prerequisite. Preserve authentication, validation and compatibility when replacing
the current generated dispatcher. SDK generation must not modify the server.

The reported docs stack overflow and bundled-facade diagnostic are repaired in the
accepted [docs profile](milestones/native-docs-development-profile.md) and
[editor agreement](milestones/portable-source-packs-and-editor-agreement.md) sections.
[Server-owned API integration](milestones/server-owned-api.md) is also accepted;
remaining requirements below stay open. Keep affected compiler/LSP diagnostics clean,
finish each section and its checks before committing, and commit each accepted section. Organize loose documentation by topic, updating actual consumers and
links; preserve owner source edits and retained historical evidence. Future UI work
should start with a finite set of reusable primitives and small composition examples
before building larger components. UI expansion remains deferred.

## Required outcomes, in order

Owner clarifications: HTTP I/O must be asynchronous, including generated Rust
clients/custom transports and CLI HTTP acquisition; do not offer a blocking HTTP
adapter as the default. Normal terminal commands may wait at their runtime boundary,
but network operations use async clients. Also audit request body readers: retain
async text/form/multipart/stream APIs, add checked typed JSON decoding with existing
limits/ownership and complete compiler/LSP documentation, and track XML separately
as parser-backed support rather than an unchecked cast from text.

Use std:testing and the normal tsrust test runner for ordinary Tsrust behavior.
Retire active executable assertion programs left from before that library. Where
async registration is missing, implement it in the library rather than maintaining
an alternative runner. Generated Rust/TypeScript consumers use their normal test
runners. External-process fixtures remain only for genuine cross-language/process
boundaries; benchmarks retain their measurement drivers and historical evidence.

1. Complete backend-only cleanup. Remove introduced frontend templates, adapters,
   dependency graphs, task UI examples, verification tools and active documentation
   references. Retain native backend compiler/LSP, independent SDK/HTTP transport,
   checked dispatch, API compatibility, authorization, resources, SQLite and useful
   arbitrary-client static serving. Preserve original Tsrust UI source/history for
   possible later revival, outside ordinary backend build/install dependencies.
   Preserve owner-created applications and unrelated edits. Prove fresh CLI backend
   generation, native execution and editor agreement without a frontend toolchain.
2. Generate SDKs from a validated HTTP API description, fetched from a running
   service or read from a saved file, without the original backend source tree.
   Initial outputs are TypeScript and Rust, plus OpenAPI specification export.
   Reuse one checked contract and existing wire/version machinery. Keep named calls,
   input/response validation, configurable authentication/transport, structured
   errors and safe rolling-deployment compatibility. No mandatory UI/cache library.
   Define supported OpenAPI mapping/import scope explicitly. Test both generated
   languages against a real native service, saved/live descriptions, invalid input,
   failures, authorization and compatible/incompatible replica versions.
3. Migrate ordinary native development from mandatory vendor copies to Cargo's
   shared registry/Git cache, exact locks and explicit preparation/resolution.
   Inventory dependency consumers before changing them. Provide actionable,
   machine-readable doctor and explicit setup using rustup/Cargo. Preserve global
   toolchains/defaults/configuration. Routine check/emit/build/dev never install or
   download silently. Preserve toolchain-free compiler operations. Keep portable
   offline source exports with runtime sources, manifests/locks, vendors, notices
   and prerequisite metadata; prove relocated offline builds and native parity.
4. Deliver reliable prebuilt CLI/LSP installation on Windows, macOS and Linux,
   and explicit latest-stable update checking/updating. Pair compiler and LSP;
   validate trusted release metadata, host architecture and artifact integrity;
   stage replacement with interruption/failure recovery and Windows locked-binary
   handling. Respect package-manager-owned installations. Test isolated installs,
   no-update, bad artifacts, rollback and tooling-free tool/app execution. Record
   platform/architecture execution separately; local mocks do not prove all OSes.

## Acceptance and work rules

Use AGENTS.md, docs/development-workflow.md, the project development/Ponytail/Caveman
skills and current docs/agent-progress.md. One agent writes this tree. Inspect Git
status and preserve owner edits. Keep local source checkpoints before risky changes
and after accepted sections; run whole-project formatting, token/idempotence checks
and formatted-source behavior as required. Use one Cargo target, one build job and
CARGO_INCREMENTAL=0. Add focused witnesses during implementation and batch affected
validation after coherent sections; complete compiler/LSP regressions and matched
performance at meaningful backend milestones. Preserve exact evidence, SQLite
analytics privacy, historical measurements and resource/ownership guarantees.

Do not control the owner's editor or stop their processes. Use terminal/files and
browser automation where applicable. Keep generated backend review projects under
D:/ai_dev/tsrust/generated-projects without overwriting previous owner projects.
Stop owned servers after testing. Keep editor diagnostics and installed consumers
current only after rebuilt tools are validated. Native deployment requires no
Tsrust/Rust/Cargo/repository/cache; document real OS runtime prerequisites.

Continue autonomously through all required outcomes. Record implementation,
acceptance, measurement and pending platform checks separately. Keep progress and
next executable action current. Do not claim completion without authoritative
requirement-by-requirement evidence. No timer pushes, public releases, release tags,
package publication or recurring automation. Remote validation follows the project
skill's specific-uncertainty decision; release publication needs explicit approval.

## Owner content-audit requirement — 2026-10-28

Audit every public documentation page for factual accuracy against current code
and examples, and replace dense prose with clear task-based explanations. Track
page-level source, example and rendered acceptance in
[the content audit](plans/public-documentation-content-audit.md). Migration and a
green site build alone do not satisfy this requirement. Preserve claims about
unchecked behavior as pending rather than treating existing prose as authoritative.
