# B5 scoped aggregates: acceptance map

Date: 2026-10-09. Ledger row: B5 in `docs/compiler-completion-plan.md`.

This map lists every storage form for records whose callback borrows local variables,
its status and its evidence. A source diagnostic is not support. Status reflects the
compiler built from the checkpoint that last changed this file.

## Owner decisions (2026-10-09)

- **Control-flow stores use scope-end release.** When a nullable callback record local,
  or a nullable field of a local record, is stored inside a branch, loop, exception
  region or labelled block, or the record is replaced as a whole, or the field belongs
  to an array element, every stored environment keeps its captures borrowed until the
  local's scope ends. Locals with only direct stores in a normal lexical sequence keep
  the precise release rule. The rule can be relaxed later without breaking programs.
- **Structural record keys reject float fields.** A record with a `number` field cannot
  be a `Set` or `Map` key.
- **Structural record keys reject callable fields.** A record that holds a function
  cannot be a key. A capturing record therefore never is a `Set` key.
- **Nullable owned results are allowed.** A function may return `Record | undefined`
  when the callback owns every capture.

## Supported, with native evidence

All rows have tests in `crates/tsrust-compiler/tests/scoped_record_composition.rs`
unless another file is named.

| Form | Test |
| --- | --- |
| Local record, record field, nested record field | earlier gates |
| Dense local array of capturing records: literal, push, indexed replacement, pop, shift, comma removal | array storage epoch tests |
| Local `Map<K, Record>`: set, delete, clear, iteration, branches, loops, `try`/`catch`/`finally`, labelled blocks | Map entry and cleanup tests |
| Nullable local: absent start, populated start, replacement, clearing, move | `nullable_scoped_record_*`, `nullable_callback_records_can_start_populated` |
| Nullable field: direct store, replacement, clearing, siblings, nested path, `null`, optional field | `nullable_record_fields_*` |
| Nullable local or field stored in a branch, loop, `for...of`, `try` block | `nullable_callback_records_join_environments_through_control_flow` |
| Whole-record replacement of a record with a nullable field | same test |
| Scope-end release rule for those stores | `control_flow_stores_keep_nullable_record_captures_until_scope_end` |
| Conditional expression selecting records, or a record and absence | `conditional_expressions_join_callback_record_environments` |
| Function result `Record \| undefined` and `Record \| null` with owned captures; record result mixing absent and populated fields | `nullable_callback_record_results_keep_one_owned_environment` |
| Nullable field of an array element: indexed store, loop, nested array field, literal with mixed elements, push, element replacement, pop | `nullable_callback_record_fields_of_array_elements_share_the_array_carrier` |
| `ReadOnly<Record>` and `Mutable<Record>` parameters reading a nullable field, nested records, joined environments | `record_parameters_read_nullable_callback_fields` |
| Different callbacks that own every capture joined in one local, field, array element, conditional value or function result; owned and borrowing callbacks mixed | `control_flow_stores_keep_nullable_record_captures_until_scope_end`, `nullable_callback_record_results_keep_one_owned_environment` |
| `Set<Record>` for a capturing record | rejected by owner decision (`set-key`) |

## Diagnosed, not supported

| Form | Current result | Needed |
| --- | --- | --- |
| Store into a nullable field through a `Mutable<Record>` parameter | `unsupported` | Caller-side environment family, with ledger row B7 |
| Two parameters that both carry a nullable callback record of one type | `unsupported` | A generic parameter per parameter leaf instead of one per callback contract |
| Parameter typed `ReadOnly<Record> \| undefined` | `unsupported` | Borrowed view of the caller's concrete union |
| `ReadOnly<Record[]>` parameter where the record has a nullable callback field | `unsupported` | Array element views over the carrier element type |
| Array element type `(Record \| undefined)[]` | `unsupported` | Nullable slot storage |
| Union of two record types where one is a capturing record | `type` error | Tagged storage for a capturing member |
| Moving a nullable callback record out of an array element | `unsupported` | Take-and-replace storage for element fields |

## Outside this row

- Structural record keys for data records (no callable field) in `Set` and `Map`. The
  policy is decided above; the feature is not started and does not involve capturing
  records. Track it as its own ledger row.
- Arbitrary comma final operands beyond the forms covered by the array storage tests.

## Implementation notes

- `nullable_record_fields.rs` holds the control-store scan, the sticky-local predicate,
  array slot carriers, result concretization, conditional joins and call retargeting.
- `record_branch_carriers.rs` holds the flow-insensitive carrier join
  (`join_nullable_record_unions`, `join_sticky_record_types`) and the in-place value
  retargeting (`retarget_nullable_store`).
- Every concrete environment of one nullable callback record uses one native enum; later
  unions are type aliases of the first.
- A precise release rule through control flow would need the Map loop-edge planner
  extended to nullable record places. The current rule accepts a subset of those
  programs, so that extension stays compatible.
