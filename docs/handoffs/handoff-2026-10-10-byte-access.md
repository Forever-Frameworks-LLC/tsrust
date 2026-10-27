# Gate 188 checkpoint: checked byte access and mutation

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and this
handoff. Gate 187 is accepted at source 6e565765/publication e394c088. Gate 188 remains
in progress. This checkpoint implements at/set; do not repeat that work.

## Implemented and verified

at borrows Bytes and returns Result<int, ByteError>. set borrows exclusively and
returns Result<void, ByteError>. Bounds are checked before the byte value. Negative,
end and oversized indices return index-range with the requested index as offset.
Valid indices with values outside 0..255 return byte-range. Failed writes change no
bytes. Successful writes preserve buffer length.

The native bridge now shares checked Result conversion across construction, scalar
reads and void mutation success. Empty success records have no field0. Error conversion
is shared rather than repeated per operation. Runtime code uses safe slices and Vec;
there are no raw pointers or new dependencies. Failure transport remains Result,
with explicit nonthrowing native effect contracts. set is not classified as a readonly
observer by the callable leaf-effect helper.

Native witnesses cover boundaries/empty buffers, i64::MAX/negative indices, validation
precedence, atomic failure, stored native callables and index/value evaluation exactly
once. The evaluation counter's callback lives in an inner lexical block so its capture
loan ends before observing the counter. Static tests reject readonly writes, bad byte
types, wrong arity and use after transfer. Editor diagnostics retain the bad-value span;
stored-function help retains Mutable/ReadOnly modes and structured Result signatures.
Both new catalog entries have complete descriptions, parameter docs and executable
examples; reading prints 255 and writing then reading prints 90.

The effective focused batch has 34 passes: 21 compiler, 11 editor and two harness
controls. Two initial failures were in new verification code: a counter read while its
capture loan was live, and a negative oracle leaf incompatible with the existing
nonnegative observer. Their original command output remains preserved separately;
only affected checks were repeated after repair.

The new native_byte_access workload retains twelve observations per iteration,
including original/updated bytes, unchanged bytes after both failed write forms,
validation error codes/offsets, failed read and unchanged length. The negative index
observation is encoded as offset + 1; no observer protocol changed. Independent
validated Uint8Array counterpart and arithmetic oracle, source guards, hashes,
registrations and runner/profile/report hooks are ready. Both native_byte_access and
the existing native_bytes workload pass five native/Node-after-tsc cases untimed.
No timing or complete compiler/editor regression was run.

SQLite records (byte-for-byte readback verified; loose copies removed):

- runtime/results/native-byte-access-2026-10-10-checks.json (original batch)
- runtime/results/native-byte-access-2026-10-10-repairs.json
- runtime/results/2026-10-10T14-27-57-640Z-verify.json

Installed consumers still represent accepted gate 187. The warm Cargo compiler has
this increment, but that is not an installed consumer publication. Refresh the five
consumers after gate 188 acceptance with atomic sibling replacement.

## Next work

Finish gate 188: alloc with optional zero fill, append, resize with optional zero fill,
independent half-open range copy and explicit independent clone acceptance. Use
checked lengths/arithmetic and fallible reservation; no arbitrary cap. Validate before
mutation and preserve original contents on validation/reservation failure. Define
precise structured errors and their precedence in focused witnesses and catalog docs.

Complete capability, aggregate/capture and exact cleanup matrices, including O0/O3
allocation restoration. Expand matched retained observations as operations land;
do not postpone benchmark support until the family run. Gate 189 owns scoped zero-copy
views and strict UTF-8; gate 190 owns consuming resource close and the useful file proof.
Close consumes ownership even on Err and rejects later use/double close. Drop fallback
is nonthrowing. S07 is not accepted or measured.

Use one warm target, /private/tmp/tsrust-gate100-editor-target, with CARGO_INCREMENTAL=0.
Node 24.11.1 first on PATH; /opt/homebrew/bin/python3 for store.py. One agent in the tree,
explicit Git paths and whole-project formatting before checkpoints. No protected owner
edits remain. Preserve original evidence; do not restart unaffected checks. Full
regressions and timing belong to the completed S07 family milestone.

## Continuation prompt

Continue Tsrust from docs/handoffs/handoff-2026-10-10-byte-access.md and docs/plans/resources-bytes-plan.md.
Gate 187 is accepted; gate 188 at/set are implemented and verified. Finish allocation,
growth, independent copy/clone and capability/cleanup acceptance without repeating the
foundation or access work. Keep editor/catalog and matched workload support current.
Then proceed through gates 189–190. Keep consuming close semantics, bounded checks,
whole-project formatting, explicit commits and exact SQLite evidence.
