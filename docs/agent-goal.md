# Goal: Tsrust backend framework, HTTP SDKs and distribution

Owner-approved replacement direction, 2026-10-27. The owner authorized continuation
of this goal after aligning it with backend-only scope. This supersedes the earlier
React/Electron/Expo application prototype objective. Follow
[the backend-only reset](plans/backend-only-reset.md) and
[Rust setup and dependency distribution](plans/rust-setup-and-dependency-distribution.md).
Historical application-platform plans and acceptance remain evidence, not active UI
requirements. No React, Electron, Expo, TanStack or frontend package-manager
orchestration work is required. Users bring their own UI and call HTTP APIs.

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
