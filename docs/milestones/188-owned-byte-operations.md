# Gate 188: owned byte operations and cleanup

Gate 188 is accepted with focused evidence on 2026-10-10. S07 remains in progress;
scoped views/UTF-8 and resource lifecycle remain gates 189 and 190. Complete family
regressions, accumulated measurements and optimization review remain pending. No byte
workload is measured.

## Observable behavior

Bytes uses independent owned contiguous byte storage. at and set check nonnegative
byte indices without wrapping; set validates the index before the value and never
changes contents on failure. alloc and resize default fill to zero, validate length
before fill even for empty or shrinking requests, preserve existing prefixes, and
reserve fallibly before growth. append readonly borrows an int array, checks combined
length and all values before reserving, then appends in order. copy creates an
independent owner for a strict half-open range, including empty end ranges. Its start
coordinate is checked before end; bounds never clamp or wrap. Structured ByteError
codes and offsets are documented and exercised by native fixtures.

Explicit clone already supports Bytes and owned descendants in arrays, records,
Maps, nullable unions and Result. New witnesses prove independence after mutation of
source and copy. Passing, binding and storing owners still transfer without implicit
cloning. Generic storage and escaped reusable/Once captures work; captured owners
release when unused callbacks are discarded or throwing callbacks finish.

Native metadata records independent cloning, collection-key eligibility and future
owned thread-transfer eligibility. Bytes and ByteError cannot be current collection
keys. Their owned backend representations can transfer across a future native thread
boundary; this does not expose source threading. Cloneability honors native capability
metadata and propagates rejection through aggregates. Opaque inspection, conversion,
reflection, spread and equality remain diagnosed source errors.

## Verification

The closing batch has 37 effective passes: 31 compiler, five editor and one harness
registration control. It includes six cloneability controls, seven native byte project
tests, seventeen declaration/capability controls and one exact cleanup test. Earlier
increments retain runtime mutation/reservation tests, native argument-mode diagnostics
and executable docs. New counters prove alloc/resize/append/copy arguments evaluate once.
All added fixtures execute after stable formatting; whole-project formatting preserves
tokens and is idempotent. Global clone hover and parameter help use the shared catalog;
its byte example prints 65 after the original buffer changes to 90.

A private Drop observer counts exactly sixteen byte owners per cleanup invocation
across normal/early return, replacement, Result failure, throw, finally, discarded
captures, throwing reusable/Once callbacks and aggregate disposal. At O0 and O3,
thirty-two invocations each restore live allocation bytes exactly to baseline. The
observer is only in generated test code; no fake resource API was published.

The native_byte_clone workload retains seven values proving direct and aggregate
independence, with explicit deep Uint8Array copies and an independent arithmetic
oracle. Five native/Node-after-tsc cases pass untimed. It complements the existing
construction, access/mutation, growth and copy/append workloads. All have source guards,
hashes, registrations and runner/profile/report hooks ready for the later family run.

## Repairs and original evidence

The first cleanup fixture lacked the language's required named record annotation;
its original failure is preserved with the focused repair. Moving clone docs into
the catalog exposed an editor vector type inference error, then a mismatch with the
global catalog's string-parameter convention. Both are repaired and the affected five
editor tests pass. The new throwing workload exposed its missing fallible ABI family
registration; the existing ABI path now supports it, with positive and rejection
controls. These are verification/support repairs, not weakened source semantics.

Exact SQLite records, verified by byte-for-byte readback:

- runtime/results/native-byte-ownership-2026-10-10-checks.json
- runtime/results/native-byte-ownership-2026-10-10-repairs.json
- runtime/results/2026-10-10T14-58-10-521Z-verify.json

Earlier operation increments and their original failed/repair evidence remain referenced
in docs/plans/resources-bytes-plan.md and their handoffs. No full regression or timing was
repeated for this individual gate. Installed consumer publication is recorded separately.


## Installed consumer publication

Source acceptance commit: 3ec1f7ce. Five consumers are rebuilt/refreshed atomically
from this checkpoint, with matching SHA-256 hashes. Debug/editor and release compiler
ownership fixture executions pass; LSP initialize/shutdown/exit protocol smoke passes.
Exact publication/build/smoke/cleanup evidence is in SQLite at
runtime/results/gate188-2026-10-10-publication.json, with byte-for-byte readback.
Superseded test executable variants and closed generated native outputs are pruned;
shared libraries, the warm target and all five installed consumers are retained.
