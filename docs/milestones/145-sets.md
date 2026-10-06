# Gate 145: typed Set collections

Status: accepted with focused compiler/editor checks and matched untimed verification.
No full regression or timing run was performed. The collection family remains open.

## Behavior

Set stores unique int, number, boolean, string, null, undefined, enum or union
keys in insertion order. Construction accepts a dense array, another Set, string
characters or an empty source; empty construction requires a key type or context.
Construction explicitly copies keys and keeps source collections usable. Compatible
keys can widen into unions. Integers do not implicitly convert to numbers.

The size property is readonly. add transfers owned keys, copies scalar keys and
returns void. has borrows its query; delete borrows its query and reports whether
it removed a key. clear empties the receiver. Number keys use SameValueZero;
negative zero is stored as positive zero. Deletion and reinsertion moves a key
last. keys and values return independent array snapshots, explicitly copying keys.
for-of borrows the receiver; owned keys are readonly borrows, and receiver mutation
or transfer during iteration is rejected. clone explicitly copies the collection.

Sets work in record fields, arrays, generic functions, return values and union
observations. Console display, explicit String conversion and array join support
Set values. See [the guide](../sets.md). Object/array/function identity keys,
constructor aliases, entries, callbacks, set algebra and live iterators remain
unsupported. This gate does not claim the complete JavaScript Set API.

## Implementation and repairs

A separate constructor parser and analysis/emission helpers keep collection work
out of shared recursive frames. Set descriptors retain key types through named
resolution, generic inference and checked editor facts. Runtime insertion slots
preserve order; hash buckets store slot indices, avoiding duplicate owned keys.
Deleted slots are tombstones. Allocation and churn costs are not measured.

Focused failing witnesses found and repaired module-renamed Set annotations,
generic validation/inference, native union-key adaptation during explicit source
copying, nested-storage array join, shadowed constructors and editor inference.
Editor support includes typed/inferred hover, completion, member and constructor
parameter help, formatting and new/Set highlighting. Catalog entries provide
observable behavior and executable examples. Eight examples run together in one
native program; an editor check ensures catalog/fixture synchronization.

## Evidence

Logs are retained under target/editor/sets-*.log.

- Final compiler Set suite: 6/6 in sets-complete-native.log. Native assertions cover
  ordering, SameValueZero, borrowed strings, copy construction, null/undefined,
  enum/union keys, generics, explicit clone, nested storage, display and examples.
  Static rows cover incorrect keys, implicit numeric conversion, active loans,
  moved keys, readonly size and shadowed constructors.
- Independent pinned Node reference: sets-node-final.log. Native snapshot methods
  have explicit Array.from adaptations in the reference. The focused union-copy
  and nested-storage additions have native assertion witnesses.
- Compiler canaries: for_of 10, higher_order_function_values 6,
  nullish_coalescing 4 and type_syntax 8 pass in sets-compiler-batch.log. These
  precede the final collection-only adaptation, shadow and editor-fact repairs;
  only the affected Set suite was rerun. Compiler unit-test code compiles with
  --lib --no-run (sets-unit-build.log); unit tests were not executed.
- Final editor Set suite: 4/4 (sets-catalog-editor.log). Adjacent enums 3/3 and
  type_assertions 2/2 pass in the earlier sets-final-editor.log, which retains
  the original Set inference failure. Only Set editor tests were rerun after repair.
- Harness: 371/371 (sets-harness.log) on refreshed consumers. A later workload
  expansion adds clear, keys, borrowed iteration and constructor-copy cells;
  affected registration and source checking pass separately. No full harness
  repeat was needed for those workload-only changes.
- Complete set_values matched verification: five cases per engine, native and
  node-tsc, results/2026-10-06T05-55-03-974Z-verify.json. verifiedOnly is true and
  startup/compilation/execution timing arrays are empty. The earlier narrower
  result is preserved with its exact sources in archive/sets-workload-expansion-start.
- Five consumers refreshed atomically, with matching final hashes in
  sets-consumers.log. Previous bytes remain in archive/sets-consumers-start.

Workload counterparts, independent oracle, semantic source contracts, registration,
runtime/profiling/report hooks, manifest fingerprints, coverage and roadmap are
complete. Measurements remain pending the collection-family milestone. Exact
original sources and metadata remain in archive/sets-*-start. No Git.
