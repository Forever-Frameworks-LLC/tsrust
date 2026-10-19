# Scalar UI state

Run `tsrust dev examples/ui-state` for the paired browser/native component.
`tsrust run examples/ui-state/src/inspect.tsr` checks formatted native output.
One `signal(initial)` API infers int, number, boolean or string. `Signal` without a
type argument means `Signal<int>`. State binds directly in the existing Text
component; callbacks contain application logic. String reads and writes copy
independent snapshots. Numeric state retains exact int/IEEE-754 values; number
text uses the language's ordinary conversion rules.

Owners remain in the returned tree. Text and callbacks use weak references and
fail after disposal. Resource, array, record, nullable and callback state is
explicitly unsupported. Computed string/tree expressions still snapshot at
construction; direct scalar bindings and disabled predicates refresh after handlers.
