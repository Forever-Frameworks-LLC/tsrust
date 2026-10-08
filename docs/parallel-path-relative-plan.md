# Parallel worker: path relative, followed by main integration

## Goal

Implement `std:path.relative(from: string, to: string): string` through runtime, compiler bindings, native execution, editor assistance, documentation and matched benchmark support. Finish by integrating accepted source, immutable evidence and executable consumers into main. Branch acceptance alone does not complete this goal.

Start from the accepted integrated checkpoint `8b9b372`, or a newer committed base agreed with the main owner. Read AGENTS.md, docs/development-workflow.md, docs/handoff-2026-10-08-stdlib-integration.md, the path contracts and current standard-library roadmap first. Inspect current exports to avoid duplicate work. The main tree currently has uncommitted callable work and fifteen protected owner fixtures; do not touch them.

## Scope and prerequisite gate

The public API returns the lexical relative path from one location to another using the target platform's path convention. Audit official Node documentation and the pinned runtime before freezing exact behavior. Relative arguments require a current working directory: this is an explicit prerequisite, not permission to pretend all inputs are absolute or to expose a reduced API as complete parity.

First establish how the existing native binding obtains the working directory and reports failure. Use the accepted error mechanism. Keep the working directory injectable in internal helpers so differential cases are deterministic. If host lookup cannot cross the current binding safely, write a minimal dependency witness and stop public acceptance at that gate; continue independent algorithms and reference cases. Do not redesign ownership, callable carriers, async support or general type analysis.

Implement dedicated runtime helpers with explicit platform and working-directory inputs. Borrow both public arguments; return an independently owned string. No filesystem traversal, canonicalization or symlink resolution. Do not add public resolve, parse, format, filesystem resources or OS APIs in this goal.

## Milestones

1. Freeze the contract and prerequisite evidence. Cover identical paths, empty inputs, relative and absolute inputs, dot segments, repeated and trailing separators, shared prefixes that are not full components, ancestor/descendant and sibling paths, roots, Unicode, Windows drive-relative paths, different drives, case handling and UNC roots. Record expected results independently using Node 24.11.1 with controlled working directories. Distinguish host-native execution from platform helper tests.
2. Add focused failing tests, then implement helpers and binding registration. Use std:testing project fixtures for direct imports, aliases, stored calls, borrowing after calls, invalid arity/types, observable output and failures. Verify formatting token preservation, idempotence and formatted execution. Preserve existing path exports. Audit native observer effects as well as binding registration: the main compiler now preserves point-sensitive callable state across explicitly listed observer calls and stored native aliases. A new relative binding must receive its correct non-callback effect summary, with a combined imported-call/factory witness, rather than silently invalidating contextual callable proofs.
3. Complete IntelliSense before acceptance: behavior, both arguments, defaults or lack thereof, errors, return value, examples and edge cases, then a short ownership note. Keep signatures, completion, hover and parameter help consistent. Verify examples natively. Keep runtime versions and evidence details out of hover text.
4. Add matched Tsrust/TypeScript workloads with deterministic inputs, independent oracles, source contracts, registrations, observers, ABI support and required runner/profile/report hooks. Append manifest rows at the end and register hashes after whole-project formatting. Run bounded untimed native and Node-after-tsc verification. At this completed family milestone, measure the new workload under the existing protocol and review comparable signals; do not retime unrelated workloads.
5. Checkpoint and integrate. Run complete compiler and LSP regressions at the family milestone, classifying intentional baseline witnesses against the latest main handoff. Preserve completed evidence and rerun affected groups after repairs. Run all eight depth canaries if shared dispatch or analysis changes. Write acceptance records, update coverage and roadmap, and commit a physical handoff.

## Parallel boundaries

Use the existing isolated stdlib worktree and a `codex/` branch. One agent edits each tree. Reuse `/private/tmp/tsrust-stdlib-path-target` with CARGO_INCREMENTAL=0 and offline/locked Cargo. Do not overwrite main binaries or write main SQLite while developing. Keep shared registry, build, catalog and benchmark edits additive and list them for integration. Keep library algorithms outside shared compiler dispatch. Record genuine language blockers with exact witnesses instead of weakening checks.

Format the entire worker project before source checkpoints with the current verified compiler. Use explicit staging paths. Never stage owner fixtures or use broad Git staging. Preserve exact historical evidence and original measurement protocols. Import closed records into the worker SQLite store, verify exact readback and remove loose copies. Use Node 24.11.1 first on PATH and /opt/homebrew/bin/python3 for storage tools.

## Required integration and ownership transfer

Integration into main is authorized and required. Before editing main, obtain an explicit exclusive ownership handoff from the main agent or the owner. Main's uncommitted callable changes are active work; do not merge through them, stash them, reset them or commit them on the worker's behalf. While waiting, finish all independent branch checks and integration preparation.

After the main agent yields a committed checkpoint, merge that checkpoint into the isolated branch and resolve conflicts there. Preserve both language and library changes. Verify the combined affected compiler/native/editor contracts, registrations and untimed workloads; run canaries after shared analysis changes. Reuse completed milestone evidence where source is unchanged. If main advances again, renew the handoff and integrate the new checkpoint before publication.

Publish the verified branch into main, preferably by fast-forward, without force updates. Preserve all protected owner bytes and index entries. Serialize SQLite integration through repository export/import tools: never replace main's database with the worker database; preserve identities and bytes, handle differing records under new explicit identities, verify original analytics and exact imported readback, then remove verified exports.

Rebuild all five main consumers from the integrated source commit using the documented main target. Refresh by copying to sibling `.refresh-tmp` files and atomically renaming, then smoke-test CLI and LSP behavior. Prune obsolete generated outputs. Commit a final handoff stating source/evidence/consumer integration, exact checks, baseline failures, platform coverage and measurement status. Return main ownership before stopping.

## Ready-to-copy prompt

Continue the parallel Tsrust stdlib worker using docs/parallel-path-relative-plan.md in the main repository. Goal: implement std:path.relative end to end, then integrate accepted source, SQLite evidence and all five executable consumers into main. Start from 8b9b372 or the main owner's newer committed checkpoint. Read AGENTS.md, docs/development-workflow.md and the accepted stdlib integration handoff first. Audit the current-working-directory and error prerequisite before implementation; freeze POSIX/Windows semantics using official documentation and Node 24.11.1 reference cases. Follow all milestones, including TDD, native assertions, formatting, complete IntelliSense, matched benchmarks and scoped milestone review. Work in your isolated checkout with its own target and SQLite store. Main currently contains active uncommitted callable work plus fifteen protected owner fixtures. Finish branch work, then obtain exclusive main ownership and integrate the latest committed main in your isolated branch before publication. Integration is required and authorized, but concurrent main mutation is prohibited. Never reset, stash, stage or commit owner changes. Provide a committed integrated handoff and return main ownership. Stop after this family; do not begin another API or create additional agents.
