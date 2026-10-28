# Gate 147: independent Map snapshots

Status: accepted with focused compiler/editor checks, runtime harness and matched
untimed verification. No full regression or timing run was performed.

## Behavior

keys(), values() and entries() return independent arrays in insertion order.
They take no arguments, read the receiver and leave it usable. entries returns
typed [Key, Value] tuples. Empty Maps return empty arrays. Replacing a key retains
its position; deletion and reinsertion moves it last. Later mutation or disposal
of the receiver does not change a snapshot.

Snapshots explicitly copy owned storage, including nested arrays and records.
values and entries require cloneable values; keys does not copy values and works
with non-cloneable owners. Receiver expressions are evaluated once. Temporary
receivers and generic functions are supported. Wrong arity or incompatible result
annotations are static errors. No access or binding starts implicitly cloning.

These operations return arrays rather than JavaScript live iterators. Owned copies
do not claim shallow alias identity. The matched TypeScript workload calls the
real Map methods and explicitly materializes their iterators with Array.from.
See [the guide](../guides/language/maps.md). Borrowed Map for-of, callbacks, live iterators and
collection-family measurements remain later work.

## Implementation and editor

Snapshot analysis is a separate inline-never helper. Entry tuple materialization
reuses the existing checked tuple machinery. Native emission walks insertion
slots once and copies only the selected columns. No runtime storage change is
needed. Return types flow through checked method facts and the shared editor
catalog. Hover and signature help show typed arrays and tuple arrays.

Three complete catalog entries include observable behavior, copy restrictions,
examples and ownership notes. The combined Map documentation fixture now contains
ten examples. An editor test checks catalog/fixture synchronization; native and
independent Node outputs match all ten. Exact accepted gate-146 fixtures are
preserved in archive/map-snapshots-tests-start.

## Evidence

Logs are retained under target/editor/map-snapshots-*.log.

- Original compiler red run fails both snapshot witnesses with unsupported Map
  members; independent pinned Node reference passes after Array.from adaptation.
  Adapted reference sources are retained alongside the logs.
- map-snapshots-native.log: snapshot tests 2/2, Map 6/6, Array.entries 9/9.
  The focused additions in map-snapshots-final-native.log pass 3/3, including
  wrong arity/result types, generic results and once-only receiver evaluation.
  These sources precede the final cloneability test addition; no compiler source
  changes follow the original passing batch.
- map-snapshots-cloneability.log: 1/1 supplement demonstrates keys on a Map of
  non-cloneable Once callable records; values/entries reject with clone-type.
  The snapshot suite has four passing tests across the final batch and supplement.
- map-snapshots-editor.log: snapshot editor 1/1 and Map editor 4/4.
- map-snapshots-documented-native.log and map-snapshots-documented-node.log match
  all ten examples, supplementing the earlier native Map example batch.
- map_snapshots passes five cases per engine, native and node-tsc, in
  results/2026-10-06T06-45-37-886Z-verify.json. verifiedOnly is true and all timing
  arrays are empty. The workload is unmeasured.
- map-snapshots-harness.log: runtime harness 373/373 on refreshed consumers.
  Final acceptance flags change afterward; final registration is checked separately.
  The exact pre-acceptance manifest remains in archive/map-snapshots-verification-start.
- Workload check passes before refresh. Debug/release builds pass; five consumers
  refresh atomically with hashes in map-snapshots-consumers.log. Previous bytes
  remain in archive/map-snapshots-consumers-start.

Matched sources, independent oracle, semantic guards, registration, runtime and
profiling hooks, reporting category and fingerprints are ready. Original compiler,
catalog, fixture and metadata snapshots remain in archive/map-snapshots-*-start.
No Git.
