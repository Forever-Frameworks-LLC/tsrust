# Handoff: standard-library integration accepted on main

Source, immutable evidence, and all five executable consumers are integrated. The bounded scope is `std:path` basename, dirname, extname, isAbsolute, normalize and variadic join, plus `std:os` platform, arch and endianness. No further API family starts in this checkpoint.

## Published source and protected edits

`75429e2` merges main callable checkpoint `09a84aa` into accepted standard-library tip `d499590`. Source publication fast-forwards main to `31e367f`; the final handoff commit follows on both main and `codex/stdlib-path`. All five consumers build from `31e367f`, whose compiler/editor source is unchanged by the documentation handoff.

The only textual merge conflict is the accepted checklist in `docs/parallel-standard-library-plan.md`; resolution retains accepted library work. Shared compiler and editor changes merge additively. Main retains checked invocation IDs, independent declaration preparation, selected environment/method failure coordinates, native generic construction, source-selected methods and exact invocation failure propagation. Native binding metadata, native-origin shapes, borrowing/rest/spread emission and pure library error effects remain intact.

The fifteen owner fixture edits remain byte-exact with their original Git index entries. None are reset, stashed, copied, staged or committed. Worker-only formatter layouts are token-verified and restored from unchanged Git baselines before publication. Main source ownership remains exclusive through publication, storage import, consumer refresh and final verification.

## Integration repairs and acceptance

Three bounded repairs complete combined acceptance:

- `dc5b04f` indexes caller binding identities once when projecting outcomes, preserving unknown identities, aliases and outgoing order.
- `b6dd9e0` returns ordinary operator frames before entering conditional arms. Actual compiled frames identify the stack failure in parser recursion. Grammar, source limits and default worker stack remain unchanged.
- `31e367f` completes only invocation-free required-parameter return summaries eagerly through the original contextual evaluator. Requests, capture validation, producer identity, effects and general recursive worklist behavior remain intact. Defaults, rest packing, captured values, writes and literal returns are negative controls. The queue-order experiment does not solve sequential discovery and is not retained.

The complete compiler milestone, with affected groups resumed after repair, accepts **2,272 passes and three intentional main witnesses**. Original completed groups remain preserved. The initial conditional stack abort and interrupted slow project check remain explicit in `stdlib-integration-compiler-full.log`; accepted replacements are `stdlib-integration-terminal-library-final.log`, `stdlib-integration-project-terminal.log` and `stdlib-integration-affected-native-final.log`. The affected library has 512 passes and one known witness; project has 87 passes; conditional expressions have 28 passes. The unchanged 128/1024 generic-count boundaries pass, and the entire project group finishes in 40.12 seconds instead of the original unfinished run exceeding thirty minutes.

The complete final LSP crate passes **346 checks with zero failures**, including executable catalog examples and imported hover, completion, parameter help and stored variadic signatures. Ten source/ABI/oracle registration checks pass. All eight rebuilt normal-stack canaries pass: closures, for-of, owned arrays, nullish coalescing, nullish assignment, higher-order function values, number operators and type syntax. Imported aliases, stored calls, spread evaluation/borrowing, ownership, diagnostics and native output are verified together.

The preserved main witnesses are:

1. `generic_call_capture_effects::generic_callable_replacement_keeps_saved_values_and_runtime_selection`.
2. `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`.
3. `generic_call_validation::branch_flow::invocation_worklist::mutable_environment_sequence_tests::owned_factory_updates_are_point_sensitive_and_instances_stay_separate`.

Their test sources match main. Baseline evidence remains `runtime/diagnostics/checked-invocation-effects-2026-10-07/tsrust-invocation-effects-native-repaired.log` and `runtime/diagnostics/rust-warning-cleanup-2026-10-07/tsrust-cleanup-unit.log`. No integration regression is hidden as a baseline witness.

## Benchmark and platform boundaries

All eleven matched path/OS workloads pass five untimed correctness cases against native and Node after TypeScript compilation under the final compiler. The final record is `runtime/results/2026-10-08T04-36-02-413Z-verify.json`: correctness-only, zero execution measurements. Original lexical-path/join timings, allocation profiles, protocols, sample counts, source hashes and toolchain identities remain exact. OS constant workloads remain untimed. Prior path workloads are not retimed.

Actual native execution is macOS ARM64. Independent Windows lexical helper checks and sixteen cross-target metadata cases remain separate from native Windows or other-target execution. Review of accepted performance signals and the one-reservation join allocation repair is retained in the lexical-path and path-join handoffs.

## Canonical SQLite integration

Repository export/import/read tools integrate every worker record through an empty temporary root. Every imported binding preserves its logical identity, kind, SHA-256 and immutable bytes, and every exported copy is removed only after exact readback. Main SQLite is never replaced. All 36,958 original main bindings and original analytic rows remain exact. All six worker analytic runs and every associated workload, sample, summary and profile row match the original worker database.

Final integration imports **253 worker bindings**. Main contains **37,211 records and 24,734 unique blobs**, including the final verification receipt. Analytics contain **14 runs, 399 workload bindings, 60,636 timing samples, 652 summaries and 588 profile samples**. The added 2,418 timing samples and 18 profile samples retain their original measurement identities. Combined verification adds no timing samples.

Main already contains 758 absolute historical record identities. The global `store.verify` path-policy check rejects these existing names; they are preserved, not renamed or overwritten. Verification uses the repository blob decoder for every unique blob, SQLite integrity and foreign-key checks, exact original-row comparison, and strict safe relative paths for every new binding. Export path restrictions remain unchanged. This compatibility limitation concerns historical record names, not blob corruption or an unverified import.

Exact inventory, source/consumer identity, original-row preservation and final counts are in `stdlib-integration-final-verification.json`. The initial storage receipt and final consumer/cleanup receipts retain their own original bytes under `stdlib-integration-` identities. The normal-frame failure profiles and rejected queue experiment remain durable evidence.

## Executable consumers and cleanup

Using `/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0` and offline/locked Cargo, rebuild the debug CLI/LSP and release CLI from `31e367f`. Copy each new executable to a sibling `.refresh-tmp`, then atomically rename all five before smoke checks:

- `target/debug/tsrust`
- `target/debug/tsrust-lsp`
- `target/editor/debug/tsrust`
- `target/editor/debug/tsrust-lsp`
- `target/release/tsrust`

All three CLIs check and execute imported path/OS assertions with `stdlib integrated` output. Both LSP consumers publish empty checked diagnostics, document all eight imported API calls, preserve the stored join rest signature, and shut down successfully. The smoke harness waits for checked diagnostics before requesting project hover. Initial premature/import-token probes are preserved separately from accepted smoke evidence. Exact executable hashes are in `stdlib-integration-consumer-refresh.json`; accepted protocol results are in `stdlib-integration-consumer-smoke.json` and `stdlib-integration-consumer-smoke-checked.log`. The executable smoke script is stored as `stdlib-integration-consumer-smoke.mjs`.

Prune 555 obsolete test/native executables, obsolete local Cargo cache copies, expired worker consumers, generated documentation executables and benchmark driver products/configurations. Retain the documented main and isolated Cargo targets, with incremental compilation disabled, and all five refreshed main consumers. No cache-warming rebuild follows cleanup. Final main status contains only the fifteen protected owner edits; the isolated source worktree is clean.

## Evidence access

Read canonical evidence from main using pinned Python:

```sh
/opt/homebrew/bin/python3 benchmarks/storage/store.py read benchmarks/records.sqlite stdlib-integration-final-verification.json
/opt/homebrew/bin/python3 benchmarks/storage/store.py read benchmarks/records.sqlite stdlib-integration-combined-acceptance.json
/opt/homebrew/bin/python3 benchmarks/storage/store.py read benchmarks/records.sqlite stdlib-integration-workloads-final-proof.json
/opt/homebrew/bin/python3 benchmarks/storage/store.py read benchmarks/records.sqlite stdlib-integration-consumer-smoke.json
```

Use Node 24.11.1 first on PATH. Executables and generated benchmark consumers rebuild from committed sources; immutable evidence restores through storage tools. Wider callable replacement and standard-library families remain open after this accepted integrated checkpoint.
