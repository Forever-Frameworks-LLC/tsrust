# Parallel standard library: variadic path joining

## Goal

Implement `std:path.join(...paths: string[]): string` through a reusable homogeneous string rest-binding bridge. Also complete `std:os.endianness(): string` through the accepted target identity bridge. Accept both slices with compiler, native, editor, documentation, benchmark support and durable evidence. Do not replace arbitrary argument support with fixed-arity overloads or require callers to pass an array instead of ordinary arguments.

Continue the isolated standard-library worktree from bd209b4. Read AGENTS.md, docs/development-workflow.md, the lexical-path and OS-identity handoffs, and this plan. Preserve the accepted branch commits and SQLite records. Main integration of those commits remains pending; branch acceptance does not establish combined acceptance. Do not modify main, refresh main consumers, overwrite its database or copy owner fixtures into either tree.

## J0: prove the native rest bridge

Inspect native binding metadata, synthetic declarations, argument preparation, direct native emission and imported function-value forwarding. Use existing language rest and spread capabilities; this work does not authorize changes to parser grammar, general type inference, ownership rules or callable carriers.

Add a minimal failing project/native witness for zero, one and several string arguments, then a reusable leaf bridge. Permit narrowly scoped changes to native binding metadata and native-call handling, preferably in separate library-owned helpers. Preserve existing fixed and optional bindings. Record every shared source overlap in the handoff. If a required change extends into compiler core or unfinished callable work, preserve the minimal witness and request that dependency before proceeding with dependent implementation.

Prove exactly-once left-to-right argument evaluation, borrowing without implicit string clones, preservation of caller values, temporary lifetime safety and argument-failure cleanup. Check wrong scalar and mixed element types. Exercise aliases, stored function values and concrete rest callable annotations within accepted language capabilities. Exercise homogeneous array spread where currently supported; record any missing native spread/forwarding path as required work rather than quietly claiming complete variadic support. Do not invent a public argument-count limit to bypass the bridge.

## J1: lexical joining

Freeze the contract against official Node documentation and independent Node reference execution. Joining uses platform separators and normalization, ignores empty segments and returns a dot for zero or entirely empty segments. It performs lexical processing without filesystem access. Research and test POSIX roots, repeated separators, dot and parent segments, trailing separators, Unicode, and Windows drive, UNC and device path behavior. Reuse accepted normalization helpers only after verifying their composition matches joining behavior; do not assume Rust PathBuf push has the reference contract.

Add std:testing project fixtures, independent TypeScript references, source diagnostics and formatter round trips. Compare POSIX and Windows helper branches with independent references on the available host. Keep cross-platform helper coverage separate from actual Windows native execution. Test long inputs and many segments with bounded work; review allocation and scanning behavior before optimizing. Preserve input strings and avoid unnecessary intermediate copies.

## E1: byte-order identity

Implement `std:os.endianness()` with no arguments and the documented BE/LE result contract. Use compilation-target byte-order metadata, not the compiler host or CPU architecture guesses. Prove both selector branches, unsupported selector behavior, imported aliases, stored zero-argument function calls, return typing and arity diagnostics. Record actual native execution separately from cross-target metadata coverage. This independent leaf can proceed if J0 exposes a compiler dependency, but does not satisfy the joining goal.

## J2: editor, guide and workload readiness

Provide complete behavioral IntelliSense for both exports. Keep rest signature help, parameter labels, completion and hover consistent with actual argument preparation and function-value behavior. Include executable examples and expected results, useful edge cases and short ownership notes after behavioral explanations. Keep versions and provenance in developer evidence.

Prepare matched Tsrust/TypeScript workload pairs, independent oracles, token-aware source contracts with mutation controls, registrations, applicable native ABI/observer support and oracle/run/profile-native/report hooks. Append manifest rows at the end after formatting and fingerprint verification. Cover ordinary joins, empty inputs, root handling, many segments and Unicode. Verify selected workloads untimed. Do not claim identity latency measurements from constant-folded endianness calls.

Keep meaningful family measurement and focused implementation acceptance separate. If the joining scope establishes a meaningful path-composition milestone, measure only defensible selected join workloads with the existing execution-only native versus Node-after-tsc protocol and original sample counts. Preserve original records and rerun only affected cases after optimization. Otherwise explicitly defer timing with a named milestone; do not retime accepted path or OS identity workloads merely for presentation.

## Acceptance and handoff

Use focused TDD and bounded compiler/native/editor checks during development. Run all eight depth canaries after shared parser/analyzer dispatch changes. Format the whole worktree before source checkpoints with the verified branch compiler, prove token preservation and idempotence, and preserve unrelated owner edits. At final acceptance run the whole compiler and editor crates, classify failures against the branch baseline and repair new regressions without hiding baseline witnesses.

Reuse /private/tmp/tsrust-stdlib-path-target with CARGO_INCREMENTAL=0 and Cargo offline/locked. Put Node 24.11.1 first on PATH and use /opt/homebrew/bin/python3 for SQLite tools. Keep branch evidence canonical: import closed logs and results, read them back exactly, then remove loose copies. Preserve all accepted path and OS record identities, analytics counts and restoration guarantees. Clean obsolete outputs without rebuilding merely to warm caches.

Commit explicit paths. Finish with an integration handoff listing commits, API contracts, bridge support, source overlaps, any compiler dependency, executed native platforms, correctness and measurement boundaries, commands and exact SQLite identities. Stop at the accepted checkpoint before adding resolve, filesystem resources, JSON, database or networking APIs. Working-directory-dependent path resolution needs its own environment and failure contract.

## Sources

- [Node path joining](https://nodejs.org/api/path.html#pathjoinpaths)
- [Node byte-order identity](https://nodejs.org/api/os.html#osendianness)

## Copyable worker prompt

Continue the isolated standard-library worktree from bd209b4. Read AGENTS.md, docs/development-workflow.md, docs/handoff-2026-10-07-lexical-path.md, docs/handoff-2026-10-07-os-identity.md and the main tree's docs/parallel-path-join-plan.md. Complete J0–J2 and E1: reusable homogeneous variadic string native binding support, std:path.join(...paths) and std:os.endianness(), with compiler/native/editor/docs, formatted executable examples, matched benchmark support and SQLite evidence. Prove native argument evaluation, borrowing, cleanup, spread and function-value forwarding before claiming the bridge complete. Native binding leaf changes are allowed; parser, general inference, ownership and callable carrier changes require a minimal dependency witness and coordination. Preserve accepted branch commits and evidence. Do not merge into main, modify its consumers or database, or touch owner fixtures. Run full compiler/editor checks at final acceptance, classify baseline failures, commit explicit paths and provide an integration handoff with actual platform and measurement limits. Arbitrary argument counts are required; fixed-arity overloads are not acceptance.
