# Gate 189 strict UTF-8 increment; views remain pending

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md,
docs/native-byte-views-implementation.md and this handoff. Gates 187–188 are accepted.
Gate 189 is in progress; strict UTF-8 is implemented, but views are not. S07 is neither
accepted nor measured. One agent works in this tree; no protected owner edits remain.

## This increment

fromUtf8(text) borrows valid UTF-8 text and returns Result<Bytes, ByteError> with
independent exact bytes. toUtf8(bytes) readonly borrows Bytes and returns independent
owned text through Result<string, ByteError>. Strict decoding reports utf8 and the
first invalid sequence start, including truncated sequences. It never replaces invalid
bytes. Both preserve NUL and a leading U+FEFF; there is no normalization or BOM stripping.
Empty inputs succeed. Failed reservation returns allocation without offset; encoding
also has checked length-range failure. No partial output escapes.

The backend lives in utf8_runtime.rs and is emitted only for selected UTF-8 operations.
Decode checks validity once, then reserves and copies valid str into output; it does
not validate again through String::from_utf8. Safe Rust representations and the shared
native Result/error bridge are retained. Both operations have explicit nonthrowing
Result effects and complete catalog docs/examples, signatures and stored-function help.

## Verification

39 effective focused passes: 35 compiler (nine byte runtime tests, eight native project
tests, eighteen declaration controls), one editor and three harness controls. Private
reservation seams cover validation-before-allocation and no partial result. Native
cases include empty, ASCII, Unicode, NUL, BOM, malformed continuation, truncation,
overlong, surrogate and out-of-range encoding. Examples print 6 and é😀. Fixtures
execute after stable formatting. No analyzer recursive dispatch changed; no full
regression or timing ran.

The native_byte_utf8 workload retains seven observations for exact round trips,
byte lengths, strict error codes/offsets and unchanged source length. Node uses fatal
TextDecoder with ignoreBOM true, plus an independent byte scanner for error offsets.
Array construction validates integers before Uint8Array conversion on both sides.
Five native/Node-after-tsc cases pass after that refinement. Existing native_bytes
also passes five cases after selective backend emission. This mixed validation scenario
is not an isolated encoding throughput benchmark; all timing remains pending.

The allocation profiler omitted nested compiler source files from identity metadata;
it now reuses the execution runner's existing recursive collector. Both harnesses
fingerprint the metadata helper code. Current verification includes the new nested
utf8_runtime.rs source. Historical profile records were preserved unchanged.

Exact SQLite records, read back byte for byte and loose copies removed:

- runtime/results/native-byte-utf8-2026-10-10-checks.json
- runtime/results/native-byte-utf8-2026-10-10-supplement.json
- runtime/results/2026-10-10T15-20-35-844Z-verify.json
- runtime/results/2026-10-10T15-22-15-777Z-verify.json

The first record retains the original missing-fromUtf8 red output, first green output
and the missing-view probe. The supplement retains affected selection/metadata and
counterpart checks. Both verification epochs remain immutable; the last has the
validated counterpart's current hash.

## Next: real borrowed views

The native-byte-views/read.tsr file is an intentionally red CLI probe. It currently
fails because view is not exported; it is not an accepted executable test. There is
no Rust test that freezes rejection as desired behavior. Promote it into an executable
acceptance test when implementing views. Do not mistake its current import failure
for proof of lifetime diagnostics.

Existing callback-oriented record views and ViewUnion cannot supply returned native
slice lifetimes. The new physical integration plan maps the necessary work: returned
borrow metadata, parameter provenance, loan propagation through Result/moves/returns,
lifetime-bearing native records/unions, shared readable/mutable byte contracts,
reborrows, storage/capture restrictions and zero-copy controls. Use actual &[u8] and
&mut [u8]; do not substitute copies, hidden shared owners, integer handles or lifetime-
erased pointers. No source lifetime syntax is planned; infer parameter relationships.

Keep length/at/toUtf8/copy useful for readable buffers/views and set useful for mutable
owners/views through typed shared contracts. Preserve direct and stored function
help. Add depth canaries after analyzer dispatch changes. Complete all view exit paths,
UTF-8/view cleanup/allocation controls and matched view support before gate acceptance.

Gate 190 remains consuming resources and a useful synchronous file proof. Close
consumes even on Err, later use/double close is static error, and drop fallback is
nonthrowing. Full regressions and accumulated measurements belong to S07 acceptance.

Installed consumers still represent gate 188 source 3ec1f7ce. The warm Cargo compiler
has this increment; it is not installed consumer publication. Reuse
/private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0; Node 24.11.1 first
on PATH; /opt/homebrew/bin/python3 for store.py. Keep exact SQLite records, explicit
Git paths and whole-project formatting. Goal stays active.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-10-byte-utf8.md and
 docs/native-byte-views-implementation.md. Gates 187–188 accepted. Gate 189 strict
UTF-8 is implemented and verified; real scoped views remain. Start returned-borrow
metadata and native lifetime/loan integration, then complete views without copies
or erased lifetimes. Keep compiler/editor/catalog/benchmark parity, bounded TDD,
normal-stack canaries when dispatch changes, full-project formatting, explicit source
commits and immutable SQLite evidence. Then gate 190 consuming resources; S07 family
regressions/measurements remain pending. Do not repeat completed UTF-8 checks without
an affected change. Provide a physical handoff at the next checkpoint.
