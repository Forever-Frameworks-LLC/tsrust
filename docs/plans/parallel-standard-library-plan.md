# Parallel standard-library implementation plan

## Goal and first deliverable

Build useful bundled standard-library APIs while the language agent finishes callable replacement. The first bounded family is lexical path operations: extend the existing `std:path.basename` integration with `dirname`, `extname`, `isAbsolute`, and `normalize`. Deliver compiler bindings, native execution, editor assistance, executable documentation, matched benchmark support, acceptance evidence, and integration-ready commits. Completion means this family is accepted, not that the entire standard library is complete.

Baseline: `8fe73b5` (gate 174). The main checkout contains fifteen protected owner fixture edits. Start from committed source in a separate worktree; do not copy, format, stage, or otherwise change those owner edits. Read the latest handoff at execution time because the compiler may advance after this plan was written.

This plan authorizes planning scope; it does not itself create an agent or worktree. The assigned agent should create or use its own checkout when instructed to begin. One agent works in each tree at a time.

## Why this family can proceed now

The existing path binding already accepts borrowed strings, an optional string argument, and an owned string result. The selected operations need only these primitive contracts plus a boolean result. They require no files, host handles, asynchronous callbacks, shared cells, database connection, JSON representation, or new ownership model. Verify boolean native return support before implementation; a missing primitive bridge is a small explicit integration request, not permission to redesign the compiler.

The roadmap's S06 status predates several accepted language gates. Audit current implementation rather than treating its old pending labels as proof of a missing feature. This path increment is a deliberate primitive-only exception to the broader S09 host family; it does not accept filesystem or resource support.

## Working boundaries and coordination

The language agent owns parser dispatch, type representation, ownership/loan analysis, callable carriers, flow analysis, and general Rust emission. The library agent owns path runtime algorithms, path binding declarations, path-focused fixtures and tests, and its documentation and benchmark additions.

Shared integration files include `src/native_bindings.rs`, native build/emission registration, `crates/tsrust-lsp/src/builtins.json`, editor tests, benchmark manifest and runner hooks, and generated coverage records. Small additive edits are allowed on the library branch. List them in each checkpoint; the integration owner resolves conflicts and checks the combined result. Do not refactor shared dispatch or repair unrelated language failures on this branch. Record a minimal failing witness and dependency request when a shared compiler defect blocks work.

Use a `codex/stdlib-path` branch in a separate managed worktree. Do not share the main checkout's live executable consumers. Reuse one library-workstream Cargo target with `CARGO_INCREMENTAL=0`; never create a target per gate. The existing main target must not be overwritten by concurrent branch builds. This workstream separation is intentional; prune obsolete outputs before handoff.

Do not write concurrently to the main checkout's SQLite store. Keep a worktree-local store for this workstream using the repository import tools, verify byte-exact readback, and remove verified loose copies. At integration, serialize canonical-store imports, preserving original identities and handling collisions without overwriting evidence. Do not copy one SQLite database over another.

Project-wide formatting runs in the library checkout before every source checkpoint. Inspect resulting changes; include only authorized feature or proven formatting support changes through explicit Git paths. Never interpret formatting as authority to commit unrelated owner changes.

## Milestones and exit gates

### P0: inventory and frozen contract

Read `AGENTS.md`, `docs/development-workflow.md`, the latest gate handoff, `docs/plans/standard-library-roadmap.md`, `docs/guides/language/standard-library-coverage.md`, and the S05/S06 milestone records. Inspect the actual binding, module loader, native build, and catalog plumbing. Confirm which path APIs already exist before adding duplicates.

Research official Node path documentation and the pinned reference runtime's implementation when edge behavior is ambiguous. Use Node 24.11.1 for differential evidence, not hover text. Write a per-export contract table covering parameters, borrowing, returns, empty input, roots, trailing/repeated separators, dot segments, leading dot filenames, Unicode, and platform behavior. Freeze observable semantics before coding. Rust `Path` behavior is not automatically Node-compatible.

The public module uses the target platform's path convention, consistent with existing basename. Internal runtime helpers take an explicit platform selector so POSIX and Windows cases can be tested on macOS. Do not claim end-to-end Windows acceptance without running its native integration. Windows drive-relative paths and UNC roots require explicit cases. No filesystem access or symlink resolution is permitted. `normalize` is lexical; it must not be advertised as a security containment check.

Proposed signatures, subject to the primitive bridge audit:

```tsrust
dirname(path: string): string
extname(path: string): string
isAbsolute(path: string): boolean
normalize(path: string): string
```

Gate: contract table, independent reference expectations, binding feasibility, shared-file list, and bounded check commands recorded. A proposal is not implemented coverage.

### P1: dirname and extname

Add focused failing compiler/project/native witnesses before implementation. Implement runtime helpers and registry entries using existing primitive binding contracts. Test root and separator handling, empty input, trailing separators, dotfiles, multiple dots, suffix-like names, drive prefixes, UNC forms, and valid Unicode. Compare to independent POSIX/Windows Node references.

Add invalid arity/type/import tests and borrowing witnesses demonstrating inputs remain usable. Verify direct imports, aliases, and stored function calls using already accepted callable forms. Avoid depending on the two known callable replacement failures.

Gate: formatted-source round trips, native assertions, reference agreement for claimed behavior, complete catalog examples and signature/hover/completion checks, guide updates, registered untimed matched workloads, and an acceptance commit.

### P2: isAbsolute and normalize

Repeat TDD and all P1 acceptance categories. Normalization tests must cover empty input, root preservation, repeated separators, trailing separators, `.` and `..`, attempts to traverse above absolute roots, relative leading `..`, Windows drive-relative versus drive-absolute paths, and UNC paths. Assert normalization idempotence in addition to explicit independent expected results. Preserve Unicode text without byte-boundary mistakes.

Gate: both exports pass the same compiler/native/editor/documentation/benchmark requirements. If normalization needs unsupported compiler facilities, continue runtime/reference work and state the exact missing public contract; do not bypass analysis or introduce an implicit clone.

### P3: path family integration and handoff

Run the complete compiler crate at acceptance as explicitly requested by the owner, and the whole editor crate. Classify the two baseline callable replacement witnesses separately; new failures cannot be dismissed as baseline. Run the path benchmark support/registration tests and untimed native versus Node-after-tsc verification. At the completed family milestone, measure this family's registered workloads under the existing protocol, preserving sample counts and recording comparable signals. Do not time the entire historical matrix. Keep implementation acceptance separate from any pending measurement or platform coverage.

Commit evidence and a physical handoff. The integration owner incorporates feature commits into the advancing compiler branch, resolves shared registries/catalog/hooks, reruns affected combined checks, and refreshes the five live consumers only after integration succeeds. A branch-local success is not evidence for the combined checkout.

## Benchmark and documentation requirements

Provide matched `.tsr`/`.ts` workload pairs, independent result oracles, source contracts, registration tests, runner/oracle/profile/report hooks where required, correct ABI observers, and manifest rows appended at the end. Use token-aware source-contract helpers introduced in gate 174. Register current source hashes after project formatting; do not weaken hashes, operation counts, guards, or historical evidence to fit an implementation.

Each new export's IntelliSense explains purpose, arguments, defaults, errors, returns, mutation, useful boundaries, and an executable example with expected results. Put a short ownership note last. Keep runtime versions and provenance in developer evidence. Verify imported hover, completion, parameter help, and diagnostics against compiler metadata. Do not create an editor-only API.

Use `std:testing` assertions for native fixtures. Run imports through the project CLI. Exercise formatting, token preservation, second-pass idempotence, and execution of formatted source. Keep independent Node references as `.ts`. Repair every defect in this family's changed behavior before accepting it; unrelated compiler defects become explicit dependency witnesses for the compiler owner.

## Compiler prerequisite ledger for subsequent families

| Family | Required proof before implementation acceptance | Parallel work possible before that proof |
|---|---|---|
| More primitive path operations | Registry support for exact arity/types and native return adaptation | Runtime algorithms, reference cases, docs; avoid variadic join/resolve until rest binding support is proven |
| Bytes and synchronous filesystem | Owned opaque handles, close/drop, byte representation, typed errors and early-return cleanup | API contracts, isolated Rust backend tests, resource/error witnesses |
| JSON | Explicit dynamic value representation, typed decoding interface, number/null/missing rules, size/depth limits and structured errors | Payload corpus, official-source semantics audit, direct Rust prototypes and independent oracles |
| Schema validation | Accepted Result/error contracts, generic output inference, JSON representation, callback ownership | Per-method coverage contract, issue-path tests, validation corpus |
| SQLite | Handles, bytes/null conversion, typed errors/rows, transaction/drop contracts | Backend research and isolated transaction/rollback tests |
| Threads | Transferred captures, Send-like capability checks, owned handles and join/failure rules | Transfer/rejection specifications and direct Rust backend tests |
| Tokio tasks and HTTP | async/await, suspend-safe loans/drop, cancellation/task lifetime, bounded buffering; JSON/schema for typed bodies | Protocol fixtures, limits/shutdown contracts and backend prototypes |
| SQLx, concurrent caches and queues | Accepted resources, async/thread contracts, pooling/backpressure/transaction rules | API/coverage inventories and backend witnesses |

These are observable contract gates, not a claim that every item is currently missing. Audit accepted features and attach exact evidence. The compiler owner implements genuinely missing language contracts. The library agent must not invent `any`, silently clone resources, weaken borrowing, or hide unsupported behavior behind Rust compilation.

After P3, stop and hand off before selecting another family. Recommend the next implementable family with prerequisites proven and missing contracts listed. Do not start JSON, HTTP, databases, or a custom goroutine scheduler merely because path work is finished.

## Environment and progress tracking

Put `/Users/seancotter/.nvm/versions/node/v24.11.1/bin` first on PATH. Use `/opt/homebrew/bin/python3` for `store.py`. Use offline/locked Cargo builds with the workstream target and incremental compilation disabled. Inspect repository commands before executing them; paths from another checkout must not select or refresh the main agent's binaries.

Maintain the following checklist with commit IDs, focused evidence identities, platform coverage, and measurement status:

- [x] P0 contract and integration boundary frozen; branch-local milestone contract.
- [x] P1 dirname/extname accepted (`3492e21`).
- [x] P2 isAbsolute/normalize accepted (`b57ec2a`).
- [x] P3 family regressions and benchmark review complete (`db30ab3`, optimized in `48ae800`).
- [x] SQLite evidence verified; obsolete outputs pruned before handoff.
- [x] Integration-ready commits and physical handoff supplied in docs/handoffs/handoff-2026-10-07-lexical-path.md.
- [ ] Integration owner verifies combined checkout and refreshes live consumers.

## Ready-to-copy agent prompt

```text
Continue Tsrust standard-library development in a separate worktree from committed HEAD 8fe73b5 or the integration owner's newer approved base. Read AGENTS.md, docs/development-workflow.md, docs/plans/parallel-standard-library-plan.md, the latest gate handoff, and the standard-library roadmap/coverage and S05/S06 records first.

Goal: finish the bounded lexical std:path family end to end: dirname, extname, isAbsolute, normalize. Existing basename remains compatible. Follow P0–P3 in the plan; audit actual support and freeze Node-compatible platform semantics before implementation. Use official reference sources and Node 24.11.1 differential witnesses. No runtime-version references in IntelliSense.

One agent per tree. Preserve the main checkout's fifteen owner fixture edits. Work on codex/stdlib-path with explicit Git staging paths. Own library runtime/bindings, fixtures, docs and benchmark additions; keep shared registry/catalog/build/runner changes additive and listed for integration. Do not alter parser, ownership, callable carrier or general type architecture. A blocking compiler defect needs a minimal witness and an exact dependency request, while independent runtime/reference work continues.

TDD; std:testing project/native assertions; formatting round trips; full IntelliSense; matched TypeScript workload, independent oracle, source contract, registrations/hooks and manifest row appended at end. Format your entire checkout before source checkpoints. Run focused checks during implementation, whole compiler at acceptance per owner request, whole editor before acceptance, and the scoped family milestone performance review using existing protocols. Preserve known baseline witness failures separately; fix new family regressions.

Use Node 24.11.1 first on PATH, /opt/homebrew/bin/python3 for store.py, CARGO_INCREMENTAL=0, and one workstream Cargo target. Do not overwrite the main agent's target/live consumers or write its SQLite store concurrently. Verify branch-local SQLite evidence byte for byte and remove closed loose copies. Serialize canonical evidence integration later.

Finish with accepted feature commits, exact checks and unresolved platform/measurement limits, a physical handoff document, and a compiler dependency ledger. Stop after this path family; recommend the next family with concrete prerequisites rather than silently expanding into JSON/HTTP/SQL. Coordinate integration before refreshing shared consumers. Do not create other agents or message other chats without owner authorization.
```
