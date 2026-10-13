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

## Schema feature boundaries

Schema checking lives in `analysis/schema`; schema emission lives in `emit/schema`.
Each feature owns a module tree, with explicit ancestor visibility for methods
that extend `Analyzer` or `Emitter`. Editor schema completion, typed paths and
constructor options live in `tsrust-lsp/src/schema`, under source intelligence.

`stdlib/schema/compiler` owns plan metadata, native namespaces and bridges.
`stdlib/schema/runtime` owns embedded validators, bounded issues, rollback trials
and JSON Schema assembly. Explicit module paths preserve their existing logical
context where shared runtime code must compile both as a backend witness and as
generated program support.

JSON's `runtime/parser.rs` is shared by compiler-time data imports and generated
programs. `runtime/operations.rs` contains dynamic runtime operations; compiler
library tests include it, while production compiler builds only the parser.
`runtime.rs` is the complete standalone backend-witness facade. Runtime assembly
concatenates the same source fragments without emitting include paths into user
programs. Typed/raw/source cursor modules are compiled directly only for backend
tests and otherwise embedded into generated programs. This boundary avoids
compiling unused native operations into the compiler binary.

Function scheduling lives in `analysis/function_scheduling.rs`. Keep function
completion queues and lexical owner continuations there; body checking and its
ownership contracts remain analysis responsibilities. The nesting architecture
plan tracks the remaining recursive dependency paths.

Callable result discovery lives in `analysis/result_discovery.rs`. Its queue
records actual unresolved function-result dependencies and wakes their consumers
when a concrete environment becomes known. Keep discovery flags, source spans
and newly instantiated function admission aligned with this queue.

Regional error annotation lives in `error_effects/traversal.rs`. Its heap
continuations separate expression point effects, catch-entry contracts and
escaping cleanup effects. Operation and callable contracts remain in
`error_effects/mod.rs`. The shared `completion.rs` worklist proves abrupt cleanup
for both analysis and emission; keep labeled-block and loop fallthrough boundaries
conservative.

Annotation storage operations live in `types/annotation_storage.rs`. Clone,
equality, nesting inspection and destruction use heap worklists. Use
`ValueAnnotation::into_kind` when consuming an annotation variant so its children
can be moved without bypassing the stack-safe destructor. Parser, substitution
and type-resolution traversal remain separate responsibilities.

Source callback/result wrappers lower through `syntax/annotation_conversion.rs`.
The reverse adapter lives in `callables/annotations/conversion.rs`. Both use
explicit construction frames; preserve the distinction between callback ownership
and inferred result invocation, as well as the established access-wrapper ranges.

Generic substitution in `types/substitution.rs` uses owned parent frames. A frame
temporarily holds the parent while one child is visited, then restores that child
before proceeding. Error unwinding reconstructs the partially substituted tree;
do not replace this with a whole-tree clone or discard earlier valid substitutions.
Containment projection and copy-mode classification also use worklists.
