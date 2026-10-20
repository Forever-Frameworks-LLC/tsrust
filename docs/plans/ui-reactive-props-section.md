# Reactive props and replacement composition

The scoped scalar-prop and replacement section passes focused acceptance.
See [acceptance](../milestones/ui-reactive-props-and-composition.md). Preserve the
accepted ownership, cancellation and cleanup rules. Full context, event/ref merging
and focus/overlay composition remain required work.

## Contract

Component authors opt scalar fields into `ReactiveValue<T>` and read them through
component-owned `useProp(...)` values. `computed(...)` creates derived getters. Callers use ordinary JSX expressions.
Computed values expose `get()` without a setter, preserve exact literal/nullable
scalar types, and own evaluators while node bindings capture weak references.
Resources and containers are not scalar computed results. No component rerun,
duplicate reactive component, public lifetime handle or implicit resource clone.

Generic styling/metadata arguments on the existing prop type aliases keep their
string defaults, preserving previously installed source. Intrinsic class/variant
bindings keep ordinary JSX argument evaluation order and node identity. Opted-in
reactive expressions defer evaluation until their getter is read.
View/Button replacement factories receive owned props once and add no wrapper.
Replacement code must forward the behavior it needs onto its returned control.

## Complete-section acceptance

- [x] Compiler/runtime values, precise diagnostics and checked generic alias bases.
- [x] Intrinsic live classes/variant metadata, validation and source evaluation order.
- [x] Replacement composition, callbacks, children and once-only cleanup.
- [x] LSP completion/hover/signatures/diagnostics for the same contracts.
- [x] Editable local examples and docs; existing component sources still compile.
- [x] Independent Node-after-tsc transition oracle and native/browser observers.
- [x] Batch focused acceptance, refresh preview/editor and verify SQLite evidence.

Compiler/native witnesses pass for typed values, normal props, replacement factories,
source order and owning callback forwarding. Runtime witnesses independently pass
for live styles, variant metadata, exact/nullable values, invalid updates, stable
nodes and evaluator release. The installed LSP and active editor are refreshed.
Optimized Wasm and the real dev guide match the independent TypeScript transitions,
including actual CSS and attribute removal. The formatted native guide builds and
passes its bounded startup observation. This is not OS gesture/pixel acceptance.
The matched transitions are untimed; no performance measurement is claimed.

Editable Alert version 2 applies the same contract. Offline installation preserves
earlier source/imports and installs a versioned file; compiler API 4 guards its
requirements. SVG icons, text balancing and full Base UI event/ref composition
remain unsupported. The preview remains the actual port-4317 dev watcher.

The implementation keeps generic callable/data union restrictions intact:
`computed` accepts an owned getter; `useProp` adapts a scalar or `Reactive<T>`
descriptor. Internal weak bindings use the checked native operation directly,
which preserves receiver type hints before comparison analysis. Owning homogeneous
scalar-union widening uses explicit tag matching; mixed or nullable unions do not
silently become strings. Replacement props are restored after extracting `render`
so children/callbacks can transfer as one complete record.

JSX spread preserves concrete callback fields, including absent environments. Only
closed owning callback families with no constructors are proven uninhabited.
Unknown borrowed callbacks and incomplete analysis retain unresolved error contracts.
The existing rejection and fallible-record witnesses continue to pass.
