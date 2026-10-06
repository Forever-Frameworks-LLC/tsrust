# Gate 151: borrowed Map callbacks

Accepted on focused evidence. No timing or full regression.

## Behavior

Map.forEach visits entries once in insertion order and supplies value, key and
readonly source Map. Replacing an existing key preserves its position. Owned
keys and values borrow, while scalar inputs copy. No callback input is implicitly
cloned. Values need not support cloning; a record containing a single-use callable
is a native witness. The receiver cannot mutate or transfer during iteration.

Callbacks accept zero through three supplied positions with ordinary default/rest
rules. Named functions, reusable abstract callbacks and mutable external captures
are supported. Readonly abstract callbacks use ReadCallback invocation. The method
returns void and discards every callback result immediately. Empty maps never call
the callback. Checked errors stop iteration and propagate through catch/finally.
No thisArg argument is supported.

The implementation shares the established forEach analysis and emission paths.
Collection traversal obtains value/key types directly from the checked Map
instead of requiring array-compatible element types. Contextual callback types
retain materialized record/Map descriptors. Emission iterates borrowed key/value
pairs, adapts callback arguments and forwards checked Result transport.

## Evidence

The initial compiler witness rejects unsupported Map.forEach; independent pinned
Node passes. Exact source snapshots are in archive/map-for-each-compiler-start.
The first native fixture passes before expansion. Expanded assertions cover
arities, replacement order, strings, generic records, unions, non-cloneable
records, default/rest parameters, named/abstract callbacks, failed assertions,
return discard, empty input and finally completion. Static witnesses reject
receiver mutation, owned input transfer, consuming parameters and extra thisArg.

Final focused compiler checks pass: Map.forEach 2/2, Array.forEach 4/4, Map
snapshots 4/4, Map 6/6 and Set.forEach 7/7 (23 checks). Existing related suites
passed in the coherent batch; only the corrected new fixture was rerun. Fixture
corrections are retained: direct Function values in Map type arguments are already
unsupported, so the non-cloneable witness uses a record with a callable field.
Abstract callback contracts use ReadOnly<string> for the owned string argument.
Neither correction changes compiler behavior or weakens the borrowing assertion.

Editor checks pass 7/7: Map.forEach 1, Map snapshots 1, Map catalog/editor 4,
Set.forEach 1. They verify typed help/hover and contextual readonly source members.
All eleven Map constructor/member catalog examples match native execution and
adapted independent Node. Only snapshot-returning methods are adapted to
Array.from for Node's iterator API; forEach example is unchanged.

Matched workload support includes typed sources, independent complete-result
oracle, fingerprint/denominator contract, registration and runner/profiler/report
hooks. Workload check passes before atomic refresh of all five consumers; hashes
are in target/editor/map-for-each-consumers.log. Harness passes 378/378. Untimed native/node-tsc verification passes five cases
per engine: runtime/results/2026-10-06T07-42-55-208Z-verify.json. All gate commands
are terminal. Original verification manifest is retained in
archive/map-for-each-verification-start; acceptance flags change afterwards with
final focused registration passing separately. Measurements remain pending for
the collection family. No live iterator or borrowed Map for-of claim is made.
