# Borrowed JSON child placement review

Three identities extend the reviewed inventory from 408 to 411. Keep these
operations in std:json while familiar typeof, Array.isArray and null comparisons
provide category checks. No new global namespace is introduced.

| Identity | Behavior and errors | Ownership and placement |
| --- | --- | --- |
| std:json::JsonView | Opaque read-only JSON child; no constructor | Retains the original owner lifetime, cannot clone; imported native view type |
| std:json::get | Object and exact string key; Result<JsonView or undefined, JsonError>; expected-object on wrong container | Borrows owner or view; no child copy or mutation; imported lookup |
| std:json::at | Array and nonnegative int index; same optional Result; expected-array or index-range | Borrows owner or view; no child copy or mutation; imported lookup |

Missing children and errors carry no child loan. JSON null is a present borrowed
value. Nested lookups and function returns preserve the source lifetime; retained
results prohibit owner invalidation. Keys are used only during lookup. Errors are
redacted fixed messages with offset zero.

Existing kind, asBoolean, asInt, asNumber and stringify imports now also accept
views with unchanged errors and output ownership. JSON.stringify and
JSON.tryStringify accept the same alternatives. takeString continues to consume
only owned JsonValue. Exact numeric checks remain mandatory; typeof is not an
unchecked cast. Native dependencies are selected from usage automatically.

Compiler/native ownership tests, executable catalog examples, editor hover and
signature checks, inventory reconciliation and matched untimed workload evidence
are recorded in [the acceptance milestone](../milestones/json-views.md).


The existing typeof keyword now also supplies a stable boolean proof. Payload
access copies a boolean while preserving JSON storage. Mutation invalidates the
proof; joins and loop back edges must preserve it. This adds no public identity
or import and does not convert exact JSON integers to floating point. String and
numeric refinement remain deferred with explicit checked readers available.
