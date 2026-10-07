# Language and comparison coverage

This inventory tracks the complete selected TypeScript declaration surface plus explicit language constructs. It is a backlog, not a claim of JavaScript compatibility or completed benchmarks. Native Rust behavior and documented quirks remain the implementation direction.

Baseline: pinned TypeScript 5.9.3. Static declarations do not prove runtime availability. APIs found only in `lib.esnext.*` are proposal/declaration candidates requiring a runtime and specification check before implementation. Inherited operations appear under their declaring interface; method overloads share one row. Host/browser/Node APIs require separate inventories; Intl declarations are included.

**Implementation:** pending, in progress, implemented, native variant, or policy. **Comparison:** missing, compound (covered inside a broader workload), paired group (matched named workloads; measurements require separate checkpoint evidence), or compile-only. These states are independent. Pending operations require edge tests and independent counterparts before acceptance; a compound workload does not close an isolated operation comparison.

Full compiler regression and accumulated performance comparisons run at completed feature-family milestones after compiler, editor and benchmark support settles. Development uses focused checks; repairs rerun affected cases. SQLite retains accepted analytics and exact original records; Markdown reports are optional exports. Presentation changes do not rerun measurements.

[Analytics storage](../benchmarks/storage/README.md) · [Historical benchmark snapshot](../BENCHMARKS.md) · [Machine-readable inventory](../benchmarks/coverage/inventory.json) · [Sources and update procedure](../benchmarks/coverage/README.md)

## Contents

[Arrays](#arrays) · [Async](#async) · [BigInt](#bigint) · [Binary and typed arrays](#binary-and-typed-arrays) · [Booleans](#booleans) · [Collections](#collections) · [Control flow](#control-flow) · [Dates](#dates) · [Declarations and syntax](#declarations-and-syntax) · [Decorators](#decorators) · [Errors](#errors) · [Functions and classes](#functions-and-classes) · [Generators](#generators) · [Globals](#globals) · [Host workstreams](#host-workstreams) · [Integer extension](#integer-extension) · [Internationalization](#internationalization) · [Iterators](#iterators) · [Null and undefined](#null-and-undefined) · [Numbers](#numbers) · [Objects](#objects) · [Primitive storage](#primitive-storage) · [Regular expressions](#regular-expressions) · [Resource management](#resource-management) · [Serialization](#serialization) · [Strings](#strings) · [Symbols](#symbols) · [TypeScript types](#typescript-types)

## Coverage summary

| Family | Operations | Implemented or native variant | In progress | Comparison gaps |
| --- | ---: | ---: | ---: | ---: |
| Arrays | 103 | 54 | 0 | 51 |
| Async | 24 | 0 | 0 | 24 |
| BigInt | 30 | 0 | 0 | 30 |
| Binary and typed arrays | 605 | 0 | 0 | 605 |
| Booleans | 18 | 15 | 0 | 3 |
| Collections | 81 | 30 | 0 | 51 |
| Control flow | 36 | 25 | 0 | 11 |
| Dates | 50 | 0 | 0 | 50 |
| Declarations and syntax | 24 | 14 | 0 | 10 |
| Decorators | 17 | 0 | 0 | 17 |
| Errors | 11 | 0 | 0 | 11 |
| Functions and classes | 37 | 17 | 0 | 21 |
| Generators | 42 | 0 | 0 | 42 |
| Globals | 37 | 29 | 0 | 8 |
| Host workstreams | 13 | 1 | 0 | 12 |
| Integer extension | 59 | 58 | 0 | 2 |
| Internationalization | 253 | 0 | 0 | 253 |
| Iterators | 26 | 1 | 0 | 25 |
| Null and undefined | 17 | 11 | 0 | 6 |
| Numbers | 132 | 129 | 0 | 9 |
| Objects | 96 | 14 | 0 | 82 |
| Primitive storage | 13 | 11 | 0 | 2 |
| Regular expressions | 53 | 0 | 0 | 53 |
| Resource management | 22 | 0 | 0 | 22 |
| Serialization | 3 | 0 | 0 | 3 |
| Strings | 77 | 67 | 0 | 10 |
| Symbols | 25 | 0 | 0 | 25 |
| TypeScript types | 53 | 10 | 2 | 41 |

## Arrays

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `Array.[index]` | pending | missing |  |
| `Array.[Symbol.iterator]` | pending | missing |  |
| `Array.[Symbol.unscopables]` | pending | missing |  |
| `Array.at` | native variant | paired group | Strict signed-i64 index, negative offsets and optional dense results. Copyable elements are snapshots; reusable owned receivers return scoped read-only views and temporary owners transfer the selected payload. Focused implementation and IntelliSense/native examples accepted. Eight source pairs registered; measurements accepted at checkpoint 91 against compiled Node.; [evidence](./milestones/87-array-at.md) · [array_at_int](../benchmarks/runtime/workloads/array_at_int.tsr) · [array_at_number](../benchmarks/runtime/workloads/array_at_number.tsr) · [array_at_boolean](../benchmarks/runtime/workloads/array_at_boolean.tsr) · [array_at_string](../benchmarks/runtime/workloads/array_at_string.tsr) · [array_at_nested](../benchmarks/runtime/workloads/array_at_nested.tsr) · [array_at_record](../benchmarks/runtime/workloads/array_at_record.tsr) · [array_at_null](../benchmarks/runtime/workloads/array_at_null.tsr) · [array_at_undefined](../benchmarks/runtime/workloads/array_at_undefined.tsr) |
| `array.callbackElementLoans` | pending | missing |  |
| `array.clone` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds. Proved number literals and direct scalar literal indexing use standard bounded fixed storage; new comparisons count whole iterations and output retention.; [evidence](./primitives-and-collections.md) · [evidence](./milestones/45-fixed-number-arrays.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) · [number_fixed_local_operations](../benchmarks/runtime/workloads/number_fixed_local_operations.tsr) |
| `Array.concat` | implemented | paired group | Dense homogeneous exact-rank arrays and non-array scalar elements; fresh independent native descendants, zero/variadic operands, contextual empty literals and once left-to-right evaluation. One 566-group regression retained with targeted scalar-record layout repair; complete O0/O3 ABI/order/disposal checks pass. Twenty-five eight-runtime comparisons and 75 zero-final-disposal profiles close the gate. Native copying differs from JavaScript reference sharing; sparse/species/coercion/Symbol spreadability remain separate.; [evidence](./milestones/53-array-concat.md) · [array_concat_int_zero](../benchmarks/runtime/workloads/array_concat_int_zero.tsr) · [array_concat_int_variadic](../benchmarks/runtime/workloads/array_concat_int_variadic.tsr) · [array_concat_number_zero](../benchmarks/runtime/workloads/array_concat_number_zero.tsr) · [array_concat_number_variadic](../benchmarks/runtime/workloads/array_concat_number_variadic.tsr) · [array_concat_boolean_zero](../benchmarks/runtime/workloads/array_concat_boolean_zero.tsr) · [array_concat_boolean_variadic](../benchmarks/runtime/workloads/array_concat_boolean_variadic.tsr) · [array_concat_string_zero](../benchmarks/runtime/workloads/array_concat_string_zero.tsr) · [array_concat_string_variadic](../benchmarks/runtime/workloads/array_concat_string_variadic.tsr) · [array_concat_record_zero](../benchmarks/runtime/workloads/array_concat_record_zero.tsr) · [array_concat_record_variadic](../benchmarks/runtime/workloads/array_concat_record_variadic.tsr) · [array_concat_nested_int_zero](../benchmarks/runtime/workloads/array_concat_nested_int_zero.tsr) · [array_concat_nested_int_variadic](../benchmarks/runtime/workloads/array_concat_nested_int_variadic.tsr) · [array_concat_nested_number_zero](../benchmarks/runtime/workloads/array_concat_nested_number_zero.tsr) · [array_concat_nested_number_variadic](../benchmarks/runtime/workloads/array_concat_nested_number_variadic.tsr) · [array_concat_nested_boolean_zero](../benchmarks/runtime/workloads/array_concat_nested_boolean_zero.tsr) · [array_concat_nested_boolean_variadic](../benchmarks/runtime/workloads/array_concat_nested_boolean_variadic.tsr) · [array_concat_nested_string_zero](../benchmarks/runtime/workloads/array_concat_nested_string_zero.tsr) · [array_concat_nested_string_variadic](../benchmarks/runtime/workloads/array_concat_nested_string_variadic.tsr) · [array_concat_nested_record_zero](../benchmarks/runtime/workloads/array_concat_nested_record_zero.tsr) · [array_concat_nested_record_variadic](../benchmarks/runtime/workloads/array_concat_nested_record_variadic.tsr) · [array_concat_deep_int_zero](../benchmarks/runtime/workloads/array_concat_deep_int_zero.tsr) · [array_concat_deep_int_variadic](../benchmarks/runtime/workloads/array_concat_deep_int_variadic.tsr) · [array_concat_recursive_record_zero](../benchmarks/runtime/workloads/array_concat_recursive_record_zero.tsr) · [array_concat_recursive_record_variadic](../benchmarks/runtime/workloads/array_concat_recursive_record_variadic.tsr) · [array_concat_ordered_trace](../benchmarks/runtime/workloads/array_concat_ordered_trace.tsr) |
| `Array.copyWithin` | implemented | paired group | Dense current array leaves and ranks use standard Rust slices, safe integer bounds and left-to-right arguments. One 581-group regression, native O0/O3 complete-output and disposal checks, eight matched comparisons and 24 zero-final-disposal profiles pass per method. Mutators return void; native owned copies remain independent and differ from JavaScript sharing.; [evidence](./milestones/56-array-fill-copy-within.md) · [array_copy_within_int](../benchmarks/runtime/workloads/array_copy_within_int.tsr) · [array_copy_within_number](../benchmarks/runtime/workloads/array_copy_within_number.tsr) · [array_copy_within_boolean](../benchmarks/runtime/workloads/array_copy_within_boolean.tsr) · [array_copy_within_string](../benchmarks/runtime/workloads/array_copy_within_string.tsr) · [array_copy_within_nested](../benchmarks/runtime/workloads/array_copy_within_nested.tsr) · [array_copy_within_record](../benchmarks/runtime/workloads/array_copy_within_record.tsr) · [array_copy_within_deep](../benchmarks/runtime/workloads/array_copy_within_deep.tsr) · [array_copy_within_recursive](../benchmarks/runtime/workloads/array_copy_within_recursive.tsr) |
| `Array.entries` | native variant | paired | Returns an eager independent array of fixed [int,T] tuples. Copies owned data descendants and leaves the receiver unchanged; callable environments cannot be copied. No live iterator or next(). Two matched scenarios pass untimed native/Node-after-tsc checks; array-family measurements remain pending.; [evidence](./milestones/118-array-entries.md) · [array_entries_string](../benchmarks/runtime/workloads/array_entries_string.tsr) · [array_entries_records](../benchmarks/runtime/workloads/array_entries_records.tsr) |
| `Array.every` | implemented | paired | All current array leaves/ranks support reusable typed boolean callbacks with zero through three parameters and exact short-circuit order. Focused O0/O3 checks and one full 535-group regression pass. Six matched workloads across eight runtimes validate complete results and callback traces; all native profiles fully dispose storage. Source and owned elements remain readonly; index misses return int -1. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/46-array-predicates.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_every_int](../benchmarks/runtime/workloads/array_every_int.tsr) · [array_every_number](../benchmarks/runtime/workloads/array_every_number.tsr) · [array_every_boolean](../benchmarks/runtime/workloads/array_every_boolean.tsr) · [array_every_string](../benchmarks/runtime/workloads/array_every_string.tsr) · [array_every_record](../benchmarks/runtime/workloads/array_every_record.tsr) · [array_every_nested](../benchmarks/runtime/workloads/array_every_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.fill` | implemented | paired group | Dense current array leaves and ranks use standard Rust slices, safe integer bounds and left-to-right arguments. One 581-group regression, native O0/O3 complete-output and disposal checks, eight matched comparisons and 24 zero-final-disposal profiles pass per method. Mutators return void; native owned copies remain independent and differ from JavaScript sharing.; [evidence](./milestones/56-array-fill-copy-within.md) · [array_fill_int](../benchmarks/runtime/workloads/array_fill_int.tsr) · [array_fill_number](../benchmarks/runtime/workloads/array_fill_number.tsr) · [array_fill_boolean](../benchmarks/runtime/workloads/array_fill_boolean.tsr) · [array_fill_string](../benchmarks/runtime/workloads/array_fill_string.tsr) · [array_fill_nested](../benchmarks/runtime/workloads/array_fill_nested.tsr) · [array_fill_record](../benchmarks/runtime/workloads/array_fill_record.tsr) · [array_fill_deep](../benchmarks/runtime/workloads/array_fill_deep.tsr) · [array_fill_recursive](../benchmarks/runtime/workloads/array_fill_recursive.tsr) |
| `Array.filter` | implemented | paired | All current array types/ranks support reusable typed callbacks with zero through three parameters. All 527 compiler groups and required checks pass. Four matched comparisons across eight runtimes retain exact outputs and callback traces, with allocation/disposal evidence. Source and owned elements remain readonly; native selected owner cloning differs from JavaScript reference sharing. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/44-array-map-filter.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_filter_int](../benchmarks/runtime/workloads/array_filter_int.tsr) · [array_filter_number](../benchmarks/runtime/workloads/array_filter_number.tsr) · [array_filter_string](../benchmarks/runtime/workloads/array_filter_string.tsr) · [array_filter_nested](../benchmarks/runtime/workloads/array_filter_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.find` | native variant | paired group | Strict reusable Boolean predicate with zero to three readonly element/original-index/source parameters. Forward/reverse short-circuit dense search returns canonical element\|undefined; Copy leaves are snapshots, named owned receivers provide scoped readonly views, and temporary owners transfer the selection. No truthiness coercion, sparse/heterogeneous/nullable elements or consuming/mutating callbacks. Source pairs and exact native IntelliSense examples accepted; measurements accepted at checkpoint 91 against compiled Node. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/88-array-find.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_find_int](../benchmarks/runtime/workloads/array_find_int.tsr) · [array_find_number](../benchmarks/runtime/workloads/array_find_number.tsr) · [array_find_boolean](../benchmarks/runtime/workloads/array_find_boolean.tsr) · [array_find_string](../benchmarks/runtime/workloads/array_find_string.tsr) · [array_find_nested](../benchmarks/runtime/workloads/array_find_nested.tsr) · [array_find_record](../benchmarks/runtime/workloads/array_find_record.tsr) · [array_find_null](../benchmarks/runtime/workloads/array_find_null.tsr) · [array_find_undefined](../benchmarks/runtime/workloads/array_find_undefined.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.findIndex` | implemented | paired | All current array leaves/ranks support reusable typed boolean callbacks with zero through three parameters and exact short-circuit order. Focused O0/O3 checks and one full 535-group regression pass. Six matched workloads across eight runtimes validate complete results and callback traces; all native profiles fully dispose storage. Source and owned elements remain readonly; index misses return int -1. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/46-array-predicates.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_find_index_int](../benchmarks/runtime/workloads/array_find_index_int.tsr) · [array_find_index_number](../benchmarks/runtime/workloads/array_find_index_number.tsr) · [array_find_index_boolean](../benchmarks/runtime/workloads/array_find_index_boolean.tsr) · [array_find_index_string](../benchmarks/runtime/workloads/array_find_index_string.tsr) · [array_find_index_record](../benchmarks/runtime/workloads/array_find_index_record.tsr) · [array_find_index_nested](../benchmarks/runtime/workloads/array_find_index_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.findLast` | native variant | paired group | Strict reusable Boolean predicate with zero to three readonly element/original-index/source parameters. Forward/reverse short-circuit dense search returns canonical element\|undefined; Copy leaves are snapshots, named owned receivers provide scoped readonly views, and temporary owners transfer the selection. No truthiness coercion, sparse/heterogeneous/nullable elements or consuming/mutating callbacks. Source pairs and exact native IntelliSense examples accepted; measurements accepted at checkpoint 91 against compiled Node. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/88-array-find.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_find_last_int](../benchmarks/runtime/workloads/array_find_last_int.tsr) · [array_find_last_number](../benchmarks/runtime/workloads/array_find_last_number.tsr) · [array_find_last_boolean](../benchmarks/runtime/workloads/array_find_last_boolean.tsr) · [array_find_last_string](../benchmarks/runtime/workloads/array_find_last_string.tsr) · [array_find_last_nested](../benchmarks/runtime/workloads/array_find_last_nested.tsr) · [array_find_last_record](../benchmarks/runtime/workloads/array_find_last_record.tsr) · [array_find_last_null](../benchmarks/runtime/workloads/array_find_last_null.tsr) · [array_find_last_undefined](../benchmarks/runtime/workloads/array_find_last_undefined.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.findLastIndex` | implemented | paired | All current array leaves/ranks support reusable typed boolean callbacks with zero through three parameters and exact short-circuit order. Focused O0/O3 checks and one full 535-group regression pass. Six matched workloads across eight runtimes validate complete results and callback traces; all native profiles fully dispose storage. Source and owned elements remain readonly; index misses return int -1. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/46-array-predicates.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_find_last_index_int](../benchmarks/runtime/workloads/array_find_last_index_int.tsr) · [array_find_last_index_number](../benchmarks/runtime/workloads/array_find_last_index_number.tsr) · [array_find_last_index_boolean](../benchmarks/runtime/workloads/array_find_last_index_boolean.tsr) · [array_find_last_index_string](../benchmarks/runtime/workloads/array_find_last_index_string.tsr) · [array_find_last_index_record](../benchmarks/runtime/workloads/array_find_last_index_record.tsr) · [array_find_last_index_nested](../benchmarks/runtime/workloads/array_find_last_index_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.flat` | native variant | paired group | Strict optional signed int depth defaults to one; nonpositive depth copies and positive depth saturates at leaf rank. Nested receivers require a signed literal; rank-one receivers permit dynamic int. Named sources are borrowed and retained owned descendants copied into independent output. Dense homogeneous null/undefined leaves supported; mixed or nullable union elements/coercion unsupported. Focused compiler/editor and exact native catalog examples accepted; matched comparisons are measured at checkpoint 96.; [evidence](./milestones/92-array-flattening.md) · [array_flat_int](../benchmarks/runtime/workloads/array_flat_int.tsr) · [array_flat_number](../benchmarks/runtime/workloads/array_flat_number.tsr) · [array_flat_boolean](../benchmarks/runtime/workloads/array_flat_boolean.tsr) · [array_flat_string](../benchmarks/runtime/workloads/array_flat_string.tsr) · [array_flat_nested](../benchmarks/runtime/workloads/array_flat_nested.tsr) · [array_flat_record](../benchmarks/runtime/workloads/array_flat_record.tsr) · [array_flat_null](../benchmarks/runtime/workloads/array_flat_null.tsr) · [array_flat_undefined](../benchmarks/runtime/workloads/array_flat_undefined.tsr) |
| `Array.flatMap` | native variant | paired group | Strict one reusable callback has zero to three Read/Copy element/original-index/source parameters and one concrete scalar, record or array result. Array results flatten exactly once; callback-produced owners transfer. Named source stays borrowed; consuming/mutating callbacks and mixed/nullable union elements unsupported. Dense homogeneous null/undefined leaves supported. Focused compiler/editor and exact native catalog examples accepted; matched comparisons are measured at checkpoint 96. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/92-array-flattening.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_flat_map_int](../benchmarks/runtime/workloads/array_flat_map_int.tsr) · [array_flat_map_number](../benchmarks/runtime/workloads/array_flat_map_number.tsr) · [array_flat_map_boolean](../benchmarks/runtime/workloads/array_flat_map_boolean.tsr) · [array_flat_map_string](../benchmarks/runtime/workloads/array_flat_map_string.tsr) · [array_flat_map_nested](../benchmarks/runtime/workloads/array_flat_map_nested.tsr) · [array_flat_map_record](../benchmarks/runtime/workloads/array_flat_map_record.tsr) · [array_flat_map_null](../benchmarks/runtime/workloads/array_flat_map_null.tsr) · [array_flat_map_undefined](../benchmarks/runtime/workloads/array_flat_map_undefined.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.forEach` | implemented | paired group | Dense homogeneous current data leaves/ranks traverse forward with reusable zero through three argument callbacks; any valid return is ignored and owned results drop per invocation. One 562-group regression and native ABI/order/disposal checks pass. Sixteen matched comparisons across eight runtimes and 48 zero-final-disposal profiles close the gate; setup/cloning/sharing costs and optimizer removal of unobservable returns are disclosed.; [evidence](./milestones/52-array-for-each.md) · [array_for_each_int](../benchmarks/runtime/workloads/array_for_each_int.tsr) · [array_for_each_number](../benchmarks/runtime/workloads/array_for_each_number.tsr) · [array_for_each_boolean](../benchmarks/runtime/workloads/array_for_each_boolean.tsr) · [array_for_each_string](../benchmarks/runtime/workloads/array_for_each_string.tsr) · [array_for_each_record](../benchmarks/runtime/workloads/array_for_each_record.tsr) · [array_for_each_nested_int](../benchmarks/runtime/workloads/array_for_each_nested_int.tsr) · [array_for_each_nested_number](../benchmarks/runtime/workloads/array_for_each_nested_number.tsr) · [array_for_each_nested_boolean](../benchmarks/runtime/workloads/array_for_each_nested_boolean.tsr) · [array_for_each_nested_string](../benchmarks/runtime/workloads/array_for_each_nested_string.tsr) · [array_for_each_nested_record](../benchmarks/runtime/workloads/array_for_each_nested_record.tsr) · [array_for_each_deep_int](../benchmarks/runtime/workloads/array_for_each_deep_int.tsr) · [array_for_each_recursive_record](../benchmarks/runtime/workloads/array_for_each_recursive_record.tsr) · [array_for_each_discard_string](../benchmarks/runtime/workloads/array_for_each_discard_string.tsr) · [array_for_each_discard_record](../benchmarks/runtime/workloads/array_for_each_discard_record.tsr) · [array_for_each_discard_nested](../benchmarks/runtime/workloads/array_for_each_discard_nested.tsr) · [array_for_each_discard_callable](../benchmarks/runtime/workloads/array_for_each_discard_callable.tsr) |
| `array.holes` | pending | missing |  |
| `Array.includes` | implemented | paired group | Dense int/bool/string subset; other element identities and sparse semantics pending.; [evidence](./milestones/32-array-search.md) · [int_array_includes](../benchmarks/runtime/workloads/int_array_includes.tsr) · [boolean_array_includes](../benchmarks/runtime/workloads/boolean_array_includes.tsr) · [string_array_includes](../benchmarks/runtime/workloads/string_array_includes.tsr) |
| `Array.indexOf` | implemented | paired group | Dense int/bool/string subset; other element identities and sparse semantics pending.; [evidence](./milestones/32-array-search.md) · [int_array_index_of](../benchmarks/runtime/workloads/int_array_index_of.tsr) · [boolean_array_index_of](../benchmarks/runtime/workloads/boolean_array_index_of.tsr) · [string_array_index_of](../benchmarks/runtime/workloads/string_array_index_of.tsr) |
| `array.indexRead` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds. Proved number literals and direct scalar literal indexing use standard bounded fixed storage; new comparisons count whole iterations and output retention.; [evidence](./primitives-and-collections.md) · [evidence](./milestones/45-fixed-number-arrays.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) · [number_fixed_local_operations](../benchmarks/runtime/workloads/number_fixed_local_operations.tsr) · [number_direct_literal_index](../benchmarks/runtime/workloads/number_direct_literal_index.tsr) |
| `array.indexWrite` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds. Proved number literals and direct scalar literal indexing use standard bounded fixed storage; new comparisons count whole iterations and output retention.; [evidence](./primitives-and-collections.md) · [evidence](./milestones/45-fixed-number-arrays.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) · [number_fixed_local_operations](../benchmarks/runtime/workloads/number_fixed_local_operations.tsr) |
| `array.iterateBorrowed` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds.; [evidence](./primitives-and-collections.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) |
| `array.iterateConsuming` | pending | missing |  |
| `array.iterateMutable` | pending | missing |  |
| `Array.join` | implemented | paired group | Strict optional string separator defaults comma; borrowed receiver and separator evaluate once in order, with dense homogeneous existing leaves/ranks and independently owned UTF-8 output. Nested commas, nullish empty slots, record labels, native Number finite formatting and signed i64 text are documented. Four compiler groups, eleven O0/O3 native documentation examples, affected editor inference/property witnesses and independent comparison sources accepted. Direct borrowed iteration writes to standard String; performance comparisons accepted at checkpoint 86.; [evidence](./milestones/82-array-join.md) · [array_join_int](../benchmarks/runtime/workloads/array_join_int.tsr) · [array_join_boolean](../benchmarks/runtime/workloads/array_join_boolean.tsr) · [array_join_string](../benchmarks/runtime/workloads/array_join_string.tsr) · [array_join_nested](../benchmarks/runtime/workloads/array_join_nested.tsr) · [array_join_number](../benchmarks/runtime/workloads/array_join_number.tsr) · [array_join_nullish](../benchmarks/runtime/workloads/array_join_nullish.tsr) |
| `Array.keys` | native variant | paired | Returns an ordinary independent array computed when called (int[] of indices); usable in for...of, Array.from, spread and array methods. No live iterator object or next(). One matched scenario is registered and verified untimed; measurements wait for the array-methods milestone.; [evidence](./milestones/103-array-keys-values.md) · [array_keys_int](../benchmarks/runtime/workloads/array_keys_int.tsr) |
| `Array.lastIndexOf` | implemented | paired group | Dense int, Number, boolean and UTF8 string arrays use standard Rust reverse iteration. Signed bounds normalize safely; NaN never matches and signed zero compares equal. One 593-group regression, O0/O3 complete outputs, eight default/bounded comparisons and 24 fully disposed allocation profiles pass. Every index is retained as index + 1 through exact array encoding. Receiver setup, query support and output retention are timed; complete observation is outside timing. Nested and record identity search remains outside this primitive contract.; [evidence](./milestones/59-array-last-index-of.md) · [array_last_index_of_default_int](../benchmarks/runtime/workloads/array_last_index_of_default_int.tsr) · [array_last_index_of_default_number](../benchmarks/runtime/workloads/array_last_index_of_default_number.tsr) · [array_last_index_of_default_boolean](../benchmarks/runtime/workloads/array_last_index_of_default_boolean.tsr) · [array_last_index_of_default_string](../benchmarks/runtime/workloads/array_last_index_of_default_string.tsr) · [array_last_index_of_bounded_int](../benchmarks/runtime/workloads/array_last_index_of_bounded_int.tsr) · [array_last_index_of_bounded_number](../benchmarks/runtime/workloads/array_last_index_of_bounded_number.tsr) · [array_last_index_of_bounded_boolean](../benchmarks/runtime/workloads/array_last_index_of_bounded_boolean.tsr) · [array_last_index_of_bounded_string](../benchmarks/runtime/workloads/array_last_index_of_bounded_string.tsr) |
| `array.length` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds.; [evidence](./primitives-and-collections.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) |
| `Array.length` | pending | missing |  |
| `array.lengthWrite` | pending | missing |  |
| `array.literal` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds. Proved number literals and direct scalar literal indexing use standard bounded fixed storage; new comparisons count whole iterations and output retention.; [evidence](./primitives-and-collections.md) · [evidence](./milestones/45-fixed-number-arrays.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) · [number_fixed_local_operations](../benchmarks/runtime/workloads/number_fixed_local_operations.tsr) · [number_direct_literal_index](../benchmarks/runtime/workloads/number_direct_literal_index.tsr) |
| `array.literalSpread` | pending | missing |  |
| `Array.map` | implemented | paired | All current array types/ranks support reusable typed callbacks with zero through three parameters. All 527 compiler groups and required checks pass. Four matched comparisons across eight runtimes retain exact outputs and callback traces, with allocation/disposal evidence. Source and owned elements remain readonly; native selected owner cloning differs from JavaScript reference sharing. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/44-array-map-filter.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_map_int](../benchmarks/runtime/workloads/array_map_int.tsr) · [array_map_number](../benchmarks/runtime/workloads/array_map_number.tsr) · [array_map_string](../benchmarks/runtime/workloads/array_map_string.tsr) · [array_map_nested](../benchmarks/runtime/workloads/array_map_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `array.move` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds.; [evidence](./primitives-and-collections.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) |
| `array.negativeIndex` | native variant | missing | Invalid index fails; JS undefined/coercion behavior is not claimed.; [evidence](./milestones/20-scalar-arrays.md) |
| `array.nested` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds.; [evidence](./primitives-and-collections.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) |
| `array.outOfBounds` | native variant | missing | Invalid index fails; JS undefined/coercion behavior is not claimed.; [evidence](./milestones/20-scalar-arrays.md) |
| `Array.pop` | native variant | paired group | Zero-argument dense removal returns canonical element \| undefined, moves owned elements and preserves remaining order. Sparse, heterogeneous and nullable element arrays remain unsupported. Focused acceptance complete; measurements accepted at checkpoint 86.; [evidence](./milestones/86-array-pop-shift.md) · [array_pop_int](../benchmarks/runtime/workloads/array_pop_int.tsr) · [array_pop_number](../benchmarks/runtime/workloads/array_pop_number.tsr) · [array_pop_boolean](../benchmarks/runtime/workloads/array_pop_boolean.tsr) · [array_pop_string](../benchmarks/runtime/workloads/array_pop_string.tsr) · [array_pop_nested](../benchmarks/runtime/workloads/array_pop_nested.tsr) · [array_pop_record](../benchmarks/runtime/workloads/array_pop_record.tsr) · [array_pop_null](../benchmarks/runtime/workloads/array_pop_null.tsr) · [array_pop_undefined](../benchmarks/runtime/workloads/array_pop_undefined.tsr) |
| `Array.push` | native variant | compound | push returns void; remove is native indexed removal, not JS splice.; [evidence](./milestones/30-array-removal.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [ordered_removal](../benchmarks/runtime/workloads/ordered_removal.tsr) |
| `array.recordFields` | native variant | compound | Dense typed ownership; independent mutable clones; checked bounds.; [evidence](./primitives-and-collections.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [nested_arrays](../benchmarks/runtime/workloads/nested_arrays.tsr) · [record_arrays](../benchmarks/runtime/workloads/record_arrays.tsr) |
| `Array.reduce` | implemented | paired | Homogeneous current data leaves/ranks and independent initial accumulator types; reusable callbacks and owned accumulator contracts. One 558-group regression, actual O0/O3 complete-output/order/failure checks, 14 matched comparisons and zero final disposal profiles pass. Native owned seed/result copies differ from JavaScript shared references; gaps are retained in the report. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/51-array-reduction.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_reduce_int](../benchmarks/runtime/workloads/array_reduce_int.tsr) · [array_reduce_number](../benchmarks/runtime/workloads/array_reduce_number.tsr) · [array_reduce_boolean](../benchmarks/runtime/workloads/array_reduce_boolean.tsr) · [array_reduce_string](../benchmarks/runtime/workloads/array_reduce_string.tsr) · [array_reduce_record](../benchmarks/runtime/workloads/array_reduce_record.tsr) · [array_reduce_nested_int](../benchmarks/runtime/workloads/array_reduce_nested_int.tsr) · [array_reduce_nested_number](../benchmarks/runtime/workloads/array_reduce_nested_number.tsr) · [array_reduce_nested_boolean](../benchmarks/runtime/workloads/array_reduce_nested_boolean.tsr) · [array_reduce_nested_string](../benchmarks/runtime/workloads/array_reduce_nested_string.tsr) · [array_reduce_nested_record](../benchmarks/runtime/workloads/array_reduce_nested_record.tsr) · [array_reduce_deep_int](../benchmarks/runtime/workloads/array_reduce_deep_int.tsr) · [array_reduce_int_to_record](../benchmarks/runtime/workloads/array_reduce_int_to_record.tsr) · [array_reduce_int_to_array](../benchmarks/runtime/workloads/array_reduce_int_to_array.tsr) · [array_reduce_int_to_nested](../benchmarks/runtime/workloads/array_reduce_int_to_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.reduceRight` | implemented | paired | Homogeneous current data leaves/ranks and independent initial accumulator types; reusable callbacks and owned accumulator contracts. One 558-group regression, actual O0/O3 complete-output/order/failure checks, 14 matched comparisons and zero final disposal profiles pass. Native owned seed/result copies differ from JavaScript shared references; gaps are retained in the report. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/51-array-reduction.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_reduce_right_int](../benchmarks/runtime/workloads/array_reduce_right_int.tsr) · [array_reduce_right_number](../benchmarks/runtime/workloads/array_reduce_right_number.tsr) · [array_reduce_right_boolean](../benchmarks/runtime/workloads/array_reduce_right_boolean.tsr) · [array_reduce_right_string](../benchmarks/runtime/workloads/array_reduce_right_string.tsr) · [array_reduce_right_record](../benchmarks/runtime/workloads/array_reduce_right_record.tsr) · [array_reduce_right_nested_int](../benchmarks/runtime/workloads/array_reduce_right_nested_int.tsr) · [array_reduce_right_nested_number](../benchmarks/runtime/workloads/array_reduce_right_nested_number.tsr) · [array_reduce_right_nested_boolean](../benchmarks/runtime/workloads/array_reduce_right_nested_boolean.tsr) · [array_reduce_right_nested_string](../benchmarks/runtime/workloads/array_reduce_right_nested_string.tsr) · [array_reduce_right_nested_record](../benchmarks/runtime/workloads/array_reduce_right_nested_record.tsr) · [array_reduce_right_deep_int](../benchmarks/runtime/workloads/array_reduce_right_deep_int.tsr) · [array_reduce_right_int_to_record](../benchmarks/runtime/workloads/array_reduce_right_int_to_record.tsr) · [array_reduce_right_int_to_array](../benchmarks/runtime/workloads/array_reduce_right_int_to_array.tsr) · [array_reduce_right_int_to_nested](../benchmarks/runtime/workloads/array_reduce_right_int_to_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `array.remove` | native variant | compound | push returns void; remove is native indexed removal, not JS splice.; [evidence](./milestones/30-array-removal.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [ordered_removal](../benchmarks/runtime/workloads/ordered_removal.tsr) |
| `Array.reverse` | native variant | paired group | Dense arrays reverse in place through standard Rust slices; returns void. Construction and order-observer costs disclosed; zero allocation during reversal.; [evidence](./milestones/36-array-reversal.md) · [fixed_int_array_reverse](../benchmarks/runtime/workloads/fixed_int_array_reverse.tsr) · [growable_int_array_reverse](../benchmarks/runtime/workloads/growable_int_array_reverse.tsr) · [owned_string_array_reverse](../benchmarks/runtime/workloads/owned_string_array_reverse.tsr) |
| `Array.shift` | native variant | paired group | Zero-argument dense removal returns canonical element \| undefined, moves owned elements and preserves remaining order. Sparse, heterogeneous and nullable element arrays remain unsupported. Focused acceptance complete; measurements accepted at checkpoint 86.; [evidence](./milestones/86-array-pop-shift.md) · [array_shift_int](../benchmarks/runtime/workloads/array_shift_int.tsr) · [array_shift_number](../benchmarks/runtime/workloads/array_shift_number.tsr) · [array_shift_boolean](../benchmarks/runtime/workloads/array_shift_boolean.tsr) · [array_shift_string](../benchmarks/runtime/workloads/array_shift_string.tsr) · [array_shift_nested](../benchmarks/runtime/workloads/array_shift_nested.tsr) · [array_shift_record](../benchmarks/runtime/workloads/array_shift_record.tsr) · [array_shift_null](../benchmarks/runtime/workloads/array_shift_null.tsr) · [array_shift_undefined](../benchmarks/runtime/workloads/array_shift_undefined.tsr) |
| `Array.slice` | implemented | paired | All current array types/ranks support zero/one/two int bounds and independent owned cloning; 519 compiler groups and required checks pass. Four matched value-domain comparisons pass across eight runtimes with shape/IEEE and allocation/disposal validation. Nested owned cloning differs from JavaScript shallow sharing; alias equivalence is not claimed.; [evidence](./milestones/42-array-slice.md) · [array_slice_int](../benchmarks/runtime/workloads/array_slice_int.tsr) · [array_slice_number](../benchmarks/runtime/workloads/array_slice_number.tsr) · [array_slice_string](../benchmarks/runtime/workloads/array_slice_string.tsr) · [array_slice_nested](../benchmarks/runtime/workloads/array_slice_nested.tsr) |
| `Array.some` | implemented | paired | All current array leaves/ranks support reusable typed boolean callbacks with zero through three parameters and exact short-circuit order. Focused O0/O3 checks and one full 535-group regression pass. Six matched workloads across eight runtimes validate complete results and callback traces; all native profiles fully dispose storage. Source and owned elements remain readonly; index misses return int -1. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/46-array-predicates.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_some_int](../benchmarks/runtime/workloads/array_some_int.tsr) · [array_some_number](../benchmarks/runtime/workloads/array_some_number.tsr) · [array_some_boolean](../benchmarks/runtime/workloads/array_some_boolean.tsr) · [array_some_string](../benchmarks/runtime/workloads/array_some_string.tsr) · [array_some_record](../benchmarks/runtime/workloads/array_some_record.tsr) · [array_some_nested](../benchmarks/runtime/workloads/array_some_nested.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.sort` | native variant | paired group | Stable in-place ordering returns void and holds exclusive receiver access; no element cloning. Defaults int numeric, Number total_cmp including signed zero/NaN bits, Boolean false<true, UTF8 String lexical and equal null/undefined. Nested/record elements require a reusable comparator even empty; zero–two readonly/Copy parameters, one concrete int or Number result, zero/NaN equal. Comparator visitation is standard-Rust-dependent; no coercion, sparse/mixed/nullable elements or consuming/once callbacks. Focused compiler/editor and six native catalog examples accepted; fourteen source comparisons registered, measurements accepted at checkpoint 91 against compiled Node. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/91-array-sorting.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_sort_default_int](../benchmarks/runtime/workloads/array_sort_default_int.tsr) · [array_sort_default_number](../benchmarks/runtime/workloads/array_sort_default_number.tsr) · [array_sort_default_boolean](../benchmarks/runtime/workloads/array_sort_default_boolean.tsr) · [array_sort_default_string](../benchmarks/runtime/workloads/array_sort_default_string.tsr) · [array_sort_default_null](../benchmarks/runtime/workloads/array_sort_default_null.tsr) · [array_sort_default_undefined](../benchmarks/runtime/workloads/array_sort_default_undefined.tsr) · [array_sort_compare_int](../benchmarks/runtime/workloads/array_sort_compare_int.tsr) · [array_sort_compare_number](../benchmarks/runtime/workloads/array_sort_compare_number.tsr) · [array_sort_compare_boolean](../benchmarks/runtime/workloads/array_sort_compare_boolean.tsr) · [array_sort_compare_string](../benchmarks/runtime/workloads/array_sort_compare_string.tsr) · [array_sort_compare_nested](../benchmarks/runtime/workloads/array_sort_compare_nested.tsr) · [array_sort_compare_record](../benchmarks/runtime/workloads/array_sort_compare_record.tsr) · [array_sort_compare_null](../benchmarks/runtime/workloads/array_sort_compare_null.tsr) · [array_sort_compare_undefined](../benchmarks/runtime/workloads/array_sort_compare_undefined.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `array.sparse` | pending | missing |  |
| `Array.splice` | native variant | paired group | Strict optional signed start/deletion and same-type variadic insertion return an independently mutable removed-elements array. Zero arguments are readonly and unchanged; omitted deletion removes the tail, explicit zero removes nothing. Arguments evaluate once before outer commit and ranges normalize against post-argument length. Both output lengths use checked native int/Rust capacity limits; owned elements transfer without implicit cloning. No coercion, sparse/heterogeneous/nullable elements. Four compiler groups, a compatibility supplement and six exact catalog examples accepted; eight source comparisons registered, measurements accepted at checkpoint 91 against compiled Node.; [evidence](./milestones/90-array-splice.md) · [array_splice_int](../benchmarks/runtime/workloads/array_splice_int.tsr) · [array_splice_number](../benchmarks/runtime/workloads/array_splice_number.tsr) · [array_splice_boolean](../benchmarks/runtime/workloads/array_splice_boolean.tsr) · [array_splice_string](../benchmarks/runtime/workloads/array_splice_string.tsr) · [array_splice_nested](../benchmarks/runtime/workloads/array_splice_nested.tsr) · [array_splice_record](../benchmarks/runtime/workloads/array_splice_record.tsr) · [array_splice_null](../benchmarks/runtime/workloads/array_splice_null.tsr) · [array_splice_undefined](../benchmarks/runtime/workloads/array_splice_undefined.tsr) |
| `array.temporaryIndex` | implemented | paired | Temporary array roots transfer owned elements without hidden clones; 523 compiler groups and required checks pass. Four matched whole-iteration comparisons pass across eight runtimes with exact output/IEEE and disposal records. Receiver construction, ownership and output retention costs included; named loans and temporary assignment restrictions retained.; [evidence](./milestones/43-temporary-array-indexing.md) · [array_temporary_index_int](../benchmarks/runtime/workloads/array_temporary_index_int.tsr) · [array_temporary_index_number](../benchmarks/runtime/workloads/array_temporary_index_number.tsr) · [array_temporary_index_string](../benchmarks/runtime/workloads/array_temporary_index_string.tsr) · [array_temporary_index_nested](../benchmarks/runtime/workloads/array_temporary_index_nested.tsr) |
| `Array.toLocaleString` | native variant | paired | No arguments; comma-separated recursive element display. Numeric leaves use fixed decimal grouping and rounding; null/undefined leaves contribute empty text. Receiver and nested values are borrowed. Locale/options arguments, sparse arrays, custom element formatting and function-source elements are unsupported. One complete all-leaf-kind scenario is verified untimed; array-family measurements remain pending.; [evidence](./milestones/115-array-locale-string.md) · [array_locale_string_values](../benchmarks/runtime/workloads/array_locale_string_values.tsr) |
| `Array.toReversed` | implemented | paired group | Dense current array leaves and ranks return fresh owned copies through standard Rust iteration. Negative replacement indices count from the end; invalid bounds fail after arguments. One 585-group regression, O0/O3 complete-output and disposal checks, eight matched comparisons and 24 zero-final-disposal profiles pass per method. Receiver setup and natural output construction/retention are timed. Native independent owned copying differs from JavaScript shallow sharing.; [evidence](./milestones/57-array-copying-updates.md) · [array_to_reversed_int](../benchmarks/runtime/workloads/array_to_reversed_int.tsr) · [array_to_reversed_number](../benchmarks/runtime/workloads/array_to_reversed_number.tsr) · [array_to_reversed_boolean](../benchmarks/runtime/workloads/array_to_reversed_boolean.tsr) · [array_to_reversed_string](../benchmarks/runtime/workloads/array_to_reversed_string.tsr) · [array_to_reversed_nested](../benchmarks/runtime/workloads/array_to_reversed_nested.tsr) · [array_to_reversed_record](../benchmarks/runtime/workloads/array_to_reversed_record.tsr) · [array_to_reversed_deep](../benchmarks/runtime/workloads/array_to_reversed_deep.tsr) · [array_to_reversed_recursive](../benchmarks/runtime/workloads/array_to_reversed_recursive.tsr) |
| `Array.toSorted` | native variant | paired group | Stable independently mutable sorted copy uses Read source access and clones owned descendants; comparator may read source but cannot mutate/consume it. Defaults int numeric, Number total_cmp including signed zero/NaN bits, Boolean false<true, UTF8 String lexical and equal null/undefined. Nested/record elements require a reusable comparator even empty; zero–two readonly/Copy parameters, one concrete int or Number result, zero/NaN equal. Comparator visitation is standard-Rust-dependent; no coercion, sparse/mixed/nullable elements or consuming/once callbacks. Focused compiler/editor and six native catalog examples accepted; fourteen source comparisons registered, measurements accepted at checkpoint 91 against compiled Node. Gate 150 repairs checked callback error transport and readonly abstract invocation. Comparator failure retains source order. New error-transport comparison verifies untimed only.; [evidence](./milestones/91-array-sorting.md) · [evidence](./milestones/150-array-callback-errors.md) · [array_to_sorted_default_int](../benchmarks/runtime/workloads/array_to_sorted_default_int.tsr) · [array_to_sorted_default_number](../benchmarks/runtime/workloads/array_to_sorted_default_number.tsr) · [array_to_sorted_default_boolean](../benchmarks/runtime/workloads/array_to_sorted_default_boolean.tsr) · [array_to_sorted_default_string](../benchmarks/runtime/workloads/array_to_sorted_default_string.tsr) · [array_to_sorted_default_null](../benchmarks/runtime/workloads/array_to_sorted_default_null.tsr) · [array_to_sorted_default_undefined](../benchmarks/runtime/workloads/array_to_sorted_default_undefined.tsr) · [array_to_sorted_compare_int](../benchmarks/runtime/workloads/array_to_sorted_compare_int.tsr) · [array_to_sorted_compare_number](../benchmarks/runtime/workloads/array_to_sorted_compare_number.tsr) · [array_to_sorted_compare_boolean](../benchmarks/runtime/workloads/array_to_sorted_compare_boolean.tsr) · [array_to_sorted_compare_string](../benchmarks/runtime/workloads/array_to_sorted_compare_string.tsr) · [array_to_sorted_compare_nested](../benchmarks/runtime/workloads/array_to_sorted_compare_nested.tsr) · [array_to_sorted_compare_record](../benchmarks/runtime/workloads/array_to_sorted_compare_record.tsr) · [array_to_sorted_compare_null](../benchmarks/runtime/workloads/array_to_sorted_compare_null.tsr) · [array_to_sorted_compare_undefined](../benchmarks/runtime/workloads/array_to_sorted_compare_undefined.tsr) · [array_callback_errors](../benchmarks/runtime/workloads/array_callback_errors.tsr) |
| `Array.toSpliced` | implemented | paired group | Dense current leaves and ranks return fresh owned arrays through standard Rust iteration. Signed bounds clamp; omitted arguments keep standard semantics. Deleted descendants are skipped before cloning and insertion owners transfer. One 589-group regression, O0/O3 complete outputs, eight matched comparisons and 24 fully disposed allocation profiles pass. Each complete result array is retained by normal ownership transfer without an extra clone; observers preserve variable result boundaries outside timing. Native owned copying differs from JavaScript shallow sharing.; [evidence](./milestones/58-array-to-spliced.md) · [array_to_spliced_int](../benchmarks/runtime/workloads/array_to_spliced_int.tsr) · [array_to_spliced_number](../benchmarks/runtime/workloads/array_to_spliced_number.tsr) · [array_to_spliced_boolean](../benchmarks/runtime/workloads/array_to_spliced_boolean.tsr) · [array_to_spliced_string](../benchmarks/runtime/workloads/array_to_spliced_string.tsr) · [array_to_spliced_nested](../benchmarks/runtime/workloads/array_to_spliced_nested.tsr) · [array_to_spliced_record](../benchmarks/runtime/workloads/array_to_spliced_record.tsr) · [array_to_spliced_deep](../benchmarks/runtime/workloads/array_to_spliced_deep.tsr) · [array_to_spliced_recursive](../benchmarks/runtime/workloads/array_to_spliced_recursive.tsr) |
| `Array.toString` | implemented | paired group | Strict zero-argument default-comma conversion of dense homogeneous existing arrays, with nested commas/nullish empty positions/record labels/native scalar text. Borrowed inputs produce an independently owned standard String through the join append trait, without diagnostic trees. Four focused compiler groups, seventeen native O0/O3 catalog examples, affected editor contracts and matched source counterparts accepted. Six comparisons measured at checkpoint 86.; [evidence](./milestones/83-array-to-string.md) · [array_to_string_int](../benchmarks/runtime/workloads/array_to_string_int.tsr) · [array_to_string_boolean](../benchmarks/runtime/workloads/array_to_string_boolean.tsr) · [array_to_string_string](../benchmarks/runtime/workloads/array_to_string_string.tsr) · [array_to_string_nested](../benchmarks/runtime/workloads/array_to_string_nested.tsr) · [array_to_string_number](../benchmarks/runtime/workloads/array_to_string_number.tsr) · [array_to_string_nullish](../benchmarks/runtime/workloads/array_to_string_nullish.tsr) |
| `Array.unshift` | native variant | paired group | Strict variadic same-type front insertion returns signed native int length; zero arguments read length without mutation. Arguments evaluate left-to-right before outer commit and post-argument length/capacity checks. Owned arguments transfer; borrowed indexed owners require explicit clone. No flattening, coercion, sparse/heterogeneous/nullable elements. Focused compiler/editor and five exact native catalog examples accepted; eight source comparisons registered, measurements accepted at checkpoint 91 against compiled Node.; [evidence](./milestones/89-array-unshift.md) · [array_unshift_int](../benchmarks/runtime/workloads/array_unshift_int.tsr) · [array_unshift_number](../benchmarks/runtime/workloads/array_unshift_number.tsr) · [array_unshift_boolean](../benchmarks/runtime/workloads/array_unshift_boolean.tsr) · [array_unshift_string](../benchmarks/runtime/workloads/array_unshift_string.tsr) · [array_unshift_nested](../benchmarks/runtime/workloads/array_unshift_nested.tsr) · [array_unshift_record](../benchmarks/runtime/workloads/array_unshift_record.tsr) · [array_unshift_null](../benchmarks/runtime/workloads/array_unshift_null.tsr) · [array_unshift_undefined](../benchmarks/runtime/workloads/array_unshift_undefined.tsr) |
| `Array.values` | native variant | paired | Returns an ordinary independent array computed when called (a copy of the elements); usable in for...of, Array.from, spread and array methods. No live iterator object or next(). One matched scenario is registered and verified untimed; measurements wait for the array-methods milestone.; [evidence](./milestones/103-array-keys-values.md) · [array_values_int](../benchmarks/runtime/workloads/array_values_int.tsr) |
| `Array.with` | implemented | paired group | Dense current array leaves and ranks return fresh owned copies through standard Rust iteration. Negative replacement indices count from the end; invalid bounds fail after arguments. One 585-group regression, O0/O3 complete-output and disposal checks, eight matched comparisons and 24 zero-final-disposal profiles pass per method. Receiver setup and natural output construction/retention are timed. Native independent owned copying differs from JavaScript shallow sharing.; [evidence](./milestones/57-array-copying-updates.md) · [array_with_int](../benchmarks/runtime/workloads/array_with_int.tsr) · [array_with_number](../benchmarks/runtime/workloads/array_with_number.tsr) · [array_with_boolean](../benchmarks/runtime/workloads/array_with_boolean.tsr) · [array_with_string](../benchmarks/runtime/workloads/array_with_string.tsr) · [array_with_nested](../benchmarks/runtime/workloads/array_with_nested.tsr) · [array_with_record](../benchmarks/runtime/workloads/array_with_record.tsr) · [array_with_deep](../benchmarks/runtime/workloads/array_with_deep.tsr) · [array_with_recursive](../benchmarks/runtime/workloads/array_with_recursive.tsr) |
| `ArrayConstructor.[Symbol.species]` | pending | missing |  |
| `ArrayConstructor.call()` | pending | missing |  |
| `ArrayConstructor.from` | native variant | paired group | Array, Unicode string, typed Set and Map sources. Unmapped owned descendants are explicitly copied into independent arrays. Mapping borrows items, including noncloneable Map values; exactly item/index are supplied with default/rest and checked error support. This does not preserve JS shallow object aliases. New collection scenario and existing mapped workloads verify untimed; measurements pending.; [evidence](./milestones/102-array-from.md) · [evidence](./milestones/156-array-from-collections.md) · [evidence](./array-from.md) · [array_from_copy_int](../benchmarks/runtime/workloads/array_from_copy_int.tsr) · [array_from_map_int](../benchmarks/runtime/workloads/array_from_map_int.tsr) · [array_from_string_characters](../benchmarks/runtime/workloads/array_from_string_characters.tsr) · [array_from_map_string](../benchmarks/runtime/workloads/array_from_map_string.tsr) · [array_from_collections](../benchmarks/runtime/workloads/array_from_collections.tsr) |
| `ArrayConstructor.fromAsync` | pending | missing | esnext declaration; availability unverified |
| `ArrayConstructor.isArray` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) |
| `ArrayConstructor.new()` | pending | missing |  |
| `ArrayConstructor.of` | native variant | paired group | Typed dense construction with inferred/contextual or explicit element type, including checked union and stored-callable families. Array/string/Set/Map spread preserves ordered staging and ordinary copy/transfer rules. Eight catalog examples pass native and Node. Original base measurements at checkpoint96 remain preserved; new iterable spread support is verified untimed and unmeasured.; [evidence](./milestones/93-array-of.md) · [evidence](./milestones/157-iterable-array-spread.md) · [evidence](./array-literal-spread.md) · [array_of_int](../benchmarks/runtime/workloads/array_of_int.tsr) · [array_of_number](../benchmarks/runtime/workloads/array_of_number.tsr) · [array_of_boolean](../benchmarks/runtime/workloads/array_of_boolean.tsr) · [array_of_string](../benchmarks/runtime/workloads/array_of_string.tsr) · [array_of_nested](../benchmarks/runtime/workloads/array_of_nested.tsr) · [array_of_record](../benchmarks/runtime/workloads/array_of_record.tsr) · [array_of_null](../benchmarks/runtime/workloads/array_of_null.tsr) · [array_of_undefined](../benchmarks/runtime/workloads/array_of_undefined.tsr) · [iterable_array_spread](../benchmarks/runtime/workloads/iterable_array_spread.tsr) |
| `ArrayConstructor.prototype` | pending | missing |  |
| `ReadonlyArray.[index]` | pending | missing |  |
| `ReadonlyArray.[Symbol.iterator]` | pending | missing |  |
| `ReadonlyArray.[Symbol.unscopables]` | pending | missing |  |
| `ReadonlyArray.at` | pending | missing |  |
| `ReadonlyArray.concat` | pending | missing |  |
| `ReadonlyArray.entries` | pending | missing |  |
| `ReadonlyArray.every` | pending | missing |  |
| `ReadonlyArray.filter` | pending | missing |  |
| `ReadonlyArray.find` | pending | missing |  |
| `ReadonlyArray.findIndex` | pending | missing |  |
| `ReadonlyArray.findLast` | pending | missing |  |
| `ReadonlyArray.findLastIndex` | pending | missing |  |
| `ReadonlyArray.flat` | pending | missing |  |
| `ReadonlyArray.flatMap` | pending | missing |  |
| `ReadonlyArray.forEach` | pending | missing |  |
| `ReadonlyArray.includes` | pending | missing |  |
| `ReadonlyArray.indexOf` | pending | missing |  |
| `ReadonlyArray.join` | pending | missing |  |
| `ReadonlyArray.keys` | pending | missing |  |
| `ReadonlyArray.lastIndexOf` | pending | missing |  |
| `ReadonlyArray.length` | pending | missing |  |
| `ReadonlyArray.map` | pending | missing |  |
| `ReadonlyArray.reduce` | pending | missing |  |
| `ReadonlyArray.reduceRight` | pending | missing |  |
| `ReadonlyArray.slice` | pending | missing |  |
| `ReadonlyArray.some` | pending | missing |  |
| `ReadonlyArray.toLocaleString` | pending | missing |  |
| `ReadonlyArray.toReversed` | pending | missing |  |
| `ReadonlyArray.toSorted` | pending | missing |  |
| `ReadonlyArray.toSpliced` | pending | missing |  |
| `ReadonlyArray.toString` | pending | missing |  |
| `ReadonlyArray.values` | pending | missing |  |
| `ReadonlyArray.with` | pending | missing |  |

## Async

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `async.await` | pending | missing |  |
| `async.cancellation` | pending | missing |  |
| `async.captures` | pending | missing |  |
| `async.cleanup` | pending | missing |  |
| `async.concurrentState` | pending | missing |  |
| `async.errorPropagation` | pending | missing |  |
| `async.microtasks` | pending | missing |  |
| `async.scheduler` | pending | missing |  |
| `Promise.[Symbol.toStringTag]` | pending | missing |  |
| `Promise.catch` | pending | missing |  |
| `Promise.finally` | pending | missing |  |
| `Promise.then` | pending | missing |  |
| `PromiseConstructor.[Symbol.species]` | pending | missing |  |
| `PromiseConstructor.all` | pending | missing |  |
| `PromiseConstructor.allSettled` | pending | missing |  |
| `PromiseConstructor.any` | pending | missing |  |
| `PromiseConstructor.new()` | pending | missing |  |
| `PromiseConstructor.prototype` | pending | missing |  |
| `PromiseConstructor.race` | pending | missing |  |
| `PromiseConstructor.reject` | pending | missing |  |
| `PromiseConstructor.resolve` | pending | missing |  |
| `PromiseConstructor.try` | pending | missing | esnext declaration; availability unverified |
| `PromiseConstructor.withResolvers` | pending | missing |  |
| `PromiseLike.then` | pending | missing |  |

## BigInt

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `BigInt.[Symbol.toStringTag]` | pending | missing |  |
| `bigint.add` | pending | missing |  |
| `bigint.assign` | pending | missing |  |
| `bigint.bitAnd` | pending | missing |  |
| `bigint.bitNot` | pending | missing |  |
| `bigint.bitOr` | pending | missing |  |
| `bigint.bitXor` | pending | missing |  |
| `bigint.comparisons` | pending | missing |  |
| `bigint.compoundAssignment` | pending | missing |  |
| `bigint.copy` | pending | missing |  |
| `bigint.decrement` | pending | missing |  |
| `bigint.divide` | pending | missing |  |
| `bigint.exponent` | pending | missing |  |
| `bigint.increment` | pending | missing |  |
| `bigint.literal` | pending | missing |  |
| `bigint.multiply` | pending | missing |  |
| `bigint.negate` | pending | missing |  |
| `bigint.numberConversion` | pending | missing |  |
| `bigint.remainder` | pending | missing |  |
| `bigint.shiftLeft` | pending | missing |  |
| `bigint.shiftRight` | pending | missing |  |
| `bigint.stringConversion` | pending | missing |  |
| `bigint.subtract` | pending | missing |  |
| `BigInt.toLocaleString` | pending | missing |  |
| `BigInt.toString` | pending | missing |  |
| `BigInt.valueOf` | pending | missing |  |
| `BigIntConstructor.asIntN` | pending | missing |  |
| `BigIntConstructor.asUintN` | pending | missing |  |
| `BigIntConstructor.call()` | pending | missing |  |
| `BigIntConstructor.prototype` | pending | missing |  |

## Binary and typed arrays

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `ArrayBuffer.[Symbol.toStringTag]` | pending | missing |  |
| `ArrayBuffer.byteLength` | pending | missing |  |
| `ArrayBuffer.detached` | pending | missing |  |
| `ArrayBuffer.maxByteLength` | pending | missing |  |
| `ArrayBuffer.resizable` | pending | missing |  |
| `ArrayBuffer.resize` | pending | missing |  |
| `ArrayBuffer.slice` | pending | missing |  |
| `ArrayBuffer.transfer` | pending | missing |  |
| `ArrayBuffer.transferToFixedLength` | pending | missing |  |
| `ArrayBufferConstructor.[Symbol.species]` | pending | missing |  |
| `ArrayBufferConstructor.isView` | pending | missing |  |
| `ArrayBufferConstructor.new()` | pending | missing |  |
| `ArrayBufferConstructor.prototype` | pending | missing |  |
| `Atomics.[Symbol.toStringTag]` | pending | missing |  |
| `Atomics.add` | pending | missing |  |
| `Atomics.and` | pending | missing |  |
| `Atomics.compareExchange` | pending | missing |  |
| `Atomics.exchange` | pending | missing |  |
| `Atomics.isLockFree` | pending | missing |  |
| `Atomics.load` | pending | missing |  |
| `Atomics.notify` | pending | missing |  |
| `Atomics.or` | pending | missing |  |
| `Atomics.pause` | pending | missing | esnext declaration; availability unverified |
| `Atomics.store` | pending | missing |  |
| `Atomics.sub` | pending | missing |  |
| `Atomics.wait` | pending | missing |  |
| `Atomics.waitAsync` | pending | missing |  |
| `Atomics.xor` | pending | missing |  |
| `BigInt64Array.[index]` | pending | missing |  |
| `BigInt64Array.[Symbol.iterator]` | pending | missing |  |
| `BigInt64Array.[Symbol.toStringTag]` | pending | missing |  |
| `BigInt64Array.at` | pending | missing |  |
| `BigInt64Array.buffer` | pending | missing |  |
| `BigInt64Array.byteLength` | pending | missing |  |
| `BigInt64Array.byteOffset` | pending | missing |  |
| `BigInt64Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `BigInt64Array.copyWithin` | pending | missing |  |
| `BigInt64Array.entries` | pending | missing |  |
| `BigInt64Array.every` | pending | missing |  |
| `BigInt64Array.fill` | pending | missing |  |
| `BigInt64Array.filter` | pending | missing |  |
| `BigInt64Array.find` | pending | missing |  |
| `BigInt64Array.findIndex` | pending | missing |  |
| `BigInt64Array.findLast` | pending | missing |  |
| `BigInt64Array.findLastIndex` | pending | missing |  |
| `BigInt64Array.forEach` | pending | missing |  |
| `BigInt64Array.includes` | pending | missing |  |
| `BigInt64Array.indexOf` | pending | missing |  |
| `BigInt64Array.join` | pending | missing |  |
| `BigInt64Array.keys` | pending | missing |  |
| `BigInt64Array.lastIndexOf` | pending | missing |  |
| `BigInt64Array.length` | pending | missing |  |
| `BigInt64Array.map` | pending | missing |  |
| `BigInt64Array.reduce` | pending | missing |  |
| `BigInt64Array.reduceRight` | pending | missing |  |
| `BigInt64Array.reverse` | pending | missing |  |
| `BigInt64Array.set` | pending | missing |  |
| `BigInt64Array.slice` | pending | missing |  |
| `BigInt64Array.some` | pending | missing |  |
| `BigInt64Array.sort` | pending | missing |  |
| `BigInt64Array.subarray` | pending | missing |  |
| `BigInt64Array.toLocaleString` | pending | missing |  |
| `BigInt64Array.toReversed` | pending | missing |  |
| `BigInt64Array.toSorted` | pending | missing |  |
| `BigInt64Array.toString` | pending | missing |  |
| `BigInt64Array.valueOf` | pending | missing |  |
| `BigInt64Array.values` | pending | missing |  |
| `BigInt64Array.with` | pending | missing |  |
| `BigInt64ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `BigInt64ArrayConstructor.from` | pending | missing |  |
| `BigInt64ArrayConstructor.new()` | pending | missing |  |
| `BigInt64ArrayConstructor.of` | pending | missing |  |
| `BigInt64ArrayConstructor.prototype` | pending | missing |  |
| `BigUint64Array.[index]` | pending | missing |  |
| `BigUint64Array.[Symbol.iterator]` | pending | missing |  |
| `BigUint64Array.[Symbol.toStringTag]` | pending | missing |  |
| `BigUint64Array.at` | pending | missing |  |
| `BigUint64Array.buffer` | pending | missing |  |
| `BigUint64Array.byteLength` | pending | missing |  |
| `BigUint64Array.byteOffset` | pending | missing |  |
| `BigUint64Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `BigUint64Array.copyWithin` | pending | missing |  |
| `BigUint64Array.entries` | pending | missing |  |
| `BigUint64Array.every` | pending | missing |  |
| `BigUint64Array.fill` | pending | missing |  |
| `BigUint64Array.filter` | pending | missing |  |
| `BigUint64Array.find` | pending | missing |  |
| `BigUint64Array.findIndex` | pending | missing |  |
| `BigUint64Array.findLast` | pending | missing |  |
| `BigUint64Array.findLastIndex` | pending | missing |  |
| `BigUint64Array.forEach` | pending | missing |  |
| `BigUint64Array.includes` | pending | missing |  |
| `BigUint64Array.indexOf` | pending | missing |  |
| `BigUint64Array.join` | pending | missing |  |
| `BigUint64Array.keys` | pending | missing |  |
| `BigUint64Array.lastIndexOf` | pending | missing |  |
| `BigUint64Array.length` | pending | missing |  |
| `BigUint64Array.map` | pending | missing |  |
| `BigUint64Array.reduce` | pending | missing |  |
| `BigUint64Array.reduceRight` | pending | missing |  |
| `BigUint64Array.reverse` | pending | missing |  |
| `BigUint64Array.set` | pending | missing |  |
| `BigUint64Array.slice` | pending | missing |  |
| `BigUint64Array.some` | pending | missing |  |
| `BigUint64Array.sort` | pending | missing |  |
| `BigUint64Array.subarray` | pending | missing |  |
| `BigUint64Array.toLocaleString` | pending | missing |  |
| `BigUint64Array.toReversed` | pending | missing |  |
| `BigUint64Array.toSorted` | pending | missing |  |
| `BigUint64Array.toString` | pending | missing |  |
| `BigUint64Array.valueOf` | pending | missing |  |
| `BigUint64Array.values` | pending | missing |  |
| `BigUint64Array.with` | pending | missing |  |
| `BigUint64ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `BigUint64ArrayConstructor.from` | pending | missing |  |
| `BigUint64ArrayConstructor.new()` | pending | missing |  |
| `BigUint64ArrayConstructor.of` | pending | missing |  |
| `BigUint64ArrayConstructor.prototype` | pending | missing |  |
| `DataView.[Symbol.toStringTag]` | pending | missing |  |
| `DataView.buffer` | pending | missing |  |
| `DataView.byteLength` | pending | missing |  |
| `DataView.byteOffset` | pending | missing |  |
| `DataView.getBigInt64` | pending | missing |  |
| `DataView.getBigUint64` | pending | missing |  |
| `DataView.getFloat16` | pending | missing | esnext declaration; availability unverified |
| `DataView.getFloat32` | pending | missing |  |
| `DataView.getFloat64` | pending | missing |  |
| `DataView.getInt16` | pending | missing |  |
| `DataView.getInt32` | pending | missing |  |
| `DataView.getInt8` | pending | missing |  |
| `DataView.getUint16` | pending | missing |  |
| `DataView.getUint32` | pending | missing |  |
| `DataView.getUint8` | pending | missing |  |
| `DataView.setBigInt64` | pending | missing |  |
| `DataView.setBigUint64` | pending | missing |  |
| `DataView.setFloat16` | pending | missing | esnext declaration; availability unverified |
| `DataView.setFloat32` | pending | missing |  |
| `DataView.setFloat64` | pending | missing |  |
| `DataView.setInt16` | pending | missing |  |
| `DataView.setInt32` | pending | missing |  |
| `DataView.setInt8` | pending | missing |  |
| `DataView.setUint16` | pending | missing |  |
| `DataView.setUint32` | pending | missing |  |
| `DataView.setUint8` | pending | missing |  |
| `DataViewConstructor.new()` | pending | missing |  |
| `DataViewConstructor.prototype` | pending | missing |  |
| `Float16Array.[index]` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.[Symbol.iterator]` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.[Symbol.toStringTag]` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.at` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.buffer` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.byteLength` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.byteOffset` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.BYTES_PER_ELEMENT` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.copyWithin` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.entries` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.every` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.fill` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.filter` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.find` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.findIndex` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.findLast` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.findLastIndex` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.forEach` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.includes` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.indexOf` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.join` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.keys` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.lastIndexOf` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.length` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.map` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.reduce` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.reduceRight` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.reverse` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.set` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.slice` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.some` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.sort` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.subarray` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.toLocaleString` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.toReversed` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.toSorted` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.toString` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.valueOf` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.values` | pending | missing | esnext declaration; availability unverified |
| `Float16Array.with` | pending | missing | esnext declaration; availability unverified |
| `Float16ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing | esnext declaration; availability unverified |
| `Float16ArrayConstructor.from` | pending | missing | esnext declaration; availability unverified |
| `Float16ArrayConstructor.new()` | pending | missing | esnext declaration; availability unverified |
| `Float16ArrayConstructor.of` | pending | missing | esnext declaration; availability unverified |
| `Float16ArrayConstructor.prototype` | pending | missing | esnext declaration; availability unverified |
| `Float32Array.[index]` | pending | missing |  |
| `Float32Array.[Symbol.iterator]` | pending | missing |  |
| `Float32Array.[Symbol.toStringTag]` | pending | missing |  |
| `Float32Array.at` | pending | missing |  |
| `Float32Array.buffer` | pending | missing |  |
| `Float32Array.byteLength` | pending | missing |  |
| `Float32Array.byteOffset` | pending | missing |  |
| `Float32Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Float32Array.copyWithin` | pending | missing |  |
| `Float32Array.entries` | pending | missing |  |
| `Float32Array.every` | pending | missing |  |
| `Float32Array.fill` | pending | missing |  |
| `Float32Array.filter` | pending | missing |  |
| `Float32Array.find` | pending | missing |  |
| `Float32Array.findIndex` | pending | missing |  |
| `Float32Array.findLast` | pending | missing |  |
| `Float32Array.findLastIndex` | pending | missing |  |
| `Float32Array.forEach` | pending | missing |  |
| `Float32Array.includes` | pending | missing |  |
| `Float32Array.indexOf` | pending | missing |  |
| `Float32Array.join` | pending | missing |  |
| `Float32Array.keys` | pending | missing |  |
| `Float32Array.lastIndexOf` | pending | missing |  |
| `Float32Array.length` | pending | missing |  |
| `Float32Array.map` | pending | missing |  |
| `Float32Array.reduce` | pending | missing |  |
| `Float32Array.reduceRight` | pending | missing |  |
| `Float32Array.reverse` | pending | missing |  |
| `Float32Array.set` | pending | missing |  |
| `Float32Array.slice` | pending | missing |  |
| `Float32Array.some` | pending | missing |  |
| `Float32Array.sort` | pending | missing |  |
| `Float32Array.subarray` | pending | missing |  |
| `Float32Array.toLocaleString` | pending | missing |  |
| `Float32Array.toReversed` | pending | missing |  |
| `Float32Array.toSorted` | pending | missing |  |
| `Float32Array.toString` | pending | missing |  |
| `Float32Array.valueOf` | pending | missing |  |
| `Float32Array.values` | pending | missing |  |
| `Float32Array.with` | pending | missing |  |
| `Float32ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Float32ArrayConstructor.from` | pending | missing |  |
| `Float32ArrayConstructor.new()` | pending | missing |  |
| `Float32ArrayConstructor.of` | pending | missing |  |
| `Float32ArrayConstructor.prototype` | pending | missing |  |
| `Float64Array.[index]` | pending | missing |  |
| `Float64Array.[Symbol.iterator]` | pending | missing |  |
| `Float64Array.[Symbol.toStringTag]` | pending | missing |  |
| `Float64Array.at` | pending | missing |  |
| `Float64Array.buffer` | pending | missing |  |
| `Float64Array.byteLength` | pending | missing |  |
| `Float64Array.byteOffset` | pending | missing |  |
| `Float64Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Float64Array.copyWithin` | pending | missing |  |
| `Float64Array.entries` | pending | missing |  |
| `Float64Array.every` | pending | missing |  |
| `Float64Array.fill` | pending | missing |  |
| `Float64Array.filter` | pending | missing |  |
| `Float64Array.find` | pending | missing |  |
| `Float64Array.findIndex` | pending | missing |  |
| `Float64Array.findLast` | pending | missing |  |
| `Float64Array.findLastIndex` | pending | missing |  |
| `Float64Array.forEach` | pending | missing |  |
| `Float64Array.includes` | pending | missing |  |
| `Float64Array.indexOf` | pending | missing |  |
| `Float64Array.join` | pending | missing |  |
| `Float64Array.keys` | pending | missing |  |
| `Float64Array.lastIndexOf` | pending | missing |  |
| `Float64Array.length` | pending | missing |  |
| `Float64Array.map` | pending | missing |  |
| `Float64Array.reduce` | pending | missing |  |
| `Float64Array.reduceRight` | pending | missing |  |
| `Float64Array.reverse` | pending | missing |  |
| `Float64Array.set` | pending | missing |  |
| `Float64Array.slice` | pending | missing |  |
| `Float64Array.some` | pending | missing |  |
| `Float64Array.sort` | pending | missing |  |
| `Float64Array.subarray` | pending | missing |  |
| `Float64Array.toLocaleString` | pending | missing |  |
| `Float64Array.toReversed` | pending | missing |  |
| `Float64Array.toSorted` | pending | missing |  |
| `Float64Array.toString` | pending | missing |  |
| `Float64Array.valueOf` | pending | missing |  |
| `Float64Array.values` | pending | missing |  |
| `Float64Array.with` | pending | missing |  |
| `Float64ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Float64ArrayConstructor.from` | pending | missing |  |
| `Float64ArrayConstructor.new()` | pending | missing |  |
| `Float64ArrayConstructor.of` | pending | missing |  |
| `Float64ArrayConstructor.prototype` | pending | missing |  |
| `Int16Array.[index]` | pending | missing |  |
| `Int16Array.[Symbol.iterator]` | pending | missing |  |
| `Int16Array.[Symbol.toStringTag]` | pending | missing |  |
| `Int16Array.at` | pending | missing |  |
| `Int16Array.buffer` | pending | missing |  |
| `Int16Array.byteLength` | pending | missing |  |
| `Int16Array.byteOffset` | pending | missing |  |
| `Int16Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Int16Array.copyWithin` | pending | missing |  |
| `Int16Array.entries` | pending | missing |  |
| `Int16Array.every` | pending | missing |  |
| `Int16Array.fill` | pending | missing |  |
| `Int16Array.filter` | pending | missing |  |
| `Int16Array.find` | pending | missing |  |
| `Int16Array.findIndex` | pending | missing |  |
| `Int16Array.findLast` | pending | missing |  |
| `Int16Array.findLastIndex` | pending | missing |  |
| `Int16Array.forEach` | pending | missing |  |
| `Int16Array.includes` | pending | missing |  |
| `Int16Array.indexOf` | pending | missing |  |
| `Int16Array.join` | pending | missing |  |
| `Int16Array.keys` | pending | missing |  |
| `Int16Array.lastIndexOf` | pending | missing |  |
| `Int16Array.length` | pending | missing |  |
| `Int16Array.map` | pending | missing |  |
| `Int16Array.reduce` | pending | missing |  |
| `Int16Array.reduceRight` | pending | missing |  |
| `Int16Array.reverse` | pending | missing |  |
| `Int16Array.set` | pending | missing |  |
| `Int16Array.slice` | pending | missing |  |
| `Int16Array.some` | pending | missing |  |
| `Int16Array.sort` | pending | missing |  |
| `Int16Array.subarray` | pending | missing |  |
| `Int16Array.toLocaleString` | pending | missing |  |
| `Int16Array.toReversed` | pending | missing |  |
| `Int16Array.toSorted` | pending | missing |  |
| `Int16Array.toString` | pending | missing |  |
| `Int16Array.valueOf` | pending | missing |  |
| `Int16Array.values` | pending | missing |  |
| `Int16Array.with` | pending | missing |  |
| `Int16ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Int16ArrayConstructor.from` | pending | missing |  |
| `Int16ArrayConstructor.new()` | pending | missing |  |
| `Int16ArrayConstructor.of` | pending | missing |  |
| `Int16ArrayConstructor.prototype` | pending | missing |  |
| `Int32Array.[index]` | pending | missing |  |
| `Int32Array.[Symbol.iterator]` | pending | missing |  |
| `Int32Array.[Symbol.toStringTag]` | pending | missing |  |
| `Int32Array.at` | pending | missing |  |
| `Int32Array.buffer` | pending | missing |  |
| `Int32Array.byteLength` | pending | missing |  |
| `Int32Array.byteOffset` | pending | missing |  |
| `Int32Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Int32Array.copyWithin` | pending | missing |  |
| `Int32Array.entries` | pending | missing |  |
| `Int32Array.every` | pending | missing |  |
| `Int32Array.fill` | pending | missing |  |
| `Int32Array.filter` | pending | missing |  |
| `Int32Array.find` | pending | missing |  |
| `Int32Array.findIndex` | pending | missing |  |
| `Int32Array.findLast` | pending | missing |  |
| `Int32Array.findLastIndex` | pending | missing |  |
| `Int32Array.forEach` | pending | missing |  |
| `Int32Array.includes` | pending | missing |  |
| `Int32Array.indexOf` | pending | missing |  |
| `Int32Array.join` | pending | missing |  |
| `Int32Array.keys` | pending | missing |  |
| `Int32Array.lastIndexOf` | pending | missing |  |
| `Int32Array.length` | pending | missing |  |
| `Int32Array.map` | pending | missing |  |
| `Int32Array.reduce` | pending | missing |  |
| `Int32Array.reduceRight` | pending | missing |  |
| `Int32Array.reverse` | pending | missing |  |
| `Int32Array.set` | pending | missing |  |
| `Int32Array.slice` | pending | missing |  |
| `Int32Array.some` | pending | missing |  |
| `Int32Array.sort` | pending | missing |  |
| `Int32Array.subarray` | pending | missing |  |
| `Int32Array.toLocaleString` | pending | missing |  |
| `Int32Array.toReversed` | pending | missing |  |
| `Int32Array.toSorted` | pending | missing |  |
| `Int32Array.toString` | pending | missing |  |
| `Int32Array.valueOf` | pending | missing |  |
| `Int32Array.values` | pending | missing |  |
| `Int32Array.with` | pending | missing |  |
| `Int32ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Int32ArrayConstructor.from` | pending | missing |  |
| `Int32ArrayConstructor.new()` | pending | missing |  |
| `Int32ArrayConstructor.of` | pending | missing |  |
| `Int32ArrayConstructor.prototype` | pending | missing |  |
| `Int8Array.[index]` | pending | missing |  |
| `Int8Array.[Symbol.iterator]` | pending | missing |  |
| `Int8Array.[Symbol.toStringTag]` | pending | missing |  |
| `Int8Array.at` | pending | missing |  |
| `Int8Array.buffer` | pending | missing |  |
| `Int8Array.byteLength` | pending | missing |  |
| `Int8Array.byteOffset` | pending | missing |  |
| `Int8Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Int8Array.copyWithin` | pending | missing |  |
| `Int8Array.entries` | pending | missing |  |
| `Int8Array.every` | pending | missing |  |
| `Int8Array.fill` | pending | missing |  |
| `Int8Array.filter` | pending | missing |  |
| `Int8Array.find` | pending | missing |  |
| `Int8Array.findIndex` | pending | missing |  |
| `Int8Array.findLast` | pending | missing |  |
| `Int8Array.findLastIndex` | pending | missing |  |
| `Int8Array.forEach` | pending | missing |  |
| `Int8Array.includes` | pending | missing |  |
| `Int8Array.indexOf` | pending | missing |  |
| `Int8Array.join` | pending | missing |  |
| `Int8Array.keys` | pending | missing |  |
| `Int8Array.lastIndexOf` | pending | missing |  |
| `Int8Array.length` | pending | missing |  |
| `Int8Array.map` | pending | missing |  |
| `Int8Array.reduce` | pending | missing |  |
| `Int8Array.reduceRight` | pending | missing |  |
| `Int8Array.reverse` | pending | missing |  |
| `Int8Array.set` | pending | missing |  |
| `Int8Array.slice` | pending | missing |  |
| `Int8Array.some` | pending | missing |  |
| `Int8Array.sort` | pending | missing |  |
| `Int8Array.subarray` | pending | missing |  |
| `Int8Array.toLocaleString` | pending | missing |  |
| `Int8Array.toReversed` | pending | missing |  |
| `Int8Array.toSorted` | pending | missing |  |
| `Int8Array.toString` | pending | missing |  |
| `Int8Array.valueOf` | pending | missing |  |
| `Int8Array.values` | pending | missing |  |
| `Int8Array.with` | pending | missing |  |
| `Int8ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Int8ArrayConstructor.from` | pending | missing |  |
| `Int8ArrayConstructor.new()` | pending | missing |  |
| `Int8ArrayConstructor.of` | pending | missing |  |
| `Int8ArrayConstructor.prototype` | pending | missing |  |
| `SharedArrayBuffer.[Symbol.toStringTag]` | pending | missing |  |
| `SharedArrayBuffer.byteLength` | pending | missing |  |
| `SharedArrayBuffer.grow` | pending | missing |  |
| `SharedArrayBuffer.growable` | pending | missing |  |
| `SharedArrayBuffer.maxByteLength` | pending | missing |  |
| `SharedArrayBuffer.slice` | pending | missing |  |
| `SharedArrayBufferConstructor.[Symbol.species]` | pending | missing |  |
| `SharedArrayBufferConstructor.new()` | pending | missing |  |
| `SharedArrayBufferConstructor.prototype` | pending | missing |  |
| `Uint16Array.[index]` | pending | missing |  |
| `Uint16Array.[Symbol.iterator]` | pending | missing |  |
| `Uint16Array.[Symbol.toStringTag]` | pending | missing |  |
| `Uint16Array.at` | pending | missing |  |
| `Uint16Array.buffer` | pending | missing |  |
| `Uint16Array.byteLength` | pending | missing |  |
| `Uint16Array.byteOffset` | pending | missing |  |
| `Uint16Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint16Array.copyWithin` | pending | missing |  |
| `Uint16Array.entries` | pending | missing |  |
| `Uint16Array.every` | pending | missing |  |
| `Uint16Array.fill` | pending | missing |  |
| `Uint16Array.filter` | pending | missing |  |
| `Uint16Array.find` | pending | missing |  |
| `Uint16Array.findIndex` | pending | missing |  |
| `Uint16Array.findLast` | pending | missing |  |
| `Uint16Array.findLastIndex` | pending | missing |  |
| `Uint16Array.forEach` | pending | missing |  |
| `Uint16Array.includes` | pending | missing |  |
| `Uint16Array.indexOf` | pending | missing |  |
| `Uint16Array.join` | pending | missing |  |
| `Uint16Array.keys` | pending | missing |  |
| `Uint16Array.lastIndexOf` | pending | missing |  |
| `Uint16Array.length` | pending | missing |  |
| `Uint16Array.map` | pending | missing |  |
| `Uint16Array.reduce` | pending | missing |  |
| `Uint16Array.reduceRight` | pending | missing |  |
| `Uint16Array.reverse` | pending | missing |  |
| `Uint16Array.set` | pending | missing |  |
| `Uint16Array.slice` | pending | missing |  |
| `Uint16Array.some` | pending | missing |  |
| `Uint16Array.sort` | pending | missing |  |
| `Uint16Array.subarray` | pending | missing |  |
| `Uint16Array.toLocaleString` | pending | missing |  |
| `Uint16Array.toReversed` | pending | missing |  |
| `Uint16Array.toSorted` | pending | missing |  |
| `Uint16Array.toString` | pending | missing |  |
| `Uint16Array.valueOf` | pending | missing |  |
| `Uint16Array.values` | pending | missing |  |
| `Uint16Array.with` | pending | missing |  |
| `Uint16ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint16ArrayConstructor.from` | pending | missing |  |
| `Uint16ArrayConstructor.new()` | pending | missing |  |
| `Uint16ArrayConstructor.of` | pending | missing |  |
| `Uint16ArrayConstructor.prototype` | pending | missing |  |
| `Uint32Array.[index]` | pending | missing |  |
| `Uint32Array.[Symbol.iterator]` | pending | missing |  |
| `Uint32Array.[Symbol.toStringTag]` | pending | missing |  |
| `Uint32Array.at` | pending | missing |  |
| `Uint32Array.buffer` | pending | missing |  |
| `Uint32Array.byteLength` | pending | missing |  |
| `Uint32Array.byteOffset` | pending | missing |  |
| `Uint32Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint32Array.copyWithin` | pending | missing |  |
| `Uint32Array.entries` | pending | missing |  |
| `Uint32Array.every` | pending | missing |  |
| `Uint32Array.fill` | pending | missing |  |
| `Uint32Array.filter` | pending | missing |  |
| `Uint32Array.find` | pending | missing |  |
| `Uint32Array.findIndex` | pending | missing |  |
| `Uint32Array.findLast` | pending | missing |  |
| `Uint32Array.findLastIndex` | pending | missing |  |
| `Uint32Array.forEach` | pending | missing |  |
| `Uint32Array.includes` | pending | missing |  |
| `Uint32Array.indexOf` | pending | missing |  |
| `Uint32Array.join` | pending | missing |  |
| `Uint32Array.keys` | pending | missing |  |
| `Uint32Array.lastIndexOf` | pending | missing |  |
| `Uint32Array.length` | pending | missing |  |
| `Uint32Array.map` | pending | missing |  |
| `Uint32Array.reduce` | pending | missing |  |
| `Uint32Array.reduceRight` | pending | missing |  |
| `Uint32Array.reverse` | pending | missing |  |
| `Uint32Array.set` | pending | missing |  |
| `Uint32Array.slice` | pending | missing |  |
| `Uint32Array.some` | pending | missing |  |
| `Uint32Array.sort` | pending | missing |  |
| `Uint32Array.subarray` | pending | missing |  |
| `Uint32Array.toLocaleString` | pending | missing |  |
| `Uint32Array.toReversed` | pending | missing |  |
| `Uint32Array.toSorted` | pending | missing |  |
| `Uint32Array.toString` | pending | missing |  |
| `Uint32Array.valueOf` | pending | missing |  |
| `Uint32Array.values` | pending | missing |  |
| `Uint32Array.with` | pending | missing |  |
| `Uint32ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint32ArrayConstructor.from` | pending | missing |  |
| `Uint32ArrayConstructor.new()` | pending | missing |  |
| `Uint32ArrayConstructor.of` | pending | missing |  |
| `Uint32ArrayConstructor.prototype` | pending | missing |  |
| `Uint8Array.[index]` | pending | missing |  |
| `Uint8Array.[Symbol.iterator]` | pending | missing |  |
| `Uint8Array.[Symbol.toStringTag]` | pending | missing |  |
| `Uint8Array.at` | pending | missing |  |
| `Uint8Array.buffer` | pending | missing |  |
| `Uint8Array.byteLength` | pending | missing |  |
| `Uint8Array.byteOffset` | pending | missing |  |
| `Uint8Array.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint8Array.copyWithin` | pending | missing |  |
| `Uint8Array.entries` | pending | missing |  |
| `Uint8Array.every` | pending | missing |  |
| `Uint8Array.fill` | pending | missing |  |
| `Uint8Array.filter` | pending | missing |  |
| `Uint8Array.find` | pending | missing |  |
| `Uint8Array.findIndex` | pending | missing |  |
| `Uint8Array.findLast` | pending | missing |  |
| `Uint8Array.findLastIndex` | pending | missing |  |
| `Uint8Array.forEach` | pending | missing |  |
| `Uint8Array.includes` | pending | missing |  |
| `Uint8Array.indexOf` | pending | missing |  |
| `Uint8Array.join` | pending | missing |  |
| `Uint8Array.keys` | pending | missing |  |
| `Uint8Array.lastIndexOf` | pending | missing |  |
| `Uint8Array.length` | pending | missing |  |
| `Uint8Array.map` | pending | missing |  |
| `Uint8Array.reduce` | pending | missing |  |
| `Uint8Array.reduceRight` | pending | missing |  |
| `Uint8Array.reverse` | pending | missing |  |
| `Uint8Array.set` | pending | missing |  |
| `Uint8Array.slice` | pending | missing |  |
| `Uint8Array.some` | pending | missing |  |
| `Uint8Array.sort` | pending | missing |  |
| `Uint8Array.subarray` | pending | missing |  |
| `Uint8Array.toLocaleString` | pending | missing |  |
| `Uint8Array.toReversed` | pending | missing |  |
| `Uint8Array.toSorted` | pending | missing |  |
| `Uint8Array.toString` | pending | missing |  |
| `Uint8Array.valueOf` | pending | missing |  |
| `Uint8Array.values` | pending | missing |  |
| `Uint8Array.with` | pending | missing |  |
| `Uint8ArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint8ArrayConstructor.from` | pending | missing |  |
| `Uint8ArrayConstructor.new()` | pending | missing |  |
| `Uint8ArrayConstructor.of` | pending | missing |  |
| `Uint8ArrayConstructor.prototype` | pending | missing |  |
| `Uint8ClampedArray.[index]` | pending | missing |  |
| `Uint8ClampedArray.[Symbol.iterator]` | pending | missing |  |
| `Uint8ClampedArray.[Symbol.toStringTag]` | pending | missing |  |
| `Uint8ClampedArray.at` | pending | missing |  |
| `Uint8ClampedArray.buffer` | pending | missing |  |
| `Uint8ClampedArray.byteLength` | pending | missing |  |
| `Uint8ClampedArray.byteOffset` | pending | missing |  |
| `Uint8ClampedArray.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint8ClampedArray.copyWithin` | pending | missing |  |
| `Uint8ClampedArray.entries` | pending | missing |  |
| `Uint8ClampedArray.every` | pending | missing |  |
| `Uint8ClampedArray.fill` | pending | missing |  |
| `Uint8ClampedArray.filter` | pending | missing |  |
| `Uint8ClampedArray.find` | pending | missing |  |
| `Uint8ClampedArray.findIndex` | pending | missing |  |
| `Uint8ClampedArray.findLast` | pending | missing |  |
| `Uint8ClampedArray.findLastIndex` | pending | missing |  |
| `Uint8ClampedArray.forEach` | pending | missing |  |
| `Uint8ClampedArray.includes` | pending | missing |  |
| `Uint8ClampedArray.indexOf` | pending | missing |  |
| `Uint8ClampedArray.join` | pending | missing |  |
| `Uint8ClampedArray.keys` | pending | missing |  |
| `Uint8ClampedArray.lastIndexOf` | pending | missing |  |
| `Uint8ClampedArray.length` | pending | missing |  |
| `Uint8ClampedArray.map` | pending | missing |  |
| `Uint8ClampedArray.reduce` | pending | missing |  |
| `Uint8ClampedArray.reduceRight` | pending | missing |  |
| `Uint8ClampedArray.reverse` | pending | missing |  |
| `Uint8ClampedArray.set` | pending | missing |  |
| `Uint8ClampedArray.slice` | pending | missing |  |
| `Uint8ClampedArray.some` | pending | missing |  |
| `Uint8ClampedArray.sort` | pending | missing |  |
| `Uint8ClampedArray.subarray` | pending | missing |  |
| `Uint8ClampedArray.toLocaleString` | pending | missing |  |
| `Uint8ClampedArray.toReversed` | pending | missing |  |
| `Uint8ClampedArray.toSorted` | pending | missing |  |
| `Uint8ClampedArray.toString` | pending | missing |  |
| `Uint8ClampedArray.valueOf` | pending | missing |  |
| `Uint8ClampedArray.values` | pending | missing |  |
| `Uint8ClampedArray.with` | pending | missing |  |
| `Uint8ClampedArrayConstructor.BYTES_PER_ELEMENT` | pending | missing |  |
| `Uint8ClampedArrayConstructor.from` | pending | missing |  |
| `Uint8ClampedArrayConstructor.new()` | pending | missing |  |
| `Uint8ClampedArrayConstructor.of` | pending | missing |  |
| `Uint8ClampedArrayConstructor.prototype` | pending | missing |  |

## Booleans

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `boolean.and` | implemented | compound | [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `boolean.andAssign` | native variant | paired group | Strict boolean operands only; value-producing in supported expression contexts, statements and for clauses. Select target/indices and check bounds once before reading old value; RHS and commit occur only on the selected branch. Preserve incoming/parent loans, conditional ownership joins and existing target/capture/parameter/iteration restrictions. Standard Rust boolean locals and if blocks allocate no scalar storage. Matched actual JavaScript operators retain expression and final target as two complete 0/1 int-array leaves; selection, conversion and output retention are timed. Nullish assignment and coercion remain pending.; [evidence](./milestones/66-boolean-logical-assignment.md) · [boolean_logical_and_assign](../benchmarks/runtime/workloads/boolean_logical_and_assign.tsr) |
| `boolean.assign` | implemented | compound | [evidence](./milestones/12-scalar-operators.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) |
| `boolean.assignmentExpression` | native variant | paired group | Ordinary = returns the committed copy scalar across supported expression contexts, statements and for clauses; target type drives RHS contextual typing with strict matching. Target/indices and bounds occur once before RHS, no old target value is read, and commit follows successful RHS. Preserve incoming/parent loans, unconditional ownership effects and existing target/capture/parameter/iteration/partial-field restrictions. Standard Rust scalar assignment allocates no heap storage in focused evidence. Actual JavaScript comparisons retain expression and final target outputs; dynamic setup, selection, conversion and complete output retention are timed. Owned and arithmetic compound assignment values remain pending.; [evidence](./milestones/67-scalar-assignment-values.md) · [boolean_assignment_value](../benchmarks/runtime/workloads/boolean_assignment_value.tsr) |
| `boolean.copy` | implemented | compound | [evidence](./milestones/12-scalar-operators.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) |
| `boolean.equal` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [boolean_equal](../benchmarks/runtime/workloads/boolean_equal.tsr) |
| `boolean.literal` | implemented | compound | [evidence](./milestones/12-scalar-operators.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) |
| `boolean.not` | implemented | compound | [evidence](./milestones/12-scalar-operators.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) |
| `boolean.notEqual` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [boolean_not_equal](../benchmarks/runtime/workloads/boolean_not_equal.tsr) |
| `boolean.or` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [boolean_or](../benchmarks/runtime/workloads/boolean_or.tsr) |
| `boolean.orAssign` | native variant | paired group | Strict boolean operands only; value-producing in supported expression contexts, statements and for clauses. Select target/indices and check bounds once before reading old value; RHS and commit occur only on the selected branch. Preserve incoming/parent loans, conditional ownership joins and existing target/capture/parameter/iteration restrictions. Standard Rust boolean locals and if blocks allocate no scalar storage. Matched actual JavaScript operators retain expression and final target as two complete 0/1 int-array leaves; selection, conversion and output retention are timed. Nullish assignment and coercion remain pending.; [evidence](./milestones/66-boolean-logical-assignment.md) · [boolean_logical_or_assign](../benchmarks/runtime/workloads/boolean_logical_or_assign.tsr) |
| `boolean.shortCircuit` | implemented | compound | Matched whole-iteration comparison passes across eight runtimes with operation/setup/observer costs disclosed. Short-circuit timing is explicitly compound.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [boolean_short_circuit](../benchmarks/runtime/workloads/boolean_short_circuit.tsr) |
| `Boolean.toString` | implemented | paired group | Primitive receiver only, strict zero arity; receiver evaluates once and stays reusable. Static true/false String result without primitive boxing. Compiler/editor, complete source/native outputs and runtime/disposal acceptance are complete; numeric formatting and boxed objects are separate.; [evidence](./milestones/72-primitive-identity.md) · [boolean_to_string](../benchmarks/runtime/workloads/boolean_to_string.tsr) |
| `boolean.truthinessConversion` | pending | missing |  |
| `Boolean.valueOf` | implemented | paired group | Primitive receiver only, strict zero arity; receiver evaluates once and stays reusable. Boolean primitive copy with natural Boolean[] observations, outside-timer b0/b1 tokens and checksum codes 260/261. Compiler/editor, complete source/native outputs and runtime/disposal acceptance are complete; numeric formatting and boxed objects are separate.; [evidence](./milestones/72-primitive-identity.md) · [boolean_value_of](../benchmarks/runtime/workloads/boolean_value_of.tsr) |
| `BooleanConstructor.call()` | implemented | paired group | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) · [boolean_string_truthiness](../benchmarks/runtime/workloads/boolean_string_truthiness.tsr) · [nullable_int_null](../benchmarks/runtime/workloads/nullable_int_null.tsr) · [nullable_boolean_undefined](../benchmarks/runtime/workloads/nullable_boolean_undefined.tsr) · [nullable_number_null](../benchmarks/runtime/workloads/nullable_number_null.tsr) · [nullable_string_undefined](../benchmarks/runtime/workloads/nullable_string_undefined.tsr) · [nullable_array_null](../benchmarks/runtime/workloads/nullable_array_null.tsr) · [nullable_record_undefined](../benchmarks/runtime/workloads/nullable_record_undefined.tsr) |
| `BooleanConstructor.new()` | pending | missing |  |
| `BooleanConstructor.prototype` | pending | missing |  |

## Collections

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `FinalizationRegistry.[Symbol.toStringTag]` | pending | missing |  |
| `FinalizationRegistry.register` | pending | missing |  |
| `FinalizationRegistry.unregister` | pending | missing |  |
| `FinalizationRegistryConstructor.new()` | pending | missing |  |
| `FinalizationRegistryConstructor.prototype` | pending | missing |  |
| `Map.[Symbol.iterator]` | pending | missing |  |
| `Map.[Symbol.toStringTag]` | pending | missing |  |
| `Map.clear` | native variant | paired | Removes all entries, returns void. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/146-maps.md) · [evidence](./maps.md) · [map_values](../benchmarks/runtime/workloads/map_values.tsr) |
| `Map.delete` | native variant | paired | Borrowed query removes an entry and reports whether it existed. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/146-maps.md) · [evidence](./maps.md) · [map_values](../benchmarks/runtime/workloads/map_values.tsr) |
| `Map.entries` | native variant | paired | Independent typed key/value tuple array with explicit owned copies. Insertion order; zero arguments; not a live iterator. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/147-map-snapshots.md) · [evidence](./maps.md) · [map_snapshots](../benchmarks/runtime/workloads/map_snapshots.tsr) |
| `Map.forEach` | native variant | paired | Insertion-ordered reusable callbacks supply borrowed value, key and readonly Map. Scalar inputs copy; owned inputs do not implicitly clone. Values need not be cloneable. Default/rest and checked error/finally contracts are supported; returns void, no thisArg. Verification is untimed; collection measurements remain pending.; [evidence](./milestones/151-map-for-each.md) · [evidence](./maps.md) · [map_for_each](../benchmarks/runtime/workloads/map_for_each.tsr) |
| `Map.get` | native variant | paired | Scalar results copy; owned values from named receivers are readonly borrows. Temporary receivers move selected owners. Missing keys return undefined. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/146-maps.md) · [evidence](./maps.md) · [map_values](../benchmarks/runtime/workloads/map_values.tsr) |
| `Map.has` | native variant | paired | Borrowed query with SameValueZero membership. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/146-maps.md) · [evidence](./maps.md) · [map_values](../benchmarks/runtime/workloads/map_values.tsr) |
| `Map.keys` | native variant | paired | Independent key array; values need not be cloneable. Insertion order; zero arguments; not a live iterator. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/147-map-snapshots.md) · [evidence](./maps.md) · [map_snapshots](../benchmarks/runtime/workloads/map_snapshots.tsr) |
| `Map.set` | native variant | paired | Transfers owned key/value inputs; returns void. Replacing a key preserves insertion order. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/146-maps.md) · [evidence](./maps.md) · [map_values](../benchmarks/runtime/workloads/map_values.tsr) |
| `Map.size` | native variant | paired | Readonly integer entry count. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/146-maps.md) · [evidence](./maps.md) · [map_values](../benchmarks/runtime/workloads/map_values.tsr) |
| `Map.values` | native variant | paired | Independent value array with explicit owned-value copies. Insertion order; zero arguments; not a live iterator. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/147-map-snapshots.md) · [evidence](./maps.md) · [map_snapshots](../benchmarks/runtime/workloads/map_snapshots.tsr) |
| `MapConstructor.[Symbol.species]` | pending | missing |  |
| `MapConstructor.groupBy` | implemented | paired group | Ordered grouping from arrays, strings, Sets and Maps, two readonly callback positions, defaults/rest, SameValueZero keys and independent owning-value copies. Noncloneable elements and identity keys rejected. Focused checks and matched verification pass untimed; collection milestone measurements pending.; [evidence](./milestones/159-map-group-by.md) · [evidence](./map-group-by.md) · [map_group_by](../benchmarks/runtime/workloads/map_group_by.tsr) |
| `MapConstructor.new()` | native variant | paired | Typed scalar/enum/union keys and owned values; explicit source copies from dense tuple arrays or Maps, contextual/inferred construction. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/146-maps.md) · [evidence](./maps.md) · [map_values](../benchmarks/runtime/workloads/map_values.tsr) |
| `MapConstructor.prototype` | pending | missing |  |
| `ReadonlyMap.[Symbol.iterator]` | pending | missing |  |
| `ReadonlyMap.entries` | pending | missing |  |
| `ReadonlyMap.forEach` | pending | missing |  |
| `ReadonlyMap.get` | pending | missing |  |
| `ReadonlyMap.has` | pending | missing |  |
| `ReadonlyMap.keys` | pending | missing |  |
| `ReadonlyMap.size` | pending | missing |  |
| `ReadonlyMap.values` | pending | missing |  |
| `ReadonlySet.[Symbol.iterator]` | pending | missing |  |
| `ReadonlySet.difference` | pending | missing | esnext declaration; availability unverified |
| `ReadonlySet.entries` | pending | missing |  |
| `ReadonlySet.forEach` | pending | missing |  |
| `ReadonlySet.has` | pending | missing |  |
| `ReadonlySet.intersection` | pending | missing | esnext declaration; availability unverified |
| `ReadonlySet.isDisjointFrom` | pending | missing | esnext declaration; availability unverified |
| `ReadonlySet.isSubsetOf` | pending | missing | esnext declaration; availability unverified |
| `ReadonlySet.isSupersetOf` | pending | missing | esnext declaration; availability unverified |
| `ReadonlySet.keys` | pending | missing |  |
| `ReadonlySet.size` | pending | missing |  |
| `ReadonlySet.symmetricDifference` | pending | missing | esnext declaration; availability unverified |
| `ReadonlySet.union` | pending | missing | esnext declaration; availability unverified |
| `ReadonlySet.values` | pending | missing |  |
| `Set.[Symbol.iterator]` | native variant | paired | Borrowed for-of iteration in insertion order. Scalar keys copy; owned keys are readonly borrows. Receiver mutation/transfer is rejected during iteration. No callable Symbol.iterator property. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `Set.[Symbol.toStringTag]` | pending | missing |  |
| `Set.add` | native variant | paired | Adds or ignores a SameValueZero key, preserves insertion order and transfers owned keys. Returns void, not a chainable receiver. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `Set.clear` | implemented | paired | Removes all keys and returns void; receiver mutates. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `Set.delete` | implemented | paired | Removes a matching key and reports whether present; argument is borrowed and receiver mutates. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `Set.difference` | native variant | paired | Explicit independent result copies with borrowed inputs. Joined key types for union/symmetricDifference; receiver type for intersection/difference. Node insertion order in ordinary key domain; all exact receiver keys retained under mixed numeric precision overlap. No set-like records. Matched verification passes untimed; collection measurements pending.; esnext declaration; availability unverified; [evidence](./milestones/153-set-algebra.md) · [evidence](./sets.md) · [set_algebra](../benchmarks/runtime/workloads/set_algebra.tsr) |
| `Set.entries` | native variant | paired | Independent typed key/key tuple array, explicitly copying owned keys into both fields. Insertion order, zero arguments, not a live iterator. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/148-set-entries.md) · [evidence](./sets.md) · [set_entries](../benchmarks/runtime/workloads/set_entries.tsr) |
| `Set.forEach` | native variant | paired | Reusable callbacks receive value, same key and readonly Set in insertion order. Scalars copy; owned inputs borrow. No receiver mutation/transfer. Default/rest rules and checked errors are supported; returns void and discards callback results. No thisArg. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/149-set-for-each.md) · [evidence](./sets.md) · [set_for_each](../benchmarks/runtime/workloads/set_for_each.tsr) |
| `Set.has` | implemented | paired | Borrowed key membership using SameValueZero; NaN and signed zero follow numeric key rules. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `Set.intersection` | native variant | paired | Explicit independent result copies with borrowed inputs. Joined key types for union/symmetricDifference; receiver type for intersection/difference. Node insertion order in ordinary key domain; all exact receiver keys retained under mixed numeric precision overlap. No set-like records. Matched verification passes untimed; collection measurements pending.; esnext declaration; availability unverified; [evidence](./milestones/153-set-algebra.md) · [evidence](./sets.md) · [set_algebra](../benchmarks/runtime/workloads/set_algebra.tsr) |
| `Set.isDisjointFrom` | native variant | paired | Borrows both typed Sets and keys; no mutation or clones. Different scalar/union key types use existing SameValueZero/numeric and enum identity rules. Set-like records unsupported. Native precision rules are documented. Final matched verification passes untimed; collection measurements pending.; esnext declaration; availability unverified; [evidence](./milestones/152-set-relations.md) · [evidence](./sets.md) · [set_relations](../benchmarks/runtime/workloads/set_relations.tsr) |
| `Set.isSubsetOf` | native variant | paired | Borrows both typed Sets and keys; no mutation or clones. Different scalar/union key types use existing SameValueZero/numeric and enum identity rules. Set-like records unsupported. Native precision rules are documented. Final matched verification passes untimed; collection measurements pending.; esnext declaration; availability unverified; [evidence](./milestones/152-set-relations.md) · [evidence](./sets.md) · [set_relations](../benchmarks/runtime/workloads/set_relations.tsr) |
| `Set.isSupersetOf` | native variant | paired | Borrows both typed Sets and keys; no mutation or clones. Different scalar/union key types use existing SameValueZero/numeric and enum identity rules. Set-like records unsupported. Native precision rules are documented. Final matched verification passes untimed; collection measurements pending.; esnext declaration; availability unverified; [evidence](./milestones/152-set-relations.md) · [evidence](./sets.md) · [set_relations](../benchmarks/runtime/workloads/set_relations.tsr) |
| `Set.keys` | native variant | paired | Independent insertion-ordered array snapshot identical to values(), not a live iterator. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `Set.size` | implemented | paired | Readonly integer count of distinct keys. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `Set.symmetricDifference` | native variant | paired | Explicit independent result copies with borrowed inputs. Joined key types for union/symmetricDifference; receiver type for intersection/difference. Node insertion order in ordinary key domain; all exact receiver keys retained under mixed numeric precision overlap. No set-like records. Matched verification passes untimed; collection measurements pending.; esnext declaration; availability unverified; [evidence](./milestones/153-set-algebra.md) · [evidence](./sets.md) · [set_algebra](../benchmarks/runtime/workloads/set_algebra.tsr) |
| `Set.union` | native variant | paired | Explicit independent result copies with borrowed inputs. Joined key types for union/symmetricDifference; receiver type for intersection/difference. Node insertion order in ordinary key domain; all exact receiver keys retained under mixed numeric precision overlap. No set-like records. Matched verification passes untimed; collection measurements pending.; esnext declaration; availability unverified; [evidence](./milestones/153-set-algebra.md) · [evidence](./sets.md) · [set_algebra](../benchmarks/runtime/workloads/set_algebra.tsr) |
| `Set.values` | native variant | paired | Independent insertion-ordered array snapshot with explicit key copies, not a live iterator. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `SetConstructor.[Symbol.species]` | pending | missing |  |
| `SetConstructor.new()` | native variant | paired | Typed scalar/enum/union keys; empty context and dense array/Set/string sources. Construction explicitly copies keys; identity keys are unsupported. Matched verification passes untimed; collection measurements remain pending.; [evidence](./milestones/145-sets.md) · [evidence](./sets.md) · [set_values](../benchmarks/runtime/workloads/set_values.tsr) |
| `SetConstructor.prototype` | pending | missing |  |
| `WeakMap.[Symbol.toStringTag]` | pending | missing |  |
| `WeakMap.delete` | pending | missing |  |
| `WeakMap.get` | pending | missing |  |
| `WeakMap.has` | pending | missing |  |
| `WeakMap.set` | pending | missing |  |
| `WeakMapConstructor.new()` | pending | missing |  |
| `WeakMapConstructor.prototype` | pending | missing |  |
| `WeakRef.[Symbol.toStringTag]` | pending | missing |  |
| `WeakRef.deref` | pending | missing |  |
| `WeakRefConstructor.new()` | pending | missing |  |
| `WeakRefConstructor.prototype` | pending | missing |  |
| `WeakSet.[Symbol.toStringTag]` | pending | missing |  |
| `WeakSet.add` | pending | missing |  |
| `WeakSet.delete` | pending | missing |  |
| `WeakSet.has` | pending | missing |  |
| `WeakSetConstructor.new()` | pending | missing |  |
| `WeakSetConstructor.prototype` | pending | missing |  |

## Control flow

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `control.assignmentExpression` | pending | missing | Umbrella remains pending. Gate68 accepts all int/Number compound values, Gate67 ordinary int/Number/boolean assignment values, Gate66 strict-boolean logical assignment values and Gate64 numeric updates. Owned ordinary/compound values, coercion and destructuring remain separate capabilities.; [evidence](./milestones/63-compound-assignment.md) |
| `control.awaitUsing` | pending | missing |  |
| `control.break` | implemented | compound | Dedicated per-construct timing still missing.; [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `control.case` | native variant | paired | case clauses end with break, return or continue, or group with the next label. Lowered to an if chain; the value is evaluated once. Running from a clause body into the next clause and a break nested inside a clause statement are rejected. One matched scenario is registered and verified untimed.; [evidence](./milestones/135-switch-statements.md) · [evidence](./switch.md) · [switch_statements](../benchmarks/runtime/workloads/switch_statements.tsr) |
| `control.catch` | native variant | paired | catch with or without a name. The caught value has the inferred alternatives of the body and needs narrowing; an unresolved callable contract is rejected. One matched scenario is registered and verified untimed.; [evidence](./milestones/142-exception-handling.md) · [evidence](./exception-handling.md) · [exception_completion](../benchmarks/runtime/workloads/exception_completion.tsr) |
| `control.commaExpression` | implemented | paired group | Ordered discarded prefixes and final values, source ownership/provenance, guard outcomes and native temporary disposal; existing assignment/destructuring for headers preserved. Compiler 22, editor 1, harness 388 and matched five-case verification pass untimed. Language milestone measurements pending. Gate162 completes named receiver guards and selected sentinel/typeof operands with RHS mutation invalidation and indexed parent loans; matched cases verified untimed.; [evidence](./milestones/161-comma-expressions.md) · [evidence](./comma-expressions.md) · [evidence](./milestones/162-comma-contexts.md) · [evidence](./comma-contexts.md) · [comma_expressions](../benchmarks/runtime/workloads/comma_expressions.tsr) · [comma_contexts](../benchmarks/runtime/workloads/comma_contexts.tsr) |
| `control.conditionalExpression` | implemented | paired group | Strict boolean predicates select one lazy same-type branch across compatible existing scalar, owned and same-origin callable values. Contextual literals, conservative ownership joins, all possible borrowed roots, invocation/capture/escape restrictions and selected temporary lifetimes remain enforced. Ten instrumented comparisons retain complete payloads and call markers; fresh owned construction and retention are timed, not existing-owner move cost. Truthiness, general heterogeneous unions and new callable representations remain pending. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/69-conditional-expressions.md) · [evidence](./milestones/84-nullable-type-foundation.md) · [conditional_int_value](../benchmarks/runtime/workloads/conditional_int_value.tsr) · [conditional_number_value](../benchmarks/runtime/workloads/conditional_number_value.tsr) · [conditional_boolean_value](../benchmarks/runtime/workloads/conditional_boolean_value.tsr) · [conditional_string_ascii](../benchmarks/runtime/workloads/conditional_string_ascii.tsr) · [conditional_string_unicode](../benchmarks/runtime/workloads/conditional_string_unicode.tsr) · [conditional_array_int](../benchmarks/runtime/workloads/conditional_array_int.tsr) · [conditional_array_number](../benchmarks/runtime/workloads/conditional_array_number.tsr) · [conditional_array_boolean](../benchmarks/runtime/workloads/conditional_array_boolean.tsr) · [conditional_array_string](../benchmarks/runtime/workloads/conditional_array_string.tsr) · [conditional_record_value](../benchmarks/runtime/workloads/conditional_record_value.tsr) · [nullable_int_null](../benchmarks/runtime/workloads/nullable_int_null.tsr) · [nullable_boolean_undefined](../benchmarks/runtime/workloads/nullable_boolean_undefined.tsr) · [nullable_number_null](../benchmarks/runtime/workloads/nullable_number_null.tsr) · [nullable_string_undefined](../benchmarks/runtime/workloads/nullable_string_undefined.tsr) · [nullable_array_null](../benchmarks/runtime/workloads/nullable_array_null.tsr) · [nullable_record_undefined](../benchmarks/runtime/workloads/nullable_record_undefined.tsr) |
| `control.continue` | implemented | compound | Dedicated per-construct timing still missing.; [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `control.default` | native variant | paired | default runs after every case has failed, in any position. Lowered to an if chain; the value is evaluated once. Running from a clause body into the next clause and a break nested inside a clause statement are rejected. One matched scenario is registered and verified untimed.; [evidence](./milestones/135-switch-statements.md) · [evidence](./switch.md) · [switch_statements](../benchmarks/runtime/workloads/switch_statements.tsr) |
| `control.doWhile` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains. Gate134 accepts a single statement without braces as the body; braceless_bodies is registered and verified untimed.; [evidence](./milestones/38-control-flow-loops.md) · [evidence](./milestones/134-statement-bodies.md) · [do_while_control](../benchmarks/runtime/workloads/do_while_control.tsr) · [braceless_bodies](../benchmarks/runtime/workloads/braceless_bodies.tsr) |
| `control.else` | native variant | compound | Conditions require boolean; JS truthiness is not implemented. Dedicated per-construct timing missing. Gate134 accepts a single statement without braces as the body; braceless_bodies is registered and verified untimed.; [evidence](./milestones/134-statement-bodies.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) · [braceless_bodies](../benchmarks/runtime/workloads/braceless_bodies.tsr) |
| `control.elseIf` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/38-control-flow-loops.md) · [else_if_control](../benchmarks/runtime/workloads/else_if_control.tsr) |
| `control.fallthrough` | pending | missing |  |
| `control.finally` | native variant | paired | finally runs before every pending exit, including uncaught errors, and can replace it. One matched scenario is registered and verified untimed.; [evidence](./milestones/142-exception-handling.md) · [evidence](./exception-handling.md) · [exception_completion](../benchmarks/runtime/workloads/exception_completion.tsr) |
| `control.forAwait` | pending | missing |  |
| `control.forClassic` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains. Gate134 accepts a single statement without braces as the body; braceless_bodies is registered and verified untimed.; [evidence](./milestones/38-control-flow-loops.md) · [evidence](./milestones/134-statement-bodies.md) · [classic_for_control](../benchmarks/runtime/workloads/classic_for_control.tsr) · [braceless_bodies](../benchmarks/runtime/workloads/braceless_bodies.tsr) |
| `control.forIn` | native variant | paired | Snapshot enumeration through Object.keys with string keys borrowed under ordinary loop access. Named const/let bindings only; source retained, break/continue supported. Matched scenario verified untimed; measurements pending.; [evidence](./milestones/139-in-operator.md) · [evidence](./in-operator.md) · [in_operator](../benchmarks/runtime/workloads/in_operator.tsr) |
| `control.forOfArray` | native variant | compound | Borrowed dense-array iteration; general iterator protocol pending. Gate134 accepts a single statement without braces as the body; braceless_bodies is registered and verified untimed.; [evidence](./milestones/134-statement-bodies.md) · [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [braceless_bodies](../benchmarks/runtime/workloads/braceless_bodies.tsr) |
| `control.forOfIterable` | pending | missing |  |
| `control.forOfMap` | native variant | paired | Borrowed stored key/value tuples and patterns in insertion order. No implicit copies, including noncloneable values; receiver loans end after the loop. Verification is untimed; live iterator objects remain unsupported.; [evidence](./milestones/154-borrowed-map-iteration.md) · [evidence](./maps.md) · [map_iteration](../benchmarks/runtime/workloads/map_iteration.tsr) |
| `control.forOfString` | native variant | paired | Readonly Unicode scalar string traversal with once evaluation, lexical source loans and normal completion. Each character string is constructed separately under the current static string ABI; no source clone or array snapshot. Well-formed text traversal agrees with JS; length remains UTF-8 bytes. Verification untimed.; [evidence](./milestones/155-string-iteration.md) · [evidence](./string-iteration.md) · [string_iteration](../benchmarks/runtime/workloads/string_iteration.tsr) |
| `control.if` | native variant | compound | Conditions require boolean; JS truthiness is not implemented. Dedicated per-construct timing missing. Gate134 accepts a single statement without braces as the body; braceless_bodies is registered and verified untimed.; [evidence](./milestones/134-statement-bodies.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) · [braceless_bodies](../benchmarks/runtime/workloads/braceless_bodies.tsr) |
| `control.labelledBreak` | native variant | paired | Labeled exits from enclosing loops and non-declaration statements; aliases, target scope, availability and definite-assignment joins. One complete matched scenario verified untimed; measurements pending.; [evidence](./milestones/140-labeled-control.md) · [evidence](./labeled-control.md) · [labeled_control](../benchmarks/runtime/workloads/labeled_control.tsr) |
| `control.labelledContinue` | native variant | paired | Labeled continuation of an enclosing loop, with the target update or post-test condition and targeted ownership/assignment checks. One complete matched scenario verified untimed; measurements pending.; [evidence](./milestones/140-labeled-control.md) · [evidence](./labeled-control.md) · [labeled_control](../benchmarks/runtime/workloads/labeled_control.tsr) |
| `control.logicalAssignment` | pending | missing | Umbrella remains pending. Strict-boolean &&= and \|\|= expressions are accepted in Gate66; nullish assignment and coercive truthiness remain separate capabilities.; [evidence](./milestones/63-compound-assignment.md) |
| `control.looseEquality` | pending | missing |  |
| `control.looseInequality` | pending | missing |  |
| `control.nullishAssignment` | native variant | paired group | Lazy once-selected nullish commit in the strict canonical typed domain. Bounds precede fallback effects; saved destinations are reacquired before commit. RHS absent facts and stable postcommit views are checked. Copy results are snapshots; owned results are lexical readonly aliases with explicit clone for independent ownership, including chains and conditional owners. Four retained compiler groups plus scoped supplements, one editor group and six O0/O3 language examples accepted. matched comparisons are measured at checkpoint 96; no coercion or general payload unions. Optional chaining is accepted at Gate96.; [evidence](./milestones/95-nullish-assignment.md) · [evidence](./nullish-assignment.md) · [evidence](../benchmarks/runtime/diagnostics/2026-10-03T21-49-17-000Z-nullish-assignment-implementation.json) · [nullish_assign_int_value](../benchmarks/runtime/workloads/nullish_assign_int_value.tsr) · [nullish_assign_number_value](../benchmarks/runtime/workloads/nullish_assign_number_value.tsr) · [nullish_assign_boolean_value](../benchmarks/runtime/workloads/nullish_assign_boolean_value.tsr) · [nullish_assign_string_value](../benchmarks/runtime/workloads/nullish_assign_string_value.tsr) · [nullish_assign_array_value](../benchmarks/runtime/workloads/nullish_assign_array_value.tsr) · [nullish_assign_record_value](../benchmarks/runtime/workloads/nullish_assign_record_value.tsr) · [nullish_assign_nullable_int_value](../benchmarks/runtime/workloads/nullish_assign_nullable_int_value.tsr) · [nullish_assign_nullable_string_value](../benchmarks/runtime/workloads/nullish_assign_nullable_string_value.tsr) |
| `control.return` | implemented | compound | Dedicated per-construct timing still missing.; [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `control.switch` | native variant | paired | switch with strict-equality cases in source order, grouped labels and default in any position. Lowered to an if chain; the value is evaluated once. Running from a clause body into the next clause and a break nested inside a clause statement are rejected. One matched scenario is registered and verified untimed.; [evidence](./milestones/135-switch-statements.md) · [evidence](./switch.md) · [switch_statements](../benchmarks/runtime/workloads/switch_statements.tsr) |
| `control.throw` | native variant | paired | throw of any checked value; the value must start on the same line. Arithmetic and bounds failures throw structured runtime errors with source coordinates. One matched scenario is registered and verified untimed.; [evidence](./milestones/142-exception-handling.md) · [evidence](./exception-handling.md) · [exception_completion](../benchmarks/runtime/workloads/exception_completion.tsr) |
| `control.try` | native variant | paired | try with catch, finally or both. Normal, return, throw, break and continue completions pass through finally; an exit from finally replaces the pending one. One matched scenario is registered and verified untimed.; [evidence](./milestones/142-exception-handling.md) · [evidence](./exception-handling.md) · [exception_completion](../benchmarks/runtime/workloads/exception_completion.tsr) |
| `control.using` | pending | missing |  |
| `control.void` | pending | missing |  |
| `control.while` | native variant | compound | Conditions require boolean; JS truthiness is not implemented. Dedicated per-construct timing missing. Gate134 accepts a single statement without braces as the body; braceless_bodies is registered and verified untimed.; [evidence](./milestones/134-statement-bodies.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) · [braceless_bodies](../benchmarks/runtime/workloads/braceless_bodies.tsr) |
| `control.with` | pending | missing |  |

## Dates

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `Date.[Symbol.toPrimitive]` | pending | missing |  |
| `Date.getDate` | pending | missing |  |
| `Date.getDay` | pending | missing |  |
| `Date.getFullYear` | pending | missing |  |
| `Date.getHours` | pending | missing |  |
| `Date.getMilliseconds` | pending | missing |  |
| `Date.getMinutes` | pending | missing |  |
| `Date.getMonth` | pending | missing |  |
| `Date.getSeconds` | pending | missing |  |
| `Date.getTime` | pending | missing |  |
| `Date.getTimezoneOffset` | pending | missing |  |
| `Date.getUTCDate` | pending | missing |  |
| `Date.getUTCDay` | pending | missing |  |
| `Date.getUTCFullYear` | pending | missing |  |
| `Date.getUTCHours` | pending | missing |  |
| `Date.getUTCMilliseconds` | pending | missing |  |
| `Date.getUTCMinutes` | pending | missing |  |
| `Date.getUTCMonth` | pending | missing |  |
| `Date.getUTCSeconds` | pending | missing |  |
| `Date.setDate` | pending | missing |  |
| `Date.setFullYear` | pending | missing |  |
| `Date.setHours` | pending | missing |  |
| `Date.setMilliseconds` | pending | missing |  |
| `Date.setMinutes` | pending | missing |  |
| `Date.setMonth` | pending | missing |  |
| `Date.setSeconds` | pending | missing |  |
| `Date.setTime` | pending | missing |  |
| `Date.setUTCDate` | pending | missing |  |
| `Date.setUTCFullYear` | pending | missing |  |
| `Date.setUTCHours` | pending | missing |  |
| `Date.setUTCMilliseconds` | pending | missing |  |
| `Date.setUTCMinutes` | pending | missing |  |
| `Date.setUTCMonth` | pending | missing |  |
| `Date.setUTCSeconds` | pending | missing |  |
| `Date.toDateString` | pending | missing |  |
| `Date.toISOString` | pending | missing |  |
| `Date.toJSON` | pending | missing |  |
| `Date.toLocaleDateString` | pending | missing |  |
| `Date.toLocaleString` | pending | missing |  |
| `Date.toLocaleTimeString` | pending | missing |  |
| `Date.toString` | pending | missing |  |
| `Date.toTimeString` | pending | missing |  |
| `Date.toUTCString` | pending | missing |  |
| `Date.valueOf` | pending | missing |  |
| `DateConstructor.call()` | pending | missing |  |
| `DateConstructor.new()` | pending | missing |  |
| `DateConstructor.now` | pending | missing |  |
| `DateConstructor.parse` | pending | missing |  |
| `DateConstructor.prototype` | pending | missing |  |
| `DateConstructor.UTC` | pending | missing |  |

## Declarations and syntax

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `syntax.arrayDestructuring` | native variant | paired | Array and tuple patterns in declarations, parameters/callbacks and for...of, including nesting, renamed record fields, skipped array positions and undefined-only lazy defaults. Names follow source access; array elements and readonly sources lend, owning record/tuple positions can move. Named rest bindings explicitly copy remaining positions/fields; ordinary selections introduce no implicit clone. Nested array-rest array/object targets are supported; classic-for initializer patterns remain unsupported. Matched comparisons pass untimed; measurements remain pending. Gate131 accepts pattern declarations in classic-for initializers; for_pattern_initializers is registered and verified untimed. Gate133 accepts pattern assignment to existing places and shorthand record properties; destructuring_assignment is registered and verified untimed.; [evidence](./milestones/124-destructuring.md) · [evidence](./destructuring.md) · [evidence](./milestones/125-parameter-patterns-and-indexed-borrows.md) · [evidence](./milestones/127-destructuring-defaults.md) · [evidence](./milestones/129-rest-and-object-spread.md) · [evidence](./milestones/130-nested-rest-patterns.md) · [evidence](./milestones/131-for-pattern-initializers.md) · [evidence](./milestones/133-destructuring-assignment.md) · [destructuring_patterns](../benchmarks/runtime/workloads/destructuring_patterns.tsr) · [destructuring_defaults](../benchmarks/runtime/workloads/destructuring_defaults.tsr) · [destructuring_rest](../benchmarks/runtime/workloads/destructuring_rest.tsr) · [nested_rest_patterns](../benchmarks/runtime/workloads/nested_rest_patterns.tsr) · [for_pattern_initializers](../benchmarks/runtime/workloads/for_pattern_initializers.tsr) · [destructuring_assignment](../benchmarks/runtime/workloads/destructuring_assignment.tsr) |
| `syntax.const` | native variant | paired | Declarations with one or several declarators, each with an initializer and an optional annotation, including pattern declarators. No uninitialized declarations. declaration_lists is registered and verified untimed; measurements pending.; [evidence](./milestones/132-declaration-lists.md) · [evidence](./declaration-lists.md) · [declaration_lists](../benchmarks/runtime/workloads/declaration_lists.tsr) |
| `syntax.defaultParameters` | pending | missing |  |
| `syntax.destructuringDefaults` | native variant | paired | Undefined-only lazy defaults in declaration, parameter/callback and for...of patterns. Null remains present. Earlier names are visible in later defaults; parameter initialization is left to right. Missing fields/positions select defaults. Fresh borrowed fallback owners persist locally without cloning; lexical source loans remain. Five matched cases verified untimed; value-family timing remains pending.; [evidence](./milestones/127-destructuring-defaults.md) · [evidence](./destructuring.md) · [destructuring_defaults](../benchmarks/runtime/workloads/destructuring_defaults.tsr) |
| `syntax.destructuringRest` | native variant | paired | Named array/tuple/record rest creates independent remaining values, preserving tuple/field types and constructor order. Declarations, nested patterns, parameters/arrows and for...of; rest last, no default or trailing comma. Earlier selected moves allowed; transferred remaining fields rejected. Nested owners copy independently under native clone semantics. Five matched cases pass untimed; measurements remain pending. Array rest may target nested array/object patterns, including defaults, parameters/arrows, tuples and for...of. Record rest requires an identifier; nesting bound32 retained.; [evidence](./milestones/129-rest-and-object-spread.md) · [evidence](./destructuring.md) · [evidence](./milestones/130-nested-rest-patterns.md) · [destructuring_rest](../benchmarks/runtime/workloads/destructuring_rest.tsr) · [nested_rest_patterns](../benchmarks/runtime/workloads/nested_rest_patterns.tsr) |
| `syntax.dynamicImport` | pending | missing |  |
| `syntax.export` | pending | missing |  |
| `syntax.hoisting` | pending | missing |  |
| `syntax.import` | pending | missing |  |
| `syntax.let` | native variant | paired | Declarations with one or several declarators and an optional annotation, including pattern declarators. A declarator with an annotation can omit its initializer; every use must follow an assignment on every path, and a type that includes undefined starts as undefined. declaration_lists and deferred_initialization are registered and verified untimed; measurements pending.; [evidence](./milestones/132-declaration-lists.md) · [evidence](./declaration-lists.md) · [evidence](./milestones/138-deferred-initialization.md) · [evidence](./deferred-initialization.md) · [declaration_lists](../benchmarks/runtime/workloads/declaration_lists.tsr) · [deferred_initialization](../benchmarks/runtime/workloads/deferred_initialization.tsr) |
| `syntax.lexicalScope` | native variant | compile-only | Implemented strict subset; no standalone compile-time comparison for this item. |
| `syntax.numericSeparators` | implemented | paired group | Decimal whole/fraction/exponent and binary/octal/hex separators, syntax diagnostics, exact int defaults and contextual f64 conversion. Original spelling/ranges and editor behavior preserved. Focused checks and matched workload verified untimed; no isolated scanner timing.; [evidence](./milestones/160-numeric-literals.md) · [evidence](./numeric-literals.md) · [numeric_literals](../benchmarks/runtime/workloads/numeric_literals.tsr) |
| `syntax.objectDestructuring` | native variant | paired | Record patterns in declarations, parameters/callbacks and for...of, including nesting, renamed record fields, skipped array positions and undefined-only lazy defaults. Names follow source access; array elements and readonly sources lend, owning record/tuple positions can move. Named rest bindings explicitly copy remaining positions/fields; ordinary selections introduce no implicit clone. Nested array-rest array/object targets are supported; classic-for initializer patterns remain unsupported. Matched comparisons pass untimed; measurements remain pending. Gate131 accepts pattern declarations in classic-for initializers; for_pattern_initializers is registered and verified untimed. Gate133 accepts pattern assignment to existing places and shorthand record properties; destructuring_assignment is registered and verified untimed.; [evidence](./milestones/124-destructuring.md) · [evidence](./destructuring.md) · [evidence](./milestones/125-parameter-patterns-and-indexed-borrows.md) · [evidence](./milestones/127-destructuring-defaults.md) · [evidence](./milestones/129-rest-and-object-spread.md) · [evidence](./milestones/130-nested-rest-patterns.md) · [evidence](./milestones/131-for-pattern-initializers.md) · [evidence](./milestones/133-destructuring-assignment.md) · [destructuring_patterns](../benchmarks/runtime/workloads/destructuring_patterns.tsr) · [destructuring_defaults](../benchmarks/runtime/workloads/destructuring_defaults.tsr) · [destructuring_rest](../benchmarks/runtime/workloads/destructuring_rest.tsr) · [nested_rest_patterns](../benchmarks/runtime/workloads/nested_rest_patterns.tsr) · [for_pattern_initializers](../benchmarks/runtime/workloads/for_pattern_initializers.tsr) · [destructuring_assignment](../benchmarks/runtime/workloads/destructuring_assignment.tsr) |
| `syntax.optionalCatchBinding` | pending | missing |  |
| `syntax.regexLiteral` | pending | missing |  |
| `syntax.restParameters` | implemented | paired group | Focused Gate99 accepts fresh homogeneous owning rest packs across current data and callable kinds, generic/imported contexts, abstract/nested adapters and real array callback tuples. Thirteen matched scenarios are registered; full regression and measurements remain queued for checkpoint101. Call spread and heterogeneous tuples remain separate.; [evidence](./milestones/99-rest-parameters.md) · [evidence](./rest-parameters.md) · [rest_parameter_int_pack](../benchmarks/runtime/workloads/rest_parameter_int_pack.tsr) · [rest_parameter_number_pack](../benchmarks/runtime/workloads/rest_parameter_number_pack.tsr) · [rest_parameter_boolean_pack](../benchmarks/runtime/workloads/rest_parameter_boolean_pack.tsr) · [rest_parameter_string_pack](../benchmarks/runtime/workloads/rest_parameter_string_pack.tsr) · [rest_parameter_nested_pack](../benchmarks/runtime/workloads/rest_parameter_nested_pack.tsr) · [rest_parameter_record_pack](../benchmarks/runtime/workloads/rest_parameter_record_pack.tsr) · [rest_parameter_null_pack](../benchmarks/runtime/workloads/rest_parameter_null_pack.tsr) · [rest_parameter_undefined_pack](../benchmarks/runtime/workloads/rest_parameter_undefined_pack.tsr) · [rest_parameter_default_prefix](../benchmarks/runtime/workloads/rest_parameter_default_prefix.tsr) · [rest_parameter_read_callback](../benchmarks/runtime/workloads/rest_parameter_read_callback.tsr) · [rest_parameter_mutate_callback](../benchmarks/runtime/workloads/rest_parameter_mutate_callback.tsr) · [rest_parameter_owned_once_callback](../benchmarks/runtime/workloads/rest_parameter_owned_once_callback.tsr) · [rest_parameter_owned_callable_pack](../benchmarks/runtime/workloads/rest_parameter_owned_callable_pack.tsr) |
| `syntax.spreadArguments` | implemented | paired group | Gate 158 extends checked argument expansion to strings, Sets and Maps; source order, generic inference and ownership remain checked. Dynamic count mismatches throw recoverable call-arity errors. Matched iterable workload verified untimed; collection milestone measurements remain pending. Historical call-spread measurements and evidence remain intact.; [evidence](./milestones/100-call-spread.md) · [evidence](./call-spread.md) · [evidence](./milestones/158-iterable-call-spread.md) · [call_spread_fixed_literal_int](../benchmarks/runtime/workloads/call_spread_fixed_literal_int.tsr) · [call_spread_dynamic_int](../benchmarks/runtime/workloads/call_spread_dynamic_int.tsr) · [call_spread_multiple_mixed_int](../benchmarks/runtime/workloads/call_spread_multiple_mixed_int.tsr) · [call_spread_math_number](../benchmarks/runtime/workloads/call_spread_math_number.tsr) · [call_spread_boolean_pack](../benchmarks/runtime/workloads/call_spread_boolean_pack.tsr) · [call_spread_string_owned](../benchmarks/runtime/workloads/call_spread_string_owned.tsr) · [call_spread_nested_owned](../benchmarks/runtime/workloads/call_spread_nested_owned.tsr) · [call_spread_record_owned](../benchmarks/runtime/workloads/call_spread_record_owned.tsr) · [call_spread_nullish_packs](../benchmarks/runtime/workloads/call_spread_nullish_packs.tsr) · [call_spread_borrowed_string_fixed](../benchmarks/runtime/workloads/call_spread_borrowed_string_fixed.tsr) · [call_spread_optional_default_counts](../benchmarks/runtime/workloads/call_spread_optional_default_counts.tsr) · [call_spread_mutate_callback](../benchmarks/runtime/workloads/call_spread_mutate_callback.tsr) · [call_spread_owned_once_optional](../benchmarks/runtime/workloads/call_spread_owned_once_optional.tsr) · [iterable_call_spread](../benchmarks/runtime/workloads/iterable_call_spread.tsr) |
| `syntax.spreadArray` | implemented | paired group | Ordered array, string, Set and Map expansion with scalar copies and owning transfers. String characters are constructed Unicode scalar values; owning readonly collection inputs cannot transfer. Contextual empty constructors and compatible union output supported. Base ten pairs and new collection scenario verified untimed. Existing checkpoint101 evidence retained; new collection measurements pending. Sparse slots and arbitrary iterator objects unsupported.; [evidence](./milestones/101-array-literal-spread.md) · [evidence](./array-literal-spread.md) · [evidence](./milestones/157-iterable-array-spread.md) · [array_literal_spread_dynamic_int](../benchmarks/runtime/workloads/array_literal_spread_dynamic_int.tsr) · [array_literal_spread_multiple_mixed_int](../benchmarks/runtime/workloads/array_literal_spread_multiple_mixed_int.tsr) · [array_literal_spread_boolean_pack](../benchmarks/runtime/workloads/array_literal_spread_boolean_pack.tsr) · [array_literal_spread_string_owned](../benchmarks/runtime/workloads/array_literal_spread_string_owned.tsr) · [array_literal_spread_nested_owned](../benchmarks/runtime/workloads/array_literal_spread_nested_owned.tsr) · [array_literal_spread_record_owned](../benchmarks/runtime/workloads/array_literal_spread_record_owned.tsr) · [array_literal_spread_nullish_packs](../benchmarks/runtime/workloads/array_literal_spread_nullish_packs.tsr) · [array_literal_spread_number](../benchmarks/runtime/workloads/array_literal_spread_number.tsr) · [array_literal_spread_callable](../benchmarks/runtime/workloads/array_literal_spread_callable.tsr) · [array_literal_spread_source_order](../benchmarks/runtime/workloads/array_literal_spread_source_order.tsr) · [iterable_array_spread](../benchmarks/runtime/workloads/iterable_array_spread.tsr) |
| `syntax.spreadObject` | native variant | paired | Fixed-record sources copy explicitly, with ordinary fields and multiple spreads. Once left-to-right evaluation, final-value overrides, first-insertion key order, inferred or annotated shapes and supported union widening. Sources remain available; partial sources and nonrecord/nullable/union receivers are unsupported. Native nested owners copy independently; JavaScript shallow copies share nested references. Five matched cases pass untimed; measurements remain pending.; [evidence](./milestones/129-rest-and-object-spread.md) · [evidence](./destructuring.md) · [destructuring_rest](../benchmarks/runtime/workloads/destructuring_rest.tsr) |
| `syntax.standaloneBlock` | implemented | compile-only | Lexical scope and cleanup guaranteed by native correctness checks at optimization levels 0 and 3; no meaningful isolated block ops/s. Full compiler acceptance passes.; [evidence](./milestones/38-control-flow-loops.md) |
| `syntax.templateStrings` | implemented | paired group | Ordinary untagged templates with nested/multiline chunks and strict primitive interpolation. Native Rust Display and UTF-8; no object coercion. Focused compiler/editor/native examples pass; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/78-template-literals.md) · [template_primitive_ascii](../benchmarks/runtime/workloads/template_primitive_ascii.tsr) · [template_string_unicode](../benchmarks/runtime/workloads/template_string_unicode.tsr) |
| `syntax.temporalDeadZone` | pending | missing |  |
| `syntax.topLevelAwait` | pending | missing |  |
| `syntax.var` | pending | missing |  |

## Decorators

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `decorator.accessor` | pending | missing |  |
| `decorator.addInitializer` | pending | missing |  |
| `decorator.class` | pending | missing |  |
| `decorator.context` | pending | missing |  |
| `decorator.evaluationOrder` | pending | missing |  |
| `decorator.field` | pending | missing |  |
| `decorator.getter` | pending | missing |  |
| `decorator.legacyClass` | pending | missing |  |
| `decorator.legacyMetadata` | pending | missing |  |
| `decorator.legacyMethod` | pending | missing |  |
| `decorator.legacyParameter` | pending | missing |  |
| `decorator.legacyProperty` | pending | missing |  |
| `decorator.metadata` | pending | missing |  |
| `decorator.method` | pending | missing |  |
| `decorator.replacement` | pending | missing |  |
| `decorator.setter` | pending | missing |  |
| `decorator.stacking` | pending | missing |  |

## Errors

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `AggregateError.errors` | pending | missing |  |
| `Error.cause` | pending | missing |  |
| `Error.message` | pending | missing |  |
| `Error.name` | pending | missing |  |
| `Error.stack` | pending | missing |  |
| `ErrorConstructor.call()` | pending | missing |  |
| `ErrorConstructor.isError` | pending | missing | esnext declaration; availability unverified |
| `ErrorConstructor.new()` | pending | missing |  |
| `ErrorConstructor.prototype` | pending | missing |  |
| `SuppressedError.error` | pending | missing | esnext declaration; availability unverified |
| `SuppressedError.suppressed` | pending | missing | esnext declaration; availability unverified |

## Functions and classes

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `class.abstract` | pending | missing |  |
| `class.constructor` | pending | missing |  |
| `class.declaration` | pending | missing |  |
| `class.extends` | pending | missing |  |
| `class.field` | pending | missing |  |
| `class.getter` | pending | missing |  |
| `class.interfaceImplementation` | pending | missing |  |
| `class.method` | pending | missing |  |
| `class.private` | pending | missing |  |
| `class.setter` | pending | missing |  |
| `class.static` | pending | missing |  |
| `class.staticBlock` | pending | missing |  |
| `class.super` | pending | missing |  |
| `function.apply` | pending | missing |  |
| `function.arguments` | pending | missing |  |
| `function.arrow` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending. Anonymous function expressions (`function (...) { ... }`) share the arrow representation and rules; one matched scenario is registered and verified untimed. Gate141 adds immediate and chained invocation of callable expression results with existing access/origin contracts. The nine-cell matched scenario is verified untimed; its measurements remain pending.; [evidence](./milestones/136-function-expressions.md) · [evidence](./arrow-functions.md) · [evidence](./milestones/141-callable-expressions.md) · [evidence](./callable-expressions.md) · [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) · [function_expressions](../benchmarks/runtime/workloads/function_expressions.tsr) · [callable_expressions](../benchmarks/runtime/workloads/callable_expressions.tsr) |
| `function.async` | pending | missing |  |
| `function.bind` | pending | missing |  |
| `function.call` | pending | missing |  |
| `function.callback` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending.; [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) |
| `function.callbackReturningCallback` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending. Gate141 adds immediate and chained invocation of callable expression results with existing access/origin contracts. The nine-cell matched scenario is verified untimed; its measurements remain pending.; [evidence](./milestones/141-callable-expressions.md) · [evidence](./callable-expressions.md) · [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) · [callable_expressions](../benchmarks/runtime/workloads/callable_expressions.tsr) |
| `function.callSpread` | implemented | paired group | Gate 158 extends checked argument expansion to strings, Sets and Maps; source order, generic inference and ownership remain checked. Dynamic count mismatches throw recoverable call-arity errors. Matched iterable workload verified untimed; collection milestone measurements remain pending. Historical call-spread measurements and evidence remain intact.; [evidence](./milestones/100-call-spread.md) · [evidence](./call-spread.md) · [evidence](./milestones/158-iterable-call-spread.md) · [call_spread_fixed_literal_int](../benchmarks/runtime/workloads/call_spread_fixed_literal_int.tsr) · [call_spread_dynamic_int](../benchmarks/runtime/workloads/call_spread_dynamic_int.tsr) · [call_spread_multiple_mixed_int](../benchmarks/runtime/workloads/call_spread_multiple_mixed_int.tsr) · [call_spread_math_number](../benchmarks/runtime/workloads/call_spread_math_number.tsr) · [call_spread_boolean_pack](../benchmarks/runtime/workloads/call_spread_boolean_pack.tsr) · [call_spread_string_owned](../benchmarks/runtime/workloads/call_spread_string_owned.tsr) · [call_spread_nested_owned](../benchmarks/runtime/workloads/call_spread_nested_owned.tsr) · [call_spread_record_owned](../benchmarks/runtime/workloads/call_spread_record_owned.tsr) · [call_spread_nullish_packs](../benchmarks/runtime/workloads/call_spread_nullish_packs.tsr) · [call_spread_borrowed_string_fixed](../benchmarks/runtime/workloads/call_spread_borrowed_string_fixed.tsr) · [call_spread_optional_default_counts](../benchmarks/runtime/workloads/call_spread_optional_default_counts.tsr) · [call_spread_mutate_callback](../benchmarks/runtime/workloads/call_spread_mutate_callback.tsr) · [call_spread_owned_once_optional](../benchmarks/runtime/workloads/call_spread_owned_once_optional.tsr) · [iterable_call_spread](../benchmarks/runtime/workloads/iterable_call_spread.tsr) |
| `function.closureBorrow` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending.; [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) |
| `function.closureOwned` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending.; [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) |
| `function.declarationHoisting` | native variant | paired group | Gate170 accepts capture-free nested declarations, lexical hoisting, self/mutual recursion, block shadowing and source/editor identity. The matched seven-observation scenario also covers typed variable and arrow-parameter shadowing with outer-binding restoration; five native/Node-after-tsc cases are verified untimed. Gates171/172 accept captures and owned escape. Gate173 adds focused nonrecursive result inference; comprehensive signature/context coverage remains pending. This is not full named-function parity or measurement.; [evidence](./milestones/170-nested-function-foundation.md) · [nested_function_declarations](../benchmarks/runtime/workloads/nested_function_declarations.tsr) |
| `function.declaredValue` | implemented | paired group | Declared functions bind, move, return and capture as zero-capture native callable values through exact existing data-argument contracts. One 571-group regression, actual O0/O3 complete output/order/IEEE checks, twelve matched comparisons and 36 zero-final-disposal profiles pass. Higher-order consumers remain separately scoped. Gate170 accepts capture-free nested declarations, lexical hoisting, self/mutual recursion, block shadowing and source/editor identity. The matched seven-observation scenario also covers typed variable and arrow-parameter shadowing with outer-binding restoration; five native/Node-after-tsc cases are verified untimed. Gates171/172 accept captures and owned escape. Gate173 adds focused nonrecursive result inference; comprehensive signature/context coverage remains pending. This is not full named-function parity or measurement.; [evidence](./milestones/54-declared-function-values.md) · [evidence](./milestones/170-nested-function-foundation.md) · [declared_callback_int_reduce](../benchmarks/runtime/workloads/declared_callback_int_reduce.tsr) · [declared_callback_number_reduce](../benchmarks/runtime/workloads/declared_callback_number_reduce.tsr) · [declared_callback_boolean_reduce](../benchmarks/runtime/workloads/declared_callback_boolean_reduce.tsr) · [declared_callback_string_map](../benchmarks/runtime/workloads/declared_callback_string_map.tsr) · [declared_callback_record_map](../benchmarks/runtime/workloads/declared_callback_record_map.tsr) · [declared_value_local_reuse](../benchmarks/runtime/workloads/declared_value_local_reuse.tsr) · [declared_value_local_move](../benchmarks/runtime/workloads/declared_value_local_move.tsr) · [declared_value_returned_function](../benchmarks/runtime/workloads/declared_value_returned_function.tsr) · [declared_value_factory_target](../benchmarks/runtime/workloads/declared_value_factory_target.tsr) · [declared_value_global_use](../benchmarks/runtime/workloads/declared_value_global_use.tsr) · [declared_value_bound_capture](../benchmarks/runtime/workloads/declared_value_bound_capture.tsr) · [declared_value_owned_tree_move](../benchmarks/runtime/workloads/declared_value_owned_tree_move.tsr) · [nested_function_declarations](../benchmarks/runtime/workloads/nested_function_declarations.tsr) |
| `function.higherOrderValue` | implemented | paired group | Declared and inline callback-consuming values, recursive callable annotations and argument-dependent returned origins pass one 577-group regression plus native O0/O3 output, order, IEEE and disposal checks. Twenty-four matched comparisons across eight runtimes and 72 zero-final-disposal profiles are complete; owned copying costs remain disclosed.; [evidence](./milestones/55-higher-order-function-values.md) · [higher_declared_reusable_int](../benchmarks/runtime/workloads/higher_declared_reusable_int.tsr) · [higher_inline_reusable_int](../benchmarks/runtime/workloads/higher_inline_reusable_int.tsr) · [higher_declared_once_int](../benchmarks/runtime/workloads/higher_declared_once_int.tsr) · [higher_inline_once_int](../benchmarks/runtime/workloads/higher_inline_once_int.tsr) · [higher_declared_owned_int](../benchmarks/runtime/workloads/higher_declared_owned_int.tsr) · [higher_inline_owned_int](../benchmarks/runtime/workloads/higher_inline_owned_int.tsr) · [higher_declared_owned_once_int](../benchmarks/runtime/workloads/higher_declared_owned_once_int.tsr) · [higher_inline_owned_once_int](../benchmarks/runtime/workloads/higher_inline_owned_once_int.tsr) · [higher_distinct_origins_int](../benchmarks/runtime/workloads/higher_distinct_origins_int.tsr) · [higher_multiple_callbacks_int](../benchmarks/runtime/workloads/higher_multiple_callbacks_int.tsr) · [higher_borrowed_declared_factory_int](../benchmarks/runtime/workloads/higher_borrowed_declared_factory_int.tsr) · [higher_borrowed_inline_factory_int](../benchmarks/runtime/workloads/higher_borrowed_inline_factory_int.tsr) · [higher_nested_consumer_int](../benchmarks/runtime/workloads/higher_nested_consumer_int.tsr) · [higher_owned_once_nested_int](../benchmarks/runtime/workloads/higher_owned_once_nested_int.tsr) · [higher_number_reusable](../benchmarks/runtime/workloads/higher_number_reusable.tsr) · [higher_boolean_reusable](../benchmarks/runtime/workloads/higher_boolean_reusable.tsr) · [higher_owned_callback_return_string](../benchmarks/runtime/workloads/higher_owned_callback_return_string.tsr) · [higher_captured_factory_string](../benchmarks/runtime/workloads/higher_captured_factory_string.tsr) · [higher_read_string](../benchmarks/runtime/workloads/higher_read_string.tsr) · [higher_consume_string](../benchmarks/runtime/workloads/higher_consume_string.tsr) · [higher_mutate_array](../benchmarks/runtime/workloads/higher_mutate_array.tsr) · [higher_owned_record](../benchmarks/runtime/workloads/higher_owned_record.tsr) · [higher_deep_array](../benchmarks/runtime/workloads/higher_deep_array.tsr) · [higher_recursive_tree](../benchmarks/runtime/workloads/higher_recursive_tree.tsr) |
| `function.mutualRecursion` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending. Gate170 accepts capture-free nested declarations, lexical hoisting, self/mutual recursion, block shadowing and source/editor identity. The matched seven-observation scenario also covers typed variable and arrow-parameter shadowing with outer-binding restoration; five native/Node-after-tsc cases are verified untimed. Gates171/172 accept captures and owned escape. Gate173 adds focused nonrecursive result inference; comprehensive signature/context coverage remains pending. This is not full named-function parity or measurement.; [evidence](./milestones/170-nested-function-foundation.md) · [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) · [nested_function_declarations](../benchmarks/runtime/workloads/nested_function_declarations.tsr) |
| `function.named` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending. Gate171 accepts nested functions that use variables of enclosing functions (see function.nestedCapture). Gate170 accepts capture-free nested declarations, lexical hoisting, self/mutual recursion, block shadowing and source/editor identity. The matched seven-observation scenario also covers typed variable and arrow-parameter shadowing with outer-binding restoration; five native/Node-after-tsc cases are verified untimed. Gates171/172 accept captures and owned escape. Gate173 adds focused nonrecursive result inference; comprehensive signature/context coverage remains pending. This is not full named-function parity or measurement.; [evidence](./milestones/170-nested-function-foundation.md) · [evidence](./milestones/171-nested-function-captures.md) · [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) · [nested_function_declarations](../benchmarks/runtime/workloads/nested_function_declarations.tsr) · [nested_function_captures](../benchmarks/runtime/workloads/nested_function_captures.tsr) |
| `function.nestedCapture` | native variant | paired group | Gates171 and172 accept per-call captured borrows and owned nested callable escape: independent factories, record/array storage, inferred callable arrays, consuming captures, once invocation, recursive retained owners, callback failures and exact native disposal. Unused inner declarations are typechecked without runtime captures or outer-call readiness/borrow requirements. Matched capture and owned-escape scenarios pass native and Node-after-tsc untimed; no measurement. Gate173 now verifies uniform nonrecursive scalar, generic, captured and array result inference. Captured optional/default values now preserve omission, default evaluation, consumption and signature help. Captured rest values preserve empty/ordered/spread packs, fixed defaults, owned factory state and signature help. Concrete callable contexts now specialize generic values with original template checking. Unannotated polymorphic generic values and the remaining context audit remain open. Concrete source signature solving retains default/rest metadata; forwarded undefined no longer constrains generic type inference, while null remains checked. Unannotated generic values now retain captured environments across data types, independent inferred factories, defaults and rest packs. Exact native cleanup and ownership failures have focused witnesses. Result-only context now propagates through generic aliases with distinct cached methods. Complete alias help, callback/storage adaptation and full gate173 acceptance remain open. Generic factory parameter help now retains concrete arguments through partial checked facts without inventing a result type; conflicting specializations remain separate. Selected fresh factory branches now join through owned callable storage, preserving selected state and one predicate/argument evaluation. Traversal storage and exact disposal have focused coverage; full context audit remains open. Nested selected callees now retain predicate order and execute only the selected path through iterative lowering. Fresh aggregate destinations have executable coverage. Fresh generic traversal infers omitted results by checking original specialized bodies from type hints; explicit result constraints remain checked. Nested callable factory input/result contracts now preserve shared ownership, invocation kinds, optional/rest metadata and void results; 88 matched observations pass five cases untimed. Wider Gate173 audit remains open. Nullable callable destinations and heterogeneous selected factory result contracts now have focused acceptance. Argument-first inference retains explicit result and invocation ownership checks. Ninety-three observations pass five native/Node-after-tsc cases untimed; full Gate173 remains open. Generic callable payloads now preserve full ownership/access/omission/rest contracts and one captured factory environment across concrete signatures. Borrowed and once controls, O0/O3 disposal, concrete editor contracts, 45 adapters, 63 depth canaries and 308 editor tests pass. Ninety-six observations pass five matched cases untimed; broader context audit and measurements remain open. Explicit named generic value applications now retain supplied types and enclosing substitutions, defaults/rest and captured ownership. Scalar/callable, mismatch, transfer and method identity controls pass. One hundred one observations pass five matched cases untimed; full Gate173 remains open. Optional/selected generic invocations now preserve short-circuit effects, one receiver evaluation, result-only context and real ownership roots. Fresh nullable factory hints no longer invent void. Fifty-nine adapter controls, all 63 depth canaries and 310 editor tests pass; O0/O3 disposal retains exact owners. One hundred six observations pass five matched cases untimed. Full Gate173 audit and measurements remain open. Focused owned pattern adapters retain concrete contracts across nested arrays/records, defaults and rest; disposal and readonly boundaries pass. Dedicated editor result/help and strict Node references pass. Matched support retains 116 observations per scenario with untimed native/Node verification. Broader Gate173 audit and measurement remain pending. Unused immutable generic aliases and known callback contracts now validate static call/type cardinality with source diagnostics. Mutable/selected flow and the full source/context audit remain pending. Closure callback contracts and guaranteed argument bounds through nested dynamic spreads are checked, with exact/lower-bound diagnostics. Common mutable/selected source contracts and straight-line assignment snapshots are checked. Structured branch/loop flow, stored fields and concrete callable environment replacement remain pending.; [evidence](./milestones/171-nested-function-captures.md) · [evidence](./nested-functions.md) · [evidence](./milestones/172-nested-function-owned-escape.md) · [nested_function_captures](../benchmarks/runtime/workloads/nested_function_captures.tsr) · [nested_function_owned_escape](../benchmarks/runtime/workloads/nested_function_owned_escape.tsr) |
| `function.nestedDeclaration` | native variant | paired group | Gate170 accepts capture-free nested declarations, lexical hoisting, self/mutual recursion, block shadowing and source/editor identity. The matched seven-observation scenario also covers typed variable and arrow-parameter shadowing with outer-binding restoration; five native/Node-after-tsc cases are verified untimed. Gates171/172 accept captures and owned escape. Gate173 adds focused nonrecursive result inference; comprehensive signature/context coverage remains pending. This is not full named-function parity or measurement.; [evidence](./milestones/170-nested-function-foundation.md) · [nested_function_declarations](../benchmarks/runtime/workloads/nested_function_declarations.tsr) |
| `function.nullableValue` | native variant | missing | One callable payload with null/undefined preserves current owning/borrowed/once contracts, origins and lifetimes through transport and lazy guarded/optional/nullish operations. Focused compiler/editor/native examples accepted; ten matched sources pass586 complete cases per O0/O3 and eight order lines. Performance measurements pending checkpoint101; no isolated call latency or allocation parity claim. General payload unions and nullable array elements remain separate.; [evidence](./milestones/97-nullable-callable-transport.md) · [evidence](./nullable-callables.md) · [evidence](../benchmarks/runtime/diagnostics/2026-10-04T04-22-29-000Z-nullable-callable-implementation.json) · [nullable_callable_declared_guard](../benchmarks/runtime/workloads/nullable_callable_declared_guard.tsr) · [nullable_callable_declared_optional](../benchmarks/runtime/workloads/nullable_callable_declared_optional.tsr) · [nullable_callable_read_forward](../benchmarks/runtime/workloads/nullable_callable_read_forward.tsr) · [nullable_callable_once_forward](../benchmarks/runtime/workloads/nullable_callable_once_forward.tsr) · [nullable_callable_owned_record](../benchmarks/runtime/workloads/nullable_callable_owned_record.tsr) · [nullable_callable_owned_once_return](../benchmarks/runtime/workloads/nullable_callable_owned_once_return.tsr) · [nullable_callable_generic_coalesce](../benchmarks/runtime/workloads/nullable_callable_generic_coalesce.tsr) · [nullable_callable_nullish_assign](../benchmarks/runtime/workloads/nullable_callable_nullish_assign.tsr) · [nullable_callable_higher_order_factory](../benchmarks/runtime/workloads/nullable_callable_higher_order_factory.tsr) · [nullable_callable_absent_higher_order_factory](../benchmarks/runtime/workloads/nullable_callable_absent_higher_order_factory.tsr) |
| `function.overload` | pending | missing |  |
| `function.recursion` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending. Gate170 accepts capture-free nested declarations, lexical hoisting, self/mutual recursion, block shadowing and source/editor identity. The matched seven-observation scenario also covers typed variable and arrow-parameter shadowing with outer-binding restoration; five native/Node-after-tsc cases are verified untimed. Gates171/172 accept captures and owned escape. Gate173 adds focused nonrecursive result inference; comprehensive signature/context coverage remains pending. This is not full named-function parity or measurement.; [evidence](./milestones/170-nested-function-foundation.md) · [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) · [nested_function_declarations](../benchmarks/runtime/workloads/nested_function_declarations.tsr) |
| `function.restParameter` | implemented | paired group | Focused Gate99 accepts fresh homogeneous owning rest packs across current data and callable kinds, generic/imported contexts, abstract/nested adapters and real array callback tuples. Thirteen matched scenarios are registered; full regression and measurements remain queued for checkpoint101. Call spread and heterogeneous tuples remain separate.; [evidence](./milestones/99-rest-parameters.md) · [evidence](./rest-parameters.md) · [rest_parameter_int_pack](../benchmarks/runtime/workloads/rest_parameter_int_pack.tsr) · [rest_parameter_number_pack](../benchmarks/runtime/workloads/rest_parameter_number_pack.tsr) · [rest_parameter_boolean_pack](../benchmarks/runtime/workloads/rest_parameter_boolean_pack.tsr) · [rest_parameter_string_pack](../benchmarks/runtime/workloads/rest_parameter_string_pack.tsr) · [rest_parameter_nested_pack](../benchmarks/runtime/workloads/rest_parameter_nested_pack.tsr) · [rest_parameter_record_pack](../benchmarks/runtime/workloads/rest_parameter_record_pack.tsr) · [rest_parameter_null_pack](../benchmarks/runtime/workloads/rest_parameter_null_pack.tsr) · [rest_parameter_undefined_pack](../benchmarks/runtime/workloads/rest_parameter_undefined_pack.tsr) · [rest_parameter_default_prefix](../benchmarks/runtime/workloads/rest_parameter_default_prefix.tsr) · [rest_parameter_read_callback](../benchmarks/runtime/workloads/rest_parameter_read_callback.tsr) · [rest_parameter_mutate_callback](../benchmarks/runtime/workloads/rest_parameter_mutate_callback.tsr) · [rest_parameter_owned_once_callback](../benchmarks/runtime/workloads/rest_parameter_owned_once_callback.tsr) · [rest_parameter_owned_callable_pack](../benchmarks/runtime/workloads/rest_parameter_owned_callable_pack.tsr) |
| `function.singleUse` | native variant | compound | Existing bounded callable/ownership contracts; broader TS signatures pending.; [local_closures](../benchmarks/runtime/workloads/local_closures.tsr) · [recursive_callbacks](../benchmarks/runtime/workloads/recursive_callbacks.tsr) · [recursive_factory](../benchmarks/runtime/workloads/recursive_factory.tsr) |
| `function.this` | pending | missing |  |

## Generators

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `AsyncGenerator.[Symbol.asyncIterator]` | pending | missing |  |
| `AsyncGenerator.next` | pending | missing |  |
| `AsyncGenerator.return` | pending | missing |  |
| `AsyncGenerator.throw` | pending | missing |  |
| `AsyncGeneratorFunction.call()` | pending | missing |  |
| `AsyncGeneratorFunction.length` | pending | missing |  |
| `AsyncGeneratorFunction.name` | pending | missing |  |
| `AsyncGeneratorFunction.new()` | pending | missing |  |
| `AsyncGeneratorFunction.prototype` | pending | missing |  |
| `AsyncGeneratorFunctionConstructor.call()` | pending | missing |  |
| `AsyncGeneratorFunctionConstructor.length` | pending | missing |  |
| `AsyncGeneratorFunctionConstructor.name` | pending | missing |  |
| `AsyncGeneratorFunctionConstructor.new()` | pending | missing |  |
| `AsyncGeneratorFunctionConstructor.prototype` | pending | missing |  |
| `Generator.[Symbol.iterator]` | pending | missing |  |
| `generator.asyncFunctionStar` | pending | missing |  |
| `generator.asyncYield` | pending | missing |  |
| `generator.cancellation` | pending | missing |  |
| `generator.captures` | pending | missing |  |
| `generator.earlyClose` | pending | missing |  |
| `generator.finally` | pending | missing |  |
| `generator.functionStar` | pending | missing |  |
| `generator.nestedDelegation` | pending | missing |  |
| `Generator.next` | pending | missing |  |
| `generator.return` | pending | missing |  |
| `Generator.return` | pending | missing |  |
| `generator.send` | pending | missing |  |
| `generator.throw` | pending | missing |  |
| `Generator.throw` | pending | missing |  |
| `generator.yield` | pending | missing |  |
| `generator.yieldStar` | pending | missing |  |
| `GeneratorFunction.[Symbol.toStringTag]` | pending | missing |  |
| `GeneratorFunction.call()` | pending | missing |  |
| `GeneratorFunction.length` | pending | missing |  |
| `GeneratorFunction.name` | pending | missing |  |
| `GeneratorFunction.new()` | pending | missing |  |
| `GeneratorFunction.prototype` | pending | missing |  |
| `GeneratorFunctionConstructor.call()` | pending | missing |  |
| `GeneratorFunctionConstructor.length` | pending | missing |  |
| `GeneratorFunctionConstructor.name` | pending | missing |  |
| `GeneratorFunctionConstructor.new()` | pending | missing |  |
| `GeneratorFunctionConstructor.prototype` | pending | missing |  |

## Globals

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `global.console.assert` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.clear` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.count` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.countReset` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.debug` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.dir` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.dirxml` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.error` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.group` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.groupCollapsed` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.groupEnd` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.info` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.log` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.profile` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.profileEnd` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.table` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.time` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.timeEnd` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.timeLog` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.timeStamp` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.trace` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.console.warn` | native variant | compile-only | Node.js 24.11.1 reference; focused O0/O3 output contracts and rich editor documentation. Native host/value/layout differences remain explicitly tracked in the console milestone. I/O throughput comparison is not meaningful; full regression remains at the five-gate checkpoint.; [evidence](./milestones/console-parity.md) |
| `global.decodeURI` | pending | missing |  |
| `global.decodeURIComponent` | pending | missing |  |
| `global.encodeURI` | pending | missing |  |
| `global.encodeURIComponent` | pending | missing |  |
| `global.escape` | pending | missing |  |
| `global.eval` | policy | missing | Original vision rejects unrestricted escape hatches; track explicit policy instead of silently omitting. |
| `global.globalThis` | pending | missing |  |
| `global.Infinity` | native variant | paired | Global Infinity is Number.POSITIVE_INFINITY; -Infinity is the negative value. A declaration with the same name takes precedence. One matched scenario is registered and verified untimed.; [evidence](./milestones/137-global-number-names.md) · [evidence](./global-number-names.md) · [global_number_names](../benchmarks/runtime/workloads/global_number_names.tsr) |
| `global.isFinite` | native variant | paired | Global isFinite is the Number.isFinite implementation and accepts a number only; no argument is converted. One matched scenario is registered and verified untimed.; [evidence](./milestones/137-global-number-names.md) · [evidence](./global-number-names.md) · [global_number_names](../benchmarks/runtime/workloads/global_number_names.tsr) |
| `global.isNaN` | native variant | paired | Global isNaN is the Number.isNaN implementation and accepts a number only; no argument is converted. One matched scenario is registered and verified untimed.; [evidence](./milestones/137-global-number-names.md) · [evidence](./global-number-names.md) · [global_number_names](../benchmarks/runtime/workloads/global_number_names.tsr) |
| `global.NaN` | native variant | paired | Global NaN is Number.NaN; a declaration with the same name takes precedence. One matched scenario is registered and verified untimed.; [evidence](./milestones/137-global-number-names.md) · [evidence](./global-number-names.md) · [global_number_names](../benchmarks/runtime/workloads/global_number_names.tsr) |
| `global.parseFloat` | native variant | paired | Global parseFloat is the Number.parseFloat implementation. One matched scenario is registered and verified untimed.; [evidence](./milestones/137-global-number-names.md) · [evidence](./global-number-names.md) · [global_number_names](../benchmarks/runtime/workloads/global_number_names.tsr) |
| `global.parseInt` | native variant | paired | Global parseInt is the Number.parseInt implementation. One matched scenario is registered and verified untimed.; [evidence](./milestones/137-global-number-names.md) · [evidence](./global-number-names.md) · [global_number_names](../benchmarks/runtime/workloads/global_number_names.tsr) |
| `global.undefined` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Nullable unions and dynamic host values remain unsupported.; [evidence](./milestones/value-types-and-namespaces.md) |
| `global.unescape` | pending | missing |  |

## Host workstreams

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `host.browserAPIs` | pending | missing |  |
| `host.console` | native variant | compile-only | Global Node console method surface implemented. Exact Node equivalence remains incomplete for JavaScript-only values, long/advanced inspection layout, complete dir options, warning scheduling, timer units, native trace frames and inspector integration; see console-parity milestone. |
| `host.crypto` | pending | missing |  |
| `host.desktop` | pending | missing |  |
| `host.encoding` | pending | missing |  |
| `host.filesystem` | pending | missing |  |
| `host.http` | pending | missing |  |
| `host.networking` | pending | missing |  |
| `host.nodeAPIs` | pending | missing |  |
| `host.streams` | pending | missing |  |
| `host.timers` | pending | missing |  |
| `host.url` | pending | missing |  |
| `host.workers` | pending | missing |  |

## Integer extension

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `int.add` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.addAssign` | implemented | paired group | Checked int compound with once-selected target, old-value snapshot before RHS effects and existing storage rules. 609 distinct regression groups pass with retained/resumed acceptance. Bounded nonnegative whole iterations retain every result; timing division is exactly divisible. Signed non-divisible division uses an explicit correctness-only Math.trunc adapter after JS /=. No overflowing or coercing timing-domain claim.; [evidence](./milestones/63-compound-assignment.md) · [compound_int_add](../benchmarks/runtime/workloads/compound_int_add.tsr) |
| `int.addAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_add](../benchmarks/runtime/workloads/compound_value_int_add.tsr) |
| `int.assign` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.assignmentExpression` | native variant | paired group | Ordinary = returns the committed copy scalar across supported expression contexts, statements and for clauses; target type drives RHS contextual typing with strict matching. Target/indices and bounds occur once before RHS, no old target value is read, and commit follows successful RHS. Preserve incoming/parent loans, unconditional ownership effects and existing target/capture/parameter/iteration/partial-field restrictions. Standard Rust scalar assignment allocates no heap storage in focused evidence. Actual JavaScript comparisons retain expression and final target outputs; dynamic setup, selection, conversion and complete output retention are timed. Owned and arithmetic compound assignment values remain pending.; [evidence](./milestones/67-scalar-assignment-values.md) · [int_assignment_value](../benchmarks/runtime/workloads/int_assignment_value.tsr) |
| `int.bitAnd` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_bit_and](../benchmarks/runtime/workloads/int_native_bit_and.tsr) |
| `int.bitAndAssign` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_compound_bit_and](../benchmarks/runtime/workloads/int_native_compound_bit_and.tsr) |
| `int.bitAndAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_and](../benchmarks/runtime/workloads/compound_value_int_and.tsr) |
| `int.bitNot` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_bit_not](../benchmarks/runtime/workloads/int_native_bit_not.tsr) |
| `int.bitOr` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_bit_or](../benchmarks/runtime/workloads/int_native_bit_or.tsr) |
| `int.bitOrAssign` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_compound_bit_or](../benchmarks/runtime/workloads/int_native_compound_bit_or.tsr) |
| `int.bitOrAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_or](../benchmarks/runtime/workloads/compound_value_int_or.tsr) |
| `int.bitXor` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_bit_xor](../benchmarks/runtime/workloads/int_native_bit_xor.tsr) |
| `int.bitXorAssign` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_compound_bit_xor](../benchmarks/runtime/workloads/int_native_compound_bit_xor.tsr) |
| `int.bitXorAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_xor](../benchmarks/runtime/workloads/compound_value_int_xor.tsr) |
| `int.checkedOverflow` | implemented | missing | Implemented and edge-tested; isolated successful-operation or failure-cost comparison still missing.; [evidence](./milestones/12-scalar-operators.md) |
| `int.copy` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.decrement` | implemented | paired group | Umbrella for separately accepted int prefix/postfix decrement. Both actual forms have matched retained-result comparisons and independent semantics, storage, diagnostics and disposal evidence.; [evidence](./milestones/64-update-expressions.md) · [update_int_prefix_decrement](../benchmarks/runtime/workloads/update_int_prefix_decrement.tsr) · [update_int_postfix_decrement](../benchmarks/runtime/workloads/update_int_postfix_decrement.tsr) |
| `int.divide` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [int_divide](../benchmarks/runtime/workloads/int_divide.tsr) |
| `int.divideAssign` | implemented | paired group | Checked int compound with once-selected target, old-value snapshot before RHS effects and existing storage rules. 609 distinct regression groups pass with retained/resumed acceptance. Bounded nonnegative whole iterations retain every result; timing division is exactly divisible. Signed non-divisible division uses an explicit correctness-only Math.trunc adapter after JS /=. No overflowing or coercing timing-domain claim.; [evidence](./milestones/63-compound-assignment.md) · [compound_int_divide](../benchmarks/runtime/workloads/compound_int_divide.tsr) |
| `int.divideAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_divide](../benchmarks/runtime/workloads/compound_value_int_divide.tsr) |
| `int.equal` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.exponent` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Exponent must fit u32; checked i64 power traps on overflow, and 0 ** 0 is 1. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_power](../benchmarks/runtime/workloads/int_native_power.tsr) |
| `int.exponentAssign` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Exponent must fit u32; checked i64 power traps on overflow, and 0 ** 0 is 1. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_compound_power](../benchmarks/runtime/workloads/int_native_compound_power.tsr) |
| `int.exponentAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_power](../benchmarks/runtime/workloads/compound_value_int_power.tsr) |
| `int.greater` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.greaterEqual` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [int_greater_equal](../benchmarks/runtime/workloads/int_greater_equal.tsr) |
| `int.increment` | implemented | paired group | Umbrella for separately accepted int prefix/postfix increment. Both actual forms have matched retained-result comparisons and independent semantics, storage, diagnostics and disposal evidence.; [evidence](./milestones/64-update-expressions.md) · [update_int_prefix_increment](../benchmarks/runtime/workloads/update_int_prefix_increment.tsr) · [update_int_postfix_increment](../benchmarks/runtime/workloads/update_int_postfix_increment.tsr) |
| `int.less` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.lessEqual` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [int_less_equal](../benchmarks/runtime/workloads/int_less_equal.tsr) |
| `int.literal` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator. Gate160 adds separators and radix spelling with preserved defaults/contextual conversion; matched result scenario verified untimed, without isolated scanner measurement.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/160-numeric-literals.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) · [numeric_literals](../benchmarks/runtime/workloads/numeric_literals.tsr) |
| `int.multiply` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.multiplyAssign` | implemented | paired group | Checked int compound with once-selected target, old-value snapshot before RHS effects and existing storage rules. 609 distinct regression groups pass with retained/resumed acceptance. Bounded nonnegative whole iterations retain every result; timing division is exactly divisible. Signed non-divisible division uses an explicit correctness-only Math.trunc adapter after JS /=. No overflowing or coercing timing-domain claim.; [evidence](./milestones/63-compound-assignment.md) · [compound_int_multiply](../benchmarks/runtime/workloads/compound_int_multiply.tsr) |
| `int.multiplyAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_multiply](../benchmarks/runtime/workloads/compound_value_int_multiply.tsr) |
| `int.negate` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [int_negate](../benchmarks/runtime/workloads/int_negate.tsr) |
| `int.notEqual` | implemented | paired | Matched operator/control workload passes across eight runtimes; setup and observer costs are included, with bounded equivalent input domains.; [evidence](./milestones/12-scalar-operators.md) · [evidence](./milestones/38-control-flow-loops.md) · [int_not_equal](../benchmarks/runtime/workloads/int_not_equal.tsr) |
| `int.parse` | pending | missing |  |
| `int.postfixDecrement` | implemented | paired group | Actual postfix decrement in all existing expression contexts with once-selected targets and old/read/compute/commit order. Explicit checked i64 arithmetic; endpoint failures are native-only O0/O3 evidence. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_int_postfix_decrement](../benchmarks/runtime/workloads/update_int_postfix_decrement.tsr) |
| `int.postfixIncrement` | implemented | paired group | Actual postfix increment in all existing expression contexts with once-selected targets and old/read/compute/commit order. Explicit checked i64 arithmetic; endpoint failures are native-only O0/O3 evidence. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_int_postfix_increment](../benchmarks/runtime/workloads/update_int_postfix_increment.tsr) |
| `int.prefixDecrement` | implemented | paired group | Actual prefix decrement in all existing expression contexts with once-selected targets and old/read/compute/commit order. Explicit checked i64 arithmetic; endpoint failures are native-only O0/O3 evidence. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_int_prefix_decrement](../benchmarks/runtime/workloads/update_int_prefix_decrement.tsr) |
| `int.prefixIncrement` | implemented | paired group | Actual prefix increment in all existing expression contexts with once-selected targets and old/read/compute/commit order. Explicit checked i64 arithmetic; endpoint failures are native-only O0/O3 evidence. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_int_prefix_increment](../benchmarks/runtime/workloads/update_int_prefix_increment.tsr) |
| `int.remainder` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.remainderAssign` | implemented | paired group | Checked int compound with once-selected target, old-value snapshot before RHS effects and existing storage rules. 609 distinct regression groups pass with retained/resumed acceptance. Bounded nonnegative whole iterations retain every result; timing division is exactly divisible. Signed non-divisible division uses an explicit correctness-only Math.trunc adapter after JS /=. No overflowing or coercing timing-domain claim.; [evidence](./milestones/63-compound-assignment.md) · [compound_int_remainder](../benchmarks/runtime/workloads/compound_int_remainder.tsr) |
| `int.remainderAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_remainder](../benchmarks/runtime/workloads/compound_value_int_remainder.tsr) |
| `int.shiftLeft` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Counts 0..63 are checked; shifted bits may be discarded. Logical right uses u64 bits and casts back to i64. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_shift_left](../benchmarks/runtime/workloads/int_native_shift_left.tsr) |
| `int.shiftLeftAssign` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Counts 0..63 are checked; shifted bits may be discarded. Logical right uses u64 bits and casts back to i64. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_compound_shift_left](../benchmarks/runtime/workloads/int_native_compound_shift_left.tsr) |
| `int.shiftLeftAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_shift_left](../benchmarks/runtime/workloads/compound_value_int_shift_left.tsr) |
| `int.shiftRight` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Counts 0..63 are checked; shifted bits may be discarded. Logical right uses u64 bits and casts back to i64. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_shift_right](../benchmarks/runtime/workloads/int_native_shift_right.tsr) |
| `int.shiftRightAssign` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Counts 0..63 are checked; shifted bits may be discarded. Logical right uses u64 bits and casts back to i64. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_compound_shift_right](../benchmarks/runtime/workloads/int_native_compound_shift_right.tsr) |
| `int.shiftRightAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_shift_right](../benchmarks/runtime/workloads/compound_value_int_shift_right.tsr) |
| `int.shiftRightUnsigned` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Counts 0..63 are checked; shifted bits may be discarded. Logical right uses u64 bits and casts back to i64. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_shift_right_unsigned](../benchmarks/runtime/workloads/int_native_shift_right_unsigned.tsr) |
| `int.shiftRightUnsignedAssign` | native variant | paired group | Native signed 64-bit bits; JavaScript timing covers only the exact shared domain. Counts 0..63 are checked; shifted bits may be discarded. Logical right uses u64 bits and casts back to i64. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_compound_shift_right_unsigned](../benchmarks/runtime/workloads/int_native_compound_shift_right_unsigned.tsr) |
| `int.shiftRightUnsignedAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_shift_right_unsigned](../benchmarks/runtime/workloads/compound_value_int_shift_right_unsigned.tsr) |
| `int.subtract` | implemented | compound | Checked i64 subset; existing mixed workload does not isolate each operator.; [evidence](./milestones/12-scalar-operators.md) · [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) |
| `int.subtractAssign` | implemented | paired group | Checked int compound with once-selected target, old-value snapshot before RHS effects and existing storage rules. 609 distinct regression groups pass with retained/resumed acceptance. Bounded nonnegative whole iterations retain every result; timing division is exactly divisible. Signed non-divisible division uses an explicit correctness-only Math.trunc adapter after JS /=. No overflowing or coercing timing-domain claim.; [evidence](./milestones/63-compound-assignment.md) · [compound_int_subtract](../benchmarks/runtime/workloads/compound_int_subtract.tsr) |
| `int.subtractAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Native checked i64 arithmetic, full-width bitwise/shifts with counts 0..63 and checked u32 power; matched JavaScript timing uses bounded exact common integer domains. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_int_subtract](../benchmarks/runtime/workloads/compound_value_int_subtract.tsr) |
| `int.toString` | native variant | paired group | All signed-i64 values, default decimal and strict optional radix 2–36. Lowercase signed digits and owned UTF-8 result; once-only operands and zero normal/unwind residue. Four JS-safe comparisons pass; Number formatting and coercion are separate.; [evidence](./milestones/73-integer-formatting.md) · [int_to_string_decimal](../benchmarks/runtime/workloads/int_to_string_decimal.tsr) · [int_to_string_binary](../benchmarks/runtime/workloads/int_to_string_binary.tsr) · [int_to_string_hex](../benchmarks/runtime/workloads/int_to_string_hex.tsr) · [int_to_string_base36](../benchmarks/runtime/workloads/int_to_string_base36.tsr) |
| `int.unaryPlus` | implemented | paired group | Identity on int values. Explicit int contexts or concrete typed int anchors select this operation; existing literal-only Number defaults and strict mixed-type rejection remain unchanged. Operands and compound target/old-value/RHS effects occur once in source order, with commit only after success. Complete retained int[] outputs use the existing observer without offsets. Setup and output construction/retention are timed; scalar operations allocate nothing in focused evidence. Separate native-only wide, trap, no-commit and disposal witnesses pass.; [evidence](./milestones/65-native-int-operators.md) · [int_native_unary_plus](../benchmarks/runtime/workloads/int_native_unary_plus.tsr) |
| `int.valueOf` | native variant | paired group | Primitive receiver only, strict zero arity; receiver evaluates once and stays reusable. Native signed i64 copy; exact JS-safe hand parity is separate from nonnegative bounded timed outputs; full i64 extremes remain native-only evidence. Compiler/editor, complete source/native outputs and runtime/disposal acceptance are complete; numeric formatting and boxed objects are separate.; [evidence](./milestones/72-primitive-identity.md) · [int_value_of](../benchmarks/runtime/workloads/int_value_of.tsr) |

## Internationalization

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `Intl.Collator.compare` | pending | missing |  |
| `Intl.Collator.resolvedOptions` | pending | missing |  |
| `Intl.CollatorConstructor.call()` | pending | missing |  |
| `Intl.CollatorConstructor.new()` | pending | missing |  |
| `Intl.CollatorConstructor.supportedLocalesOf` | pending | missing |  |
| `Intl.CollatorOptions.caseFirst` | pending | missing |  |
| `Intl.CollatorOptions.collation` | pending | missing |  |
| `Intl.CollatorOptions.ignorePunctuation` | pending | missing |  |
| `Intl.CollatorOptions.localeMatcher` | pending | missing |  |
| `Intl.CollatorOptions.numeric` | pending | missing |  |
| `Intl.CollatorOptions.sensitivity` | pending | missing |  |
| `Intl.CollatorOptions.usage` | pending | missing |  |
| `Intl.DateTimeFormat.format` | pending | missing |  |
| `Intl.DateTimeFormat.formatRange` | pending | missing |  |
| `Intl.DateTimeFormat.formatRangeToParts` | pending | missing |  |
| `Intl.DateTimeFormat.formatToParts` | pending | missing |  |
| `Intl.DateTimeFormat.resolvedOptions` | pending | missing |  |
| `Intl.DateTimeFormatConstructor.call()` | pending | missing |  |
| `Intl.DateTimeFormatConstructor.new()` | pending | missing |  |
| `Intl.DateTimeFormatConstructor.prototype` | pending | missing |  |
| `Intl.DateTimeFormatConstructor.supportedLocalesOf` | pending | missing |  |
| `Intl.DateTimeFormatOptions.calendar` | pending | missing |  |
| `Intl.DateTimeFormatOptions.dateStyle` | pending | missing |  |
| `Intl.DateTimeFormatOptions.day` | pending | missing |  |
| `Intl.DateTimeFormatOptions.dayPeriod` | pending | missing |  |
| `Intl.DateTimeFormatOptions.era` | pending | missing |  |
| `Intl.DateTimeFormatOptions.formatMatcher` | pending | missing |  |
| `Intl.DateTimeFormatOptions.fractionalSecondDigits` | pending | missing |  |
| `Intl.DateTimeFormatOptions.hour` | pending | missing |  |
| `Intl.DateTimeFormatOptions.hour12` | pending | missing |  |
| `Intl.DateTimeFormatOptions.hourCycle` | pending | missing |  |
| `Intl.DateTimeFormatOptions.localeMatcher` | pending | missing |  |
| `Intl.DateTimeFormatOptions.minute` | pending | missing |  |
| `Intl.DateTimeFormatOptions.month` | pending | missing |  |
| `Intl.DateTimeFormatOptions.numberingSystem` | pending | missing |  |
| `Intl.DateTimeFormatOptions.second` | pending | missing |  |
| `Intl.DateTimeFormatOptions.timeStyle` | pending | missing |  |
| `Intl.DateTimeFormatOptions.timeZone` | pending | missing |  |
| `Intl.DateTimeFormatOptions.timeZoneName` | pending | missing |  |
| `Intl.DateTimeFormatOptions.weekday` | pending | missing |  |
| `Intl.DateTimeFormatOptions.year` | pending | missing |  |
| `Intl.DateTimeFormatPart.type` | pending | missing |  |
| `Intl.DateTimeFormatPart.value` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.day` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.dayPeriod` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.era` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.fractionalSecond` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.hour` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.literal` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.minute` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.month` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.second` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.timeZoneName` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.unknown` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.weekday` | pending | missing |  |
| `Intl.DateTimeFormatPartTypesRegistry.year` | pending | missing |  |
| `Intl.DateTimeRangeFormatPart.source` | pending | missing |  |
| `Intl.DisplayNames.of` | pending | missing |  |
| `Intl.DisplayNames.resolvedOptions` | pending | missing |  |
| `Intl.DisplayNamesOptions.fallback` | pending | missing |  |
| `Intl.DisplayNamesOptions.languageDisplay` | pending | missing |  |
| `Intl.DisplayNamesOptions.localeMatcher` | pending | missing |  |
| `Intl.DisplayNamesOptions.style` | pending | missing |  |
| `Intl.DisplayNamesOptions.type` | pending | missing |  |
| `Intl.getCanonicalLocales` | pending | missing |  |
| `Intl.ListFormat.format` | pending | missing |  |
| `Intl.ListFormat.formatToParts` | pending | missing |  |
| `Intl.ListFormat.resolvedOptions` | pending | missing |  |
| `Intl.ListFormatOptions.localeMatcher` | pending | missing |  |
| `Intl.ListFormatOptions.style` | pending | missing |  |
| `Intl.ListFormatOptions.type` | pending | missing |  |
| `Intl.Locale.baseName` | pending | missing |  |
| `Intl.Locale.language` | pending | missing |  |
| `Intl.Locale.maximize` | pending | missing |  |
| `Intl.Locale.minimize` | pending | missing |  |
| `Intl.Locale.toString` | pending | missing |  |
| `Intl.LocaleOptions.baseName` | pending | missing |  |
| `Intl.LocaleOptions.calendar` | pending | missing |  |
| `Intl.LocaleOptions.caseFirst` | pending | missing |  |
| `Intl.LocaleOptions.collation` | pending | missing |  |
| `Intl.LocaleOptions.hourCycle` | pending | missing |  |
| `Intl.LocaleOptions.language` | pending | missing |  |
| `Intl.LocaleOptions.numberingSystem` | pending | missing |  |
| `Intl.LocaleOptions.numeric` | pending | missing |  |
| `Intl.LocaleOptions.region` | pending | missing |  |
| `Intl.LocaleOptions.script` | pending | missing |  |
| `Intl.NumberFormat.format` | pending | missing |  |
| `Intl.NumberFormat.formatRange` | pending | missing |  |
| `Intl.NumberFormat.formatRangeToParts` | pending | missing |  |
| `Intl.NumberFormat.formatToParts` | pending | missing |  |
| `Intl.NumberFormat.resolvedOptions` | pending | missing |  |
| `Intl.NumberFormatConstructor.call()` | pending | missing |  |
| `Intl.NumberFormatConstructor.new()` | pending | missing |  |
| `Intl.NumberFormatConstructor.prototype` | pending | missing |  |
| `Intl.NumberFormatConstructor.supportedLocalesOf` | pending | missing |  |
| `Intl.NumberFormatOptions.compactDisplay` | pending | missing |  |
| `Intl.NumberFormatOptions.currency` | pending | missing |  |
| `Intl.NumberFormatOptions.currencyDisplay` | pending | missing |  |
| `Intl.NumberFormatOptions.currencySign` | pending | missing |  |
| `Intl.NumberFormatOptions.localeMatcher` | pending | missing |  |
| `Intl.NumberFormatOptions.maximumFractionDigits` | pending | missing |  |
| `Intl.NumberFormatOptions.maximumSignificantDigits` | pending | missing |  |
| `Intl.NumberFormatOptions.minimumFractionDigits` | pending | missing |  |
| `Intl.NumberFormatOptions.minimumIntegerDigits` | pending | missing |  |
| `Intl.NumberFormatOptions.minimumSignificantDigits` | pending | missing |  |
| `Intl.NumberFormatOptions.notation` | pending | missing |  |
| `Intl.NumberFormatOptions.numberingSystem` | pending | missing |  |
| `Intl.NumberFormatOptions.roundingIncrement` | pending | missing |  |
| `Intl.NumberFormatOptions.roundingMode` | pending | missing |  |
| `Intl.NumberFormatOptions.roundingPriority` | pending | missing |  |
| `Intl.NumberFormatOptions.signDisplay` | pending | missing |  |
| `Intl.NumberFormatOptions.style` | pending | missing |  |
| `Intl.NumberFormatOptions.trailingZeroDisplay` | pending | missing |  |
| `Intl.NumberFormatOptions.unit` | pending | missing |  |
| `Intl.NumberFormatOptions.unitDisplay` | pending | missing |  |
| `Intl.NumberFormatOptions.useGrouping` | pending | missing |  |
| `Intl.NumberFormatOptionsCurrencyDisplayRegistry.code` | pending | missing |  |
| `Intl.NumberFormatOptionsCurrencyDisplayRegistry.name` | pending | missing |  |
| `Intl.NumberFormatOptionsCurrencyDisplayRegistry.narrowSymbol` | pending | missing |  |
| `Intl.NumberFormatOptionsCurrencyDisplayRegistry.symbol` | pending | missing |  |
| `Intl.NumberFormatOptionsSignDisplayRegistry.always` | pending | missing |  |
| `Intl.NumberFormatOptionsSignDisplayRegistry.auto` | pending | missing |  |
| `Intl.NumberFormatOptionsSignDisplayRegistry.exceptZero` | pending | missing |  |
| `Intl.NumberFormatOptionsSignDisplayRegistry.negative` | pending | missing |  |
| `Intl.NumberFormatOptionsSignDisplayRegistry.never` | pending | missing |  |
| `Intl.NumberFormatOptionsStyleRegistry.currency` | pending | missing |  |
| `Intl.NumberFormatOptionsStyleRegistry.decimal` | pending | missing |  |
| `Intl.NumberFormatOptionsStyleRegistry.percent` | pending | missing |  |
| `Intl.NumberFormatOptionsStyleRegistry.unit` | pending | missing |  |
| `Intl.NumberFormatOptionsUseGroupingRegistry.always` | pending | missing |  |
| `Intl.NumberFormatOptionsUseGroupingRegistry.auto` | pending | missing |  |
| `Intl.NumberFormatOptionsUseGroupingRegistry.min2` | pending | missing |  |
| `Intl.NumberFormatPart.type` | pending | missing |  |
| `Intl.NumberFormatPart.value` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.compact` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.currency` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.decimal` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.exponentInteger` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.exponentMinusSign` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.exponentSeparator` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.fraction` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.group` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.infinity` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.integer` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.literal` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.minusSign` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.nan` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.percent` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.percentSign` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.plusSign` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.unit` | pending | missing |  |
| `Intl.NumberFormatPartTypeRegistry.unknown` | pending | missing |  |
| `Intl.NumberRangeFormatPart.source` | pending | missing |  |
| `Intl.PluralRules.resolvedOptions` | pending | missing |  |
| `Intl.PluralRules.select` | pending | missing |  |
| `Intl.PluralRulesConstructor.call()` | pending | missing |  |
| `Intl.PluralRulesConstructor.new()` | pending | missing |  |
| `Intl.PluralRulesConstructor.supportedLocalesOf` | pending | missing |  |
| `Intl.PluralRulesOptions.localeMatcher` | pending | missing |  |
| `Intl.PluralRulesOptions.maximumFractionDigits` | pending | missing |  |
| `Intl.PluralRulesOptions.maximumSignificantDigits` | pending | missing |  |
| `Intl.PluralRulesOptions.minimumFractionDigits` | pending | missing |  |
| `Intl.PluralRulesOptions.minimumIntegerDigits` | pending | missing |  |
| `Intl.PluralRulesOptions.minimumSignificantDigits` | pending | missing |  |
| `Intl.PluralRulesOptions.type` | pending | missing |  |
| `Intl.RelativeTimeFormat.format` | pending | missing |  |
| `Intl.RelativeTimeFormat.formatToParts` | pending | missing |  |
| `Intl.RelativeTimeFormat.resolvedOptions` | pending | missing |  |
| `Intl.RelativeTimeFormatOptions.localeMatcher` | pending | missing |  |
| `Intl.RelativeTimeFormatOptions.numeric` | pending | missing |  |
| `Intl.RelativeTimeFormatOptions.style` | pending | missing |  |
| `Intl.ResolvedCollatorOptions.caseFirst` | pending | missing |  |
| `Intl.ResolvedCollatorOptions.collation` | pending | missing |  |
| `Intl.ResolvedCollatorOptions.ignorePunctuation` | pending | missing |  |
| `Intl.ResolvedCollatorOptions.locale` | pending | missing |  |
| `Intl.ResolvedCollatorOptions.numeric` | pending | missing |  |
| `Intl.ResolvedCollatorOptions.sensitivity` | pending | missing |  |
| `Intl.ResolvedCollatorOptions.usage` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.calendar` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.dateStyle` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.day` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.dayPeriod` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.era` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.formatMatcher` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.fractionalSecondDigits` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.hour` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.hour12` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.hourCycle` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.locale` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.minute` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.month` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.numberingSystem` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.second` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.timeStyle` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.timeZone` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.timeZoneName` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.weekday` | pending | missing |  |
| `Intl.ResolvedDateTimeFormatOptions.year` | pending | missing |  |
| `Intl.ResolvedDisplayNamesOptions.fallback` | pending | missing |  |
| `Intl.ResolvedDisplayNamesOptions.languageDisplay` | pending | missing |  |
| `Intl.ResolvedDisplayNamesOptions.locale` | pending | missing |  |
| `Intl.ResolvedDisplayNamesOptions.style` | pending | missing |  |
| `Intl.ResolvedDisplayNamesOptions.type` | pending | missing |  |
| `Intl.ResolvedListFormatOptions.locale` | pending | missing |  |
| `Intl.ResolvedListFormatOptions.style` | pending | missing |  |
| `Intl.ResolvedListFormatOptions.type` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.compactDisplay` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.currency` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.currencyDisplay` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.currencySign` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.locale` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.maximumFractionDigits` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.maximumSignificantDigits` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.minimumFractionDigits` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.minimumIntegerDigits` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.minimumSignificantDigits` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.notation` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.numberingSystem` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.roundingIncrement` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.roundingMode` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.roundingPriority` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.signDisplay` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.style` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.trailingZeroDisplay` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.unit` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.unitDisplay` | pending | missing |  |
| `Intl.ResolvedNumberFormatOptions.useGrouping` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.locale` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.maximumFractionDigits` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.maximumSignificantDigits` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.minimumFractionDigits` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.minimumIntegerDigits` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.minimumSignificantDigits` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.pluralCategories` | pending | missing |  |
| `Intl.ResolvedPluralRulesOptions.type` | pending | missing |  |
| `Intl.ResolvedRelativeTimeFormatOptions.locale` | pending | missing |  |
| `Intl.ResolvedRelativeTimeFormatOptions.numberingSystem` | pending | missing |  |
| `Intl.ResolvedRelativeTimeFormatOptions.numeric` | pending | missing |  |
| `Intl.ResolvedRelativeTimeFormatOptions.style` | pending | missing |  |
| `Intl.ResolvedSegmenterOptions.granularity` | pending | missing |  |
| `Intl.ResolvedSegmenterOptions.locale` | pending | missing |  |
| `Intl.SegmentData.index` | pending | missing |  |
| `Intl.SegmentData.input` | pending | missing |  |
| `Intl.SegmentData.isWordLike` | pending | missing |  |
| `Intl.SegmentData.segment` | pending | missing |  |
| `Intl.Segmenter.resolvedOptions` | pending | missing |  |
| `Intl.Segmenter.segment` | pending | missing |  |
| `Intl.SegmenterOptions.granularity` | pending | missing |  |
| `Intl.SegmenterOptions.localeMatcher` | pending | missing |  |
| `Intl.SegmentIterator.[Symbol.iterator]` | pending | missing |  |
| `Intl.Segments.[Symbol.iterator]` | pending | missing |  |
| `Intl.Segments.containing` | pending | missing |  |
| `Intl.supportedValuesOf` | pending | missing |  |

## Iterators

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `AsyncIterable.[Symbol.asyncIterator]` | pending | missing |  |
| `AsyncIterableIterator.[Symbol.asyncIterator]` | pending | missing |  |
| `AsyncIterator.next` | pending | missing |  |
| `AsyncIterator.return` | pending | missing |  |
| `AsyncIterator.throw` | pending | missing |  |
| `AsyncIteratorObject.[Symbol.asyncIterator]` | pending | missing |  |
| `Iterable.[Symbol.iterator]` | pending | missing |  |
| `IterableIterator.[Symbol.iterator]` | pending | missing |  |
| `iterator.asyncEarlyClose` | pending | missing |  |
| `iterator.asyncNext` | pending | missing |  |
| `iterator.asyncProtocol` | pending | missing |  |
| `iterator.borrowed` | native variant | compound | Borrowed dense-array iteration; general iterator protocol pending.; [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) |
| `iterator.cleanup` | pending | missing |  |
| `iterator.consuming` | pending | missing |  |
| `iterator.customIterable` | pending | missing |  |
| `iterator.earlyClose` | pending | missing |  |
| `iterator.lazyFusion` | pending | missing |  |
| `iterator.mutable` | pending | missing |  |
| `iterator.next` | pending | missing |  |
| `Iterator.next` | pending | missing |  |
| `iterator.protocol` | pending | missing |  |
| `iterator.return` | pending | missing |  |
| `Iterator.return` | pending | missing |  |
| `iterator.throw` | pending | missing |  |
| `Iterator.throw` | pending | missing |  |
| `IteratorObject.[Symbol.iterator]` | pending | missing |  |

## Null and undefined

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `null.assignment` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) |
| `null.equality` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) |
| `null.literal` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) |
| `null.narrowing` | pending | missing |  |
| `nullish.assignment` | pending | missing |  |
| `nullish.coalescing` | native variant | paired group | Lazy once-ordered null/undefined selection with strict canonical typed results, existing owning/scoped-read contracts and callable origins. Stable nullable names/direct fields get fallback-only absence facts; writes invalidate and facts do not leak. Four retained compiler groups, editor plus scoped RHS supplement and six O0/O3 language examples accepted; final generated-source correspondence verified. matched comparisons are measured at checkpoint 96. Nullish assignment is accepted at Gate95; optional chaining is accepted at Gate96; general payload unions and coercion remain pending. Gate126 accepts a static string literal fallback for a borrowed optional string selection without cloning, including nullable record array elements. Source loans remain lexical. The matched scenario is verified untimed; existing measurement records are preserved.; [evidence](./milestones/94-nullish-coalescing.md) · [evidence](./nullish-coalescing.md) · [evidence](../benchmarks/runtime/diagnostics/2026-10-03T20-48-38-000Z-nullish-coalescing-implementation.json) · [evidence](./milestones/126-borrowed-optional-fallback.md) · [nullish_int_value](../benchmarks/runtime/workloads/nullish_int_value.tsr) · [nullish_number_value](../benchmarks/runtime/workloads/nullish_number_value.tsr) · [nullish_boolean_value](../benchmarks/runtime/workloads/nullish_boolean_value.tsr) · [nullish_string_value](../benchmarks/runtime/workloads/nullish_string_value.tsr) · [nullish_array_value](../benchmarks/runtime/workloads/nullish_array_value.tsr) · [nullish_record_value](../benchmarks/runtime/workloads/nullish_record_value.tsr) · [nullish_nullable_fallback_int](../benchmarks/runtime/workloads/nullish_nullable_fallback_int.tsr) · [nullish_null_undefined_value](../benchmarks/runtime/workloads/nullish_null_undefined_value.tsr) · [borrowed_optional_fallback](../benchmarks/runtime/workloads/borrowed_optional_fallback.tsr) |
| `optional.call` | native variant | paired group | Optional calls evaluate callee/receiver once and skip arguments when absent. Existing function, method, namespace and generic call contracts remain strict; statically present calls retain concrete results. Nullable callable storage and general coercion are not introduced. Focused compiler/editor and six O0/O3 language examples accepted; matched comparisons are measured at checkpoint 96.; [evidence](./milestones/96-optional-chaining.md) · [evidence](./optional-chaining.md) · [evidence](../benchmarks/runtime/diagnostics/2026-10-03T23-30-44-000Z-optional-chaining-implementation.json) · [optional_chain_int_field](../benchmarks/runtime/workloads/optional_chain_int_field.tsr) · [optional_chain_number_field](../benchmarks/runtime/workloads/optional_chain_number_field.tsr) · [optional_chain_boolean_field](../benchmarks/runtime/workloads/optional_chain_boolean_field.tsr) · [optional_chain_string_length](../benchmarks/runtime/workloads/optional_chain_string_length.tsr) · [optional_chain_array_index](../benchmarks/runtime/workloads/optional_chain_array_index.tsr) · [optional_chain_array_join](../benchmarks/runtime/workloads/optional_chain_array_join.tsr) · [optional_chain_record_field](../benchmarks/runtime/workloads/optional_chain_record_field.tsr) · [optional_chain_nullable_chain](../benchmarks/runtime/workloads/optional_chain_nullable_chain.tsr) |
| `optional.chaining` | native variant | paired group | Typed contiguous optional chains evaluate the base once and lazily skip later steps on null/undefined; parentheses end the chain. Successful hops use checked payload types, and reachable absence adds canonical undefined. Strict native types, UTF8 and bounds remain; owned selected values use scoped readonly views or ordinary fresh-result ownership, with explicit clone for independent ownership. Focused compiler/editor and six O0/O3 language examples accepted; matched comparisons are measured at checkpoint 96. Gate126 accepts a static string literal fallback for a borrowed optional string selection without cloning, including nullable record array elements. Source loans remain lexical. The matched scenario is verified untimed; existing measurement records are preserved.; [evidence](./milestones/96-optional-chaining.md) · [evidence](./optional-chaining.md) · [evidence](../benchmarks/runtime/diagnostics/2026-10-03T23-30-44-000Z-optional-chaining-implementation.json) · [evidence](./milestones/126-borrowed-optional-fallback.md) · [optional_chain_int_field](../benchmarks/runtime/workloads/optional_chain_int_field.tsr) · [optional_chain_number_field](../benchmarks/runtime/workloads/optional_chain_number_field.tsr) · [optional_chain_boolean_field](../benchmarks/runtime/workloads/optional_chain_boolean_field.tsr) · [optional_chain_string_length](../benchmarks/runtime/workloads/optional_chain_string_length.tsr) · [optional_chain_array_index](../benchmarks/runtime/workloads/optional_chain_array_index.tsr) · [optional_chain_array_join](../benchmarks/runtime/workloads/optional_chain_array_join.tsr) · [optional_chain_record_field](../benchmarks/runtime/workloads/optional_chain_record_field.tsr) · [optional_chain_nullable_chain](../benchmarks/runtime/workloads/optional_chain_nullable_chain.tsr) · [borrowed_optional_fallback](../benchmarks/runtime/workloads/borrowed_optional_fallback.tsr) |
| `optional.field` | pending | missing |  |
| `optional.index` | native variant | paired group | Optional indexing evaluates the index only when the base is present. Present indexing retains ordinary native signed-int bounds checks; absent access yields canonical undefined. Copy elements are snapshots and owned elements obey scoped readonly view or temporary-owner transfer contracts. Focused compiler/editor and six O0/O3 language examples accepted; matched comparisons are measured at checkpoint 96.; [evidence](./milestones/96-optional-chaining.md) · [evidence](./optional-chaining.md) · [evidence](../benchmarks/runtime/diagnostics/2026-10-03T23-30-44-000Z-optional-chaining-implementation.json) · [optional_chain_int_field](../benchmarks/runtime/workloads/optional_chain_int_field.tsr) · [optional_chain_number_field](../benchmarks/runtime/workloads/optional_chain_number_field.tsr) · [optional_chain_boolean_field](../benchmarks/runtime/workloads/optional_chain_boolean_field.tsr) · [optional_chain_string_length](../benchmarks/runtime/workloads/optional_chain_string_length.tsr) · [optional_chain_array_index](../benchmarks/runtime/workloads/optional_chain_array_index.tsr) · [optional_chain_array_join](../benchmarks/runtime/workloads/optional_chain_array_join.tsr) · [optional_chain_record_field](../benchmarks/runtime/workloads/optional_chain_record_field.tsr) · [optional_chain_nullable_chain](../benchmarks/runtime/workloads/optional_chain_nullable_chain.tsr) |
| `optional.parameter` | implemented | paired group | Gate98 accepts optional markers and lazy expression defaults across current data/callable ownership contracts, required-position arity, generic/imported signatures and verified editor help. Thirteen source comparisons are accepted and registered; measurements remain pending checkpoint101. Destructuring and broader heterogeneous unions remain separate.; [evidence](./milestones/98-optional-default-parameters.md) · [evidence](./optional-default-parameters.md) · [optional_parameter_int_guard](../benchmarks/runtime/workloads/optional_parameter_int_guard.tsr) · [default_parameter_number_value](../benchmarks/runtime/workloads/default_parameter_number_value.tsr) · [default_parameter_boolean_value](../benchmarks/runtime/workloads/default_parameter_boolean_value.tsr) · [default_parameter_string_value](../benchmarks/runtime/workloads/default_parameter_string_value.tsr) · [default_parameter_array_value](../benchmarks/runtime/workloads/default_parameter_array_value.tsr) · [default_parameter_record_value](../benchmarks/runtime/workloads/default_parameter_record_value.tsr) · [default_parameter_dependent_positions](../benchmarks/runtime/workloads/default_parameter_dependent_positions.tsr) · [optional_parameter_read_callback](../benchmarks/runtime/workloads/optional_parameter_read_callback.tsr) · [optional_parameter_mutate_callback](../benchmarks/runtime/workloads/optional_parameter_mutate_callback.tsr) · [default_parameter_owned_reusable_callback](../benchmarks/runtime/workloads/default_parameter_owned_reusable_callback.tsr) · [optional_parameter_once_callback](../benchmarks/runtime/workloads/optional_parameter_once_callback.tsr) · [default_parameter_owned_once_callback](../benchmarks/runtime/workloads/default_parameter_owned_once_callback.tsr) · [default_parameter_stored_family](../benchmarks/runtime/workloads/default_parameter_stored_family.tsr) |
| `optional.return` | pending | missing |  |
| `optional.value` | pending | missing |  |
| `undefined.assignment` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) |
| `undefined.equality` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) |
| `undefined.literal` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/84-nullable-type-foundation.md) |
| `undefined.narrowing` | pending | missing |  |

## Numbers

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `Math.[Symbol.toStringTag]` | pending | missing |  |
| `Math.abs` | implemented | paired | Standard Rust f64 transform accepted by 515 compiler groups and required checks. Matched whole iterations pass across eight runtimes with exact IEEE validation and allocation/disposal records; table setup, selection, loop, and accumulation costs included.; [evidence](./milestones/41-math-unary.md) · [math_abs](../benchmarks/runtime/workloads/math_abs.tsr) |
| `Math.acos` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_acos](../benchmarks/runtime/workloads/math_acos.tsr) |
| `Math.acosh` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_acosh](../benchmarks/runtime/workloads/math_acosh.tsr) |
| `Math.asin` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_asin](../benchmarks/runtime/workloads/math_asin.tsr) |
| `Math.asinh` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_asinh](../benchmarks/runtime/workloads/math_asinh.tsr) |
| `Math.atan` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_atan](../benchmarks/runtime/workloads/math_atan.tsr) |
| `Math.atan2` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_atan2](../benchmarks/runtime/workloads/math_atan2.tsr) |
| `Math.atanh` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_atanh](../benchmarks/runtime/workloads/math_atanh.tsr) |
| `Math.cbrt` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_cbrt](../benchmarks/runtime/workloads/math_cbrt.tsr) |
| `Math.ceil` | implemented | paired | Standard Rust f64 transform accepted by 515 compiler groups and required checks. Matched whole iterations pass across eight runtimes with exact IEEE validation and allocation/disposal records; table setup, selection, loop, and accumulation costs included.; [evidence](./milestones/41-math-unary.md) · [math_ceil](../benchmarks/runtime/workloads/math_ceil.tsr) |
| `Math.clz32` | implemented | paired | Gate48 compiler acceptance passes one 544-group regression and required Rust checks. Actual matched eight-runtime scalar IEEE comparisons and three native allocation/disposal samples pass; results and performance gaps remain recorded.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) · [math_clz32](../benchmarks/runtime/workloads/math_clz32.tsr) |
| `Math.cos` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_cos](../benchmarks/runtime/workloads/math_cos.tsr) |
| `Math.cosh` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_cosh](../benchmarks/runtime/workloads/math_cosh.tsr) |
| `Math.E` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.exp` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_exp](../benchmarks/runtime/workloads/math_exp.tsr) |
| `Math.expm1` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_expm1](../benchmarks/runtime/workloads/math_expm1.tsr) |
| `Math.f16round` | implemented | paired | One number argument rounded to the nearest half-precision value with ties to even; overflow gives a signed infinity and underflow a signed zero. 341 values checked against Node. One matched scenario is registered and verified untimed; measurements wait for the math-functions milestone.; esnext declaration; availability unverified; [evidence](./milestones/108-math-f16round.md) · [math_f16round_values](../benchmarks/runtime/workloads/math_f16round_values.tsr) |
| `Math.floor` | implemented | paired | Standard Rust f64 transform accepted by 515 compiler groups and required checks. Matched whole iterations pass across eight runtimes with exact IEEE validation and allocation/disposal records; table setup, selection, loop, and accumulation costs included.; [evidence](./milestones/41-math-unary.md) · [math_floor](../benchmarks/runtime/workloads/math_floor.tsr) |
| `Math.fround` | implemented | paired | Gate48 compiler acceptance passes one 544-group regression and required Rust checks. Actual matched eight-runtime scalar IEEE comparisons and three native allocation/disposal samples pass; results and performance gaps remain recorded.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) · [math_fround](../benchmarks/runtime/workloads/math_fround.tsr) |
| `Math.hypot` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_hypot](../benchmarks/runtime/workloads/math_hypot.tsr) |
| `Math.imul` | implemented | paired | Gate48 compiler acceptance passes one 544-group regression and required Rust checks. Actual matched eight-runtime scalar IEEE comparisons and three native allocation/disposal samples pass; results and performance gaps remain recorded.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) · [math_imul](../benchmarks/runtime/workloads/math_imul.tsr) |
| `Math.LN10` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.LN2` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.log` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_log](../benchmarks/runtime/workloads/math_log.tsr) |
| `Math.log10` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_log10](../benchmarks/runtime/workloads/math_log10.tsr) |
| `Math.LOG10E` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.log1p` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_log1p](../benchmarks/runtime/workloads/math_log1p.tsr) |
| `Math.log2` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_log2](../benchmarks/runtime/workloads/math_log2.tsr) |
| `Math.LOG2E` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.max` | implemented | paired | Gate48 compiler acceptance passes one 544-group regression and required Rust checks. Actual matched eight-runtime scalar IEEE comparisons and three native allocation/disposal samples pass; results and performance gaps remain recorded.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) · [math_max](../benchmarks/runtime/workloads/math_max.tsr) |
| `Math.min` | implemented | paired | Gate48 compiler acceptance passes one 544-group regression and required Rust checks. Actual matched eight-runtime scalar IEEE comparisons and three native allocation/disposal samples pass; results and performance gaps remain recorded.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) · [math_min](../benchmarks/runtime/workloads/math_min.tsr) |
| `Math.PI` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.pow` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_pow](../benchmarks/runtime/workloads/math_pow.tsr) |
| `Math.random` | implemented | paired | No arguments; uniform number in [0, 1) from a per-thread xorshift128+ state seeded per process. Not seedable and not for security use. Values differ from Node by design; range and uniformity are tested. One matched scenario is registered and verified untimed; measurements wait for the math-functions milestone.; [evidence](./milestones/109-math-random.md) · [math_random_range](../benchmarks/runtime/workloads/math_random_range.tsr) |
| `Math.round` | implemented | paired | Gate48 compiler acceptance passes one 544-group regression and required Rust checks. Actual matched eight-runtime scalar IEEE comparisons and three native allocation/disposal samples pass; results and performance gaps remain recorded.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) · [math_round](../benchmarks/runtime/workloads/math_round.tsr) |
| `Math.sign` | implemented | paired | Gate48 compiler acceptance passes one 544-group regression and required Rust checks. Actual matched eight-runtime scalar IEEE comparisons and three native allocation/disposal samples pass; results and performance gaps remain recorded.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) · [math_sign](../benchmarks/runtime/workloads/math_sign.tsr) |
| `Math.sin` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_sin](../benchmarks/runtime/workloads/math_sin.tsr) |
| `Math.sinh` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_sinh](../benchmarks/runtime/workloads/math_sinh.tsr) |
| `Math.sqrt` | implemented | paired | Standard Rust f64 transform accepted by 515 compiler groups and required checks. Matched whole iterations pass across eight runtimes with exact IEEE validation and allocation/disposal records; table setup, selection, loop, and accumulation costs included.; [evidence](./milestones/41-math-unary.md) · [math_sqrt](../benchmarks/runtime/workloads/math_sqrt.tsr) |
| `Math.SQRT1_2` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.SQRT2` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `Math.tan` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_tan](../benchmarks/runtime/workloads/math_tan.tsr) |
| `Math.tanh` | implemented | paired | Standard Rust f64 methods; typed operands evaluate once in source order. One 549-group regression and independent bounded numerical validation across eight runtimes pass. Matched execution-only comparison and zero-allocation/disposal profiles accepted; precision is platform dependent.; [evidence](./milestones/49-native-math-functions.md) · [math_tanh](../benchmarks/runtime/workloads/math_tanh.tsr) |
| `Math.trunc` | implemented | paired | Standard Rust f64 transform accepted by 515 compiler groups and required checks. Matched whole iterations pass across eight runtimes with exact IEEE validation and allocation/disposal records; table setup, selection, loop, and accumulation costs included.; [evidence](./milestones/41-math-unary.md) · [math_trunc](../benchmarks/runtime/workloads/math_trunc.tsr) |
| `number.add` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_add](../benchmarks/runtime/workloads/number_add.tsr) |
| `number.addAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_add](../benchmarks/runtime/workloads/compound_number_add.tsr) |
| `number.addAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_add](../benchmarks/runtime/workloads/compound_value_number_add.tsr) |
| `number.assign` | implemented | compound | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Covered within arithmetic workloads; no isolated folded scalar throughput claim.; [evidence](./milestones/39-native-number.md) · [number_add](../benchmarks/runtime/workloads/number_add.tsr) |
| `number.assignmentExpression` | native variant | paired group | Ordinary = returns the committed copy scalar across supported expression contexts, statements and for clauses; target type drives RHS contextual typing with strict matching. Target/indices and bounds occur once before RHS, no old target value is read, and commit follows successful RHS. Preserve incoming/parent loans, unconditional ownership effects and existing target/capture/parameter/iteration/partial-field restrictions. Exact Number IEEE bits, including signed zero and nonfinite values, are preserved. Standard Rust scalar assignment allocates no heap storage in focused evidence. Actual JavaScript comparisons retain expression and final target outputs; dynamic setup, selection, conversion and complete output retention are timed. Owned and arithmetic compound assignment values remain pending.; [evidence](./milestones/67-scalar-assignment-values.md) · [number_assignment_value](../benchmarks/runtime/workloads/number_assignment_value.tsr) |
| `number.bitAnd` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_and](../benchmarks/runtime/workloads/number_and.tsr) |
| `number.bitAndAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_and](../benchmarks/runtime/workloads/compound_number_and.tsr) |
| `number.bitAndAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_and](../benchmarks/runtime/workloads/compound_value_number_and.tsr) |
| `number.bitNot` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_not](../benchmarks/runtime/workloads/number_not.tsr) |
| `number.bitOr` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_or](../benchmarks/runtime/workloads/number_or.tsr) |
| `number.bitOrAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_or](../benchmarks/runtime/workloads/compound_number_or.tsr) |
| `number.bitOrAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_or](../benchmarks/runtime/workloads/compound_value_number_or.tsr) |
| `number.bitXor` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_xor](../benchmarks/runtime/workloads/number_xor.tsr) |
| `number.bitXorAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_xor](../benchmarks/runtime/workloads/compound_number_xor.tsr) |
| `number.bitXorAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_xor](../benchmarks/runtime/workloads/compound_value_number_xor.tsr) |
| `number.compoundAssignment` | implemented | paired group | Umbrella for the twelve individually tracked Number compound statements, each with a named retained IEEE comparison. Gate68 separately accepts all twelve value-producing forms and their matched comparisons. Strict-boolean logical assignment and numeric updates have separate Gate66/Gate64 acceptance; owned compound values and nullish operations remain pending.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_add](../benchmarks/runtime/workloads/compound_number_add.tsr) · [compound_number_subtract](../benchmarks/runtime/workloads/compound_number_subtract.tsr) · [compound_number_multiply](../benchmarks/runtime/workloads/compound_number_multiply.tsr) · [compound_number_divide](../benchmarks/runtime/workloads/compound_number_divide.tsr) · [compound_number_remainder](../benchmarks/runtime/workloads/compound_number_remainder.tsr) · [compound_number_power](../benchmarks/runtime/workloads/compound_number_power.tsr) · [compound_number_and](../benchmarks/runtime/workloads/compound_number_and.tsr) · [compound_number_or](../benchmarks/runtime/workloads/compound_number_or.tsr) · [compound_number_xor](../benchmarks/runtime/workloads/compound_number_xor.tsr) · [compound_number_shift_left](../benchmarks/runtime/workloads/compound_number_shift_left.tsr) · [compound_number_shift_right](../benchmarks/runtime/workloads/compound_number_shift_right.tsr) · [compound_number_shift_right_unsigned](../benchmarks/runtime/workloads/compound_number_shift_right_unsigned.tsr) |
| `number.copy` | implemented | compound | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Covered within arithmetic workloads; no isolated folded scalar throughput claim.; [evidence](./milestones/39-native-number.md) · [number_add](../benchmarks/runtime/workloads/number_add.tsr) |
| `number.decrement` | implemented | paired group | Umbrella for separately accepted number prefix/postfix decrement. Both actual forms have matched retained-result comparisons and independent semantics, storage, diagnostics and disposal evidence.; [evidence](./milestones/64-update-expressions.md) · [update_number_prefix_decrement](../benchmarks/runtime/workloads/update_number_prefix_decrement.tsr) · [update_number_postfix_decrement](../benchmarks/runtime/workloads/update_number_postfix_decrement.tsr) |
| `number.divide` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_divide](../benchmarks/runtime/workloads/number_divide.tsr) |
| `number.divideAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_divide](../benchmarks/runtime/workloads/compound_number_divide.tsr) |
| `number.divideAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_divide](../benchmarks/runtime/workloads/compound_value_number_divide.tsr) |
| `number.equal` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_equal](../benchmarks/runtime/workloads/number_equal.tsr) |
| `number.exponent` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_exponent](../benchmarks/runtime/workloads/number_exponent.tsr) |
| `number.exponentAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_power](../benchmarks/runtime/workloads/compound_number_power.tsr) |
| `number.exponentAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_power](../benchmarks/runtime/workloads/compound_value_number_power.tsr) |
| `number.floatRounding` | implemented | missing | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests.; [evidence](./milestones/39-native-number.md) |
| `number.greater` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_greater](../benchmarks/runtime/workloads/number_greater.tsr) |
| `number.greaterEqual` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_greater_equal](../benchmarks/runtime/workloads/number_greater_equal.tsr) |
| `number.increment` | implemented | paired group | Umbrella for separately accepted number prefix/postfix increment. Both actual forms have matched retained-result comparisons and independent semantics, storage, diagnostics and disposal evidence.; [evidence](./milestones/64-update-expressions.md) · [update_number_prefix_increment](../benchmarks/runtime/workloads/update_number_prefix_increment.tsr) · [update_number_postfix_increment](../benchmarks/runtime/workloads/update_number_postfix_increment.tsr) |
| `number.infinity` | implemented | missing | Positive and negative infinity values are implemented through Number.POSITIVE_INFINITY/NEGATIVE_INFINITY and IEEE arithmetic. Constant bits and native boundary checks pass; this does not imply a global Infinity identifier or isolated throughput comparison.; [evidence](./milestones/39-native-number.md) · [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `number.intConversion` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_from_int](../benchmarks/runtime/workloads/number_from_int.tsr) |
| `number.less` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_less](../benchmarks/runtime/workloads/number_less.tsr) |
| `number.lessEqual` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_less_equal](../benchmarks/runtime/workloads/number_less_equal.tsr) |
| `number.literal` | implemented | compound | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Covered within arithmetic workloads; no isolated folded scalar throughput claim. Gate160 adds separators and radix spelling with preserved defaults/contextual conversion; matched result scenario verified untimed, without isolated scanner measurement.; [evidence](./milestones/39-native-number.md) · [evidence](./milestones/160-numeric-literals.md) · [number_add](../benchmarks/runtime/workloads/number_add.tsr) · [numeric_literals](../benchmarks/runtime/workloads/numeric_literals.tsr) |
| `number.multiply` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_multiply](../benchmarks/runtime/workloads/number_multiply.tsr) |
| `number.multiplyAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_multiply](../benchmarks/runtime/workloads/compound_number_multiply.tsr) |
| `number.multiplyAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_multiply](../benchmarks/runtime/workloads/compound_value_number_multiply.tsr) |
| `number.NaN` | implemented | missing | NaN values are implemented through Number.NaN, invalid Number(string) conversion and IEEE arithmetic. Exact constant classification and native boundary checks pass; this does not imply a global NaN identifier or isolated throughput comparison.; [evidence](./milestones/39-native-number.md) · [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `number.negate` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_negate](../benchmarks/runtime/workloads/number_negate.tsr) |
| `number.notEqual` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_not_equal](../benchmarks/runtime/workloads/number_not_equal.tsr) |
| `number.postfixDecrement` | implemented | paired group | Actual postfix decrement in all existing expression contexts with once-selected targets and old/read/compute/commit order. Ordinary f64 arithmetic; signed-zero saved-old and 2^53 rounding cases retain exact IEEE observation. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_number_postfix_decrement](../benchmarks/runtime/workloads/update_number_postfix_decrement.tsr) |
| `number.postfixIncrement` | implemented | paired group | Actual postfix increment in all existing expression contexts with once-selected targets and old/read/compute/commit order. Ordinary f64 arithmetic; signed-zero saved-old and 2^53 rounding cases retain exact IEEE observation. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_number_postfix_increment](../benchmarks/runtime/workloads/update_number_postfix_increment.tsr) |
| `number.prefixDecrement` | implemented | paired group | Actual prefix decrement in all existing expression contexts with once-selected targets and old/read/compute/commit order. Ordinary f64 arithmetic; signed-zero saved-old and 2^53 rounding cases retain exact IEEE observation. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_number_prefix_decrement](../benchmarks/runtime/workloads/update_number_prefix_decrement.tsr) |
| `number.prefixIncrement` | implemented | paired group | Actual prefix increment in all existing expression contexts with once-selected targets and old/read/compute/commit order. Ordinary f64 arithmetic; signed-zero saved-old and 2^53 rounding cases retain exact IEEE observation. Existing captures, loans and storage restrictions remain enforced. Independent complete expression/committed outputs and source-order evidence pass. Output construction/retention is timed; output storage costs are separate from allocation-free scalar updates.; [evidence](./milestones/64-update-expressions.md) · [update_number_prefix_increment](../benchmarks/runtime/workloads/update_number_prefix_increment.tsr) |
| `number.remainder` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_remainder](../benchmarks/runtime/workloads/number_remainder.tsr) |
| `number.remainderAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_remainder](../benchmarks/runtime/workloads/compound_number_remainder.tsr) |
| `number.remainderAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_remainder](../benchmarks/runtime/workloads/compound_value_number_remainder.tsr) |
| `number.safeIntegerBoundary` | implemented | missing | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests.; [evidence](./milestones/39-native-number.md) |
| `number.shiftLeft` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_shift_left](../benchmarks/runtime/workloads/number_shift_left.tsr) |
| `number.shiftLeftAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_shift_left](../benchmarks/runtime/workloads/compound_number_shift_left.tsr) |
| `number.shiftLeftAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_shift_left](../benchmarks/runtime/workloads/compound_value_number_shift_left.tsr) |
| `number.shiftRight` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_shift_right](../benchmarks/runtime/workloads/number_shift_right.tsr) |
| `number.shiftRightAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_shift_right](../benchmarks/runtime/workloads/compound_number_shift_right.tsr) |
| `number.shiftRightAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_shift_right](../benchmarks/runtime/workloads/compound_value_number_shift_right.tsr) |
| `number.shiftRightUnsigned` | implemented | paired | Standard Rust Number lowering passes one 553-group regression and O0/O3 edge/effect checks. Eight-runtime comparisons and zero-allocation profiles pass. Exponentiation retains the independently referenced bounded approximate protocol; native precision remains platform dependent.; [evidence](./milestones/50-number-operators.md) · [number_shift_right_unsigned](../benchmarks/runtime/workloads/number_shift_right_unsigned.tsr) |
| `number.shiftRightUnsignedAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_shift_right_unsigned](../benchmarks/runtime/workloads/compound_number_shift_right_unsigned.tsr) |
| `number.shiftRightUnsignedAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_shift_right_unsigned](../benchmarks/runtime/workloads/compound_value_number_shift_right_unsigned.tsr) |
| `number.signedZero` | implemented | missing | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests.; [evidence](./milestones/39-native-number.md) |
| `number.stringConversion` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_parse_decimal](../benchmarks/runtime/workloads/number_parse_decimal.tsr) |
| `number.subnormal` | implemented | missing | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests.; [evidence](./milestones/39-native-number.md) |
| `number.subtract` | implemented | paired | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Matched whole-iteration comparison passes across eight runtimes; setup and observation costs are disclosed.; [evidence](./milestones/39-native-number.md) · [number_subtract](../benchmarks/runtime/workloads/number_subtract.tsr) |
| `number.subtractAssign` | implemented | paired group | Actual Number compound under the existing IEEE arithmetic/conversion contract. 609 distinct regression groups and matched complete number[] outputs pass. Runtime operand setup, selection and retention are counted. Canonical NaN, signed zero/infinity and shift-count boundaries have independent correctness evidence. Power timing covers exact positive integer bases/exponents 1 through 8, without an approximate-domain claim. Raw normalized resources and absolute zero-disposal checks are retained.; [evidence](./milestones/63-compound-assignment.md) · [compound_number_subtract](../benchmarks/runtime/workloads/compound_number_subtract.tsr) |
| `number.subtractAssignExpression` | native variant | paired group | Compound assignment returns the exact committed numeric scalar in supported expression contexts, statements and for clauses, with strict target-directed RHS typing and right-associated chains. Target/index selection, bounds and copied old value precede RHS; successful standard Rust computation precedes commit. Preserve incoming/ancestor/index-evaluation loans, destination reacquisition, capture/parameter/iteration and partial-field rules. Existing IEEE f64 arithmetic and Number 32-bit bitwise/shift/power contract; returned and committed IEEE bits match within native execution, while cross-runtime NaN observations use canonical NaN. Focused scalar operations allocate no heap storage. Actual JavaScript comparisons retain both result and final target; dynamic setup, selection and complete output retention are timed. Owned compound values remain pending.; [evidence](./milestones/68-numeric-compound-assignment-values.md) · [compound_value_number_subtract](../benchmarks/runtime/workloads/compound_value_number_subtract.tsr) |
| `Number.toExponential` | native variant | paired group | Strict number receiver and optional integer precision; full ranges, once-only ordered operands, source failures and normal/unwind disposal accepted. Standard Rust f64 formatting uses native ties-to-even and shortest-digit choices; matched shared-domain comparisons do not claim general JavaScript conversion parity.; [evidence](./milestones/74-number-decimal-formatting.md) · [number_to_exponential_default](../benchmarks/runtime/workloads/number_to_exponential_default.tsr) · [number_to_exponential_2](../benchmarks/runtime/workloads/number_to_exponential_2.tsr) |
| `Number.toFixed` | native variant | paired group | Strict number receiver and optional integer precision; full ranges, once-only ordered operands, source failures and normal/unwind disposal accepted. Standard Rust f64 formatting uses native ties-to-even and shortest-digit choices; matched shared-domain comparisons do not claim general JavaScript conversion parity.; [evidence](./milestones/74-number-decimal-formatting.md) · [number_to_fixed_default](../benchmarks/runtime/workloads/number_to_fixed_default.tsr) · [number_to_fixed_2](../benchmarks/runtime/workloads/number_to_fixed_2.tsr) |
| `Number.toLocaleString` | native variant | paired | No arguments; fixed comma grouping and period decimal separator, up to three fractional digits with shortest-decimal ties away from zero. Signed zero and infinity text are preserved. Native integers retain all i64 digits. Locale/options arguments are rejected. One matched scenario is verified untimed; measurements remain pending.; [evidence](./milestones/114-number-locale-string.md) · [number_locale_string_decimal](../benchmarks/runtime/workloads/number_locale_string_decimal.tsr) |
| `Number.toPrecision` | native variant | paired group | Strict number receiver and optional integer precision; full ranges, once-only ordered operands, source failures and normal/unwind disposal accepted. Standard Rust f64 formatting uses native ties-to-even and shortest-digit choices; matched shared-domain comparisons do not claim general JavaScript conversion parity.; [evidence](./milestones/74-number-decimal-formatting.md) · [number_to_precision_default](../benchmarks/runtime/workloads/number_to_precision_default.tsr) · [number_to_precision_5](../benchmarks/runtime/workloads/number_to_precision_5.tsr) |
| `Number.toString` | implemented | paired | Shortest decimal text by default and int radix 2 through 36 with fraction digits; output matches Node on the reference fixtures. An invalid radix stops the program instead of throwing. One matched scenario is registered and verified untimed; measurements wait for the number-methods milestone.; [evidence](./milestones/106-number-to-string.md) · [number_to_string_radix](../benchmarks/runtime/workloads/number_to_string_radix.tsr) |
| `number.unaryPlus` | implemented | paired group | Standard Rust f64 accepted by 509 compiler groups and required checks; IEEE boundary behavior is covered by native correctness tests. Gate168 accepts scalar unary conversion and scalar unions through Number, preserving existing numeric domains and borrowed once-only sources. The matched seven-observation scenario is verified untimed; no new measurement. Gate169 shares whole-string scalar/union/enum conversion with exact power-of-two radix rounding. Matched retained observations verified untimed; no new timing.; [evidence](./milestones/39-native-number.md) · [evidence](./milestones/168-scalar-unary-plus.md) · [evidence](./milestones/169-numeric-string-conversion.md) · [scalar_unary_plus](../benchmarks/runtime/workloads/scalar_unary_plus.tsr) · [numeric_string_conversion](../benchmarks/runtime/workloads/numeric_string_conversion.tsr) |
| `Number.valueOf` | implemented | paired group | Primitive receiver only, strict zero arity; receiver evaluates once and stays reusable. Number copies preserve native bits including signed zero, subnormal and nonfinite values; comparison normalizes NaN without JavaScript payload identity. Compiler/editor, complete source/native outputs and runtime/disposal acceptance are complete; numeric formatting and boxed objects are separate.; [evidence](./milestones/72-primitive-identity.md) · [number_value_of](../benchmarks/runtime/workloads/number_value_of.tsr) |
| `NumberConstructor.call()` | implemented | compound | Primitive Number() and Number(int\|number\|boolean\|string) are implemented. Three matched conversion workloads pass across eight runtimes. Empty construction and number identity have native correctness coverage, without fabricated folded throughput. Whole numeric strings follow the supported decimal/radix grammar and exact numeric whitespace; object/array coercion is unsupported. Gate169 shares whole-string scalar/union/enum conversion with exact power-of-two radix rounding. Matched retained observations verified untimed; no new timing.; [evidence](./milestones/39-native-number.md) · [evidence](./milestones/169-numeric-string-conversion.md) · [number_from_int](../benchmarks/runtime/workloads/number_from_int.tsr) · [number_from_boolean](../benchmarks/runtime/workloads/number_from_boolean.tsr) · [number_parse_decimal](../benchmarks/runtime/workloads/number_parse_decimal.tsr) · [numeric_string_conversion](../benchmarks/runtime/workloads/numeric_string_conversion.tsr) |
| `NumberConstructor.EPSILON` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.isFinite` | implemented | paired | Standard Rust f64 predicate accepted by 512 compiler groups and required checks. One number argument is evaluated once. Matched whole-iteration comparison passes across eight runtimes with input table, loop, and digest costs included; allocation/disposal records retained.; [evidence](./milestones/40-number-predicates.md) · [number_is_finite](../benchmarks/runtime/workloads/number_is_finite.tsr) |
| `NumberConstructor.isInteger` | implemented | paired | Standard Rust f64 predicate accepted by 512 compiler groups and required checks. One number argument is evaluated once. Matched whole-iteration comparison passes across eight runtimes with input table, loop, and digest costs included; allocation/disposal records retained.; [evidence](./milestones/40-number-predicates.md) · [number_is_integer](../benchmarks/runtime/workloads/number_is_integer.tsr) |
| `NumberConstructor.isNaN` | implemented | paired | Standard Rust f64 predicate accepted by 512 compiler groups and required checks. One number argument is evaluated once. Matched whole-iteration comparison passes across eight runtimes with input table, loop, and digest costs included; allocation/disposal records retained.; [evidence](./milestones/40-number-predicates.md) · [number_is_nan](../benchmarks/runtime/workloads/number_is_nan.tsr) |
| `NumberConstructor.isSafeInteger` | implemented | paired | Standard Rust f64 predicate accepted by 512 compiler groups and required checks. One number argument is evaluated once. Matched whole-iteration comparison passes across eight runtimes with input table, loop, and digest costs included; allocation/disposal records retained.; [evidence](./milestones/40-number-predicates.md) · [number_is_safe_integer](../benchmarks/runtime/workloads/number_is_safe_integer.tsr) |
| `NumberConstructor.MAX_SAFE_INTEGER` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.MAX_VALUE` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.MIN_SAFE_INTEGER` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.MIN_VALUE` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.NaN` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.NEGATIVE_INFINITY` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.new()` | pending | missing |  |
| `NumberConstructor.parseFloat` | implemented | paired group | Exactly one borrowed string argument; ECMAScript leading whitespace and longest signed decimal/Infinity prefix, incomplete-exponent rollback, invalid NaN, signed zero and standard Rust f64 rounding. Once-only evaluation, contexts, source diagnostics and normal/unwind disposal pass. Six complete-result comparisons; no implicit coercion or global alias.; [evidence](./milestones/75-number-prefix-parsing.md) · [number_parse_float_decimal](../benchmarks/runtime/workloads/number_parse_float_decimal.tsr) · [number_parse_float_exponent](../benchmarks/runtime/workloads/number_parse_float_exponent.tsr) · [number_parse_float_whitespace](../benchmarks/runtime/workloads/number_parse_float_whitespace.tsr) · [number_parse_float_trailing](../benchmarks/runtime/workloads/number_parse_float_trailing.tsr) · [number_parse_float_nonfinite_invalid](../benchmarks/runtime/workloads/number_parse_float_nonfinite_invalid.tsr) · [number_parse_float_mixed](../benchmarks/runtime/workloads/number_parse_float_mixed.tsr) |
| `NumberConstructor.parseInt` | implemented | paired | Borrowed string and optional int radix 2 through 36 with 0x detection; result is number and NaN when no digit is read. Cases checked against Node. One matched scenario is registered and verified untimed; measurements wait for the number-methods milestone.; [evidence](./milestones/105-number-parse-int.md) · [number_parse_int_radix](../benchmarks/runtime/workloads/number_parse_int_radix.tsr) |
| `NumberConstructor.POSITIVE_INFINITY` | implemented | compile-only | Gate48 exact constant-bit semantics pass at O0/O3 and within the single full regression. Constants are compile-only coverage; constant-folded throughput would not measure a meaningful runtime operation.; [evidence](./milestones/48-math-rounding-and-integer-functions.md) |
| `NumberConstructor.prototype` | pending | missing |  |

## Objects

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `CallableFunction.apply` | pending | missing |  |
| `CallableFunction.bind` | pending | missing |  |
| `CallableFunction.call` | pending | missing |  |
| `Function.[Symbol.hasInstance]` | pending | missing |  |
| `Function.[Symbol.metadata]` | pending | missing | esnext declaration; availability unverified |
| `Function.apply` | pending | missing |  |
| `Function.arguments` | pending | missing |  |
| `Function.bind` | pending | missing |  |
| `Function.call` | pending | missing |  |
| `Function.caller` | pending | missing |  |
| `Function.length` | pending | missing |  |
| `Function.name` | pending | missing |  |
| `Function.prototype` | pending | missing |  |
| `Function.toString` | pending | missing |  |
| `FunctionConstructor.call()` | pending | missing |  |
| `FunctionConstructor.new()` | pending | missing |  |
| `FunctionConstructor.prototype` | pending | missing |  |
| `NewableFunction.apply` | pending | missing |  |
| `NewableFunction.bind` | pending | missing |  |
| `NewableFunction.call` | pending | missing |  |
| `object.computedProperty` | pending | missing |  |
| `Object.constructor` | pending | missing |  |
| `object.delete` | pending | missing |  |
| `Object.hasOwnProperty` | pending | missing |  |
| `object.identity` | pending | missing |  |
| `object.in` | native variant | paired | Own-key membership on fixed records, tuples and dense arrays; string/int/number keys, once-only key-before-value evaluation. Inherited properties are excluded. Matched scenario verified untimed; measurements pending.; [evidence](./milestones/139-in-operator.md) · [evidence](./in-operator.md) · [in_operator](../benchmarks/runtime/workloads/in_operator.tsr) |
| `object.instanceof` | pending | missing |  |
| `Object.isPrototypeOf` | pending | missing |  |
| `object.propertyEnumeration` | pending | missing |  |
| `Object.propertyIsEnumerable` | pending | missing |  |
| `object.prototypeMutation` | policy | missing | Original vision rejects unrestricted escape hatches; track explicit policy instead of silently omitting. |
| `Object.toLocaleString` | pending | missing |  |
| `Object.toString` | pending | missing |  |
| `object.typeof` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Nullable unions and dynamic host values remain unsupported.; [evidence](./milestones/value-types-and-namespaces.md) |
| `Object.valueOf` | pending | missing |  |
| `ObjectConstructor.assign` | pending | missing |  |
| `ObjectConstructor.call()` | pending | missing |  |
| `ObjectConstructor.create` | pending | missing |  |
| `ObjectConstructor.defineProperties` | pending | missing |  |
| `ObjectConstructor.defineProperty` | pending | missing |  |
| `ObjectConstructor.entries` | implemented | paired | Returns [name, value] pairs in constructor order as independent tuples; mixed field types give a union value type. Records and tuples (and arrays for values); values are copied rather than shared. Strings, primitives, nullable values and records with function fields are unsupported. One matched scenario is registered and verified untimed; measurements wait for the object-functions milestone.; [evidence](./milestones/122-object-values-entries.md) · [object_values_entries](../benchmarks/runtime/workloads/object_values_entries.tsr) |
| `ObjectConstructor.freeze` | pending | missing |  |
| `ObjectConstructor.fromEntries` | pending | missing |  |
| `ObjectConstructor.getOwnPropertyDescriptor` | pending | missing |  |
| `ObjectConstructor.getOwnPropertyDescriptors` | pending | missing |  |
| `ObjectConstructor.getOwnPropertyNames` | pending | missing |  |
| `ObjectConstructor.getOwnPropertySymbols` | pending | missing |  |
| `ObjectConstructor.getPrototypeOf` | pending | missing |  |
| `ObjectConstructor.groupBy` | pending | missing |  |
| `ObjectConstructor.hasOwn` | implemented | paired | Own-property membership for fixed records, tuples, dense arrays, UTF-16 strings and numeric/Boolean receivers. Two required arguments, primitive keys including primitive unions, once left-to-right evaluation, no receiver mutation or clone. Nullable/union/function receivers and object/callable keys are unsupported. Five matched cases verified untimed; object-family measurements remain pending.; [evidence](./milestones/128-object-has-own.md) · [object_has_own](../benchmarks/runtime/workloads/object_has_own.tsr) |
| `ObjectConstructor.is` | implemented | paired | Two borrowed values; numbers use same-value rules (NaN matches NaN, 0 differs from -0), strings compare by content, arrays and records by storage identity; callables rejected. Cases checked against Node. One matched scenario is registered and verified untimed; measurements wait for the object-functions milestone.; [evidence](./milestones/107-object-is.md) · [object_is_values](../benchmarks/runtime/workloads/object_is_values.tsr) |
| `ObjectConstructor.isExtensible` | pending | missing |  |
| `ObjectConstructor.isFrozen` | pending | missing |  |
| `ObjectConstructor.isSealed` | pending | missing |  |
| `ObjectConstructor.keys` | implemented | paired | Returns independent own enumerable names for fixed records in each constructor literal order, arrays/tuples in position order, strings in UTF-16 code-unit position order, and empty results for numbers/booleans. Record clones and field replacement preserve order. Dynamic/quoted/numeric/symbol record keys, prototypes, functions and nullable receivers remain unsupported. One matched scenario passes untimed native/Node-after-tsc checks; object-family measurements remain pending.; [evidence](./milestones/119-object-keys.md) · [object_keys_order](../benchmarks/runtime/workloads/object_keys_order.tsr) |
| `ObjectConstructor.new()` | pending | missing |  |
| `ObjectConstructor.preventExtensions` | pending | missing |  |
| `ObjectConstructor.prototype` | pending | missing |  |
| `ObjectConstructor.seal` | pending | missing |  |
| `ObjectConstructor.setPrototypeOf` | pending | missing |  |
| `ObjectConstructor.values` | implemented | paired | Returns field values in constructor order as an independent array; mixed field types give a union element type. Records and tuples (and arrays for values); values are copied rather than shared. Strings, primitives, nullable values and records with function fields are unsupported. One matched scenario is registered and verified untimed; measurements wait for the object-functions milestone.; [evidence](./milestones/122-object-values-entries.md) · [object_values_entries](../benchmarks/runtime/workloads/object_values_entries.tsr) |
| `ProxyConstructor.new()` | pending | missing |  |
| `ProxyConstructor.revocable` | pending | missing |  |
| `ProxyHandler.apply` | pending | missing |  |
| `ProxyHandler.construct` | pending | missing |  |
| `ProxyHandler.defineProperty` | pending | missing |  |
| `ProxyHandler.deleteProperty` | pending | missing |  |
| `ProxyHandler.get` | pending | missing |  |
| `ProxyHandler.getOwnPropertyDescriptor` | pending | missing |  |
| `ProxyHandler.getPrototypeOf` | pending | missing |  |
| `ProxyHandler.has` | pending | missing |  |
| `ProxyHandler.isExtensible` | pending | missing |  |
| `ProxyHandler.ownKeys` | pending | missing |  |
| `ProxyHandler.preventExtensions` | pending | missing |  |
| `ProxyHandler.set` | pending | missing |  |
| `ProxyHandler.setPrototypeOf` | pending | missing |  |
| `record.clone` | native variant | compound | [evidence](./milestones/31-record-field-moves.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [field_restore](../benchmarks/runtime/workloads/field_restore.tsr) · [recursive_clone](../benchmarks/runtime/workloads/recursive_clone.tsr) |
| `record.literal` | native variant | compound | [evidence](./milestones/31-record-field-moves.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [field_restore](../benchmarks/runtime/workloads/field_restore.tsr) · [recursive_clone](../benchmarks/runtime/workloads/recursive_clone.tsr) |
| `record.moveField` | native variant | compound | [evidence](./milestones/31-record-field-moves.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [field_restore](../benchmarks/runtime/workloads/field_restore.tsr) · [recursive_clone](../benchmarks/runtime/workloads/recursive_clone.tsr) |
| `record.readField` | native variant | compound | [evidence](./milestones/31-record-field-moves.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [field_restore](../benchmarks/runtime/workloads/field_restore.tsr) · [recursive_clone](../benchmarks/runtime/workloads/recursive_clone.tsr) |
| `record.recursiveLayout` | native variant | compound | [evidence](./milestones/31-record-field-moves.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [field_restore](../benchmarks/runtime/workloads/field_restore.tsr) · [recursive_clone](../benchmarks/runtime/workloads/recursive_clone.tsr) |
| `record.restoreField` | native variant | compound | [evidence](./milestones/31-record-field-moves.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [field_restore](../benchmarks/runtime/workloads/field_restore.tsr) · [recursive_clone](../benchmarks/runtime/workloads/recursive_clone.tsr) |
| `record.writeField` | native variant | compound | [evidence](./milestones/31-record-field-moves.md) · [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [field_restore](../benchmarks/runtime/workloads/field_restore.tsr) · [recursive_clone](../benchmarks/runtime/workloads/recursive_clone.tsr) |
| `Reflect.apply` | pending | missing |  |
| `Reflect.construct` | pending | missing |  |
| `Reflect.defineProperty` | pending | missing |  |
| `Reflect.deleteProperty` | pending | missing |  |
| `Reflect.get` | pending | missing |  |
| `Reflect.getOwnPropertyDescriptor` | pending | missing |  |
| `Reflect.getPrototypeOf` | pending | missing |  |
| `Reflect.has` | pending | missing |  |
| `Reflect.isExtensible` | pending | missing |  |
| `Reflect.ownKeys` | pending | missing |  |
| `Reflect.preventExtensions` | pending | missing |  |
| `Reflect.set` | pending | missing |  |
| `Reflect.setPrototypeOf` | pending | missing |  |

## Primitive storage

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `storage.bigint` | pending | missing |  |
| `storage.boolean` | native variant | compound | Normalized allocation probe also exists; heap metrics exclude stack/static storage.; [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) · [allocation: boolean_value](../benchmarks/runtime/allocations/boolean_value.tsr) |
| `storage.booleanArray` | native variant | compound | Normalized allocation probe also exists; heap metrics exclude stack/static storage.; [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [allocation: boolean_array](../benchmarks/runtime/allocations/boolean_array.tsr) |
| `storage.int` | native variant | compound | Normalized allocation probe also exists; heap metrics exclude stack/static storage.; [scalar_control](../benchmarks/runtime/workloads/scalar_control.tsr) · [allocation: int_value](../benchmarks/runtime/allocations/int_value.tsr) |
| `storage.intArray` | native variant | compound | Normalized allocation probe also exists; heap metrics exclude stack/static storage.; [scalar_arrays](../benchmarks/runtime/workloads/scalar_arrays.tsr) · [allocation: int_array](../benchmarks/runtime/allocations/int_array.tsr) |
| `storage.null` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Nullable unions and dynamic host values remain unsupported.; [evidence](./milestones/value-types-and-namespaces.md) |
| `storage.number` | native variant | compound | Native f64 is 8 bytes and Copy. Normalized number-value probe exists; allocation counters exclude stack and static storage.; [evidence](./milestones/39-native-number.md) · [evidence](./milestones/45-fixed-number-arrays.md) · [number_add](../benchmarks/runtime/workloads/number_add.tsr) · [allocation: number_value](../benchmarks/runtime/allocations/number_value.tsr) |
| `storage.numberArray` | native variant | compound | Proved fixed f64 arrays and ordinary escaping/growing vectors. Whole-operation allocation/disposal profiles exist; a separate normalized number-array memory probe remains missing.; [evidence](./milestones/39-native-number.md) · [evidence](./milestones/45-fixed-number-arrays.md) · [number_fixed_local_operations](../benchmarks/runtime/workloads/number_fixed_local_operations.tsr) · [number_array_operations](../benchmarks/runtime/workloads/number_array_operations.tsr) |
| `storage.record` | native variant | compound | Normalized allocation probe also exists; heap metrics exclude stack/static storage.; [record_mutation](../benchmarks/runtime/workloads/record_mutation.tsr) · [allocation: record_value](../benchmarks/runtime/allocations/record_value.tsr) |
| `storage.string` | native variant | compound | Normalized allocation probe also exists; heap metrics exclude stack/static storage.; [dynamic_string_clone](../benchmarks/runtime/workloads/dynamic_string_clone.tsr) · [allocation: string_value](../benchmarks/runtime/allocations/string_value.tsr) |
| `storage.stringArray` | native variant | compound | Normalized allocation probe also exists; heap metrics exclude stack/static storage.; [owned_arrays](../benchmarks/runtime/workloads/owned_arrays.tsr) · [allocation: string_array](../benchmarks/runtime/allocations/string_array.tsr) |
| `storage.symbol` | pending | missing |  |
| `storage.undefined` | implemented | compile-only | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Nullable unions and dynamic host values remain unsupported.; [evidence](./milestones/value-types-and-namespaces.md) |

## Regular expressions

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `RegExp.[Symbol.match]` | pending | missing |  |
| `RegExp.[Symbol.matchAll]` | pending | missing |  |
| `RegExp.[Symbol.replace]` | pending | missing |  |
| `RegExp.[Symbol.search]` | pending | missing |  |
| `RegExp.[Symbol.split]` | pending | missing |  |
| `RegExp.compile` | pending | missing |  |
| `RegExp.dotAll` | pending | missing |  |
| `RegExp.exec` | pending | missing |  |
| `RegExp.flags` | pending | missing |  |
| `RegExp.global` | pending | missing |  |
| `RegExp.hasIndices` | pending | missing |  |
| `RegExp.ignoreCase` | pending | missing |  |
| `RegExp.lastIndex` | pending | missing |  |
| `RegExp.multiline` | pending | missing |  |
| `RegExp.source` | pending | missing |  |
| `RegExp.sticky` | pending | missing |  |
| `RegExp.test` | pending | missing |  |
| `RegExp.unicode` | pending | missing |  |
| `RegExp.unicodeSets` | pending | missing |  |
| `RegExpConstructor."$_"` | pending | missing |  |
| `RegExpConstructor."$'"` | pending | missing |  |
| `RegExpConstructor."$&"` | pending | missing |  |
| `RegExpConstructor."$`"` | pending | missing |  |
| `RegExpConstructor."$+"` | pending | missing |  |
| `RegExpConstructor."$1"` | pending | missing |  |
| `RegExpConstructor."$2"` | pending | missing |  |
| `RegExpConstructor."$3"` | pending | missing |  |
| `RegExpConstructor."$4"` | pending | missing |  |
| `RegExpConstructor."$5"` | pending | missing |  |
| `RegExpConstructor."$6"` | pending | missing |  |
| `RegExpConstructor."$7"` | pending | missing |  |
| `RegExpConstructor."$8"` | pending | missing |  |
| `RegExpConstructor."$9"` | pending | missing |  |
| `RegExpConstructor."input"` | pending | missing |  |
| `RegExpConstructor."lastMatch"` | pending | missing |  |
| `RegExpConstructor."lastParen"` | pending | missing |  |
| `RegExpConstructor."leftContext"` | pending | missing |  |
| `RegExpConstructor."prototype"` | pending | missing |  |
| `RegExpConstructor."rightContext"` | pending | missing |  |
| `RegExpConstructor.[Symbol.species]` | pending | missing |  |
| `RegExpConstructor.call()` | pending | missing |  |
| `RegExpConstructor.new()` | pending | missing |  |
| `RegExpExecArray.0` | pending | missing |  |
| `RegExpExecArray.groups` | pending | missing |  |
| `RegExpExecArray.index` | pending | missing |  |
| `RegExpExecArray.indices` | pending | missing |  |
| `RegExpExecArray.input` | pending | missing |  |
| `RegExpIndicesArray.groups` | pending | missing |  |
| `RegExpMatchArray.0` | pending | missing |  |
| `RegExpMatchArray.groups` | pending | missing |  |
| `RegExpMatchArray.index` | pending | missing |  |
| `RegExpMatchArray.indices` | pending | missing |  |
| `RegExpMatchArray.input` | pending | missing |  |

## Resource management

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `AsyncDisposable.[Symbol.asyncDispose]` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.[Symbol.asyncDispose]` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.[Symbol.toStringTag]` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.adopt` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.defer` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.disposeAsync` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.disposed` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.move` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStack.use` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStackConstructor.new()` | pending | missing | esnext declaration; availability unverified |
| `AsyncDisposableStackConstructor.prototype` | pending | missing | esnext declaration; availability unverified |
| `Disposable.[Symbol.dispose]` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.[Symbol.dispose]` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.[Symbol.toStringTag]` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.adopt` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.defer` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.dispose` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.disposed` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.move` | pending | missing | esnext declaration; availability unverified |
| `DisposableStack.use` | pending | missing | esnext declaration; availability unverified |
| `DisposableStackConstructor.new()` | pending | missing | esnext declaration; availability unverified |
| `DisposableStackConstructor.prototype` | pending | missing | esnext declaration; availability unverified |

## Serialization

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `JSON.[Symbol.toStringTag]` | pending | missing |  |
| `JSON.parse` | pending | missing |  |
| `JSON.stringify` | pending | missing |  |

## Strings

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `String.[index]` | native variant | paired | text[position] gives the character at a Unicode character position as a new string. A missing position is a catchable range error, like an array index; JavaScript gives undefined. Writes through a position are rejected; the optional form gives undefined for an absent text. One matched scenario verified untimed.; [evidence](./milestones/164-string-indexing.md) · [evidence](./string-indexing.md) · [string_indexing](../benchmarks/runtime/workloads/string_indexing.tsr) |
| `String.[Symbol.iterator]` | pending | missing |  |
| `string.addAssign` | implemented | paired group | Matching String/String += reads an available mutable target before RHS evaluation, borrows both operands and commits a fresh independent owned result. Self-append, source order, existing loans and normal/unwind disposal pass. Separate ASCII/Unicode comparisons time cloning/repetition, target initialization, actual append and complete string[] retention. Native ownership costs differ from possible JavaScript immutable sharing; no allocation-parity claim. Gate167 accepts scalar operands (int, number, boolean, null, undefined and scalar unions) via String conversion, preserving source order and borrowing. Arrays, records and functions require explicit supported conversion. New matched scenario verified untimed; no new timing.; [evidence](./milestones/63-compound-assignment.md) · [evidence](./milestones/167-string-concatenation.md) · [compound_string_add_ascii](../benchmarks/runtime/workloads/compound_string_add_ascii.tsr) · [compound_string_add_unicode](../benchmarks/runtime/workloads/compound_string_add_unicode.tsr) · [string_concatenation](../benchmarks/runtime/workloads/string_concatenation.tsr) |
| `String.anchor` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_anchor](../benchmarks/runtime/workloads/string_html_anchor.tsr) |
| `string.assign` | native variant | compound | Valid UTF-8; ownership transfers; explicit owned clone copies bytes.; [evidence](./string-semantics.md) · [dynamic_string_clone](../benchmarks/runtime/workloads/dynamic_string_clone.tsr) |
| `String.at` | native variant | paired | Required int position counted in Unicode characters from either end; result is string \| undefined. Node counts UTF-16 code units. One matched scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/104-string-at.md) · [string_at_characters](../benchmarks/runtime/workloads/string_at_characters.tsr) |
| `String.big` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_big](../benchmarks/runtime/workloads/string_html_big.tsr) |
| `String.blink` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_blink](../benchmarks/runtime/workloads/string_html_blink.tsr) |
| `String.bold` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_bold](../benchmarks/runtime/workloads/string_html_bold.tsr) |
| `String.charAt` | native variant | paired group | Native byte/scalar or standard Rust Unicode contract.; [evidence](./string-semantics.md) · [string_char_at](../benchmarks/runtime/workloads/string_char_at.tsr) |
| `String.charCodeAt` | native variant | paired | Optional int position; result is number, the first UTF-16 unit of the character, or NaN when no character is there. Positions count Unicode characters; Node counts UTF-16 code units. One matched scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/111-string-code-at.md) · [string_code_at_positions](../benchmarks/runtime/workloads/string_code_at_positions.tsr) |
| `string.clone` | native variant | compound | Valid UTF-8; ownership transfers; explicit owned clone copies bytes.; [evidence](./string-semantics.md) · [dynamic_string_clone](../benchmarks/runtime/workloads/dynamic_string_clone.tsr) |
| `String.codePointAt` | native variant | paired | Optional int position; result is int \| undefined, and a negative or out-of-range position gives undefined. Positions count Unicode characters; Node counts UTF-16 code units. One matched scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/111-string-code-at.md) · [string_code_at_positions](../benchmarks/runtime/workloads/string_code_at_positions.tsr) |
| `string.concat` | implemented | paired group | String addition: exact UTF8 contents, once left-to-right borrowed operands and fresh independent owned result using standard capacity/push_str. Chains construct real owned intermediates. One 605-group regression and four binary/chained ASCII/Unicode comparisons pass with complete result retention and normalized resources. Native copying differs from possible JavaScript immutable sharing. Scalar concatenation is defined conversion, not coercive comparison. The separate String.concat method is accepted in Milestone 70. Matching String/String += has separate Milestone 63 acceptance and comparisons. Gate167 accepts scalar operands (int, number, boolean, null, undefined and scalar unions) via String conversion, preserving source order and borrowing. Arrays, records and functions require explicit supported conversion. New matched scenario verified untimed; no new timing.; [evidence](./milestones/62-string-addition.md) · [evidence](./milestones/167-string-concatenation.md) · [string_add_direct_ascii](../benchmarks/runtime/workloads/string_add_direct_ascii.tsr) · [string_add_direct_unicode](../benchmarks/runtime/workloads/string_add_direct_unicode.tsr) · [string_add_chained_ascii](../benchmarks/runtime/workloads/string_add_chained_ascii.tsr) · [string_add_chained_unicode](../benchmarks/runtime/workloads/string_add_chained_unicode.tsr) · [string_concatenation](../benchmarks/runtime/workloads/string_concatenation.tsr) |
| `String.concat` | implemented | paired group | Strict String operands borrow once left to right; all operands evaluate even on no-ops. Changed construction uses one checked fallible reservation. Standard Cow no-op cloning preserves static borrowed storage and independent owned copies. Padding accepts strict int UTF-8 byte targets, defaults to space and rejects a partial filler cutting a scalar boundary; negative/short targets and empty filler clone before capacity/boundary checks. Ten whole-workload comparisons retain complete String results; Unicode counterpart byte/UTF-16 target tuple setup and selection are timed. JavaScript coercion, storage/allocation parity and isolated method latency are not claimed. Editor completion, hover and signature help are accepted.; [evidence](./milestones/70-string-concat-padding.md) · [string_concat_zero_ascii](../benchmarks/runtime/workloads/string_concat_zero_ascii.tsr) · [string_concat_zero_unicode](../benchmarks/runtime/workloads/string_concat_zero_unicode.tsr) · [string_concat_variadic_ascii](../benchmarks/runtime/workloads/string_concat_variadic_ascii.tsr) · [string_concat_variadic_unicode](../benchmarks/runtime/workloads/string_concat_variadic_unicode.tsr) |
| `String.endsWith` | native variant | paired | Standard Rust UTF-8 search with checked byte positions. Compiler, eight-runner comparisons and allocation/disposal acceptance pass; Unicode mapping costs and timed position subset are explicit.; [evidence](./milestones/37-string-search.md) · [string_ends_with_ascii](../benchmarks/runtime/workloads/string_ends_with_ascii.tsr) · [string_ends_with_unicode](../benchmarks/runtime/workloads/string_ends_with_unicode.tsr) |
| `string.equal` | implemented | paired group | [evidence](./milestones/34-string-comparisons.md) · [string_equality](../benchmarks/runtime/workloads/string_equality.tsr) |
| `string.escapeDecoding` | native variant | compound | Valid UTF-8; ownership transfers; explicit owned clone copies bytes.; [evidence](./string-semantics.md) · [dynamic_string_clone](../benchmarks/runtime/workloads/dynamic_string_clone.tsr) |
| `String.fixed` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_fixed](../benchmarks/runtime/workloads/string_html_fixed.tsr) |
| `String.fontcolor` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_fontcolor](../benchmarks/runtime/workloads/string_html_fontcolor.tsr) |
| `String.fontsize` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_fontsize](../benchmarks/runtime/workloads/string_html_fontsize.tsr) |
| `string.graphemes` | pending | missing |  |
| `string.greater` | native variant | paired group | Rust UTF-8 lexical order; timed counterpart restricted to ASCII.; [evidence](./milestones/34-string-comparisons.md) · [string_ordering_ascii](../benchmarks/runtime/workloads/string_ordering_ascii.tsr) |
| `string.greaterEqual` | native variant | paired group | Rust UTF-8 lexical order; timed counterpart restricted to ASCII.; [evidence](./milestones/34-string-comparisons.md) · [string_ordering_ascii](../benchmarks/runtime/workloads/string_ordering_ascii.tsr) |
| `String.includes` | native variant | paired | Standard Rust UTF-8 search with checked byte positions. Compiler, eight-runner comparisons and allocation/disposal acceptance pass; Unicode mapping costs and timed position subset are explicit.; [evidence](./milestones/37-string-search.md) · [string_includes_ascii](../benchmarks/runtime/workloads/string_includes_ascii.tsr) · [string_includes_unicode](../benchmarks/runtime/workloads/string_includes_unicode.tsr) |
| `String.indexOf` | native variant | paired | Standard Rust UTF-8 search with checked byte positions. Compiler, eight-runner comparisons and allocation/disposal acceptance pass; Unicode mapping costs and timed position subset are explicit.; [evidence](./milestones/37-string-search.md) · [string_index_of_ascii](../benchmarks/runtime/workloads/string_index_of_ascii.tsr) · [string_index_of_unicode](../benchmarks/runtime/workloads/string_index_of_unicode.tsr) |
| `String.isWellFormed` | native variant | paired group | Strict zero-argument borrowed String receiver evaluated once, with effects, traps, loans and temporary disposal preserved. Safe native UTF-8 guarantees a true predicate without a validation scan. No representable lone surrogates or JavaScript coercion. Four focused groups, editor/source/native/runtime and disposal acceptance complete.; [evidence](./milestones/76-native-string-well-formedness.md) · [string_is_well_formed_ascii](../benchmarks/runtime/workloads/string_is_well_formed_ascii.tsr) · [string_is_well_formed_unicode](../benchmarks/runtime/workloads/string_is_well_formed_unicode.tsr) |
| `String.italics` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_italics](../benchmarks/runtime/workloads/string_html_italics.tsr) |
| `string.iteration` | pending | missing |  |
| `String.lastIndexOf` | native variant | paired group | Native UTF8 reverse substring search returns the greatest eligible byte start and preserves overlaps. Positions clamp safely and require character boundaries even with empty needles. Receiver and needle remain borrowed. One 597-group regression, O0/O3 complete outputs, four default/bounded ASCII/Unicode comparisons and twelve fully disposed allocation profiles pass. All byte indices are retained through index + 1 encoding. Unicode TypeScript mapping setup/lookups and bounded support searches are timed; ASCII offsets coincide and call the operation directly.; [evidence](./milestones/60-string-last-index-of.md) · [string_last_index_of_default_ascii](../benchmarks/runtime/workloads/string_last_index_of_default_ascii.tsr) · [string_last_index_of_default_unicode](../benchmarks/runtime/workloads/string_last_index_of_default_unicode.tsr) · [string_last_index_of_bounded_ascii](../benchmarks/runtime/workloads/string_last_index_of_bounded_ascii.tsr) · [string_last_index_of_bounded_unicode](../benchmarks/runtime/workloads/string_last_index_of_bounded_unicode.tsr) |
| `String.length` | native variant | paired group | Native byte/scalar or standard Rust Unicode contract.; [evidence](./string-semantics.md) · [string_length](../benchmarks/runtime/workloads/string_length.tsr) |
| `string.less` | native variant | paired group | Rust UTF-8 lexical order; timed counterpart restricted to ASCII.; [evidence](./milestones/34-string-comparisons.md) · [string_ordering_ascii](../benchmarks/runtime/workloads/string_ordering_ascii.tsr) |
| `string.lessEqual` | native variant | paired group | Rust UTF-8 lexical order; timed counterpart restricted to ASCII.; [evidence](./milestones/34-string-comparisons.md) · [string_ordering_ascii](../benchmarks/runtime/workloads/string_ordering_ascii.tsr) |
| `String.link` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_link](../benchmarks/runtime/workloads/string_html_link.tsr) |
| `string.literal` | native variant | compound | Valid UTF-8; both quote delimiters, strict ordinary escapes, Unicode code points and line continuations decode at compile time. Focused compiler/editor/O0/O3 acceptance passes; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles. Ownership transfers and explicit owned cloning retain native Cow behavior.; [evidence](./string-semantics.md) · [evidence](./milestones/77-quoted-string-literals.md) · [dynamic_string_clone](../benchmarks/runtime/workloads/dynamic_string_clone.tsr) · [quoted_string_literals_ascii](../benchmarks/runtime/workloads/quoted_string_literals_ascii.tsr) · [quoted_string_literals_unicode](../benchmarks/runtime/workloads/quoted_string_literals_unicode.tsr) |
| `String.localeCompare` | pending | missing |  |
| `String.match` | pending | missing |  |
| `String.matchAll` | pending | missing |  |
| `string.move` | native variant | compound | Valid UTF-8; ownership transfers; explicit owned clone copies bytes.; [evidence](./string-semantics.md) · [dynamic_string_clone](../benchmarks/runtime/workloads/dynamic_string_clone.tsr) |
| `String.normalize` | implemented | paired | All four Unicode normalization forms, NFC omission/undefined defaults, optional forms and strict form errors are supported. Complete pinned Unicode tables, composition exclusions, Hangul handling and conformance checks are present. Strings contain valid Unicode scalars. One seven-result scenario is verified untimed; string-family measurements remain pending.; [evidence](./milestones/116-string-normalize.md) · [string_normalize_forms](../benchmarks/runtime/workloads/string_normalize_forms.tsr) |
| `string.notEqual` | implemented | paired group | [evidence](./milestones/34-string-comparisons.md) · [string_equality](../benchmarks/runtime/workloads/string_equality.tsr) |
| `String.padEnd` | native variant | paired group | Strict String operands borrow once left to right; all operands evaluate even on no-ops. Changed construction uses one checked fallible reservation. Standard Cow no-op cloning preserves static borrowed storage and independent owned copies. Padding accepts strict int UTF-8 byte targets, defaults to space and rejects a partial filler cutting a scalar boundary; negative/short targets and empty filler clone before capacity/boundary checks. Ten whole-workload comparisons retain complete String results; Unicode counterpart byte/UTF-16 target tuple setup and selection are timed. JavaScript coercion, storage/allocation parity and isolated method latency are not claimed. Editor completion, hover and signature help are accepted.; [evidence](./milestones/70-string-concat-padding.md) · [string_pad_end_default](../benchmarks/runtime/workloads/string_pad_end_default.tsr) · [string_pad_end_ascii](../benchmarks/runtime/workloads/string_pad_end_ascii.tsr) · [string_pad_end_unicode](../benchmarks/runtime/workloads/string_pad_end_unicode.tsr) |
| `String.padStart` | native variant | paired group | Strict String operands borrow once left to right; all operands evaluate even on no-ops. Changed construction uses one checked fallible reservation. Standard Cow no-op cloning preserves static borrowed storage and independent owned copies. Padding accepts strict int UTF-8 byte targets, defaults to space and rejects a partial filler cutting a scalar boundary; negative/short targets and empty filler clone before capacity/boundary checks. Ten whole-workload comparisons retain complete String results; Unicode counterpart byte/UTF-16 target tuple setup and selection are timed. JavaScript coercion, storage/allocation parity and isolated method latency are not claimed. Editor completion, hover and signature help are accepted.; [evidence](./milestones/70-string-concat-padding.md) · [string_pad_start_default](../benchmarks/runtime/workloads/string_pad_start_default.tsr) · [string_pad_start_ascii](../benchmarks/runtime/workloads/string_pad_start_ascii.tsr) · [string_pad_start_unicode](../benchmarks/runtime/workloads/string_pad_start_unicode.tsr) |
| `String.repeat` | implemented | paired | Standard Rust owned string construction; one full 539-group regression and required Rust checks pass. Three actual TypeScript comparisons per method validate complete content, shape and operand effects across eight runtimes. All native allocation profiles fully dispose storage. Empty-pattern/separator comparisons use the shared BMP domain; native UTF-8 scalar semantics, clone costs and unsupported regex/callback/coercion behavior remain explicit.; [evidence](./milestones/47-string-construction.md) · [string_repeat_plain](../benchmarks/runtime/workloads/string_repeat_plain.tsr) · [string_repeat_unicode](../benchmarks/runtime/workloads/string_repeat_unicode.tsr) · [string_repeat_empty_zero](../benchmarks/runtime/workloads/string_repeat_empty_zero.tsr) |
| `String.replace` | implemented | paired | Standard Rust owned string construction; one full 539-group regression and required Rust checks pass. Three actual TypeScript comparisons per method validate complete content, shape and operand effects across eight runtimes. All native allocation profiles fully dispose storage. Empty-pattern/separator comparisons use the shared BMP domain; native UTF-8 scalar semantics, clone costs and unsupported regex/callback/coercion behavior remain explicit.; [evidence](./milestones/47-string-construction.md) · [string_replace_first_plain](../benchmarks/runtime/workloads/string_replace_first_plain.tsr) · [string_replace_first_tokens](../benchmarks/runtime/workloads/string_replace_first_tokens.tsr) · [string_replace_first_empty](../benchmarks/runtime/workloads/string_replace_first_empty.tsr) |
| `String.replaceAll` | implemented | paired | Standard Rust owned string construction; one full 539-group regression and required Rust checks pass. Three actual TypeScript comparisons per method validate complete content, shape and operand effects across eight runtimes. All native allocation profiles fully dispose storage. Empty-pattern/separator comparisons use the shared BMP domain; native UTF-8 scalar semantics, clone costs and unsupported regex/callback/coercion behavior remain explicit.; [evidence](./milestones/47-string-construction.md) · [string_replace_all_plain](../benchmarks/runtime/workloads/string_replace_all_plain.tsr) · [string_replace_all_tokens](../benchmarks/runtime/workloads/string_replace_all_tokens.tsr) · [string_replace_all_empty](../benchmarks/runtime/workloads/string_replace_all_empty.tsr) |
| `String.search` | pending | missing |  |
| `String.slice` | native variant | paired group | Native byte/scalar or standard Rust Unicode contract.; [evidence](./string-semantics.md) · [string_slice_unicode](../benchmarks/runtime/workloads/string_slice_unicode.tsr) |
| `String.small` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_small](../benchmarks/runtime/workloads/string_html_small.tsr) |
| `String.split` | implemented | paired group | Standard Rust owned string construction; one full 539-group regression and required Rust checks pass. Three actual TypeScript comparisons per method validate complete content, shape and operand effects across eight runtimes. All native allocation profiles fully dispose storage. Empty-pattern/separator comparisons use the shared BMP domain; native UTF-8 scalar semantics, clone costs and unsupported regex/callback/coercion behavior remain explicit. Gate79 preserves wrapped limits and growable arrays while static Cow pieces now borrow safely; owned pieces remain independent. Complete split IntelliSense documentation, five native examples and focused ownership/disposal checks pass. Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/47-string-construction.md) · [evidence](./milestones/79-split-storage-and-documentation.md) · [string_split_normal](../benchmarks/runtime/workloads/string_split_normal.tsr) · [string_split_limits](../benchmarks/runtime/workloads/string_split_limits.tsr) · [string_split_empty_delimiter](../benchmarks/runtime/workloads/string_split_empty_delimiter.tsr) · [string_cow_split_pieces](../benchmarks/runtime/workloads/string_cow_split_pieces.tsr) · [string_scalar_split_unicode](../benchmarks/runtime/workloads/string_scalar_split_unicode.tsr) |
| `String.startsWith` | native variant | paired | Standard Rust UTF-8 search with checked byte positions. Compiler, eight-runner comparisons and allocation/disposal acceptance pass; Unicode mapping costs and timed position subset are explicit.; [evidence](./milestones/37-string-search.md) · [string_starts_with_ascii](../benchmarks/runtime/workloads/string_starts_with_ascii.tsr) · [string_starts_with_unicode](../benchmarks/runtime/workloads/string_starts_with_unicode.tsr) |
| `String.strike` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_strike](../benchmarks/runtime/workloads/string_html_strike.tsr) |
| `String.sub` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_sub](../benchmarks/runtime/workloads/string_html_sub.tsr) |
| `String.substr` | implemented | paired group | Relative UTF8 byte start clamps safely; optional strict int length caps remaining bytes, with nonpositive empty ranges. Full/partial static and owned Cow storage, ordered loans, UTF8 source-span failures and disposal accepted by four focused compiler groups. Complete hover/signature/completion and five native documentation examples pass. Four independently matched sources accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/80-string-substr.md) · [string_substr_default_ascii](../benchmarks/runtime/workloads/string_substr_default_ascii.tsr) · [string_substr_default_unicode](../benchmarks/runtime/workloads/string_substr_default_unicode.tsr) · [string_substr_bounded_ascii](../benchmarks/runtime/workloads/string_substr_bounded_ascii.tsr) · [string_substr_bounded_unicode](../benchmarks/runtime/workloads/string_substr_bounded_unicode.tsr) |
| `String.substring` | native variant | paired group | Required strict int start and optional int end clamp to zero/UTF-8 byte length and swap reversed endpoints. Receiver and bounds evaluate once in source order under existing loans. Empty results skip boundary access, including equal interior bytes. Full range uses standard Cow cloning; partial static text borrows and owned text copies independently. Nonempty invalid scalar boundaries report source bytes. Six default/bounded/swapped ASCII/Unicode comparisons retain complete String results with Unicode byte/UTF-16 endpoint tuple setup/selection timed. Compiler/editor discovery, complete outputs and normal/unwind resources are accepted; coercion and storage/allocation parity are not claimed.; [evidence](./milestones/71-string-substring.md) · [string_substring_default_ascii](../benchmarks/runtime/workloads/string_substring_default_ascii.tsr) · [string_substring_default_unicode](../benchmarks/runtime/workloads/string_substring_default_unicode.tsr) · [string_substring_bounded_ascii](../benchmarks/runtime/workloads/string_substring_bounded_ascii.tsr) · [string_substring_bounded_unicode](../benchmarks/runtime/workloads/string_substring_bounded_unicode.tsr) · [string_substring_swapped_ascii](../benchmarks/runtime/workloads/string_substring_swapped_ascii.tsr) · [string_substring_swapped_unicode](../benchmarks/runtime/workloads/string_substring_swapped_unicode.tsr) |
| `String.sup` | implemented | paired group | Legacy HTML wrapper with exact tag, raw UTF8 receiver and quote-only attributes where applicable. Strict signatures and independently owned output; complete IntelliSense/examples and focused O0/O3 native behavior/ownership/disposal accepted. Matched source comparison accepted; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/81-string-html-wrappers.md) · [string_html_sup](../benchmarks/runtime/workloads/string_html_sup.tsr) |
| `string.taggedTemplate` | pending | missing |  |
| `string.templateLiteral` | implemented | paired group | Ordinary untagged templates with nested/multiline chunks and strict primitive interpolation. Native Rust Display and UTF-8; no object coercion. Focused compiler/editor/native examples pass; Checkpoint81 accepts matched execution comparisons and fully disposed allocation profiles.; [evidence](./milestones/78-template-literals.md) · [template_primitive_ascii](../benchmarks/runtime/workloads/template_primitive_ascii.tsr) · [template_string_unicode](../benchmarks/runtime/workloads/template_string_unicode.tsr) |
| `String.toLocaleLowerCase` | native variant | paired | No arguments; applies the locale-independent Unicode mapping, the same result as the plain case method. A locale argument is rejected because locale-specific rules are not provided. One matched scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/112-string-locale-case.md) · [string_locale_case_text](../benchmarks/runtime/workloads/string_locale_case_text.tsr) |
| `String.toLocaleUpperCase` | native variant | paired | No arguments; applies the locale-independent Unicode mapping, the same result as the plain case method. A locale argument is rejected because locale-specific rules are not provided. One matched scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/112-string-locale-case.md) · [string_locale_case_text](../benchmarks/runtime/workloads/string_locale_case_text.tsr) |
| `String.toLowerCase` | native variant | paired group | Native byte/scalar or standard Rust Unicode contract.; [evidence](./string-semantics.md) · [dynamic_normalization](../benchmarks/runtime/workloads/dynamic_normalization.tsr) |
| `String.toString` | implemented | paired group | Primitive receiver only, strict zero arity; receiver evaluates once and stays reusable. Exact text through standard Cow cloning: static results borrow and owned results copy independently; no JavaScript allocation/identity parity claim. Compiler/editor, complete source/native outputs and runtime/disposal acceptance are complete; numeric formatting and boxed objects are separate.; [evidence](./milestones/72-primitive-identity.md) · [string_to_string_ascii](../benchmarks/runtime/workloads/string_to_string_ascii.tsr) · [string_to_string_unicode](../benchmarks/runtime/workloads/string_to_string_unicode.tsr) |
| `String.toUpperCase` | native variant | paired group | Rust Unicode uppercase with owned UTF-8 results; literal and dynamic comparisons preserve returned content and actual engine costs.; [evidence](./milestones/35-string-uppercase.md) · [literal_ascii_uppercase](../benchmarks/runtime/workloads/literal_ascii_uppercase.tsr) · [dynamic_unicode_uppercase](../benchmarks/runtime/workloads/dynamic_unicode_uppercase.tsr) |
| `String.toWellFormed` | native variant | paired group | Strict zero-argument borrowed String receiver evaluated once; content remains exact through standard Cow clone. Static storage stays borrowed, nonempty owned storage copies independently, and owned empty cloning allocates nothing. Valid native UTF-8 has no representable lone surrogates; no JavaScript coercion or allocation parity claim. Complete-result source/native/runtime and disposal acceptance complete.; [evidence](./milestones/76-native-string-well-formedness.md) · [string_to_well_formed_ascii](../benchmarks/runtime/workloads/string_to_well_formed_ascii.tsr) · [string_to_well_formed_unicode](../benchmarks/runtime/workloads/string_to_well_formed_unicode.tsr) |
| `String.trim` | native variant | paired group | Native byte/scalar or standard Rust Unicode contract.; [evidence](./string-semantics.md) · [ascii_strings](../benchmarks/runtime/workloads/ascii_strings.tsr) |
| `String.trimEnd` | native variant | paired group | Native Rust standard directional whitespace trimming; U+0085 trims and U+FEFF remains. trimLeft/trimRight alias trimStart/trimEnd exactly. Static Cow results can borrow; owned results are independent and receiver storage remains reusable. One 601-group regression passes. This method has separate ASCII/Unicode comparisons using actual JavaScript built-ins over common whitespace. Receiver pool setup/cloning, dynamic selection, trimming and complete String[] retention are timed. Three-sample normalized resources retain raw profiles; output and final disposal reach zero.; [evidence](./milestones/61-string-trim-sides.md) · [string_trim_end_ascii](../benchmarks/runtime/workloads/string_trim_end_ascii.tsr) · [string_trim_end_unicode](../benchmarks/runtime/workloads/string_trim_end_unicode.tsr) |
| `String.trimLeft` | native variant | paired group | Native Rust standard directional whitespace trimming; U+0085 trims and U+FEFF remains. trimLeft/trimRight alias trimStart/trimEnd exactly. Static Cow results can borrow; owned results are independent and receiver storage remains reusable. One 601-group regression passes. This method has separate ASCII/Unicode comparisons using actual JavaScript built-ins over common whitespace. Receiver pool setup/cloning, dynamic selection, trimming and complete String[] retention are timed. Three-sample normalized resources retain raw profiles; output and final disposal reach zero.; [evidence](./milestones/61-string-trim-sides.md) · [string_trim_left_ascii](../benchmarks/runtime/workloads/string_trim_left_ascii.tsr) · [string_trim_left_unicode](../benchmarks/runtime/workloads/string_trim_left_unicode.tsr) |
| `String.trimRight` | native variant | paired group | Native Rust standard directional whitespace trimming; U+0085 trims and U+FEFF remains. trimLeft/trimRight alias trimStart/trimEnd exactly. Static Cow results can borrow; owned results are independent and receiver storage remains reusable. One 601-group regression passes. This method has separate ASCII/Unicode comparisons using actual JavaScript built-ins over common whitespace. Receiver pool setup/cloning, dynamic selection, trimming and complete String[] retention are timed. Three-sample normalized resources retain raw profiles; output and final disposal reach zero.; [evidence](./milestones/61-string-trim-sides.md) · [string_trim_right_ascii](../benchmarks/runtime/workloads/string_trim_right_ascii.tsr) · [string_trim_right_unicode](../benchmarks/runtime/workloads/string_trim_right_unicode.tsr) |
| `String.trimStart` | native variant | paired group | Native Rust standard directional whitespace trimming; U+0085 trims and U+FEFF remains. trimLeft/trimRight alias trimStart/trimEnd exactly. Static Cow results can borrow; owned results are independent and receiver storage remains reusable. One 601-group regression passes. This method has separate ASCII/Unicode comparisons using actual JavaScript built-ins over common whitespace. Receiver pool setup/cloning, dynamic selection, trimming and complete String[] retention are timed. Three-sample normalized resources retain raw profiles; output and final disposal reach zero.; [evidence](./milestones/61-string-trim-sides.md) · [string_trim_start_ascii](../benchmarks/runtime/workloads/string_trim_start_ascii.tsr) · [string_trim_start_unicode](../benchmarks/runtime/workloads/string_trim_start_unicode.tsr) |
| `string.unicodeValidity` | native variant | compound | Valid UTF-8; ownership transfers; explicit owned clone copies bytes.; [evidence](./string-semantics.md) · [dynamic_string_clone](../benchmarks/runtime/workloads/dynamic_string_clone.tsr) |
| `String.valueOf` | implemented | paired group | Primitive receiver only, strict zero arity; receiver evaluates once and stays reusable. Exact text through standard Cow cloning: static results borrow and owned results copy independently; no JavaScript allocation/identity parity claim. Compiler/editor, complete source/native outputs and runtime/disposal acceptance are complete; numeric formatting and boxed objects are separate.; [evidence](./milestones/72-primitive-identity.md) · [string_value_of_ascii](../benchmarks/runtime/workloads/string_value_of_ascii.tsr) · [string_value_of_unicode](../benchmarks/runtime/workloads/string_value_of_unicode.tsr) |
| `StringConstructor.call()` | implemented | paired group | Supported statically typed value domain; Node differential O0/O3, ownership, editor and documentation checks accepted. Dynamic host values remain unsupported. Gate83 preserves array conversion text/extra-argument order and removes diagnostic traversal/allocation through direct borrowed iteration; the array conversion row is refreshed at checkpoint 86. Scalar conversion remains unchanged. Gate84 accepts canonical nullable types with one existing payload plus null/undefined, and whole-tag observation without narrowing. Owned payloads transfer on injection; existing tags borrow for observations. Strict stable-place payload projection is accepted in Gate85. General unions and nullable callables/elements remain pending.; [evidence](./milestones/value-types-and-namespaces.md) · [evidence](./milestones/83-array-to-string.md) · [evidence](./milestones/84-nullable-type-foundation.md) · [string_explicit_int](../benchmarks/runtime/workloads/string_explicit_int.tsr) · [string_array_conversion](../benchmarks/runtime/workloads/string_array_conversion.tsr) · [nullable_int_null](../benchmarks/runtime/workloads/nullable_int_null.tsr) · [nullable_boolean_undefined](../benchmarks/runtime/workloads/nullable_boolean_undefined.tsr) · [nullable_number_null](../benchmarks/runtime/workloads/nullable_number_null.tsr) · [nullable_string_undefined](../benchmarks/runtime/workloads/nullable_string_undefined.tsr) · [nullable_array_null](../benchmarks/runtime/workloads/nullable_array_null.tsr) · [nullable_record_undefined](../benchmarks/runtime/workloads/nullable_record_undefined.tsr) |
| `StringConstructor.fromCharCode` | native variant | paired | Int UTF-16 code units (low 16 bits used), with spread arrays; no value fails. Surrogate pairs form one character; a lone surrogate becomes U+FFFD because strings hold valid Unicode. Cases checked against Node. One matched scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/110-string-from-codes.md) · [string_from_codes_text](../benchmarks/runtime/workloads/string_from_codes_text.tsr) |
| `StringConstructor.fromCodePoint` | native variant | paired | Int code points 0 through 1114111, with spread arrays; a code outside the range stops the program. Surrogate pairs form one character; a lone surrogate becomes U+FFFD because strings hold valid Unicode. Cases checked against Node. One matched scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/110-string-from-codes.md) · [string_from_codes_text](../benchmarks/runtime/workloads/string_from_codes_text.tsr) |
| `StringConstructor.new()` | pending | missing |  |
| `StringConstructor.prototype` | pending | missing |  |
| `StringConstructor.raw` | native variant | paired | Joins a record’s raw string[] segments with String() conversions and optional string-array spread. Missing substitutions insert empty text; extra argument expressions still evaluate. Tagged templates and arbitrary array-like raw fields are unsupported. One scenario is registered and verified untimed; measurements wait for the string-methods milestone.; [evidence](./milestones/113-string-raw.md) · [string_raw_segments](../benchmarks/runtime/workloads/string_raw_segments.tsr) |

## Symbols

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `Symbol.[Symbol.toPrimitive]` | pending | missing |  |
| `Symbol.[Symbol.toStringTag]` | pending | missing |  |
| `Symbol.description` | pending | missing |  |
| `Symbol.toString` | pending | missing |  |
| `Symbol.valueOf` | pending | missing |  |
| `SymbolConstructor.asyncDispose` | pending | missing | esnext declaration; availability unverified |
| `SymbolConstructor.asyncIterator` | pending | missing |  |
| `SymbolConstructor.call()` | pending | missing |  |
| `SymbolConstructor.dispose` | pending | missing | esnext declaration; availability unverified |
| `SymbolConstructor.for` | pending | missing |  |
| `SymbolConstructor.hasInstance` | pending | missing |  |
| `SymbolConstructor.isConcatSpreadable` | pending | missing |  |
| `SymbolConstructor.iterator` | pending | missing |  |
| `SymbolConstructor.keyFor` | pending | missing |  |
| `SymbolConstructor.match` | pending | missing |  |
| `SymbolConstructor.matchAll` | pending | missing |  |
| `SymbolConstructor.metadata` | pending | missing | esnext declaration; availability unverified |
| `SymbolConstructor.prototype` | pending | missing |  |
| `SymbolConstructor.replace` | pending | missing |  |
| `SymbolConstructor.search` | pending | missing |  |
| `SymbolConstructor.species` | pending | missing |  |
| `SymbolConstructor.split` | pending | missing |  |
| `SymbolConstructor.toPrimitive` | pending | missing |  |
| `SymbolConstructor.toStringTag` | pending | missing |  |
| `SymbolConstructor.unscopables` | pending | missing |  |

## TypeScript types

| Operation | Implementation | Comparison | Scope / evidence |
| --- | --- | --- | --- |
| `type.alias` | native variant | paired | type Name = Type; for any written non-generic type, in any declaration order and across modules; expanded where a written type is parsed. No type parameters or self-reference. One matched scenario verified untimed.; [evidence](./milestones/163-type-aliases.md) · [evidence](./type-aliases.md) · [type_aliases](../benchmarks/runtime/workloads/type_aliases.tsr) |
| `type.ambient` | pending | missing |  |
| `type.any` | policy | missing | Original vision rejects unrestricted escape hatches; track explicit policy instead of silently omitting. |
| `type.array` | native variant | compile-only | Implemented strict subset; no standalone compile-time comparison for this item. |
| `type.asConst` | pending | missing |  |
| `type.assertion` | native variant | paired | Compatible contextual typing and explicit scalar int-to-number widening; unsafe narrowing rejected. Ownership and lexical borrows are preserved. Matched native/node-tsc verification passes five untimed cases per engine; unmeasured.; [evidence](./milestones/143-checked-type-assertions.md) · [evidence](./type-assertions.md) · [type_assertions](../benchmarks/runtime/workloads/type_assertions.tsr) |
| `type.conditional` | pending | missing |  |
| `type.constEnum` | pending | missing |  |
| `type.declarationMerging` | pending | missing |  |
| `type.discriminatedUnion` | pending | missing |  |
| `type.enum` | native variant | paired | Closed nominal numeric or string enums with constant initializers, copied values, arrays, record fields and imported aliases. Primitive conversion is explicit; raw primitive assignments, heterogeneous enums, runtime enum objects and reverse lookup are unsupported. Matched native/node-tsc verification passes untimed.; [evidence](./milestones/144-enums.md) · [evidence](./enums.md) · [enum_values](../benchmarks/runtime/workloads/enum_values.tsr) |
| `type.generic` | pending | missing |  |
| `type.genericConstraint` | pending | missing |  |
| `type.genericDefault` | pending | missing |  |
| `type.indexedAccess` | pending | missing |  |
| `type.infer` | pending | missing |  |
| `type.interface` | native variant | paired | interface declarations are nominal record types with optional fields, type parameters and extends over record types, in any order and across modules. Record types stay distinct by name; method signatures, generic bases and declaration merging are rejected. One matched scenario verified untimed.; [evidence](./milestones/166-interfaces.md) · [evidence](./interfaces.md) · [interfaces](../benchmarks/runtime/workloads/interfaces.tsr) |
| `type.intersection` | pending | missing |  |
| `type.keyof` | pending | missing |  |
| `type.literal` | pending | missing |  |
| `type.mapped` | pending | missing |  |
| `type.namespace` | pending | missing |  |
| `type.narrowing` | in progress | paired group | Gate85 accepts strict null/undefined stable-place guards. Gate120 adds strict typeof category comparisons in either operand order, residual data subsets and checked member access after narrowing. Physical tags stay unchanged; replacement and overlapping mutation invalidate facts. typeof cannot distinguish alternatives in the same category. Truthiness/loose-equality guards and reusable indexed/call guards remain pending. The new matched data comparison is verified untimed; six earlier comparisons were measured at checkpoint86.; [evidence](./milestones/85-nullable-flow-narrowing.md) · [evidence](./milestones/120-data-unions.md) · [narrowing_int](../benchmarks/runtime/workloads/narrowing_int.tsr) · [narrowing_number](../benchmarks/runtime/workloads/narrowing_number.tsr) · [narrowing_boolean](../benchmarks/runtime/workloads/narrowing_boolean.tsr) · [narrowing_string](../benchmarks/runtime/workloads/narrowing_string.tsr) · [narrowing_array](../benchmarks/runtime/workloads/narrowing_array.tsr) · [narrowing_record](../benchmarks/runtime/workloads/narrowing_record.tsr) · [data_union_values](../benchmarks/runtime/workloads/data_union_values.tsr) |
| `type.never` | pending | missing |  |
| `type.nominalRecord` | native variant | compile-only | Implemented strict subset; no standalone compile-time comparison for this item. |
| `type.nonNullAssertion` | pending | missing |  |
| `type.optional` | native variant | paired | Optional record fields: name?: Type has type Type \| undefined and may be omitted from a literal, where it holds undefined. The record keeps its fixed shape, so an omitted field is still listed and printed; JavaScript omits the property. Optional parameters are covered separately. One matched scenario verified untimed.; [evidence](./milestones/165-optional-fields.md) · [evidence](./optional-fields.md) · [optional_fields](../benchmarks/runtime/workloads/optional_fields.tsr) |
| `type.primitive` | native variant | compile-only | Implemented strict subset; no standalone compile-time comparison for this item. |
| `type.readonlyTuple` | pending | missing |  |
| `type.satisfies` | native variant | paired | Checks compatibility while preserving the source type; literals receive required annotation context. Ownership and lexical borrows are preserved. Matched native/node-tsc verification passes five untimed cases per engine; unmeasured.; [evidence](./milestones/143-checked-type-assertions.md) · [evidence](./type-assertions.md) · [type_assertions](../benchmarks/runtime/workloads/type_assertions.tsr) |
| `type.structuralObject` | pending | missing |  |
| `type.templateLiteral` | pending | missing |  |
| `type.tuple` | native variant | paired | Fixed owning positional data tuples have contextual literals, structural identities, constant signed indices, length, mutation, explicit cloning, nullable/nested positions, tuple arrays, generic/imported contexts and editor support. Computed indices, named tuple aliases, callable/optional/rest positions and variable-length tuple iteration remain unsupported. One construction/copy/mutation/map scenario is verified untimed; family measurements remain pending.; [evidence](./milestones/117-tuple-types.md) · [tuple_types_values](../benchmarks/runtime/workloads/tuple_types_values.tsr) |
| `type.typeof` | pending | missing |  |
| `type.union` | in progress | paired group | Gates84/85/97 retain nullable and callable transport. Gate120 accepts canonical general data alternatives, all-present projections/coalescing, independently cloned recursive graphs, readonly views and required/defaulted mutable borrowing including mixed spread staging. Callable unions retain one callable payload. Union array leaves, general union equality/arithmetic and broader inference remain pending. The new matched data comparison is verified untimed; checkpoint86 measurements remain historical. Gate121 accepts union array elements with storage, iteration, narrowing, optional element results, primitive-union search, text conversion and cloning; union_array_values is registered and verified untimed.; [evidence](./milestones/84-nullable-type-foundation.md) · [evidence](./milestones/85-nullable-flow-narrowing.md) · [evidence](./milestones/97-nullable-callable-transport.md) · [evidence](./nullable-callables.md) · [evidence](./milestones/120-data-unions.md) · [evidence](./milestones/121-union-array-elements.md) · [nullable_int_null](../benchmarks/runtime/workloads/nullable_int_null.tsr) · [nullable_boolean_undefined](../benchmarks/runtime/workloads/nullable_boolean_undefined.tsr) · [nullable_number_null](../benchmarks/runtime/workloads/nullable_number_null.tsr) · [nullable_string_undefined](../benchmarks/runtime/workloads/nullable_string_undefined.tsr) · [nullable_array_null](../benchmarks/runtime/workloads/nullable_array_null.tsr) · [nullable_record_undefined](../benchmarks/runtime/workloads/nullable_record_undefined.tsr) · [data_union_values](../benchmarks/runtime/workloads/data_union_values.tsr) · [union_array_values](../benchmarks/runtime/workloads/union_array_values.tsr) |
| `type.unknown` | pending | missing |  |
| `type.utilityAwaited` | pending | missing |  |
| `type.utilityConstructorParameters` | pending | missing |  |
| `type.utilityExclude` | pending | missing |  |
| `type.utilityExtract` | pending | missing |  |
| `type.utilityInstanceType` | pending | missing |  |
| `type.utilityNonNullable` | pending | missing |  |
| `type.utilityOmit` | pending | missing |  |
| `type.utilityOmitThisParameter` | pending | missing |  |
| `type.utilityParameters` | pending | missing |  |
| `type.utilityPartial` | pending | missing |  |
| `type.utilityPick` | pending | missing |  |
| `type.utilityReadonly` | pending | missing |  |
| `type.utilityRecord` | pending | missing |  |
| `type.utilityRequired` | pending | missing |  |
| `type.utilityReturnType` | pending | missing |  |
| `type.utilityThisParameterType` | pending | missing |  |
| `type.utilityThisType` | pending | missing |  |
