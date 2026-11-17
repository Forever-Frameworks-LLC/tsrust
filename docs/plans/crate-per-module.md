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
- Async source functions are emitted as closure structs with `async fn call`.
  Families whose dispatch awaits closures: Studio 1 of 78 (2 closures),
  Worklist desktop 0 of 77, Worklist API 0.

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
2. **Stable names.** *(Done 2026-11-16 for development builds: ledger-based
   renaming in `native/stable_names.rs`, per-unit part files; Studio
   add-function rustc+link 32-34 s -> 20-22 s. New closures still invalidate
   all bodies through the shared callback traits, which step 3 removes.)* Source names are already unique and stable, so generated
   items reuse them: `<module path>::<name>`, with a default export as an alias.
   Only compiler-made items need derived names, each from a stable parent:
   - generic specializations: base name + hash of the type arguments
     (today `computed<specialization#7>`, numbered in creation order);
   - generic record instances: base name + hash of the argument types;
   - structural records and unions: hash of their field names and types;
   - closures: the enclosing function's stable name + ordinal within it;
   - module prefixes: the module path instead of its index
     (`__tsrust_module_37_` shifts when modules are added).
   Studio today: functions 478 of 1,263 and records 946 of 1,877 already have
   unique descriptive names; all 1,489 closures are anonymous. The renaming
   is one post-pass over the generated program (name construction spans ~260
   emitter sites). Verify that an edit to one module changes only that
   module's items, and that program output is unchanged across the suites.
3. **Split emission behind a development flag.** *(Prototype done 2026-11-16:
   `experiments/crate-split-spike/split_modules.py` splits Studio into 103 crates
   that build in 15.8 s cold; a literal edit in one module rebuilds and links in
   1.1-1.2 s; a change to the shared types crate rebuilds everything in 9.7 s.
   Ported 2026-11-16: `native/crate_split.rs` splits development builds when
   `TSRUST_CRATE_SPLIT=1`; `runtime_build::SplitCrates` writes crates into the
   incremental session, skips fresh crates and builds module crates in
   parallel. CI suites 23/23 and Studio 172/177 (same five tool-dependent
   failures as single-crate) pass split. Studio `tsrust dev`: string edit
   3.2-3.5 s rustc+link (2 crates), adding a function with a closure 3.1-3.3 s
   (5-6 crates); single crate: ~14 s and ~37 s. Check (~14.5 s) is now the
   largest cost.)* Emit the types, module and
   program crates for `TSRUST_CRATE_SPLIT=1` builds:
   - closure data structs, families, records and unions in the types crate;
   - closure bodies as impls of a per-closure trait defined in the module crate
     (allowed by the orphan rule, body text unchanged), plus a
     `#[unsafe(no_mangle)]` shim per synchronous body for family dispatch in
     the types crate;
   - back-edge calls through `unsafe extern "Rust"` declarations with
     `[declared, defined]` checks in the program crate;
   - closures dispatched asynchronously from a family (Studio: 1 family, 2
     closures; Worklist: none) keep their bodies in the types crate, with
     whatever they need pulled in after them.
   Compare program output and suites against single-crate builds.
4. **Builds.** *(Partly done: rustc driver per crate in `SplitCrates::build`.
   Remaining: cancel a split build on a newer edit in `tsrust dev`, Linux/Windows
   link validation, then make the split the default.)* Cargo workspace per project in the native workspace cache for
   `dev`, `run`, `build` and `test`, reusing the staged runtime. Measure Studio
   and gallery edit loops, then make the split the default.
5. **Release.** Same crates with thin or fat LTO; compare binary size and the
   SQLite benchmark suite against single-crate release. If any regression
   remains, release compiles the identical source as one crate.
6. **Tooling.** `tsrust emit` keeps a single readable file; `--debug` source
   mapping, packaging and Windows path limits are verified on all platforms.

## Risks and checks

| Risk | Check |
|---|---|
| Undefined behavior from a mismatched declaration | Compile-time `[declared, defined]` checks in the program crate |
| Runtime regression | Benchmarks (SQLite store) gate step 6 |
| Crate graph recreates a giant cycle | `crate-split-analysis.py` on Studio and gallery in CI |
| Stale or extra rebuilds | Edit-loop measurements per step; emitted-source identity for untouched modules |
| Build orchestration failures | Cargo-based build; three-platform validation before merge |
