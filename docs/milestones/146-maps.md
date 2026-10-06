# Gate 146: typed Map collections

Status: accepted with focused checks and matched untimed verification. No full
regression or timing run was performed. The collection family remains open.

## Behavior

Map stores scalar, enum or union keys with ordinary owned values in insertion
order. Construction accepts dense key/value tuple arrays, another Map or empty
sources. Explicit constructor copies leave sources usable. Empty constructors
need types or context; typed sources and literal tuple entries can infer types.
Number keys use SameValueZero, including NaN and normalized zero. Replacement
retains a key's position; deletion and reinsertion moves it last.

size is readonly. set transfers owned inputs and returns void. get returns a
value or undefined: scalars copy, owned values from named receivers borrow, and
temporary receivers move the selected owner. Borrowed reads block receiver
mutation or transfer until their lexical scope ends. has and delete borrow query
keys; delete reports removal. clear disposes entries. Explicit clone is supported.
Maps work in generic functions, returns, records, arrays and union observations.
Console inspection, String conversion and array join are supported.

See [the guide](../maps.md). Identity keys, live iterators, for-of, keys, values,
entries and callbacks are not included in this gate. Source copies do not claim
JavaScript shallow alias identity. Runtime storage keeps one owned key per slot;
hash buckets contain indices and deleted slots remain tombstones. Churn and
allocation costs are unmeasured.

## Repairs and editor integration

Focused failing tests drove constructor/type analysis, borrowed-get loans, native
storage and emission. Later witnesses repaired literal-entry inference, explicit
Map clone, optional borrowed-record field hints, console display and array join.
Owned storage tests demonstrate replacement/deletion/clear dispose non-Clone
values exactly once; runtime get itself requires no Clone bound.

Editor facts preserve key/value types. Catalog entries cover all six members and
construction, with seven executable examples. Hover, completion, constructor and
member parameter help, formatting and generic-constructor highlighting are
checked. A failing incomplete-source test repaired copied-Map fallback inference.

## Evidence

Logs are retained under target/editor/maps-*.log.

- maps-expanded-native.log: Map 6/6, higher_order_function_values 6/6,
  nullish_coalescing 4/4 and type_syntax 8/8. Assertions cover keys, NaN/zero,
  enum/union keys, generic/contextual construction, copies, explicit clone,
  borrowed records and arrays, temporary get, disposal, display and examples.
  Static rows cover incorrect types, transfers, active borrows, readonly size and
  shadowed construction. Earlier Set 6/6 remains in maps-current-native.log.
- maps-nested-native.log: the expanded fixture runs again after adding record,
  array and union Map storage. maps-expanded-node.log independently checks the
  complete final fixture with pinned Node. Both finish successfully without output.
- maps-documented-node.log and the native example test match all seven catalog
  examples. The editor synchronization test also checks each example's source.
- maps-final-editor.log: Map 4/4 and adjacent Set 4/4. Original incomplete-source
  failure is retained in maps-editor-red.log.
- maps-harness.log: runtime harness 372/372 on refreshed consumers. Final
  acceptance flags changed only after this pass; the affected registration test
  passes again in maps-registration-final.log.
- map_values: five correctness cases per engine, native and node-tsc, in
  results/2026-10-06T06-36-33-009Z-verify.json. verifiedOnly is true;
  startup/compilation/execution timing arrays are empty. The exact pre-acceptance
  manifest is preserved in archive/maps-verification-start; sources are unchanged.
- maps-workload-check.log checks the workload before consumer refresh.
  Debug/release builds pass; five consumers are atomically refreshed with hashes
  in maps-consumers.log. Previous bytes remain in archive/maps-consumers-start.

Matched sources, independent oracle, source contracts, registration, runtime and
profiling hooks, reporting category, fingerprints, coverage and roadmap are ready.
Original compiler, editor and metadata snapshots remain in archive/maps-*-start.
No Git. Collection milestone measurements remain pending.
