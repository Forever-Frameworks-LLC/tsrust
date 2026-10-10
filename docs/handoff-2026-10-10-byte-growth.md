# Gate 188 allocation and resize continuation

Read AGENTS.md, docs/development-workflow.md, docs/resources-bytes-plan.md and this
handoff. Gate 187 remains accepted. Gate 188 remains in progress. This checkpoint
adds allocation and resizing after the checked at/set checkpoint 4305c72d. No other
agent is working in the tree, and no protected owner edits remain.

## Implemented increment

alloc(length, fill?) returns Result<Bytes, ByteError>. resize(bytes, length, fill?)
exclusively borrows Bytes and returns Result<void, ByteError>. Fill defaults to zero.
Both validate length before fill, including empty, unchanged and shrinking requests.
Invalid length produces length-range; invalid fill produces byte-range. These errors
omit offset. Growth reserves fallibly before mutation and initializes only new bytes.
Shrinking preserves the retained prefix and may retain capacity. Reservation failure
produces allocation without changing contents or length. There is no arbitrary size
cap and no promise of recovering from every process-wide memory exhaustion.

Both native bindings use the existing typed registry and shared structured Result
bridge, including optional argument lowering and stored native functions. They have
explicit nonthrowing effects. A private Rust reservation seam verifies failure leaves
contents, length, pointer and capacity unchanged without huge actual allocations.
No public testing backend or fake operation was added.

## Evidence

The closing focused batch passes 29 tests: 24 compiler (four runtime, five native
project and fifteen declaration tests), three editor and two harness registration
controls. Two new catalog examples execute and print 3. Native fixture formatting
is stable and formatted source executes. Whole-project formatting changed two new
files and refreshed the new workload fingerprint; the next formatting pass was clean.

The new native_byte_growth workload retains eleven observations per iteration. It
has a matched validated Uint8Array counterpart, independent arithmetic oracle,
source guards, registration controls and runner/profile/report hooks. Five cases
pass on native and Node-after-tsc, untimed. No full regression or timing ran.

SQLite retains these records with byte-for-byte readback; loose copies were removed:

- runtime/results/native-byte-growth-2026-10-10-checks.json
- runtime/results/2026-10-10T14-40-45-623Z-verify.json

The first source witness failed before implementation because alloc was not exported.
Only the diagnostic excerpt was retained, explicitly labelled as such in evidence.
A new static rejection test initially omitted main on helper-only cases; the entry
error was a test setup defect, repaired before the closing batch.

## Continue

Finish gate 188 with append, independent half-open range copy and explicit clone
acceptance. Extend capability, aggregate/capture, evaluation-once and exact O0/O3
cleanup witnesses. Allocation/resize already work; do not reimplement or repeat their
completed batch without an affected change. Keep independent matched workload support
current as operations land. Full family acceptance remains pending.

Gate 189 covers scoped zero-copy readonly/exclusive views and strict UTF-8. Gate 190
covers consuming resources and a useful synchronous file proof. Owner confirmed that
close transfers ownership even when it returns Err; later use and double close are
static errors. The drop fallback must be nonthrowing.

Installed consumers still represent gate 187. The warm Cargo compiler has this source
increment; that is not consumer publication. Refresh the five consumers atomically
from a committed gate acceptance. Use one warm target at
/private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0, Node 24.11.1 first
on PATH, and /opt/homebrew/bin/python3 for store.py. Preserve exact SQLite evidence,
format the whole project before source checkpoints, and use explicit Git paths.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-10-byte-growth.md and
 docs/resources-bytes-plan.md. Gate 187 accepted; gate 188 at/set/alloc/resize verified.
Finish append, independent copy/clone and capability/cleanup acceptance, then gates
189–190. Keep consuming close, complete editor docs, matched workload support,
bounded checks, project formatting and exact SQLite records. Do not restart completed
checks or describe untimed workloads as measured. S07 remains open.
