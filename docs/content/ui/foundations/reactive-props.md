<!-- tsrust-docs
{"id":"ui-reactive-props","group":"foundations","order":80,"status":"partial","targets":["web","native"],"examples":["reactive-props"]}
-->
# Reactive props and composition

A component runs once to build its tree. A live prop supplies a checked getter;
it does not rerun the component or replace its children when state changes.

## Author a live prop

Declare `ReactiveValue<T>` for a scalar prop that should stay live. Read it through
`useProp<T>(...)` in the component. Callers write ordinary JSX expressions such as
`tone={active.get() ? 'destructive' : 'default'}`. Constants work through the same
contract. Fields with ordinary scalar types keep their construction-time behavior.

`useProp` and `computed` expose `get()` without a setter. `computed<T>(() => ...)`
defines a derived value. Reads reevaluate the getter, without memoization. Keep
getters free of state changes and I/O. Errors propagate when the value is read.

Supported results are exact signed 64-bit ints, numbers, strings, booleans, enums,
literal unions and nullable scalars. Resource, record, array and callback results
are rejected. No implicit conversion loses integer precision.

## Forward and derive styles

The primitive controls accept live `className` and `data-variant` expressions.
Custom components can opt inherited fields into the same contract with
`ViewProps<ReactiveValue<string>, ReactiveValue<string>>`. Use `useProp` to combine
incoming classes with derived classes, or forward the incoming descriptor intact.
Invalid utilities and variant values fail on the read that produces them.

Adapting an owned prop transfers it. Before forwarding the entire props record,
restore a consumed optional field to `undefined`. This makes its replacement
explicit and prevents reusing the transferred evaluator. No callback is cloned.

An existing component extending plain `ViewProps` continues to receive strings.
Updating the compiler does not silently change editable local component contracts.

## Replace the base node

`View` and `Button` accept a `render` factory. It receives owned props once. Return
one node and forward needed behavior with JSX spread. Anonymous factories can
forward live descriptors intact. Put additional `useProp` or `computed` declarations
in a named component; creating state inside anonymous callbacks is rejected.
There is no extra wrapper,
button or focus stop. Forwarded callbacks and children transfer once. Override
props after the spread to replace those fields deliberately.

`Button` replacements must forward `onPress`, disabled state and children to retain
those behaviors. Replacing a button with another kind of node does not invent the
missing control behavior. Replacement rendering is a base composition primitive;
it does not supply overlay focus management or every Base UI composition contract.

## Lifetime

Declare state and derived values with direct `const` declarations in the component,
outside anonymous event callbacks. Returning the tree retains their owners.
Bindings and callbacks hold weak references. Removing the tree releases evaluators
and handlers once; reading a disposed binding fails. Neither web nor native needs
a public lifetime handle.
