# Gate 159: Map.groupBy

Map.groupBy groups ordered array, string, Set or Map elements by a reusable
callback key. It returns a Map of independently owned arrays, retaining first-key
order and input order within each group. Strings supply Unicode scalar characters,
Sets supply keys and Maps supply entry tuples. Empty input invokes no callback.

## Ownership and implementation

The callback receives a readonly element and its zero-based int position. Fewer
parameters, defaults and compatible rest arrays use the existing two-position
Array.from callback contract. Sources and callback expressions evaluate once in
that order. The callback can mutate unrelated captures; it cannot mutate or consume
the source or owning elements. Callback failures stop traversal and propagate
through catch/finally. Scalar, enum and compatible union keys reuse Map key rules,
including SameValueZero for numbers. Identity keys remain unsupported.

Grouping explicitly copies source values into independently owned arrays. This
matches existing explicit collection construction/snapshot operations; access and
bindings still never implicitly clone. Noncloneable elements are rejected. Native
owning records do not retain JavaScript's shared object references. A callback
returning a borrowed owning key must clone it explicitly or construct a fresh key.

Analysis and emission reuse checked traversal contracts, callback count handling,
registered failure transport and array representations. A dedicated checked target
produces grouped Map output; it is excluded from fixed-array eligibility. Internal
Map mutable lookup appends to groups without replacing their first insertion order
or copying stored keys. Nested grouping is recognized as a Map source. No parser
changes, arbitrary iterator protocol or groupBy spread arguments were added.

## Focused evidence

- TDD red reports unsupported Map.groupBy. Independent pinned Node std:testing
  fixture passes unchanged. Native assertions cover arrays, Sets, strings, Map
  tuples, empty inputs, callback index/captures, source reuse, defaults/rest,
  explicit contextual Maps, nested grouping, Unicode and NaN/infinite/zero keys.
  Supplemental final native/Node checks cover string and Boolean keys.
- Static witnesses reject source mutation, mutable element callback parameters,
  noncloneable values and a third required callback position. A separate native
  owning-record witness proves source mutation cannot change a copied group.
  Its Node adaptation explicitly snapshots the source before grouping; no shared
  JavaScript object-identity parity is claimed.
- Compiler: 10 distinct focused checks across retained batches: map_group_by 3,
  array_from_collections 2, map_for_each 2 and iterable_call_spread 3. Final new
  owning-copy check and supplemental key assertions pass. Existing successful
  related checks were retained rather than repeated after editor-only changes.
- Editor: map_group_by 2 and map_iteration 1 pass. Static namespace completion,
  namespace/member hover, signature documentation and checked Map result types
  are covered. MapConstructor.groupBy has complete behavioral documentation and
  an executable example. Existing instance Map documentation remains intact.
- Documented example matches native and pinned Node output: 2 groups, then 4,6.
- Harness 386/386 after five-consumer refresh; accepted-manifest repeat retained
  with final logs. No full regression or timing.
- Matched native/node-tsc verification passes five cases in
  runtime/results/2026-10-06T10-52-12-159Z-verify.json, marked verifiedOnly with
  no timed execution rows. Independent oracle uses closed-form numeric results
  and scalar character classification, without Map.groupBy. Source fingerprints,
  guards, registration and runtime/profile/report hooks are complete. TypeScript
  counterpart references esnext.collection for declarations; Node supplies the
  actual method. Initial workload ABI formatting failure was corrected without
  changing scenario semantics.
- Workload checked before atomic refresh of five consumers. Previous executable
  bytes remain in archive/map-group-by-consumers-start; current hashes are in
  target/editor/map-group-by-consumers.log.

Exact relative-path source, editor, support and record snapshots remain in archive.
Logs are retained under target/editor/map-group-by-* including initial failures.
Collection milestone measurements remain pending; this workload is unmeasured.
