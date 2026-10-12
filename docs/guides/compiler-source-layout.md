# Compiler source layout

Compiler implementation lives in `crates/tsrust-compiler/src`. Place additions
under the module that owns their behavior. Keep public compiler entry points in
`lib.rs`; its re-exports preserve existing CLI, editor, and library imports.

| Directory | Responsibility |
| --- | --- |
| `syntax` | Parsing, syntax trees, and syntax-specific lowering |
| `types` | Type syntax, aliases, substitution, and union validation |
| `analysis` | Type checking, ownership analysis, and checked-program facts |
| `callables` | Function and closure contracts, capture ownership, storage, and dispatch |
| `generics` | Specialization, defaults, inference, and generic invocation validation |
| `records` | Record layout, recursive storage, field access, and view provenance |
| `definite_assignment`, `error_effects` | Assignment and exception-flow checks |
| `emit` | Generated Rust expressions, statements, and program assembly |
| `runtime` | Support code embedded into generated programs |
| `native` | Binding schemas, dependency selection, runtime emission, and cache support |
| `project` | Module resolution, linking, sessions, and typed data imports |
| `cli` | Command orchestration, scaffolding, watching, builds, and package export |
| `formatting` | Source formatting and syntax-specific layout |
| `stdlib` | Standard-library feature bindings, emission, and runtime support |

Shared root files contain diagnostics, control-flow facts, storage rules,
parameter rules, traversal, and public projections. A file belongs there only
when it represents a shared compiler concept without a more specific owner.

## Module and runtime boundaries

Use ordinary Rust module declarations. Child filenames follow their module names;
use a directory with `mod.rs` when a module owns children. Keep implementation
helpers beside their owner instead of adding another prefixed root file.

Embedded runtime files are compiler inputs, even when they are not compiled as
ordinary modules in the compiler library. Keep their `include_str!`/`include!`
references and native integration fixtures aligned when moving them. The CLI
binary starts at `src/cli/main.rs`; standalone exported programs still use their
own `src/main.rs`.

Standard-library features own their bindings, emission, and runtime fragments.
Byte-view emitter tests live under `stdlib/bytes/emit`; byte and testing runtime
fragments have dedicated `runtime` directories. Collection checker/emitter
extensions intentionally use explicit paths from analysis and emission: their
logical module context grants access to the relevant compiler state.

Source moves must preserve public APIs, runtime contents, diagnostics, ownership,
and evaluation order. Batch structural verification after coherent moves; do not
rerun tests for each directory. Follow the development workflow for formatting,
regression evidence, and checkpoints.
