# Compiler source organization

Status: owner-requested, scheduled. Begin after focused acceptance of the approved
JSON namespace and before additional S10 API expansion. This is a required cleanup
checkpoint, not an optional task deferred until the project ends.

## Scope

Organize all of `crates/tsrust-compiler/src`, including existing subdirectories.
Group modules by responsibility and use ordinary Rust module boundaries. Keep
`lib.rs` as a readable public facade and preserve public imports used by the CLI,
LSP, tests and tools.

The initial inspection confirms inconsistent organization inside `stdlib`:
`fs.rs`, `buffered_fs.rs` and `resource.rs` are test modules, while
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
