# One Rust crate per source module

Owner decision, 2026-11-16: pursue per-module crates in every build, with no
runtime cost and no difference between development and release behavior.
Evidence is in `experiments/crate-split-spike/` and `tools/crate-split-analysis.py`.

## Goal

An edit to one source module recompiles that module's crate and relinks.
Today Studio's generated program is one 20 MB crate: rustc+link takes ~14 s
per edit (19 s inside `tsrust dev`), and ~11 s of that is fixed cost that grows
with the whole program. The spike rebuilt and linked an edited Studio module
in its own crate in 0.6-0.7 s.

## Non-negotiable constraints

- **No runtime cost.** Generated code keeps today's representations: enum
  families for stored callables, concrete closure structs, direct calls. No
  boxing, no trait objects, no dynamic dispatch introduced by the split.
- **One program, every build.** Development and release compile the same
  generated source. Only crate boundaries and optimization settings differ.
- **Soundness of link-time calls.** Every `unsafe extern "Rust"` declaration is
  checked against its definition at compile time (`let _ = [declared, defined];`
  in a crate that sees both), so a mismatch cannot reach the linker.

## Design

Crates, in dependency order:

1. `tsrust_runtime` (exists): bundled runtime modules and Rust dependencies.
2. **Types crate**: records, unions, view unions, error and completion enums,
   stored-callable family enums, closure data structs, and the trait impls
   Rust's orphan rule requires next to them. Family dispatch and closure
   `call` methods delegate to link-time-declared body functions.
3. **Module crates**: one per source module, or per group of modules joined by
   a cycle that link-time calls cannot break (async closure bodies called
   against the dependency direction). Each holds the module's generated
   functions and closure body functions. Calls along the import direction are
   ordinary crate dependencies; calls against it, and calls from the types
   crate into bodies, use link-time declarations.
4. **Program crate**: `main`, the source-location table, and the signature
   checks for every link-time declaration.

Names: generated items get module-scoped, content-stable names so that an edit
to one module leaves every other crate's source byte-identical (the existing
padded source bases already keep offsets stable).

## Measured facts that shape the work

- Studio cross-file references: 1,539 to plain functions, 344 to closure types.
- Async closures returned across files (types inferred, not named): Studio 12,
  Worklist desktop 6, Worklist API 0. Only those on dependency back-edges force
  modules into one crate.
- `extern "Rust"` declarations cannot name async future types, generic
  functions, or `impl Trait` returns. Generated functions are monomorphic; the
  async case is handled by merging.

## Steps

Each step lands with byte-identical behavior: the existing CI suites, the
compiler library and integration tests, Studio/gallery/Worklist test suites,
and an A/B check that `check` output is unchanged. Steps 1-4 keep emitting a
single crate so they can be verified against today's output.

1. **Provenance and grouping.** *(1a done 2026-11-16: `ProjectAnalysis::crate_plan`
   and `tsrust emit --crate-plan` map every generated function and closure to its
   source module; on Studio it agrees with the source-location mapping for all
   2,402 items, and 147 compiler-generated functions have no module.)* The emitter records the source module of every
   generated item and builds the module dependency graph from its references,
   including inferred return types. Output: a crate assignment (module or
   merged group) per item, reported by `tsrust emit --crate-plan`. Verify on
   Studio, the gallery and Worklist with `crate-split-analysis.py`.
2. **Closure bodies as functions.** Emit each closure's `call`/method bodies
   as free functions taking the closure data, with the inherent methods
   delegating to them. Single crate still; generated behavior identical.
3. **Stable names.** Replace program-wide counters for functions, closures,
   records and unions with module-scoped or content-derived names. Verify that
   an edit to one module changes only that module's items.
4. **Link-time declarations in one crate.** Route back-edge calls through
   `extern "Rust"` declarations plus `#[unsafe(no_mangle)]` definitions while
   still emitting one crate, proving the declarations, checks and symbol
   naming before splitting.
5. **Multi-crate emission and builds.** Emit the types, module and program
   crates; build them with Cargo (one workspace per project in the existing
   native workspace cache) for `dev`, `run`, `build` and `test`, reusing the
   staged runtime. Measure Studio and the gallery edit loops.
6. **Release.** Build the same crates with thin or fat LTO; compare binary size
   and the SQLite benchmark suite against single-crate release. If any
   regression remains, release compiles the identical source as one crate.
7. **Tooling.** `tsrust emit` keeps a single readable file; `--debug` source
   mapping, packaging and Windows path limits are verified on all platforms.

## Risks and checks

| Risk | Check |
|---|---|
| Undefined behavior from a mismatched declaration | Compile-time `[declared, defined]` checks in the program crate |
| Runtime regression | Benchmarks (SQLite store) gate step 6 |
| Crate graph recreates a giant cycle | `crate-split-analysis.py` on Studio and gallery in CI |
| Stale or extra rebuilds | Edit-loop measurements per step; emitted-source identity for untouched modules |
| Build orchestration failures | Cargo-based build; three-platform validation before merge |
