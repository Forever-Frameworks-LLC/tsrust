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

Data annotation resolution lives in `analysis/value_type_resolution.rs`.
`callables/resolution/traversal.rs` owns the shared worklist for callable results,
parameters, unions and storage declarations. Its caches are limited to one run
over immutable annotations. Contract IDs are still published after children;
grouped storage errors retain their enclosing-range remapping.

Generic record declaration validation and source annotation wrapper traversal live
in `generics/records/mod.rs`. Their worklists preserve constructor-before-child
validation and parameter-before-result materialization. Concrete annotation and
record instance expansion use owned parent and field continuations in
`generics/records/materialization.rs`. Publish nominal identities before visiting
fields; publish completed field lists only after all fields succeed. Result
alternative allocation remains in `generics/records/result_types.rs`.

Checked annotation construction shares `types/reconstruction.rs` between
`analysis/checked_value_annotations.rs` and `generics/inference.rs`. Describe one
checked node at a time; the builder schedules children in source order and assembles
parents after their children. Active-path type identities reject structural cycles
without rejecting repeated acyclic subgraphs. Preserve named recursive union
boundaries, callable ownership/access metadata and cached annotation span behavior.

Generic function cache keys live in `generics/functions/contract_keys.rs`. A local
arena canonicalizes contract nodes; only a flat prefix-token sequence survives in
the cache. Keep construction, cloning, hashing, equality and disposal independent
of source nesting. Group removal, stable absent-member ordering, adjacent-only
union deduplication and callable metadata normalization preserve existing identity.

Constraint name materialization in `generics/inference.rs` uses owned parent
frames for postorder nominal resolution and exact partial-tree restoration on
failure. Its nullable deferral classifier uses a read-only worklist. Callable
contracts remain opaque to these two visitors; constraint unification traverses
them through its separate worklist.

Generic constraint unification now shares `ConstraintWork` in
`generics/inference.rs`. Keep tuple/application resolution as deferred work so
later invalid annotations cannot preempt earlier constraint diagnostics. Source
patterns borrow annotations; substituted union payloads own annotations and move
children through `constraint_children`. Union-member continuations retain one
pending template rather than cloning every member in advance.

Source wrapper disposal lives in `syntax/annotation_storage.rs`; cloning shares
`syntax/tree_clone.rs`. Callback and result annotations use explicit worklists;
parameter annotations reach them through their callback variants. Consume callback
fields through `CallbackAnnotation::into_parts` so the emptied wrapper still runs
its destructor. Parameter defaults retain their expression trees in the shared AST clone worklist;
recursive debug formatting remains a separate pending conversion.

AST disposal lives in `syntax/tree_storage.rs`. Its owned worklist crosses
expressions, statements, local functions, callable annotations and default
initializers, including checked spread-plan children. `Expr::into_kind` and
`Stmt::into_kind` consume a node safely; `Expr::take_kind` retains metadata while
emptying its payload. Test a variant before extraction when an unmatched node
must remain intact. AST cloning shares signature/default continuations; debug formatting remains pending.

AST and source wrapper cloning share `syntax/tree_clone.rs`. `Source` describes
borrowed nodes; construction frames retain continuations, and `Values` drains
completed children in source order. Keep signature/default edges in this shared
worklist rather than calling a separate recursive clone visitor. Expression
reconstruction copies stored depth and checked metadata exactly. Leaf variants
may use their derived clone only when they contain no AST ownership edges.


Checker loop ownership and flow boundaries live in `analysis/continuations/loops.rs`.
`analysis/continuations/driver/tail.rs` schedules counted and post-test loop bodies,
updates and conditions on the shared checker worklist. Unreachable tails use a
separate context on that same scheduler; dispose it before joining reachable exits.
Keep iterator-loan cleanup and control-target cleanup active on diagnostic paths.
Statement assignments dispatch into the existing assignment/store tasks rather
than starting a nested checker driver.


Exception suspension lives in `analysis/continuations/driver/region.rs`, with
exit-frame boundaries in `analysis/continuations/exceptions.rs`. Try/catch bodies
and canonical/per-destination finally replay use the same checker worklist.
Preserve replay order, declaration prefixes, catch shadow restoration and delayed
publication of successful handler bookkeeping. Exception-bearing closure bodies
use the callable task in the driver instead of opening another checker driver.

Record and scoped-map carrier type joins share `analysis/carrier_type_join.rs`.
Keep nested record fields, array record elements and map values on that module's
continuation stack so mixed aggregate joins cannot re-enter a recursive driver.

Mutable carrier retargeting lives in `analysis/carrier_retarget.rs`. Owned parent
frames restore every extracted child before returning, including diagnostic
unwinds; completed sibling mutations preserve the existing partial-error behavior.

`analysis/carrier_literal_join.rs` joins paired record expressions with owned
parent frames. Child joins and fallback conversions complete in declaration field
order; both trees are restored before a diagnostic returns.

Nullable/sticky record joins also use `analysis/carrier_type_join.rs`.
`analysis/empty_scoped_maps.rs` shares work frames for empty record, map and union
environments. Absent callback return contracts use parent frames in
`Analyzer::absent_callback`, retaining bottom-up publication and cache identity.

Callable result source-scope traversal lives in `analysis/discovery_work.rs`.
Body, branch and exception tasks preserve cloned nested contexts and the mutable
root context. Keep cleanup-first discovery and catch refinement/restart order
there; expression and region checks still enter the checker through existing
interfaces and require a separate re-entry audit.

Closure parameter preparation lives in `analysis/continuations/closure.rs` and
`analysis/continuations/driver/closure_parameters.rs`. Defaults and lowered
parameter patterns suspend on the same driver as closure bodies. Retain the
child context until parameter metadata, pattern bindings and body completion
finish; restore extracted pattern statements on diagnostic exits.
`analysis/continuations/defaults.rs` retains supplied/default availability and
flow joins, including nullable callback access contracts. Scheduled expression
and block frames do not participate in recursive-entry depth accounting.

Contextual expression preparation uses `analysis/continuations/prelude.rs`.
Mutation footprints, flow projections, borrowed expectations, nullable injection
and contextual stored-callable conversions yield child work to the driver.
Projection frames retain flow suppression and mutation frames retain access state
until completion; both restore their boundaries on diagnostic paths. Keep these
transitions on the existing driver rather than starting nested expression checks.

Optional-chain state lives in `analysis/continuations/optional.rs`; its receiver
and branch-body tasks live in `analysis/continuations/driver/optional.rs`.
Receiver checks use the enclosing checker worklist. Completed branch frames join
availability and flow in reverse selection order. Diagnostic unwinding releases
capture-loan boundaries and selected-access state. Receiver-spine normalization,
selection and named-source traversal remain in `analysis/optional_chain.rs` and
use explicit cursors or parent/work frames.

Borrow-source predicates live in `analysis/borrow_hints.rs`. The borrowed-place
worklist preserves left-to-right disjunction and defers indexed/query fallbacks
until the earlier predicate fails. Indexed-element hints unwrap sequence and
assertion sources with a cursor before inspecting the field/index path. General
type-hint dependencies remain separate and must not be described as stack-safe
merely because these source walks are iterative.

Structural type hints use `analysis/type_hints.rs`. Its continuation stack owns
conditional, coalescing/logical, unary, arithmetic, array, field/index, pattern
and optional-hint dependencies. Metadata and flow projections are checked before
structural descent. Keep lazy branches lazy and retain original join spans.
The memo belongs to one immutable source/context query and must never survive a
context or program-state change. Native/schema call hint dispatch and other
compatibility helpers still require a separate re-entry audit.
