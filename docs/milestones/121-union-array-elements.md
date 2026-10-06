# Gate 121: union array elements

Status: focused compiler/native, editor, documentation and benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior

An array can declare a union element type: `(int | string)[]`, `(Item | null)[]`, `(number | string | boolean | null | undefined)[]`, at any rank. Each element carries its own alternative.

- A literal, `push`, `unshift`, `fill`, `with`, `splice` and index assignment accept a value of any alternative. A value of another type is a `type` error.
- Indexing, `for...of` and callback parameters give the union type. `typeof`, `!== null` and `!== undefined` narrow it as for any data union.
- `at`, `find`, `findLast`, `pop` and `shift` return the element alternatives plus `undefined` as one flat union.
- `includes`, `indexOf` and `lastIndexOf` work when every alternative is int, number, boolean, string, null or undefined. The searched value has one alternative type. An element with another active alternative does not match, except that int and number compare by value. `includes` matches `NaN`. A union that contains a record or array alternative is an `unsupported` error for these three methods; use a predicate method.
- `join`, `toString`, `String()` and templates write the text of each active alternative; null and undefined write nothing.
- `map`, `filter`, `forEach`, `reduce`, `some`, `every`, `findIndex`, `findLastIndex`, `slice`, `concat`, `reverse`, `toReversed`, `sort` and `toSorted` with a comparator, `toSpliced`, `copyWithin`, `flat`, `Array.from`, `Array.isArray`, `Array.of<A | B>(...)`, array spread, rest parameters and `clone()` accept union arrays. Console output shows each element in its own form.
- A record literal inside a union context takes the record alternative whose field names match it. A literal that matches two record alternatives is a `type` error with advice to build it through a typed binding.
- Ownership rules are unchanged: indexing an array with owned alternatives borrows the element, spreading transfers owned elements, and `clone()` gives an independent copy.

Not included: strict equality between two union values, an untyped literal such as `[1, "a"]` without a declared element type, destructuring, and default `sort()` without a comparator on mixed alternatives.

Both compiler fixtures match Node v24.11.1 output, with annotations removed and `clone()` supplied as a deep copy.

## Implementation

`ArrayElement::Union(UnionId, depth)` is a new array leaf. The array annotation resolver interns the element union; a union that collapses to one alternative becomes an ordinary leaf. Native storage is a vector of the existing owner enum, so cloning, dropping and transport reuse gate 120.

Emitter changes: array callback inputs pass a union element as itself or as a readonly view; optional element results widen an element union into the flat result union through `optional_element_present`; `union_search_predicate` compares one element with a searched alternative; every data union now has a text conversion for joins. The checker adds `union_collection_search` and `record_alternative`.

## Repairs

- A record literal assigned to a union such as `Item | int` was rejected. It now selects the matching record alternative. This applies outside arrays too.
- Stale expectations: the union arena unit test still rejected `int | string | null`; three tests expected `(int | null)[]` to be unsupported; one test still expected `(1.0).toString()` to be unsupported.
- The catalog witness test had not been updated for `Array.entries`, `Array.toLocaleString`, `int.toLocaleString`, `Object.keys`, `String.raw` and the `Tuple` family. All have witnesses now.
- Editor type names for arrays of unions lacked grouping (`int | string[]`). They now read `(int | string)[]`.

## Editor

Hover, completion and parameter help work on union arrays. `at`, `find`, `pop` and `shift` show the flat optional union. `includes`, `indexOf` and `lastIndexOf` are offered for primitive unions and show the alternatives as the value type. The editor now shows result types for `slice`, `concat`, `filter`, `with`, `toSpliced`, `toReversed`, `includes`, `some`, `every`, `indexOf`, `lastIndexOf`, `findIndex` and `findLastIndex` calls that have arguments, for every array type. The catalog entries for `includes`, `indexOf` and `lastIndexOf` were rewritten with union examples, and eight entries no longer say that nullable elements are unsupported.

## Focused evidence

- `crates/tsrust-compiler/tests/union_array_elements.rs`: three tests; two Node-matched fixtures in `tests/fixtures/union-array-elements/` and seven static rejections.
- Compiler suites passed: unit tests, `union_array_elements`, `data_unions`, `nullable_unions`, `nullable_narrowing`, `nullable_callables`, `array_at`, `array_find`, `array_pop_shift`, `array_search`, `array_last_index_of`, `array_join`, `array_to_string`, `array_locale_string`, `array_transform`, `array_for_each`, `array_predicates`, `array_reduce`, `array_entries`, `tuple_types`, `object_keys`, `array_literal_spread`, `call_spread`, `fixed_storage`, `generic_default_builtins`.
- All 181 editor tests passed, including `union_array_elements_show_types_members_and_search_help` and the native catalog examples.
- Benchmark support: `union_array_values` with an independent reasoned oracle, source contract and registration test. Untimed native and Node-after-tsc verification passed for it and for `data_union_values`. All 324 harness checks passed in one run.

The byte-exact baseline is `archive/union-array-elements-start`. No full compiler regression, timing or allocation profiling was run. SQLite measurement records are unchanged. The new comparison is ready and unmeasured.

Next gate: `Object.values` and `Object.entries` over records with mixed field types.
