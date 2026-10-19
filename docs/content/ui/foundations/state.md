<!-- tsrust-docs
{"id":"ui-state","group":"foundations","order":10,"status":"ready","targets":["web","native"],"examples":["counter"]}
-->
# State and ownership

Component-local signals hold state across events. Application callbacks perform updates; the compiler retains their checked captures and attaches state ownership to the returned subtree.

## Reading and writing

`signal(0)` creates independent integer state. `count.get()` reads it; `count.set(count.get() + 1)` updates it with checked arithmetic. String, number and boolean signals are also supported.

Text can bind a signal directly. Supported JSX scalar expressions reading directly declared local state update after handlers. State-reading conditions for `disabled`, `readOnly` and `hidden` also refresh. Precomputed values, arbitrary prop-record reads and unknown helper expressions remain construction-time snapshots.

## Hiding and removing

`View.hidden` removes a subtree from layout, interaction and accessibility while keeping its state and callbacks. Its scroll position returns when shown.

A supported state-reading JSX conditional mounts only the selected branch. Removing that branch releases its state once. Selecting it later constructs a fresh branch. Choose hiding only when retaining that state is intended.

## Callbacks and cleanup

UI callbacks refer to state without becoming additional state owners. Removing a subtree releases its callbacks and native resources through the checked ownership path. There is no public handle or state-attachment step in application code.

Keeping a shared layout mounted preserves its state. Persistent route/layout identity and general keyed lists remain framework work; the current counter is not evidence of a completed router.
