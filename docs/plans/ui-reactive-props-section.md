# Reactive props and replacement composition

Implementation in progress; no acceptance claim. Continue from `41a8ae7f` without
another approval. Preserve the accepted ownership, cancellation and cleanup rules.

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

- [ ] Compiler/runtime values, precise diagnostics and checked generic alias bases.
- [ ] Intrinsic live classes/variant metadata, validation and source evaluation order.
- [ ] Replacement composition, callbacks, children and once-only cleanup.
- [ ] LSP completion/hover/signatures/diagnostics for the same contracts.
- [ ] Editable local examples and docs; existing component sources still compile.
- [ ] Independent Node-after-tsc transition oracle and native/browser observers.
- [ ] Batch focused acceptance, refresh preview/editor and verify SQLite evidence.

Before expanding examples, one narrow compiler probe resolves two implementation
uncertainties: generic reactive-value inference and owned callback-prop projection.
That probe is not a partial feature acceptance or a full regression pass.

The implementation keeps generic callable/data union restrictions intact:
`computed` accepts an owned getter; `useProp` adapts a scalar or `Reactive<T>`
descriptor. Internal weak bindings use the checked native operation directly,
which preserves receiver type hints before comparison analysis. Owning homogeneous
scalar-union widening uses explicit tag matching; mixed or nullable unions do not
silently become strings. Replacement props are restored after extracting `render`
so children/callbacks can transfer as one complete record.
