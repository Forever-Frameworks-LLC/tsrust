# Modules foundation: S01/S02

Status: module contract and graph-loader implementation accepted with focused source tests. Project type checking/emission (S03), CLI module execution and project editor support (S04) remain pending. This parallel work does not renumber or replace compiler language gates or checkpoint 86.

## Accepted first-release contract

Explicit named imports from relative .tsr paths; import aliases and trailing binding commas; direct export function and export type record declarations. Imports may appear between declarations because there is no executable top-level initialization. Empty/side-effect/default/namespace imports, export lists/re-exports, package/std imports, inferred extensions/index files and executable top-level declarations are outside this gate.

File provider canonicalizes filesystem paths, including symlinks, using host filesystem case behavior. UTF-8 path identities and source text are required; aliases resolving to the same canonical identity load once. Snapshot providers supply their own stable identities. Exported nominal type identity will include defining SourceId in S03. Import cycles are rejected and related locations retain their source identities. Depth is bounded to 128 import edges.

Only the selected entry's zero-argument procedure main runs automatically in S03; each dependency's main is ordinary private unless exported/imported. The current loader executes nothing, and does not validate main signatures or infer ownership/types. Existing check(source) and CLI remain unchanged.

## Public contract

project::SourceProvider resolves identities and reads snapshots. project::FileSourceProvider is the optional disk adapter; the loader itself does not read disk directly. load_project returns ProjectGraph with entry identity, independently retained modules and deterministic dependency-first order. Modules expose import bindings/targets, export name spans, original source and byte-aligned declaration_source with only module markers blanked. Parsing uses the existing compiler lexer and parser. No concatenated-file coordinate model is introduced.

ProjectDiagnostic carries a primary SourceLocation and separately source-indexed related locations. S03 must preserve this mapping for type/ownership errors, and S04 must map those source snapshots to UTF-16/versioned edits. Current loading stops at the first graph error; aggregation and editor recovery are not supplied here.

## Focused evidence

Retained tests: crates/tsrust-compiler/tests/project.rs. Six passing tests cover diamond dependency identity/read-once/order, aliases and exports, UTF-8 byte coordinates, cycles, missing sources, private exports, duplicate declarations/imports, malformed/unsupported forms, comment/string/template isolation, actual filesystem fixture loading and separation of loading from semantic checking.

Retained physical graph: tests/fixtures/modules/diamond/{main,left,right,shared}.tsr. Future S03 execution must print 4 only; shared's private main must not run. This output is an acceptance requirement, not a measured or executed result today.

Commands: CARGO_TARGET_DIR=target/editor cargo test --offline --locked -p tsrust-compiler --test project --test formatting (6 loader + 5 formatting tests pass); focused loader rerun after token-kind hardening; cargo clippy --offline --locked -p tsrust-compiler --lib succeeds with five existing warnings in analysis.rs/syntax.rs and none in project.rs. Rustfmt applied to the two new Rust files. Logs: target/editor/module-loader-tests.log, module-loader-tests-final.log and module-loader-clippy.log. Original first test run retained a mismatch expecting syntax instead of the existing unsupported top-level diagnostic; expectation repaired, compiler behavior preserved.

No runtime benchmark is meaningful for this frontend-only loader gate. Project-load scaling/caching measurements remain pending for subsequent project tooling acceptance. No full regression or runtime benchmark was restarted. No new built-in method/property was introduced, so catalog additions are not required at this gate; std bindings will require complete catalog coverage under S05.

## Next handoff

S03 must resolve imported aliases to defining-module symbols, preserve nominal type identity and privacy, propagate cross-module ownership contracts, emit unique native symbols, map semantic errors to original sources and select only entry main. CLI check/lint/emit/build/run should consume the same project analysis API after that contract passes focused execution and negative ownership/type fixtures. S04 follows with unsaved dependency invalidation and source-indexed navigation.
