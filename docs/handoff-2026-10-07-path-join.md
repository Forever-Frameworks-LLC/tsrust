# Handoff: variadic lexical path joining and byte order

This isolated slice accepts `std:path.join(...paths: string[]): string` and `std:os.endianness(): string`. [The contract](path-join-contract.md) freezes behavior, ownership, reference coverage and measurement boundaries. It does not accept the rest of either module. Main integration remains separate.

## Checkpoints and integration

Continue from `bd209b4` in `/Users/seancotter/.codex/worktrees/stdlib-path/tsrust`, branch `codex/stdlib-path`.

- `0a436b7`: reusable homogeneous native string rest bindings, lexical joining, target byte order, compiler/native/editor documentation and matched workload support.
- `de94c6d`: repair the pre-existing missing native profile classifier import and capture the active path/OS oracle inputs. Original failed and preliminary profile records remain preserved.
- `d1b6ff8`: reserve composed text capacity once after original timing/profile review; add an executable failing-to-passing allocation witness and rerun affected semantics, documentation and workload measurements.
- `7713745`: align the independent Unicode checksum oracle with the established Unicode-scalar observer. The original failed verification is retained; path output comparisons already passed.

Shared source overlaps are compiler `native_bindings.rs`, `analysis.rs`, `native_emit.rs`, `emit.rs`, `syntax.rs` (internal spread classification only), `spread_builtins.rs`, `spread_calls.rs`, `spread_emit.rs`, `spread_builtin_emit.rs`, `error_effects.rs`, project and native-binding tests; LSP `builtins.json`, `intelligence.rs` and editor tests; runtime manifest, oracle, run/profile/report hooks and OS source contracts; standard-library guide and coverage. Library-owned path and OS helpers hold lexical/target behavior. No grammar, general inference, ownership rule or callable carrier representation changed. Native-origin argument/shape handling is the narrow analyzer overlap. No compiler dependency remains for the accepted slice.

Integrate these commits after the accepted path and OS series in order. Resolve shared files narrowly against the advancing language branch and rerun affected combined checks. Branch acceptance does not establish combined acceptance. Main sources, consumers, owner fixtures and SQLite are untouched.

## Native bridge and behavior

The bridge implements a single homogeneous read-only string rest tail. Ordinary calls accept zero, one and arbitrary many strings; no fixed-arity overload ceiling is introduced. Native metadata checks all argument elements. Aliases, stored values, homogeneous array spread and concrete rest Function annotations execute. Known native calls borrow caller strings and arrays. Abstract language Function rest forwarding uses its existing consuming pack contract, lends pack contents to the helper and disposes the pack. No implicit string clone or new read-only language rest syntax is required.

Both optimization levels prove exactly-once left-to-right evaluation, short-circuit argument failure, caller preservation, temporary lifetime safety and balanced cleanup. Wrong scalar/mixed element types reject. A 128-ordinary-argument native witness and 1,024-segment helper case prove unbounded metadata rather than fixed overloads. Existing fixed/optional bindings retain their controls.

Joining ignores empty segments, inserts the target separator and normalizes lexically. Zero/all-empty input returns dot; later rooted segments append. Roots, parents, trailing separators, Unicode, Windows drive/UNC/device behavior and long inputs are frozen against independent Node references. The corpus has 707 input vectors and 1,414 POSIX/Windows comparisons. Actual native execution is macOS ARM64 only; Windows helper execution is not Windows native acceptance.

Byte order uses compiled `target_endian` metadata, returning BE or LE with explicit unsupported-selector compilation errors. Sixteen target-metadata cases exercise both branches independently. Aliases, stored calls, result typing and zero arity pass; actual native execution returns LE. No host substitution or CPU-name guess is used.

## Editor, examples and formatted sources

Completion, imported hover and rest parameter help agree on `join(...paths: string[]): string`; later arguments retain the rest parameter label. Byte order exposes a zero-argument string signature. Complete behavioral documentation precedes ownership notes. Catalog examples, project fixtures and CLI execution pass after token-preserving, idempotent formatting. Matched TypeScript fixtures remain independent references.

Whole-project formatting precedes every source checkpoint. Fifteen pre-existing fixture layout changes have identical tokens and are excluded from source commits and restored before handoff. No owner fixture is copied.

## Acceptance and measurement

The complete compiler crate has 2,232 passes and exactly the two accepted baseline callable failures:

1. `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`.
2. `generic_call_validation::branch_flow::invocation_worklist::mutable_environment_sequence_tests::owned_factory_updates_are_point_sensitive_and_instances_stay_separate`.

The new capacity witness adds one passing test after this full run, for 2,233 unique accepted compiler passes. Affected native/borrow/spread, independent reference and executable editor documentation checks pass after optimization; completed unaffected groups are preserved. No new compiler regression remains. The complete LSP crate has 345 passes and zero failures. The focused normal-stack depth batch passes 44 tests; all depth guards also execute in the complete compiler run. Existing worker stacks and source-depth limits remain unchanged. Full regression logs and supplemental checks remain canonical; successful groups are not restarted.

Original execution record: `runtime/results/2026-10-08T03-26-23-155Z.json` (SHA-256 `ce496a24bad0c9fa16eb705f275ee44e3bf21113da31e256792d2483bfaae0cf`). Optimized record: `runtime/results/2026-10-08T03-31-28-788Z.json` (SHA-256 `f66b6b757ab8bf0d4b42896239020ff9c3e02e4fb83937074bdb6143d89c43fc`). Correctness-only record: `runtime/results/2026-10-08T03-22-24-689Z-verify.json`. Each timing uses 3 processes, 8 warmups and 31 samples per process: 93 samples per engine/workload, 558 per record. Original and optimized profiles each have 3 samples per workload, zero bytes after output/final disposal; profiles are `runtime/diagnostics/2026-10-08T03-27-44-622Z-native-allocations.json` and `runtime/diagnostics/2026-10-08T03-31-32-304Z-native-allocations.json`. Their accepted proof identities are `runtime/diagnostics/path-join-original-proof.json` and `runtime/diagnostics/path-join-capacity-proof.json`.

Native average throughput before/after reservation is 7.32/11.56 million ordinary joins/s, 10.66/11.40 million empty/root joins/s and 0.99/1.19 million Unicode joins/s. Final matched native/Node-after-tsc ratios are 2.03×, 1.22× and 3.23×. These host-run signals preserve contemporaneous matching; no cross-run hardware/toolchain aggregation or isolated-latency claim is made. Composition helper reallocations fall to zero; native profile totals retain only 12/12/16 output-array setup growth events across 20,000 operations, with unchanged allocation counts. No further optimization is justified by this bounded review.

The three new join workloads cover ordinary rooted joins, empty/root cases and 67-segment Unicode spread. Matched sources, source mutation controls, formatted hashes, native ABI observers, independent scalar-checksum oracles and oracle/run/profile/report registrations are complete. All four new workloads pass untimed native versus Node-after-tsc verification over five cases each. Endianness remains untimed: optimized LLVM retains LE and eliminates its wrapper. Optimized Unicode joining retains the lexical join and normalization operations.

The execution-only join protocol includes input setup, reference adaptation, lexical work, retained outputs and observation. It does not isolate API latency or establish allocation equivalence. Existing process/warmup/sample counts remain unchanged. Prior path/OS workloads are not retimed.

## Reproduction and evidence integration

Use Node 24.11.1 first on PATH, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/private/tmp/tsrust-stdlib-path-target`, Cargo offline/locked and the verified branch compiler through TSRUST. Export both Cargo variables to the runtime harness; its branch executable consumer is `target/release/tsrust`.

```sh
cargo test --offline --locked -p tsrust-compiler --test project native_join
cargo test --offline --locked -p tsrust-compiler --test project variadic_native_path_join
cargo test --offline --locked -p tsrust-compiler --test project os_endianness
cargo test --offline --locked -p tsrust-compiler --test native_bindings
cargo test --offline --locked -p tsrust-compiler --test native_path join_matches_independent
cargo test --offline --locked -p tsrust-compiler --tests depth
cargo test --offline --locked -p tsrust-lsp path_join
cargo test --offline --locked -p tsrust-lsp os_identity
TSRUST=/private/tmp/tsrust-stdlib-path-target/debug/tsrust node tools/format-project.mjs
node --test benchmarks/runtime/path-join-registration.test.mjs benchmarks/runtime/os-identity-registration.test.mjs
node benchmarks/runtime/run.mjs --verify --engines=native,node-tsc --only=std_path_join_ascii,std_path_join_empty_roots,std_path_join_many_unicode,std_os_endianness
node benchmarks/runtime/run.mjs --execution-only --engines=native,node-tsc --only=std_path_join_ascii,std_path_join_empty_roots,std_path_join_many_unicode
cargo test --offline --locked --no-fail-fast -p tsrust-compiler
cargo test --offline --locked --no-fail-fast -p tsrust-lsp
```

Branch `benchmarks/records.sqlite` is canonical. Serialize integration: export into an empty temporary evidence root, preserve original logical identities/kinds, compare existing bindings, deduplicate identical bytes and preserve differing identities under explicit new addresses. Never replace main SQLite with branch SQLite. Every closed loose record is imported and read back exactly before deletion. Earlier 95 record bindings and original analytics rows are preserved byte for byte: 2 runs, 7 workloads, 1,302 timing samples and 14 summaries remain exact prefixes. Current analytics contain 6 runs (4 execution, 2 profile), 19 workload bindings, 2,418 timing samples, 26 summaries and 18 profile samples. Exact export/restoration of the optimized raw record passes; every blob verifies. Diagnostic records use `stdlib-join-` / `stdlib-endianness-` identities, including full logs, original red witnesses, layout proof, performance review and preservation proof.

Cleanup removes 713 obsolete files, 6,307,439,487 bytes (5.874 GiB). The shared Cargo target remains about 1.2 GiB; branch outputs about 31 MiB. Reusable dependency libraries and current branch debug/release CLI consumers remain. No Cargo rebuild follows cleanup. A concise `target/editor/stdlib-current.log` is the only current-gate log outside SQLite. Dependency links and verified unrelated layout changes are removed/restored at final handoff. No new timing is derived from correctness-only or constant-folded identity records.

Final canonical storage contains 170 record bindings and 154 unique blobs, all verified. The final six matched workload registration checks pass after cleanup.

Stop here before resolve, filesystem resources, JSON, database or networking APIs. Working-directory-dependent resolution needs its own environment and failure contract.
