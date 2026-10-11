# Scoped callable records: borrowing callback fields

Status: accepted. This closes the record gate of
[the scoped callable aggregate plan](../plans/scoped-callable-aggregate-plan.md) and
milestone 8 of [the field storage plan](../plans/scoped-record-field-storage-plan.md).
The array and composition gates of that plan stay open and are rejected at source
level today. The [guide](../callable-fields.md) describes the language behavior.

## What is accepted

A record field declared `ReadOnly<Function<...>>` or `Mutable<Function<...>>` holds
a callback that borrows the variables it uses. The record stays in the function that
created it.

- Each field keeps its own borrow and its own native storage. Read-only and mutable
  calls, sibling fields as two arguments of one call, nested records, tuples, moves of
  a field or of the whole record, destructuring and scope exit work natively.
- A record parameter `ReadOnly<Holder>` calls fields; `Mutable<Holder>` also replaces
  callbacks. A replacement in a callee releases the caller's old capture at the call
  when every path replaces it. Relays, both branches, comma operands and one
  evaluation of arguments are covered.
- A local replacement, also inside `try`, `catch` and `finally` bodies, keeps the
  error layout of the field's current carrier. A field moved out after a replacement
  keeps that carrier, and calls on the new variable dispatch through it.
- Return, throw and retaining transfers of a record with a borrowing callback report
  `callback-escape`.

## Completed in the closing pass

- Callee replacement with a capture-free or scalar-capturing callback (the pending
  witness of the previous handoff), with nine native and rejection controls.
- Source rejections that replace native compile failures:
  - a callback stored through a record parameter that borrows a parameter or local of
    the storing function, and a callback parameter stored there (`callback-escape`);
  - an array, Map, Set or union that holds a record with a borrowing callback
    (`unsupported`);
  - replacement of a field declared as a borrowing callback or null/undefined
    (`unsupported`).
- A call of a field that carrier planning widened to a family now uses the family's
  error layout (generated Rust E0308 inside exception regions).
- A local that receives a moved, widened field is retyped with its carrier.
- The independent TypeScript reference now runs all 31 project witnesses.

## Defects repaired in passing

These were regressions in accepted features, found by running the whole compiler
crate for the first time since gate 171 (52 failures). None came from this gate.

- `break` and `continue` kept the capture loans of callbacks declared inside the loop
  body alive after the loop. Exits now leave the bindings of the loop scope.
- Moving a `ReadOnly<Function<...>>` parameter into a local turned the parameter into
  a mutable reference and failed in generated Rust.
- A `catch` body after a `try` body that cannot fail was emitted unchecked and
  panicked in the emitter. Such a handler never runs; its body is now dropped. Its
  statements are not type checked.
- A recursive factory with a `Once` callback parameter reported mixed closure
  origins, because the symbolic template body compared its own literal with the
  literal of its concrete copy.
- A result annotation nested deeper than the limit reported `Expected an expression`
  instead of the nesting limit. A statement keyword used as a value reported
  `Unknown value`; it is now a syntax error.
- Carrier planning panicked on unchecked expressions of unused generic functions.
- About 30 tests still expected rejections for features accepted since: string and
  scalar concatenation, numeric literal forms, unary plus, block shadowing, arrays of
  functions, sibling field arguments, borrowing field declarations, scoped `Once`
  parameters and nested function captures. Each was checked against native execution
  before its expectation changed.

## Limits

- Arrays, Maps, Sets and unions of records with borrowing callbacks (array and
  composition gates).
- Replacement of a nullable or optional borrowing callback field.
- A callback stored through a record parameter cannot borrow from the caller through
  another parameter; that needs lifetime relations between parameters.
- At this gate a function that calls a callback field of a record parameter had no
  resolved error contract for that callback. Gate 180 resolves it.
- Statements of an unreachable `catch` body were not type checked at this gate; gate 177 checks them.

## Open witnesses kept red

- `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`
  and the unit test `owned_factory_updates_are_point_sensitive_and_instances_stay_separate`:
  replacement of a generic callable local by a different origin, and point-sensitive
  mutable factory state. Both belong to the callable replacement work after gate 173.

## Evidence

- Whole compiler crate: 2175 pass, 4 fail before the last repair; the two failures of
  that repair are fixed and the affected batch (unit tests, record, exception and
  callable suites, eight depth canaries) then passes 647 with only the unit witness
  above red. The two open witnesses are the only remaining failures.
- Whole editor crate: 339 of 339.
- Matched `scoped_callable_records` workload, appended at the end of the manifest:
  source contract, independent oracle, registration test, run, profile, oracle and
  report hooks. Native and Node-after-tsc agree with the oracle on five untimed cases.
  No timing.
- The guide example runs natively and passes the pinned Node reference.
- Benchmark harness: 363 of 419 pass. The 56 failures are layout-sensitive source
  contracts left by the project formatting milestone; 96 others were repaired here by
  propagating the rehashed source fingerprints. The rest is tracked as separate work.
- Logs `target/editor/record-gate-{final,compiler,editor,consumers,verify,harness}.log`
  and both verify results are in SQLite, read back byte for byte. Five consumers were
  refreshed from the accepted sources. Project formatting changes no file.
