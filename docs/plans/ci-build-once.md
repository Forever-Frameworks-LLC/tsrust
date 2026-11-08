# CI build-once and `tsrust test` migration

Owner direction, 2026-11-08: CI compiles the compiler once per OS, reuses that
build for every runtime test through `tsrust test`, and fails fast. Rust
integration tests stay only for compiler internals and real process/language
boundaries. Do not push PRs or dispatch CI while this plan is in progress; push
once, to merge, when it is complete.

## Why

The 965de83f full run spent about 9,700s building 557 Rust test binaries. Each
links the whole compiler. It then spent about 11,700s in cases that mostly
compile their own native program: about 650 `tsrust run` calls, 371
`support::execute` calls and 205 direct `rustc` calls, many inside variant loops.
Only 24 `.test.tsr` suites use the compile-once runner. Each CI job also rebuilt
the compiler from source, so the tests never exercised the binary that ships.

## Target shape

| Layer | What runs | Where |
|---|---|---|
| Build | `tsrust` + `tsrust-lsp` release build per OS, packaged with `tools/package-toolchain.mjs`, smoke-tested, uploaded as an artifact | CI `build` matrix (4 OS), fail fast |
| Compiler library | `cargo test --lib` (unit, rejection and diagnostic-code checks, in process) | one Linux job, in parallel with the build |
| Runtime suites | `tsrust test` on `tests/suites/<family>/` with the built artifact; one native program per family | CI `suites` matrix, after `build`, per OS |
| Host tests | 21 named Rust targets (`tools/host-tests.py`) for CLI, signals, processes, offline export, packages, Rust adapters and LSP stdio | CI `host` matrix, after `build` |
| Docs, editor depth, frozen UI | catalog example execution, completion/agreement matrices, docs and UI programs | path-filtered jobs, or local only |

Diagnostics in CI assert codes and spans, not message prose. Editor typing-speed
and presentation matrices are development tools.

## Steps

1. [x] Policy: AGENTS.md, the project skill and memory say a push/PR exists to
   merge finished work toward a release, never to run checks on other OSes.
2. [x] `std:testing` output capture: `captureOutput(callback)` for non-throwing
   callbacks, and `startOutputCapture()`/`finishOutputCapture()` for code that
   throws or awaits. The runner discards an unfinished capture before each test.
   The bindings are in the non-failing effect table. Catalog entries and examples.
3. [x] Superseded: per-case recording. `support` is included by 433 test
   binaries, so a recorder forces a multi-hour rebuild and full run. The owner
   allowed consolidation instead of 1:1 conversion.
4. [x] `tools/generate-catalog-suites.py`: every verified non-UI catalog example
   becomes one test (1,093 after dedupe) asserting its recorded stdout, one
   directory per catalog group. Examples that need an input file, re-run their
   own executable or read their arguments are host-only (14).
5. [x] `tools/run-suites.py`: every tracked `.test.tsr` directory plus the
   generated groups, one `tsrust test` build each, fresh working directory,
   closed stdin, 600s per suite. The intentional failure fixture is excluded
   (its host test asserts the report).
6. [x] `tools/host-tests.py`: 19 compiler and 2 LSP targets for real
   process/language boundaries (CLI, projects, packages, toolchain install,
   signals, child processes, relocated export, Rust adapters, test reporter,
   LSP stdio).
7. [x] CI (`.github/workflows/ci.yml`): `lint`, `lib`, `build` (release build per
   OS, packaged, smoke-tested, uploaded), `suites` and `host` per OS using that
   build, `editor`, and one `CI` gate; fail fast. The shard runner, the
   per-target platform runner with its retained-evidence chain, and the
   supporting-host timing workflow are removed.
8. [x] Local validation (aarch64 macOS, jobs=1, one shared target), all PASS:
   compiler library 1,546 (~2 min run), compiler CLI 54, LSP library 27 and
   binary 6; suites 133 (cold 17.7 min, warm 301s); host targets 21
   (1,635s, dominated by relocated offline export). It found and fixed three
   defects: `console.table` failed in test programs (private runtime function);
   the console runtime was omitted when a test program captured output but
   never printed; the rust-regex example's bindings lock had drifted.
9. [ ] One push to merge, then report CI wall time per OS from that run.

Legacy per-case Rust integration targets (about 530) stay in the repository as
local deep-regression witnesses, outside CI. Fold them into family suites
incrementally, keeping unique emit-level witnesses (for example the map-loop
divergence case) until an equivalent suite case exists. Pure rejection checks
belong in compiler library tests.

## Acceptance

- One compiler build per OS per CI run; tests use that build.
- Every verified catalog example and every tracked `.test.tsr` suite runs in CI;
  the generator reports converted and host-only counts. Legacy targets are
  retained locally, not deleted.
- CI wall time per OS is reported from the first run, not estimated.
