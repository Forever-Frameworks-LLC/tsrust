# Gate 157: iterable array spread

Array literals and Array.of now accept string, Set and Map spread sources beside
arrays. Each segment evaluates once, from left to right, and finishes capturing
its elements before the next segment runs. Strings construct Unicode scalar
character strings; combining marks remain separate values. Sets and Maps preserve
insertion order, including overwritten Map keys and removed collection slots.

## Ownership and implementation

The existing array-spread ownership rule remains intact. Scalar elements copy,
including readonly Set scalar keys. Owning Set elements transfer. Map key/value
tuple owners transfer even when their fields are scalar, matching record-array
spread. Noncloneable Map values transfer without copying. Readonly owners cannot
transfer. String spread constructs character strings and leaves the source usable;
there is no implicit source clone or borrowed substring ABI change.

Map expansion reuses stored entry records. Consuming Set/Map iterators move live
slots and skip deleted slots; ordinary borrowed traversal remains intact. Checked
context supplies empty constructor types, while union result elements receive
checked variant injection. Typed numeric values do not gain implicit widening.
Ordinary function-call spread remains array-only; arbitrary iterator objects are
not added. The syntax guide and Array.of/Array namespace catalog entries explain
source forms, order, Unicode behavior, mutation, ownership and limitations.

## Defects and focused evidence

- Exact Node assertion fixture passes unchanged. The original red CLI witness
  rejects string spread. Native assertions now pass for characters, empty/contextual
  collections, once-only source effects, later scalar source mutation, explicit
  readonly scalar copying, union output, ordered Map entries and noncloneable owners.
  Static witnesses reject moved reuse, readonly owning Set/Map transfers and records.
- Existing call-spread depth test uncovered a parser stack overflow while rejecting
  an over-limit nested call chain. Crash frames and a phase witness identify parsing.
  Boxed callee storage and a separate final construction helper reduce the recursively
  retained call_expression frame from 1,840 to 1,120 bytes. Original 100-level accepted
  and 200-level rejected tests pass with the original worker stack and 128 limit.
  Temporary phase prints were removed; the test file is restored byte-for-byte.
- Compiler: 41 distinct focused checks across retained batches: array_literal_spread
  10, call_spread 10, higher_order_function_values 6, iterable_array_spread 2,
  map_iteration 2, string_iteration 3 and type_syntax 8. Final parser-affected groups
  and the expanded new fixture pass after repair. Unchanged array/iteration groups
  retain their earlier passing evidence rather than being repeated after the split.
- Editor: 5 distinct checks: iterable_array_spread 2, array_from_collections 2 and
  map_iteration 1. Final new syntax/help/example checks pass after the parser repair.
  Existing checked facts supply spread types; no additional LSP type heuristic is
  needed. Only globals.Array and ArrayConstructor.of catalog entries changed.
- All eight Array.of catalog examples match native and pinned Node output; fixture
  and expected output live under tests/fixtures/iterable-array-spread/documented.*.
- Harness 384/384 after consumer refresh. An earlier harness failure used old gate156
  consumers; its log is retained. Updated consumers resolve that failure.
- Untimed native/node-tsc verification passes five cases in
  runtime/results/2026-10-06T09-54-03-556Z-verify.json. The result records verifiedOnly
  with no timed execution rows. Matched sources, independent oracle, fingerprints,
  source contract, registration and runtime/profile/report hooks are complete.
- Workload checked before atomic refresh of all five consumers. Exact previous bytes
  remain in archive/iterable-array-spread-consumers-start; final hashes are in
  target/editor/iterable-array-spread-consumers.log.

Source, support, depth and record snapshots are retained under descriptive archive
paths. Logs remain under target/editor/iterable-array-spread-*; initial failures are
preserved. No full regression or timing. Existing Array.of measurement records stay
intact; new iterable spread comparisons remain pending the collection milestone.
