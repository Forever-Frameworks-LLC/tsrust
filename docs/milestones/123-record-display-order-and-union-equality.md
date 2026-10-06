# Gate 123: record display order, union strict equality and mixed array literals

Status: focused compiler/native and editor acceptance complete. Full regressions and timing remain deferred. No new benchmark pair; four affected pairs were verified untimed.

## Record display order

Console display of a record now lists fields in the order of that value's constructor literal, the same order that `Object.keys`, `Object.values` and `Object.entries` use. Display used the type's declaration order before. The owner chose this for consistency inside the language.

Every record type that a console call can reach (directly, or through fields, arrays, tuples and unions of its arguments) now carries the constructor-order index array. Record types that are never displayed or enumerated keep their previous layout. The added index array costs one machine word per field per value of those types; this cost is unmeasured.

## Union strict equality

`===` and `!==` accept a data union operand.

- The other operand is an int, number, boolean or string, or another union whose alternatives are all int, number, boolean, string, null or undefined.
- Values with different active alternatives are unequal. Int and number compare by value. `NaN` is not equal to `NaN`.
- Comparing two unions when either has a record, array or function alternative is an `unsupported` error with advice to narrow with `typeof` first. A union with such alternatives can still be compared with a primitive; those alternatives never match.
- Both operands are only read. Equality does not narrow the union.

This makes predicates such as `values.findIndex((x) => x === "b")` work on union arrays.

## Untyped mixed array literals

An array literal with no declared type whose values have different data types takes the union of those types as its element type: `[1, "a", true, null]` is `(boolean | int | string | null)[]`.

- An int beside a number stays a number, so `[1, 2.5]` is `number[]` and `[1.5, "x", 2]` is `(number | string)[]`.
- A literal with a declared element type still rejects other values, and `push` on an inferred `int[]` still rejects a string.
- A literal that contains an empty array literal keeps the plain rules, so `[[true, false], [], [true]]` is `boolean[][]`.
- `Array.of(1, true)` follows the same rule.
- An integer literal assigned to a union that has number and no int is a number.

Six rejection rows in older tests described the previous rule and were removed. The fixture `fixtures/union-array-elements/mixed-literals.tsr` matches Node.

## Repairs

- `flag || "Guest"` and `1 && true` are valid since gate 120 and produce a union value; three tests still expected a `logical-result` error.
- Two tests expected `x === 1` and `x === x` on `int | null` to be unsupported.

## Not changed

`acceptance::missing_backend_reports_setup_without_fallback_or_output` fails in this environment: with an empty `PATH` the expected rustup install hint is not in the message. The test does not involve records or unions. It was not investigated.

## Focused evidence

- `crates/tsrust-compiler/tests/union_array_elements.rs`: `union_values_compare_with_strict_equality`, a Node-matched fixture (`fixtures/union-array-elements/equality.tsr`) and two rejections.
- Display order: a probe with nested records and arrays of records matches Node. 27 of 29 suites that display or store records passed in one run; the two failures were the stale logical test (repaired) and the environment test above. After the repairs these suites passed: `union_array_elements`, `logical_values`, `data_unions`, `nullable_unions`, `nullable_narrowing`, `nullable_callables`, `console_output`, `nested_records`, `object_values_entries`, `object_keys`, `operators`, `string_operators`, `number_operators`.
- Mixed literals: `untyped_mixed_array_literals_take_a_union_element_type` passes; `arrays`, `array_of`, `array_flat`, `nested_arrays`, `owned_arrays`, `numbers`, `array_literal_spread`, `array_from`, `array_concat`, `array_search`, `fixed_storage`, `fixed_number_storage`, `analysis`, `generic_default_builtins`, `call_spread`, `tuple_types`, `array_fields`, `array_transform`, `templates` and the unit tests pass.
- All 182 editor tests passed. The `typeof` catalog entry now describes union equality.
- All 326 harness checks passed after rebuilding; `object_values_entries`, `union_array_values`, `data_union_values` and `object_keys_order` verified untimed on native and Node-after-tsc.

Baselines: `archive/record-display-order-start` and `archive/union-equality-start`.
