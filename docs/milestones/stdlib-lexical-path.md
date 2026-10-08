# Lexical path family acceptance

Base: `8fe73b5`. Branch: `codex/stdlib-path`. Scope: dirname, extname, isAbsolute, normalize. Implementation and measurements remain pending.

## Frozen public contract

All exports accept exactly one borrowed string argument named `path`. Missing, extra, and non-string arguments are compile-time errors. No input mutation, filesystem access, symlink resolution, or current-directory lookup occurs. String results are independently owned valid UTF-8 text; isAbsolute returns boolean. The target selects POSIX or Windows conventions. Internal helpers accept an explicit platform selector. Windows helper tests on macOS do not establish Windows native integration acceptance.

| Export | Purpose and return | Empty input | Roots and separators | Dot components and Unicode |
|---|---|---|---|---|
| dirname | Remove final nonseparator component; retain parent spelling | `.` | POSIX `/` remains `/`; `//a` gives `//`; repeated parent separators remain. Windows recognizes both separators, drive prefixes, and UNC share roots. | No dot resolution; UTF-8 text retained. |
| extname | Final component suffix starting at last dot | empty string | Ignore trailing separators; Windows drive prefix excluded | `.hidden`, `.`, `..` give empty; `...` and `a.` give `.`; `a.tar.gz` gives `.gz`. Unicode retained. |
| isAbsolute | Test lexical root syntax | false | POSIX leading `/`; Windows leading separator or letter-colon-separator. `C:a` is relative; UNC is absolute. | Dot segments do not change result. |
| normalize | Collapse separators and resolve dot components lexically | `.` | Preserve absolute root and final separator; POSIX uses `/`, Windows emits backslash and retains drive/UNC device. | Absolute parents stop at root; relative leading parents survive. Unicode retained. Idempotent for ordinary paths; special Windows device/colon forms can change on a second pass. Not a containment check. |

Reference: official [Node path documentation](https://nodejs.org/api/path.html) and [pinned implementation](https://github.com/nodejs/node/blob/v24.11.1/lib/path.js). Differential checks use Node 24.11.1. Algorithms are independently written; Rust Path semantics are not used as an oracle.

## Integration boundary

Additive shared edits: native binding registry, path bridge emission, explicit native error-effect registry, builtin catalog, project/editor fixtures, benchmark manifest and runner hooks. Parser, ownership representation, callable carriers, and general flow/emission architecture remain outside scope.

Known baseline failures: generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier; generic_call_validation::branch_flow::invocation_worklist::mutable_environment_sequence_tests::owned_factory_updates_are_point_sensitive_and_instances_stay_separate.

Use `/private/tmp/tsrust-stdlib-path-target`, CARGO_INCREMENTAL=0, offline locked Cargo, and worktree-local SQLite. Main consumers and store remain untouched. Focused commands: compiler native_path/native_bindings/project lexical tests; editor project intelligence lexical tests; catalog examples; benchmark registration and untimed matched verification. At P3 run whole compiler/editor and scoped family timing under existing sample protocol.

## Progress

- P0 contract frozen; primitive metadata supports boolean. Native boolean adaptation awaits public execution witness.
- P1 red runtime evidence: unresolved dirname/extname imports in `/private/tmp/stdlib-path-red.log`.
- P1/P2/P3 acceptance, evidence import, commits and handoff remain pending.

## P1 accepted focused checks

Runtime boundary tests and independent Node POSIX/Windows corpus pass: 293 paths, two operations, two conventions (1,172 comparisons), plus unchanged basename corpus (4,316 comparisons). Project test passes native O0/O3, alias/stored calls, input reuse, and exact arity/type diagnostics. The formatted std:testing project fixture passes. Imported editor test passes hover, completion and parameter help. Both catalog examples execute with their stated output after formatting; initial test API naming errors and repaired verification are retained.

Matched dirname/extname sources have exact registered hashes, token-aware scenario guards, independent checksum oracles, native ABI checks, runner/profile/report hooks and appended manifest rows. Registration checks pass at n=0,2,17. Timing remains pending P3. Whole fixture formatter corpus passes token preservation, idempotence and valid standalone checks. Project formatting changes 18 files on first pass, zero on second; 15 baseline fixture layout changes remain unstaged and are excluded from feature commits. No owner edits were copied.

Evidence identities: worktree SQLite records `stdlib-path-{red,green,project-red,project-green,reference,editor,registration,fixture,docs,docs-repair,docs-final,format,format-second,format-corpus,build}.log`. Main store and consumers remain untouched.

## P2 accepted focused checks

The native boolean bridge is verified at O0/O3 through direct and stored imported calls and std:testing assertions. The registry pins exactly one borrowed string input and boolean/string output types. All four exports have exact arity/type diagnostic checks, input reuse, stored alias fixtures, and imported editor hover/completion/parameter help. Four catalog examples execute after formatting with expected output.

Normalization/root differential corpus has 1,710 paths per convention (3,420 pairs), comparing normalization, isAbsolute and second normalization against Node. Ordinary paths also satisfy explicit idempotence assertions. Initial idempotence expectation failed on a reference-compatible device path; corrected scope preserves both first/second differential assertions. Reserved names and special device roots deliberately retain the reference's non-idempotent lexical safeguards. No end-to-end Windows acceptance is claimed.

All four benchmark registrations pass exact hashes, mutation guards, ABI checks and n=0,2,17 native/TypeScript results. isAbsolute observes each boolean through toString and includes that conversion cost. P3 untimed whole-run verification and timing remain pending.

P2 evidence identities are worktree SQLite `stdlib-path-p2-*.log`; exact readback and loose log removal precede the acceptance commit.

## P3 initial milestone review

Whole compiler: 2,221 passes and exactly the two named baseline callable failures; 234 completed targets, no new failures. Whole editor: 341/341. Supplemented runtime tests retain the independent Node reference in `tests/fixtures/lexical-path/reference.ts`. Five-case untimed verification covers basename and all four new exports. Initial result-directory setup failure is preserved; the repaired record is `runtime/results/2026-10-08T00-53-21-791Z-verify.json`.

Initial execution-only record: `runtime/results/2026-10-08T00-59-07-862Z.json`. Five workloads, native/Node-after-tsc, original 8 warmups, 31 samples, 3 processes (93 samples per engine/workload; 930 individual samples). Native/Node mean reciprocal-throughput ratios: basename 0.743, dirname 1.486, extname 0.707, isAbsolute 4.051, normalize 0.714. No historical normalized path rows exist in the canonical main SQLite store, so these are engine comparisons, not historical regression claims. No allocation profile was measured.

Review identifies avoidable intermediate allocation in extname and normalization. Preserve these originals and supplement only affected checks and measurements after optimization. SQLite uses the existing `analytics.derive` API for accepted execution-only rows, with profilesAccepted=false; no nonexistent allocation/profile acceptance is fabricated to satisfy the older combined CLI ingestion contract.
