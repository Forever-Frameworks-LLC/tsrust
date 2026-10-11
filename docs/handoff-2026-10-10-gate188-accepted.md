# Gate 188 accepted: owned bytes

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md,
docs/b11-resources-bytes-acceptance-map.md and this handoff. Gates 187 and 188 are
accepted. S07 remains in progress. One agent works in this tree; no protected owner
edits remain. The goal remains active.

## Accepted behavior

std:bytes exports Bytes, ByteError, fromArray, length, at, set, alloc, resize, append
and copy. All operations have catalog documentation and executable examples. at/set
use strict nonnegative byte indices, index-before-value validation and atomic writes.
alloc/resize default fill to zero and validate length before fill, even for empty or
shrinking requests. append validates combined length and every int-array element
before reservation or mutation. copy uses strict half-open bounds, start before end,
including empty end ranges. Its owners are independent. No clamping or negative wrapping.

Ordinary clone independently copies Bytes and owned aggregate descendants. Generic,
record/array/Map/nullable/Result storage and escaped reusable/Once captures execute.
Native clone eligibility propagates through aggregates. Metadata explicitly describes
collection-key eligibility and future owned native thread transfer; it does not
implement source threading. Opaque inspection, conversion, reflection, spread, keys
and equality remain source errors. Passing/storing values never implicitly clones.

## Closing evidence

37 effective focused passes: 31 compiler, five editor, one harness registration.
The compiler count includes six cloneability controls, seven native project tests,
seventeen declaration controls and one exact cleanup test. A State counter proves
alloc/resize/append/copy arguments evaluate once. Exact cleanup checks sixteen byte
owner releases per invocation and restored live allocation bytes across thirty-two
invocations at each of O0 and O3. It covers return, replacement, Result failure, throw,
finally, discarded and failing captures, Once and aggregate disposal. Only generated
test code has the Drop/allocator observers; no public fake resource backend exists.

The clone catalog example prints 65 after the original is changed to 90. Global clone
help now comes from builtins.json. Five native/Node-after-tsc clone workload cases pass
untimed, with seven retained observations and independent arithmetic expectations.
The construction, access, growth and append/copy workloads retain earlier focused
verification. All five workload pairs have guards/hashes/ABI/oracle/harness support.
No workload has been measured. No complete regressions ran for this individual gate.

Preserved failures: a cleanup fixture omitted its required record annotation; moving
clone docs left an ambiguous empty vector type; its first catalog parameter shape did
not match global help conventions; the throwing clone workload lacked its fallible
ABI registration. Each is repaired with affected checks. Original command logs and
explicitly labelled diagnostic excerpts remain in SQLite rather than being hidden.

SQLite exact records, byte-for-byte readback verified and loose copies removed:

- runtime/results/native-byte-ownership-2026-10-10-checks.json
- runtime/results/native-byte-ownership-2026-10-10-repairs.json
- runtime/results/2026-10-10T14-58-10-521Z-verify.json

The milestone and acceptance map contain the requirement/evidence mapping. Earlier
increment records and source checkpoints remain in the resource plan and handoffs.
Source acceptance and installed consumer publication are separate; publication status
is added below after the source checkpoint and atomic refresh.

## Next: gate 189

Implement scoped zero-copy readonly/exclusive views with real owner provenance.
Reuse existing loan/lifetime machinery; do not substitute owned copies or erased raw
pointers. Validate ranges and release loans on every exit. Cover nested calls, returned
views, storage/capture escapes, overlapping mutation, resize and owner transfer.
Readonly and exclusive views must expose usable checked access; exclusive writes
must affect the owner. Prove no allocation/sharing copies with native controls.

Implement explicit UTF-8 encoding and strict decoding with exact invalid-sequence
byte offsets. Cover ASCII, multibyte scalars, NUL, malformed/truncated sequences and
empty buffers. Add complete IntelliSense, diagnostics, guide and matched Uint8Array /
TextEncoder / fatal TextDecoder support as implementation lands, then untimed checks.
Do not postpone benchmark support until the family run.

Gate 190 owns consuming resources and a useful synchronous file proof. Owner confirmed
close consumes even on Err; later use/double close are static errors; destructor
fallback never throws. Distinguish close from flush/sync. Bytes itself needs no close.

Finish gates 189–190 before the S07 complete regression, accumulated measurements and
optimization review. Preserve completed evidence and resume affected failures rather
than restarting broad runs. Reuse /private/tmp/tsrust-gate100-editor-target with
CARGO_INCREMENTAL=0; Node 24.11.1 first on PATH; /opt/homebrew/bin/python3 for store.py.
Use explicit Git paths, whole-project formatting and exact SQLite readback.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-10-gate188-accepted.md. Read AGENTS.md,
docs/development-workflow.md, docs/plans/resources-bytes-plan.md and the B11 acceptance map.
Gates 187–188 accepted; do not redo owned bytes. Implement gate 189 scoped zero-copy
views/provenance and strict UTF-8, then gate 190 consuming resources/file proof. Keep
compiler/editor/docs and matched benchmark support current, bounded TDD checks,
project formatting, explicit commits and exact SQLite evidence. S07 family acceptance
and measurements remain pending; goal stays active. Provide the next physical handoff.


## Installed consumer publication

Source acceptance commit: 3ec1f7ce. Five consumers are rebuilt/refreshed atomically
from this checkpoint, with matching SHA-256 hashes. Debug/editor and release compiler
ownership fixture executions pass; LSP initialize/shutdown/exit protocol smoke passes.
Exact publication/build/smoke/cleanup evidence is in SQLite at
runtime/results/gate188-2026-10-10-publication.json, with byte-for-byte readback.
Superseded test executable variants and closed generated native outputs are pruned;
shared libraries, the warm target and all five installed consumers are retained.
