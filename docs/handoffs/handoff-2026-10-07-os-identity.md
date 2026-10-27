# Handoff: executable OS target identity

This slice accepts `std:os.platform()` and `std:os.arch()` only. Both take exactly zero arguments and return strings using the existing primitive native bridge. [The contract](../milestones/os-identity-contract.md) freezes exact mappings, unsupported components, and executable-target semantics. The whole OS module remains outside scope.

## Checkpoints and source overlaps

Continue from the accepted path tip `6c5fcc7` on `codex/stdlib-path` in `/Users/seancotter/.codex/worktrees/stdlib-path/tsrust`.

- `9ad333b`: compiler bindings, per-export target emission, independent metadata witnesses, executable fixtures, complete catalog/editor support, guide, and matched benchmark support.
- `424c4b4`: update the native-effect audit witness to explicitly accept these two pure target constants. The original audit correctly exposed its stale allowlist during the whole compiler run; no unreviewed binding was accepted.
- `368f6eb`: project-level unknown-export diagnostics and final imported API checks.

Shared additive paths: compiler `emit.rs`, `error_effects.rs`, `native_bindings.rs`, `stdlib/mod.rs`, compiler project/registry tests, LSP `builtins.json` and intelligence tests, runtime `manifest.json`, `oracle.mjs`, `run.mjs`, `profile-native.mjs`, `report.mjs`, and standard-library guide/coverage. Library-owned OS files contain all mappings and bridge logic. No parser, general analyzer dispatch, callable carrier, type representation, ownership, or loan change was made. Native error-effect dispatch adds an audited leaf classification.

Main is untouched: no merge, main consumer refresh, owner fixture copy, or main SQLite write. Main has advanced since the branch base, so these branch checks do not establish combined acceptance. Integrate these commits after the path series in order, resolve shared files narrowly, then rerun affected combined checks. Refresh main live consumers only after combined acceptance.

## Execution and acceptance

Native execution is macOS ARM64 with Node 24.11.1 and Rust 1.96.1: `darwin arm64`. The imported fixture proves aliases, stored zero-argument calls, string results, equality assertions, and project CLI execution after full formatting. Arity and wrong return-type assignments reject; a missing export reports `module-native` with `does not export`. A referenced export emits only its own target component check.

Independent metadata cases cover 16 actual Rust target triples: macOS, Windows, Linux x86/ARM/LoongArch/RISC-V/s390x/MIPS/PPC, Android, AIX, FreeBSD, OpenBSD, Solaris, and illumos. Rust `--print cfg --target` supplies compilation metadata; test predicates are renamed and selected bridges execute together on this host. Big/little MIPS produce `mips`/`mipsel`; both PPC64 byte orders produce `ppc64`. Unsupported selectors fail compilation explicitly. Cross-target metadata is not native execution on Windows or any other platform.

Both catalog examples execute after formatter idempotence checks. Imported hover, completion and zero-parameter signature help pass. Full formatting also changes fifteen pre-existing branch fixture layouts with proven identical tokens; those unrelated layout changes are restored before handoff and excluded from all source commits.

Complete compiler acceptance has 2,227 unique passes and the two baseline failures below. The original whole pass had 2,226 passes and three failures; the affected whole-library rerun has 478 passes and the one library baseline failure, replacing the repaired stale audit failure without restarting successful integration groups. Final focused project and registry checks also pass. The whole editor crate has 343 passes and no failures. The compiler formatter corpus proves token preservation, idempotence, and formatted standalone behavior; imported native/project checks prove formatted-source execution. Two accepted baseline callable failures remain the language owner's work:

1. `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`.
2. `generic_call_validation::branch_flow::invocation_worklist::mutable_environment_sequence_tests::owned_factory_updates_are_point_sensitive_and_instances_stay_separate`.

## Workload support and measurement boundary

`std_os_platform` and `std_os_arch` have matched `.tsr`/`.ts` sources, token-aware source guards and mutation controls, current fingerprints, existing string-array ABI observers, registration checks, independent process-binary metadata oracles, and run/oracle/profile/report hooks. Both pass native versus Node-after-tsc harness verification over five cases each. Correctness record: `runtime/results/2026-10-08T02-22-23-441Z-verify.json`; `measurementScope=correctness-only`, startup/compilation/execution measurement arrays empty.

Optimized LLVM retains static `darwin` and `arm64` strings and contains no identity wrapper symbols. Generated Rust retains target predicates rather than substituted compiler-host names. These workloads exercise output retention and observation; no isolated identity latency, allocation equivalence, or measured performance claim is made. Timing is deferred to a meaningful OS family milestone. Comparing native and Node results requires matching compiled targets.

## Reproduction commands

Use this worktree, Node 24.11.1 first on PATH, `CARGO_INCREMENTAL=0`, and `CARGO_TARGET_DIR=/private/tmp/tsrust-stdlib-path-target`. Cargo is offline/locked. One target is reused; main targets and consumers are never selected.

```sh
cargo test --offline --locked -p tsrust-compiler --test project os_identity
cargo test --offline --locked -p tsrust-compiler --test native_bindings os_identity
cargo test --offline --locked -p tsrust-compiler --lib executable_target_mappings_follow_rust_target_metadata
cargo test --offline --locked -p tsrust-lsp os_identity
TSRUST=/private/tmp/tsrust-stdlib-path-target/debug/tsrust node tools/format-project.mjs
node --test benchmarks/runtime/os-identity-registration.test.mjs
node benchmarks/runtime/run.mjs --verify --engines=native,node-tsc --only=std_os_platform,std_os_arch
cargo test --offline --locked --no-fail-fast -p tsrust-compiler
cargo test --offline --locked --no-fail-fast -p tsrust-lsp
```

Whole acceptance commands already completed should not be restarted merely for presentation. The repaired native-effect group and supplemental project/registry checks preserve earlier completed checks. No additional depth run is required by this leaf-only bridge; existing depth tests remain in the whole compiler pass.

## Evidence integration and cleanup

Branch `benchmarks/records.sqlite` is canonical. The original 67 path record identities and original analytics counts (2 runs, 7 workloads, 1,302 samples, 14 summaries) are preserved. OS adds correctness and diagnostic records only. Every closed record is read back byte for byte before its loose copy is removed. Diagnostic identities begin `stdlib-os-`; the preservation inventory and layout/optimized-code proofs are also stored there.

Serialize canonical integration. Export branch records with the repository storage tools into an empty temporary evidence root; import under their original logical identities and kinds. Compare existing bindings first, deduplicate identical bytes, and preserve a differing identity under an explicit new content address. Never copy one SQLite database over another, overwrite an old record, or derive timing samples from this correctness-only run. Remove exports only after exact readback. Final storage has 95 record bindings and 85 unique blobs; all original identities and analytics counts are unchanged. Cleanup prunes 322 obsolete executable/intermediate outputs, 5.979 GiB. The shared workstream Cargo target remains about 1.2 GiB, and branch outputs about 31 MiB. The existing runtime harness unconditionally built incidental worktree-local release intermediates; those are removed. Export both Cargo environment variables before later harness reproduction. Reusable libraries and current branch compiler consumers remain; no cache rebuild follows cleanup. Only a concise current OS gate log remains outside SQLite.

## Next bounded candidate

Propose `std:os.endianness(): string` next, without implementing it here. [Official Node documentation](https://nodejs.org/api/os.html#osendianness) defines compiled CPU byte order as `BE` or `LE`. Rust target-endian metadata and the accepted zero-argument string bridge already supply its primitive prerequisites; freeze exact unsupported-selector behavior, then add independent big/little native branch witnesses and full editor/docs/workload support. It is another constant identity API, so timing still needs a meaningful OS workload.

Home and temporary directory APIs require explicit missing-variable/empty-variable precedence, POSIX account and Windows profile fallbacks, invalid Unicode handling, native failure transport, and trailing-separator/platform contracts. Existing primitive strings alone do not prove those contracts. Filesystem resources, databases, networking and concurrency remain behind their respective resource/runtime gates. Variadic paths still need a separate rest-binding proof. Stop at this accepted slice before expanding scope.
