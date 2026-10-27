# Gate 188 append and independent copy continuation

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and this
handoff. Gate 187 is accepted. Gate 188 remains in progress. The preceding allocation
and resize checkpoint is a5080dd5; this increment adds append and copy. No protected
owner edits remain. One agent works in this tree.

## Behavior and implementation

append(bytes, values) exclusively borrows Bytes and readonly borrows int[]. It checks
combined length first, validates every value in 0..255, reserves fallibly, then
appends in order. Empty input succeeds without reservation. Invalid values return
byte-range with their input-array index as offset; length-range and allocation omit
offset. No failure changes the destination's length or contents. This is an explicit
integer-to-byte copy, not conversion by wrapping.

copy(bytes, start, end) readonly borrows Bytes and returns Result<Bytes, ByteError>
with independent owned storage. Bounds satisfy 0 <= start <= end <= length(bytes).
Equal bounds, including the source end, return an empty buffer without reservation.
Check start before end; invalid start reports range with start as offset, and invalid
or reversed end reports range with end as offset. No clamping or negative wrapping.
Failed reservation returns allocation without offset. The source stays unchanged,
and later mutation of either owner cannot change the other.

Both bindings reuse the native type registry and Result bridge. Append's int-array
argument is lowered to a slice through the shared native call path, including stored
native callables. Both have explicit nonthrowing effects; copy is an observer in
callable leaf-effect analysis, whereas append is not. Private runtime reservation
seams cover failure atomicity without huge actual allocations.

## Verification

The focused batch passes 34 tests: 28 compiler (six runtime, six native project,
sixteen declaration tests), four editor and two registration controls. New native
witnesses cover valid and invalid append, input-array offsets, empty input, strict
ranges, stored operations and bidirectional mutation independence. Static witnesses
reject readonly append, wrong types/arity and use after transfer. Editor help retains
read/exclusive modes and structured Result types. Append and copy catalog examples
execute and print 3 and 2 respectively.

Whole-project formatting changes the two new Tsrust files and updates the new workload
fingerprint only after proving token preservation. Native acceptance fixtures use
stable format round trips and execute the formatted source. No full regression or
timing run belongs to this source increment.

The native_byte_copy_append workload retains thirteen observations: original length,
append success, invalid-byte code/offset, preserved trailing byte, copy length, both
write successes, independent copy/source values, empty end copy, invalid range offset
and final length. Matched Uint8Array operations explicitly validate before writes or
slice; an independent arithmetic oracle checks retained results. Source guards,
registrations, hashes and runner/profile/report hooks are in place. See the closing
verification record for native/Node correctness evidence. No workload is measured.

SQLite evidence paths, with byte-for-byte readback and loose copies removed:

- runtime/results/native-byte-copy-append-2026-10-10-checks.json
- runtime/results/2026-10-10T14-47-13-584Z-verify.json

The focused record preserves the full original red command output and first green
output. The initial witness failed because append was not exported. The runner-generated verification record proves five native and Node-after-tsc cases
pass untimed.

## Remaining gate 188 work

Explicit clone capability and independence need acceptance. Complete capability,
aggregate/capture, one-evaluation and exact O0/O3 cleanup/allocation restoration
matrices. fromArray, length, at, set, alloc, resize, append and copy already work;
repeat their completed checks only after an affected change. Do not accept gate 188
until its remaining criteria are proved. Views and strict UTF-8 remain gate 189;
consuming resource close and a useful synchronous file proof remain gate 190.

Owner chose consuming close, including on Err. Later use and double close are static
errors; unclosed resources have a nonthrowing drop fallback. Bytes itself needs no
close. S07 family acceptance, complete regressions and timing remain pending.

Installed consumers still represent gate 187. The warm Cargo compiler has this
source increment; it is not installed consumer publication. Refresh five consumers
atomically from accepted committed source. Reuse /private/tmp/tsrust-gate100-editor-target
with CARGO_INCREMENTAL=0; Node 24.11.1 first on PATH; /opt/homebrew/bin/python3 for
store.py. Preserve immutable SQLite records and format the entire project before
explicit-path source commits.

## Continuation prompt

Continue Tsrust from docs/handoffs/handoff-2026-10-10-byte-copy-append.md and
 docs/plans/resources-bytes-plan.md. Gate 187 accepted. Gate 188 basic byte operations through
append and independent copy verified. Finish explicit clone and capability/storage/
capture/evaluation/cleanup acceptance, then gates 189–190. Keep native/editor/catalog
parity, matched workload support, bounded checks, whole-project formatting and exact
SQLite evidence. Consuming close remains fixed. Do not reimplement accepted increments
or describe untimed workloads as measured.
