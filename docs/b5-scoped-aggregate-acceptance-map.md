# B5 scoped aggregates: remaining acceptance map

Date: 2026-10-09. Ledger row: B5 in `docs/compiler-completion-plan.md`.

B5 is not complete. This map lists every storage form for records whose callback
borrows local variables, its current status and the evidence. A source diagnostic is
not support. Status was probed against the compiler built from the checkpoint that
adds this file.

## Supported, with native and disposal evidence

| Form | Evidence |
| --- | --- |
| Local record, record field, nested record field | `tests/scoped_record_composition.rs`, earlier gates |
| Dense local array of capturing records: literal, push, indexed replacement, pop, shift, discarded and retained comma removal | array storage epoch tests in the same suite |
| Local `Map<K, Record>`: set, delete, clear, iteration, branches, loops, `try`/`catch`/`finally`, labelled blocks | Map entry and cleanup tests in the same suite |
| Nullable local, absent start, replacement, clearing, move | `nullable_scoped_record_*` |
| Nullable local and nullable field, populated start | `nullable_callback_records_can_start_populated` |
| Nullable field: direct store, replacement, clearing, sibling independence, nested path, `null` variant, optional field, ordinary siblings, right operand reading the old field | `nullable_record_fields_*` |
| Nullable field next to a scoped callback array | `nullable_record_fields_compose_with_scoped_array_storage` |
| Record with a populated nullable field whose closure owns its captures, returned from a function | `nullable_callback_records_can_start_populated` |
| `Map<K, Record>` whose record has a populated nullable field | probe only; add a test before relying on it |

## Diagnosed, not supported

Each row needs storage and lifetime planning, tests, editor witness, guide text and
a workload helper before B5 can close.

| Form | Current result | Needed |
| --- | --- | --- |
| Whole-record replacement when a field is nullable (`box = {...}`) | `unsupported` | Dispose only live old leaves after the right operand; respect moved fields |
| Nullable field store in a branch | `unsupported` | Environment join at the branch exit |
| Nullable field store in a loop | `unsupported` | Loop edge environments, same model as Map loop edges |
| Nullable field store through an index (`boxes[0].saved = ...`) | `unsupported` | Slot-specific element environments |
| Nullable field store through a `Mutable<Record>` parameter | `unsupported` | Caller-side environment family (shared with ledger row B7) |
| `ReadOnly<Record>` parameter where the record has a nullable callback field | `unsupported` (container message) | Borrowed view of the concrete environment |
| Array element type `(Record \| undefined)[]` | `unsupported` | Nullable slot storage |
| Conditional expression as the stored value (`flag ? {...} : undefined`) | `type` error | Join of branch environments in a value position |
| Union of two record types where one is a capturing record | `type` error | Tagged storage for a capturing member |
| Function result `Record \| undefined` holding a capturing record | `callback-escape` | Decide whether owned-capture results may escape; then carrier |
| `Set<Record>` | `set-key` | Structural record keys (owner direction below) |

## Not started

- Structural record keys for `Set` and `Map`. Owner direction: structural equality
  and hashing, ownership transfer on insertion, explicit clone, protection of stored
  keys against equality-changing mutation, diagnostics for unsupported fields. Float
  and callable field policy is not decided.
- Arbitrary comma final operands beyond the forms covered by the array storage tests.

## Recommended order

Prefer shared planning over one guard per form.

1. One environment-join model for nullable field stores at branch and loop exits.
   The Map loop edge planner already records entry, back and break environments per
   place; extend its places to nullable field leaves instead of adding a parallel
   planner.
2. Whole-record replacement, using the same live-leaf facts that the join needs.
3. Indexed and nullable array slots, on the existing array storage epochs.
4. Parameter forms, together with ledger row B7.
5. Value-position joins: conditional expressions and capturing union members.
6. Structural record keys, after the owner decides the float and callable policy.

Run the complete compiler and editor regressions, timing review and consumer refresh
when rows 1 to 5 are accepted.
