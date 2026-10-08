# Lexical path family acceptance

Base: `8fe73b5`. Branch: `codex/stdlib-path`. Scope: dirname, extname, isAbsolute, normalize. Implementation and measurements remain pending.

## Frozen public contract

All exports accept exactly one borrowed string argument named `path`. Missing, extra, and non-string arguments are compile-time errors. No input mutation, filesystem access, symlink resolution, or current-directory lookup occurs. String results are independently owned valid UTF-8 text; isAbsolute returns boolean. The target selects POSIX or Windows conventions. Internal helpers accept an explicit platform selector. Windows helper tests on macOS do not establish Windows native integration acceptance.

| Export | Purpose and return | Empty input | Roots and separators | Dot components and Unicode |
|---|---|---|---|---|
| dirname | Remove final nonseparator component; retain parent spelling | `.` | POSIX `/` remains `/`; `//a` gives `//`; repeated parent separators remain. Windows recognizes both separators, drive prefixes, and UNC share roots. | No dot resolution; UTF-8 text retained. |
| extname | Final component suffix starting at last dot | empty string | Ignore trailing separators; Windows drive prefix excluded | `.hidden`, `.`, `..` give empty; `...` and `a.` give `.`; `a.tar.gz` gives `.gz`. Unicode retained. |
| isAbsolute | Test lexical root syntax | false | POSIX leading `/`; Windows leading separator or letter-colon-separator. `C:a` is relative; UNC is absolute. | Dot segments do not change result. |
| normalize | Collapse separators and resolve dot components lexically | `.` | Preserve absolute root and final separator; POSIX uses `/`, Windows emits backslash and retains drive/UNC device. | Absolute parents stop at root; relative leading parents survive. Unicode retained. Idempotent for accepted corpus. Not a containment check. |

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
