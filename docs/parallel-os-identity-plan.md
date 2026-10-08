# Parallel standard library: operating-system identity

## Goal and boundary

Implement `std:os.platform()` and `std:os.arch()` as a complete, documented operating-system identity slice. Both return strings and take no arguments. Determine and document their exact target mappings before implementation. This is focused acceptance, not acceptance of the entire Node OS module.

Continue in the isolated standard-library worktree from `6c5fcc7`. One agent may work in that tree while the main agent handles callable replacement in the main tree. Preserve the accepted path commits and their SQLite evidence. Do not integrate into main, refresh main executable consumers, or modify the main database. Combined integration and Windows native acceptance of the path family remain separate pending work.

## Gate O1: contract and bridge proof

Read AGENTS.md, docs/development-workflow.md, docs/parallel-standard-library-plan.md and the lexical-path handoff. Inspect the existing native binding bridge before adding an API. Research official Node documentation and Rust target configuration documentation. Record references and freeze a mapping table for supported operating systems and architectures. Distinguish the executable's compilation target from its host and from the operating system's physical hardware. State unsupported targets explicitly; do not silently invent a fallback.

Add a failing project/native witness for an imported zero-argument string-returning binding, plus argument-count rejection and unknown-export diagnostics. Identify whether the existing bridge already supports this shape. If implementation requires parser, ownership, callable, general type-system or unrelated backend changes, provide a minimal witness and dependency request instead of editing those systems.

## Gate O2: implementation and editor acceptance

Implement the two bindings with narrowly scoped registration and emission changes. Verify target mappings independently from the implementation, including cross-target mapping tests where practical. Native execution on the available host must match the reference. Cross-target mapping tests do not establish Windows native acceptance.

Add std:testing project fixtures, independent TypeScript references, formatter round trips, import alias tests, return-type checks and arity diagnostics. Provide complete catalog documentation, hover, completion and signature help: purpose, no arguments, return values, useful examples and target semantics, followed by ownership notes. Keep reference versions and implementation provenance out of IntelliSense. Execute the examples and update the guide.

## Gate O3: evidence and handoff

Prepare matched workload pairs, independent result oracles, source contracts, registration checks and all applicable oracle/run/profile-native/report hooks. Append manifest rows at the end after project formatting. Verify representative workloads untimed. Identity queries can be optimized into constants, so inspect generated code and describe benchmark scope honestly. Do not introduce timing solely to measure two constant-returning wrappers; defer measurements to a meaningful OS family milestone unless a defensible workload measures actual work.

Run bounded affected compiler/native/editor checks during implementation. At final acceptance run the whole compiler crate as requested by the owner and the whole editor crate. Compare failures with the accepted branch baseline; repair new regressions and preserve baseline evidence. Run depth canaries if shared parser/analyzer dispatch changes. Format the entire worktree before source checkpoints, preserving protected owner edits.

Import closed evidence into the branch SQLite database, read it back exactly, then remove loose copies. Reuse the branch Cargo target with incremental compilation disabled. Commit explicit paths only. Finish with a handoff listing commits, source overlaps, commands, accepted target mappings, native platforms actually executed, deferred measurements and integration instructions. Preserve the original path evidence; never replace the main SQLite database with the branch database.

## Follow-on selection

After this slice, propose the next OS family from researched APIs that fit existing primitive bindings. Environment-dependent operations such as home and temporary directories require explicit missing-variable, Unicode, failure and platform contracts. Filesystem handles, databases, networking and concurrency remain dependent on their resource and runtime foundations. Variadic path operations require a separate rest-binding bridge proof. Stop at the accepted checkpoint and hand off before expanding scope.

## Agent prompt

Continue the isolated standard-library worktree from HEAD 6c5fcc7. Read AGENTS.md, docs/development-workflow.md, docs/handoff-2026-10-07-lexical-path.md and the main tree's docs/parallel-os-identity-plan.md. Implement gates O1–O3 for std:os.platform() and std:os.arch(): official-source contracts, zero-argument native bridge proof, focused TDD, compiler/native/editor support, complete behavioral IntelliSense, executable examples, matched untimed benchmark support, formatting and SQLite evidence. Keep compiler core and callable work untouched. Preserve the accepted path commits and measurements. Do not refresh main consumers, merge into main or replace its database. Run the complete compiler and editor crates at final acceptance, classify baseline failures, commit explicit paths, and provide an integration handoff. Windows native acceptance remains pending unless actually executed.
