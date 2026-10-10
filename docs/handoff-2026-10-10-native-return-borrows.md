# Gate 189 returned-borrow metadata and slice helpers

Read AGENTS.md, docs/development-workflow.md, docs/resources-bytes-plan.md and
native-byte-views-implementation.md first. This checkpoint follows 56de6342, the
strict UTF-8 increment. Gates 187–188 remain accepted. Gate 189 and S07 remain open.
One agent works in this tree. There are no protected owner fixtures or unrelated
source edits.

## Implemented in this checkpoint

NativeBinding has an explicit optional returned-borrow contract. The isolated
native_return_borrows module identifies a zero-based source parameter, readonly
or exclusive access, nominal payload identity and direct or Result Ok return slot.
Validation rejects missing, optional, rest, consuming, copy and storage-free source
parameters, including sources after an optional or rest parameter. Exclusive return access requires a mutate parameter. The selected result
slot must contain the exact payload identity. Existing native bindings explicitly
have no returned borrow; their API and owned results are unchanged.

Export lookup validates contracts and preserves validation errors. Native type
projection validates before interning records. It deliberately rejects even valid
returned-borrow contracts until provenance and lifetime-bearing emission are wired.
Do not remove this guard merely to make an import compile. The unit tests use Bytes
as a synthetic payload identity, not as a published borrowed representation. When
introducing ByteView and ByteViewMut, validate their actual borrowed representation
and access capability as well as the signature slot.

views_runtime.rs contains safe Rust slice helpers returning &[u8] and &mut [u8].
They share checked_range with copy: start is checked before end, strict half-open
bounds apply, empty end ranges succeed, and errors preserve the owner. Nested views
retain exact storage identity. Mutable writes reach the source. Existing at/set,
copy and UTF-8 helpers already take slices internally, so they can reuse this backend
without a hidden copy.

These helpers are not emitted or exported by the Tsrust compiler yet. No source view
syntax, native identity, stored factory relationship or lifetime loan is accepted.
The existing native-byte-views/read.tsr probe still fails at import lookup. Safe Rust
helper tests are not evidence of Tsrust source-level borrow checking.

## Verification and retained failures

46 effective focused checks pass: five returned-borrow controls, one recursive native
signature test, eighteen declaration checks, eight native byte project tests, twelve
byte runtime tests, one native slice integration test and one editor UTF-8 test.
The integration test compiles and executes at O0 and O3. Across 32 iterations per
binary, successful and failed view operations allocate/reallocate zero times, preserve
source pointers and capacity, and support nested mutable writes. UTF-8 decode succeeds
on a Unicode subrange and reports a slice-relative offset for a truncated sequence.

TDD evidence includes the missing metadata API, projection unexpectedly succeeding
before its guard, and missing view helpers. The first unregistered metadata test
invocation ran zero tests; it is retained but is not counted as evidence. A native
integration harness initially passed Command where &mut Command was required. That
failed build and the repaired successful check are both retained. No behavior assertion
was weakened. An additional red witness catches a source after an optional parameter;
the repair also rejects sources after rest parameters. Existing native project fixtures execute after stable formatting.
A focused matched native_byte_copy_append run passes all five native/Node-after-tsc
cases after the shared range helper change. This is correctness-only evidence.
Whole-project formatting changes zero files and zero benchmark hashes. No analyzer
recursive dispatch changed; no depth-canary batch, whole regression or timing ran.

Exact records imported into benchmarks/records.sqlite, verified byte for byte, and
loose copies removed:

- runtime/results/native-return-borrows-2026-10-10-checks.json
- runtime/results/native-byte-view-runtime-2026-10-10-checks.json
- runtime/results/native-byte-view-runtime-2026-10-10-repair.json
- runtime/results/native-return-borrows-2026-10-10-checkpoint.json
- runtime/results/2026-10-10T15-37-46-103Z-verify.json

No view workload is registered yet because source views are not executable. Add the
matched Uint8Array subarray family, independent oracle and native ABI support during
source integration, before gate 189 acceptance. Existing UTF-8 workload remains
implemented and unmeasured.

## Next implementation

Introduce dedicated ByteView and ByteViewMut identities and borrowed storage metadata.
Then integrate returned source provenance with native calls, Result success payloads,
bindings, moves, branch joins, forwarding returns and stored factory values. Preserve
all possible source roots. Borrowed parameter returns can escape with inferred lifetime
relationships; local owners and consuming parameters cannot supply escaping views.
Lifetime-bearing native records/unions and aggregate wrappers must hold actual Rust
slices. Never use erased pointers, integer handles, hidden shared owners or copies.

Connect shared readable/mutable byte contracts so length/at/toUtf8/copy work on owners
and views, and set works on mutable owners/views. Preserve readonly concurrency and
exclusive conflicts, nested reborrows, Err loan release, all exit paths and scoped
aggregate/capture storage. Publish factories only when analyzer and backend agree.
Add complete catalog/editor coverage with the first source implementation, not later.
Run the normal-stack depth canaries after recursive analyzer dispatch changes.

Gate 190 remains consuming resource handles and a useful synchronous file proof.
Close consumes even when it returns Err; later use and double close are static errors.
Fallback drop is nonthrowing. Full regressions, accumulated measurements and performance
review belong to the completed S07 milestone. The overall goal stays active.

## Environment and consumers

Reuse /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0. Node 24.11.1
must be first on PATH; use /opt/homebrew/bin/python3 for store.py. Warm Cargo artifacts
include this checkpoint; executable publication remains separate. Installed five consumers still represent gate 188 source
3ec1f7ce; they were not refreshed here. Standalone O0/O3 outputs use disposable test
directories; no per-gate target or archive was created. Keep the active shared Cargo
target and existing consumers. Use explicit Git paths and format the whole project
before source checkpoints.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-10-native-return-borrows.md. Read AGENTS.md,
docs/development-workflow.md and docs/native-byte-views-implementation.md first.
Gate 189 UTF-8, returned-borrow metadata validation and safe slice helpers are verified;
source views remain unimplemented. Integrate native borrowed identities, provenance,
loans and lifetime-bearing ABI without copies or erased lifetimes. Keep the projection
guard until those layers agree. Complete editor/catalog and matched view benchmark
support before accepting gate 189, then continue gate 190 consuming resources. Use
bounded TDD, whole-project formatting, explicit source checkpoints and immutable
SQLite evidence. Provide a physical handoff at the next checkpoint.
