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
| Local array of nullable callback records `(Record \| undefined)[]`: literal, indexed store, loop, push, pop, shift, iteration, element view; also as a record field | `arrays_of_nullable_callback_records_share_one_element_environment` |
| Array, nullable-element array and `Map<K, Record>` of capturing records as `ReadOnly` or `Mutable` parameters; empty arrays; array results with owned captures | `arrays_and_maps_of_capturing_records_cross_function_boundaries` |
| Parameter typed `ReadOnly<Record> \| undefined` or `ReadOnly<Record> \| null`, including concrete, absent, owned and conditional values | `nullable_callback_record_readonly_parameter_preserves_caller_storage`, `nullable_callback_record_borrowed_views_cover_absence_null_and_owned_values` |
| Reading a nullable callback-record field of an array element into a local readonly view, including passing that view to a parameter | `nullable_callback_record_element_field_view_preserves_array_storage`, `nullable_callback_record_views_protect_their_source_storage` |
| `Set<Record>` for a capturing record | rejected by owner decision (`set-key`) |
| Store an owned callback record into a nullable field through a `Mutable<Record>` parameter, including named nullable values, replacement, nested relay calls and null clearing | `mutable_record_parameter_stores_owned_nullable_callback_environments`, `nullable_parameter_stores_propagate_and_reject_borrowed_environments` |

## Diagnosed, not supported

| Form | Current result | Needed |
| --- | --- | --- |
| Two parameters that both carry a nullable callback record of one type | `unsupported` | A generic parameter per parameter leaf instead of one per callback contract |
| Union of two record types where one is a capturing record | `type` error | Tagged storage for a capturing member |

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

## Borrowed-view checkpoint, 2026-10-09

The composition suite passes 195 tests. Eight focused compiler tests, the editor
witness, the guide example and all eight depth canaries pass. Later direct null and
undefined argument witnesses also pass. Registration and mutation guards pass before
and after project formatting. Five native and Node-after-tsc correctness cases pass
untimed with 134 observations. The exact result is stored in SQLite as
`runtime/results/2026-10-09T14-59-51-827Z-verify.json`; timing arrays are empty.
Complete compiler/LSP regressions and family timing were not run for this isolated
checkpoint. Three forms remain diagnosed; mixed-record union scope awaits the owner.

## Mutable parameter store checkpoint, 2026-10-09

Owned nullable callback records can be installed and replaced through mutable
record parameters. Named nullable values retain concrete environments; readonly
union injection preserves borrowing. Nested relay calls, null clearing and capture
escape diagnostics are covered. The initial composition run had 194 passes and
three failures; the two injection regressions were repaired, all 24 affected
nullable tests pass, and the subprocess-launch failure passes on retry. The editor
witness, eight depth canaries, guide native/Node reference and final registration
checks pass. Five untimed native/Node-after-tsc workload cases pass with 135
observations. SQLite key:
`runtime/results/2026-10-09T15-20-51-970Z-verify.json`.
Two forms remain diagnosed. Full family regressions and timing remain pending.
See `docs/handoff-2026-10-09-b5-parameter-stores.md`.
