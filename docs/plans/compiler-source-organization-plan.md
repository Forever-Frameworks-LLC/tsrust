# Compiler source organization

Status: in progress after focused JSON namespace acceptance, before additional
S10 API expansion. This is a required cleanup
checkpoint, not an optional task deferred until the project ends.

## Scope

Organize all of `crates/tsrust-compiler/src`, including existing subdirectories.
Group modules by responsibility and use ordinary Rust module boundaries. Keep
`lib.rs` as a readable public facade and preserve public imports used by the CLI,
LSP, tests and tools.

The initial inspection confirms inconsistent organization inside `stdlib`:
`fs.rs`, `buffered_fs.rs` and `resource.rs` contain embedded runtime code compiled
as modules for host-side tests, while
`fs_bindings.rs` and `fs_emit.rs` are implementation modules beside feature
directories. Consolidate filesystem implementation and related tests under
`stdlib/fs/`; place shared resource tests according to their actual dependencies.
Also inspect collections, byte views and testing helpers for coherent internal
submodules. Do not assume every file at a directory root is misplaced: retain
genuinely shared entry points there and document their role.

Review top-level compiler modules for syntax/types, analysis and ownership,
callables, code generation/runtime, native bindings, and project loading/linking.
Choose final boundaries from dependency inspection, rather than filenames alone.
Avoid replacing a flat directory with a collection of path-attribute shims.

## Execution and acceptance

1. Inventory module dependencies and embedded-source paths, including
   `include_str!`, `include!`, test path attributes, runtime packaging and tools.
2. Move one coherent family at a time, beginning with standard-library strays.
   Preserve behavior, public API paths, diagnostics and generated runtime output.
3. Run affected compiler/native/editor checks after each batch. Verify embedded
   runtime generation and package export when their source paths move. Add tests
   only for uncovered behavior; do not add tests that merely assert folder names.
4. Format the whole project and checkpoint each accepted batch. Preserve owner
   edits, historical evidence paths and SQLite records. Create no archive copies.
5. Record completed families and remaining work here. At completion, run complete
   compiler and LSP regressions once as the structural milestone. Use representative
   untimed workloads to verify runtime behavior; directory moves alone do not
   require new performance measurements.

Acceptance requires working module boundaries, updated references, retained public
API compatibility and passing checks. A directory proposal alone is not completion.

## Filesystem batch

Filesystem binding metadata and emission now live in `stdlib/fs/bindings.rs` and
`stdlib/fs/emit.rs`. Runtime ownership is explicit under `stdlib/fs/runtime/`,
with file, buffered-file and resource lifecycle modules. The five relocated files
retain exact bytes. Public compiler APIs and generated native module names remain
unchanged. JSONL tests use the new module paths and a test-only descriptor accessor;
the backend field stays private.

Four backend integration tests pass before and after the move. Two generated file
programs retain identical SHA-256 hashes. All 87 standard-library unit checks pass, including JSONL reader/writer integration.
SQLite evidence: tsrust-fs-organization-before.log, tsrust-fs-organization-after.log,
tsrust-fs-organization-preservation.log, tsrust-fs-organization-hashes.json and
tsrust-fs-organization-unit-3.log. Failed path/visibility compilation attempts are
retained in the preceding unit logs. The filesystem batch is accepted.

Next inspect the analysis/emission child modules currently held at the source root
through path attributes. Move each existing module family into its actual owner
directory, preserving Rust module identity before considering deeper regrouping.
Compiler-wide cleanup and the final structural regression milestone remain open.

Inventory after filesystem relocation: 327 source-root files. Analysis has 98 root-level child files,
emission 68 and syntax five, attached through path attributes. These existing
module boundaries provide the next concrete relocation batches.

## Syntax batch

The parser parent and its five children now live under `src/syntax/`. Ordinary
module declarations replace all five path attributes; the public match_syntax
re-export retains its identity. Child files retain exact bytes. Parent edits are
limited to module paths and two embedded test fixture paths.

The same 14 match/type-syntax integration tests pass before and after relocation.
SQLite records use the tsrust-syntax-organization prefix. All 11 parser unit tests also pass; the syntax batch is accepted. The source root now has 321 files.

## Analysis batch

The analyzer parent and 98 child modules now live under `src/analysis/`. All 98
child files retain exact bytes. Parent edits only remove obsolete path overrides
and adjust the three shared stdlib collection checker paths. Internal Rust module
identities, visibility and public APIs are preserved.

The same 31 focused integration checks pass before and after the move: callback
returns, recursive storage, JSON namespace and match syntax. Full unit-test
compilation and two callee-transport witnesses are checked before checkpointing.
SQLite evidence uses tsrust-analysis-organization and tsrust-analysis-ownership
prefixes. Root file count is now 222; emission modules are next.
