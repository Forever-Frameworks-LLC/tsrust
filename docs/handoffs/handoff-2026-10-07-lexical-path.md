# Handoff: accepted lexical path family

Date: 2026-10-07. The UTC result filenames fall on 2026-10-08.

## Outcome and integration boundary

The bounded family is accepted on `codex/stdlib-path`, based on `8fe73b5`. `std:path` now exports dirname, extname, isAbsolute and normalize alongside compatible basename. Compiler bindings, generated native execution, imported editor assistance, executable catalog/guide examples, matched benchmark support and evidence are complete. This does not accept the entire standard library or filesystem resources.

Worktree: `/Users/seancotter/.codex/worktrees/stdlib-path/tsrust`. The main checkout and its protected owner changes were never written or staged by this workstream. No other agent or chat was created or messaged. Combined-checkout integration and refresh of the five main consumers remain the integration owner's work. Branch-local success is not combined acceptance.

Feature checkpoints, in order:

- `3492e21`: dirname/extname compiler, runtime, editor, examples, benchmark support and focused acceptance.
- `b57ec2a`: isAbsolute/normalize, boolean native bridge and equivalent focused acceptance.
- `db30ab3`: family regression and initial measurement review; independent Node reference source and stronger source guards.
- `48ae800`: verified extname/normalize allocation optimization and affected-only measurements.
- The closing documentation commit supplies this handoff, current coverage and the completed checklist.

Shared additive integration files are `crates/tsrust-compiler/src/native_bindings.rs`, the path support block in `emit.rs`, the explicit native effect registry in `error_effects.rs`, `crates/tsrust-lsp/src/builtins.json`, existing project/editor tests, and benchmark manifest/oracle/runner/profile/report hooks. Resolve these small additions against advancing language work. Parser, type representation, ownership/loan analysis and callable carriers are unchanged.

## Acceptance

Whole compiler: 2,221 passes and exactly two known baseline callable-replacement failures. Whole editor: 341/341. No new failure is dismissed as baseline. Optimized-source supplements pass six runtime/reference tests, two allocation tests, two O0/O3 project/native tests, imported editor help and four executable catalog examples. Both guide programs and both assertion fixtures execute through the current project CLI after formatting.

Portable helper checks compare 1,172 dirname/extname cases and 3,420 normalization/root pairs with independent Node POSIX/Windows references, alongside 4,316 existing basename comparisons. Public native integration is accepted on macOS. Windows native execution is not claimed. Ordinary normalization is idempotent; special Windows device/colon forms retain the reference's two-pass lexical behavior. No filesystem access, symlink resolution or containment guarantee is introduced.

Project-wide formatting runs before every source checkpoint. The fixture corpus verifies token preservation, idempotence and valid-source behavior. Fifteen baseline fixture layout changes are token-identical, excluded from commits and restored after the closing checkpoint. Protected main owner files are untouched.

## Measurements and review

The original protocol remains 8 warmups, 31 samples and 3 processes. Each engine/workload has 93 samples. Native and Node-after-tsc are the only measured engines; execution-only scope excludes startup and compilation timing.

| Operation | Latest native/Node mean throughput ratio | Record |
|---|---:|---|
| basename | 0.743 | Original family run |
| dirname | 1.486 | Original family run |
| extname | 1.758 | Optimized affected run |
| isAbsolute | 4.051 | Original family run |
| normalize | 1.362 | Optimized affected run |

Original run: `runtime/results/2026-10-08T00-59-07-862Z.json`, 930 samples. Optimized affected run: `runtime/results/2026-10-08T01-04-01-192Z.json`, 372 samples. They remain separate. Original extname/normalize ratios are 0.707/0.714; native mean throughput improves 2.483x/1.910x in the supplemental run. Node mean rates differ by less than 0.3%. The unchanged basename remains a slower signal; no historical comparable path measurement exists in canonical analytics. These bounded workloads are not isolated API latency or allocation-parity claims.

Focused native helper witnesses reduce extname allocations from two to one and representative normalization from four to one. No matched allocation profile is measured. Normalization reserves input length, so shorter output can retain spare capacity.

Untimed five-case records: `runtime/results/2026-10-08T00-53-21-791Z-verify.json` for all five operations; `runtime/results/2026-10-08T01-03-57-385Z-verify.json` for the two optimized operations. Current manifest rows carry accepted measurement identities and unchanged source hashes/operation counts.

## Evidence and reproduction

`benchmarks/records.sqlite` in the library worktree is the durable evidence store. It retains original failures, repairs, regression outputs, source/layout proofs, raw verification/timing records and acceptance proofs. All 64 pre-cleanup records are physically exported and verified by exact hash/size, then scratch exports are removed. Restoration and cleanup receipts bring the store to 67 records. Analytics has two measured runs, seven workload rows, 1,302 samples, 14 summaries and zero profiles. Exact originals remain restorable.

The measured runs are indexed through existing `analytics.derive`, with explicit execution-only acceptance and profilesAccepted=false. The older combined ingestion CLI requires runtime/profile acceptance together; no profile acceptance is invented. Correctness-only records create no timing rows.

Use Node `/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node`, Homebrew `/opt/homebrew/bin/python3`, `CARGO_TARGET_DIR=/private/tmp/tsrust-stdlib-path-target`, `CARGO_INCREMENTAL=0`, and Cargo `--offline --locked`. Benchmark dependencies were read through a temporary symlink to the main checkout's dependency installation; that symlink is removed. Restore dependencies before running the formatter or benchmark tools.

Representative affected commands:

```sh
cargo test --offline --locked -p tsrust-compiler --test native_path --test native_path_allocations
cargo test --offline --locked -p tsrust-compiler --test project lexical_path
cargo test --offline --locked -p tsrust-lsp --test intelligence lexical_path
cargo test --offline --locked -p tsrust-lsp --test project_intelligence lexical_path
node --test benchmarks/runtime/lexical-path-registration.test.mjs
node benchmarks/runtime/run.mjs --verify --engines=native,node-tsc --only=std_path_basename_ascii,std_path_dirname_ascii,std_path_extname_ascii,std_path_is_absolute_ascii,std_path_normalize_ascii
node benchmarks/runtime/run.mjs --execution-only --engines=native,node-tsc --only=std_path_basename_ascii,std_path_dirname_ascii,std_path_extname_ascii,std_path_is_absolute_ascii,std_path_normalize_ascii
```

Do not repeat the completed whole or timing passes merely for presentation. Source changes after integration require the affected combined checks. The main consumer refresh follows successful integration and uses atomic replacement.

For serialized canonical evidence integration, export branch records to an empty temporary root with `store.py restore`, then import using the same logical paths and original kinds. Compare existing canonical identities first. Identical bindings deduplicate; a differing binding must retain new bytes under an explicit content-addressed record identity, never overwrite history. Import proofs and raw bytes before deriving each measured run with its original proof hash. Do not copy one SQLite database over another or merge samples across runs. Remove temporary exports only after exact readback.

Cleanup removes 309 obsolete executable outputs/intermediate trees, about 5.95 GiB. The library Cargo target remains about 1.2 GiB; branch outputs about 57 MiB. Current debug/release CLI consumers and reusable libraries remain. An initial harness invocation briefly created worktree-local release intermediates; those are removed. Rebuilding pruned test executables is partly cold. No cache rebuild follows cleanup. Only one concise current-gate log remains outside SQLite.

## Compiler dependency ledger and next candidate

No compiler prerequisite blocks this accepted family. The two preserved baseline failures remain the language owner's work:

1. `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`.
2. `generic_call_validation::branch_flow::invocation_worklist::mutable_environment_sequence_tests::owned_factory_updates_are_point_sensitive_and_instances_stay_separate`.

A bounded next candidate is target identity through `std:os.platform` and `std:os.arch`. Primitive string results, fixed-arity metadata, bundled feature selection and zero-argument native calls already exist. Before accepting that family, prove a zero-argument string-return execution witness and freeze platform/architecture mappings. No resource, async or dynamic-value prerequisite is required. Variadic path join/resolve still require an audited rest-binding bridge. Filesystem, JSON, database and networking families retain the plan's separate prerequisites. No next family is started.
