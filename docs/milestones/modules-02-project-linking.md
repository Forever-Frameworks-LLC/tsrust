# S03: cross-module checking and native emission

Status: focused implementation accepted. Project-aware CLI and generated native execution are verified. S04 project editor support is pending. Frontend scaling measurements and full regressions remain queued at the existing numbered compiler checkpoint; this parallel milestone does not renumber language gates or restart their regression.

## Observable behavior

Explicit named relative imports and aliases resolve exported functions/record types across files. Private symbols remain module-local. Same-shaped nominal records from different defining modules are distinct types. Local variables, parameters, loop bindings and closures can shadow imported names according to existing lexical scopes. Existing type, callback and ownership analysis remains authoritative across module boundaries.

Only the selected entry's zero-argument procedure main runs automatically; imported modules' main declarations remain ordinary functions. Import cycles and private/missing exports are rejected. CLI check/lint/emit/build/run operate on the selected entry's graph. Single-file sources retain their existing JSON/contracts and successful lint remains silent. Output targets cannot overwrite any loaded source, including filesystem aliases.

Compile-time errors retain original file identities and UTF-8 ranges, including cross-file related locations and clone fix candidates. Project runtime errors with compiler source ranges identify the original file and local byte range. Unknown native panic payloads retain the original panic hook.

## Isolated implementation boundaries

project.rs loads source snapshots and validates the graph. project_link.rs parses each module separately, resolves global/type aliases structurally with lexical shadowing, assigns unique native names/closure IDs, and feeds linked AST declarations to existing strict/recursive analysis. It does not concatenate source text. Explicit coordinate segments map checker ranges to original sources. Field/property names are preserved.

ProjectAnalysis exposes graph, source-qualified contracts, mapped closure metadata/JSON, source_location and emit_rust. The underlying linked Analysis is crate-private. ProjectDiagnostic carries primary/related SourceLocations and immutable ProjectFix candidates. Fixes still require whole-project rechecking and source-version validation before S04 can offer them in the editor.

project_cli.rs is the filesystem host adapter, preserving single-file CLI output while supplying project errors and metadata. project_runtime.rs generates entry panic-hook setup containing source segments only, not file contents; direct emitted binaries get the same mapping as CLI run. emit.rs has an optional entry setup boundary; ordinary emission uses empty setup and retains its behavior.

## Focused acceptance evidence

19 project tests pass in crates/tsrust-compiler/tests/project.rs. Coverage includes graph identity/read-once/order; aliases/exports; Unicode byte ranges; cycles/missing/private/duplicate declarations; malformed syntax; lexical comment/string/template isolation; nominal mismatch and ownership transfers; original-file diagnostics/fixes; local shadowing and closures; nullable imported types and recursion; record arrays/field chains/loops; multiple callback specializations; mutation contracts and retained capture loans; reserved names and missing entry; source-facing capture metadata; output overwrite protection; actual CLI check/lint/emit/build/run with space-containing paths.

Native witnesses retain exact output: diamond 4 (no dependency main); alias/shadowed/local closures 9,3,9; nullable record/recursion/callback Ada,4,9; record-array loops/callback specializations 6,3,6; imported field mutation 2; source-mapped clone fix Ada. Nullable/callback and record-array witnesses run at O0/O3. Runtime bounds failures inside dependencies identify original files/ranges at O0/O3 and omit generated program.rs frames.

Commands use CARGO_TARGET_DIR=target/editor and offline/locked Cargo. Logs: module-boundary-tests.log (19 tests); module-cli-verified.log (earlier 17 project plus 5 formatting/CLI tests); module-editor-compatibility.log (4 existing quick-fix tests); module-cli-clippy-final.log (only five pre-existing warnings in analysis.rs/syntax.rs). Rustfmt applied to changed Rust files. Original failed attempts and intermediate logs remain retained. No new built-in methods/properties were added, so no catalog entry is implied by this compiler-only milestone.

## Scaling registration and pending measurements

project_scaling.rs registers deterministic shared-dependency fanout workloads with 16,64,256 leaf modules (18,66,258 total). Each leaf imports one shared seed function; entry imports all leaves and accumulates their results. Expected native totals are 136,2080,32896. The timed workload separates source-provider loading, AST/checking and emission, recording source bytes, module counts, provider reads and generated bytes per sample. This measures frontend work, not native runtime throughput.

A normal correctness test checks 18/66-module graphs, exactly one provider read per module and successful checking/emission. Log: target/editor/module-scaling-correctness.log (1 passes, measurement test intentionally ignored). Larger timed runs are registered, not measured or claimed passing until executed.

At the scheduled checkpoint run: CARGO_TARGET_DIR=target/editor cargo test --offline --locked -p tsrust-compiler --test project_scaling measure_module_frontend_fanout -- --ignored --nocapture. TSRUST_MODULE_SAMPLES optionally sets bounded sample count (default 10). Retain machine/toolchain/source identity and sample outputs before computing distributions. No runtime comparison is meaningful for module-resolution code itself; corresponding generated native behavior is already verified by focused execution.

## Next handoff: S04

Supply project source symbol/type metadata and imported-symbol editor assistance, unsaved dependency providers, dependency invalidation, cross-file diagnostics/navigation, project fix validation and stale/versioned result protection. CLI acceptance does not supply these capabilities. Preserve existing single-file source assistance, quick fixes, formatter and quiet automatic checking. No installed global CLI or VSIX refresh was performed by this gate; development builds contain this implementation.
