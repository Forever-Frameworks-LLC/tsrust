# Gate 154: borrowed Map iteration

Map for-of visits readonly stored key/value tuples in insertion order. Both entry
variables and key/value destructuring work; readonly tuple parameters accept the
entry. Noncloneable owned values can be read without copies. Empty Maps run no
body. Receiver mutation and borrowed owner transfers are rejected during the loop;
mutation works after exit. Explicit `clone(entry)` creates an independent cloneable
tuple. The iterable expression is evaluated once.

## Implementation and defects

Map storage takes a private entry representation parameter. Ordinary Maps keep
native pairs; Maps used in borrowed iteration store their existing owned tuple
record. A private trait supplies borrowed fields and owned construction/extraction.
The iterator yields references to stored records without allocating or cloning
entries. Existing tuple ABI, Map callback and snapshot paths stay intact.

Editor iteration inference now recognizes typed Map and Set inputs, including
Map tuple bindings and patterns. Map string conversion accepts every storage
representation. The raw runtime storage witness needs explicit default-entry type
annotations after the new generic parameter; its ordering, replacement and owner
drop assertions remain unchanged. Original red and failed checks are retained.
The expanded fixture initially used unsupported `entry.clone()`; it now exercises
the existing explicit `clone(entry)` operation. The benchmark ABI signature was
corrected to the runner's required spelling before final verification.

## Focused evidence

- Initial red integration and CLI checks reject Map iteration. Independent pinned
  Node accepts the assertion fixture; final Node and installed native runs pass.
- Final compiler checks: higher_order_function_values 6, maps 6, type_syntax 8,
  corrected map_iteration 2. Map callbacks 2 and snapshots 4 passed before the
  final Map string-conversion trait generalization; their analysis was unchanged.
  Thus 28 distinct focused checks pass across preserved runs.
- Editor: map_iteration 1, map_snapshots 1 and maps 4 pass (6 total).
- Harness: 381/381. Matched independent-array oracle verification passes untimed
  for native and node-tsc: runtime/results/2026-10-06T08-39-54-559Z-verify.json.
- Workload checked before refresh. Debug/compiler/editor and release consumers
  atomically refreshed; hashes in target/editor/map-iteration-consumers.log.
- Syntax guide: docs/guides/language/maps.md. No new built-in member is introduced. Source pair,
  source contracts, registration, oracle and all runtime/profiling/report hooks
  are complete. Original manifest retained before acceptance flags change.

Evidence logs are retained under target/editor/map-iteration-*.log. Sources,
previous consumers and records have exact archive/map-iteration-*-start snapshots.
No full regression or timing was run. Collection measurements remain pending.
Live iterator objects and the general iterator protocol remain unsupported.
