# B11/S07: resources and bytes

Status: gates 187–189 native declarations, owned bytes, scoped views and strict
UTF-8 have focused implementation acceptance. B11/S07 remains in progress; gate
190 resource lifecycle and the S07 family milestone remain pending. No byte workload is measured. B10/S06 is
accepted at gate 186. Resource close consumes ownership by owner decision.
See the gate 187 record and current continuation handoff.

## Audit of the current compiler

`native_bindings.rs` has scalar String/Boolean/Int/Number/Void signatures and three
private callback/observation intrinsics. There are no native owned types, structured
native results or borrowed native views. `syntax.rs::Type` and `ArrayElement` have
no native opaque identity. `project.rs` validates native imports through function
lookup, and `project_link.rs` maps every native import to a callable symbol. Thus a
native exported type needs a real type-export path, not an ordinary user record with
a forged implementation field. `analysis.rs::native_parameter_type`/`native_type`
and the editor's native signature rendering currently enumerate primitive cases.

The accepted type arena, Result constructors, generic specialization, ownership,
aggregate storage and project facts are reusable. Do not redo them or replace typed
native values with integers, dynamic payloads or user-constructible pointer records.
Existing std:path/os/testing surfaces must retain their identities and behavior.

## Public direction

Use named imports from `std:bytes`. `Bytes` is an owned contiguous byte buffer with
one byte per element. Source integers remain checked int; byte inputs must be in
0..255. Converting int[] into Bytes explicitly copies and validates values. Binding,
passing and storing Bytes never implicitly copies its allocation. `clone(bytes)`
means an independent buffer, not a shared reference-counted handle. Borrowed views
are scoped, zero-copy windows; they cannot outlive their owner or survive a conflicting
resize/mutation/transfer. They are not cheaply cloned owners.

Proposed initial free functions keep registry dispatch uniform:

```ts
import { Bytes, ByteError, fromArray, length } from "std:bytes";

function main() {
    const encoded: Result<Bytes, ByteError> = fromArray([0, 65, 255]);
    match (encoded) {
        Ok(bytes) => { console.log(length(bytes)); },
        Err(error) => { console.error(error.message); },
    }
}
```

Import ByteError when naming it; there is no implicit global native type namespace.
Type aliases and re-exports retain native declaration identity.
The first witness uses inferred constructor Result so it needs only Bytes explicitly.

Initial API contracts, to be registered together with their documentation:

| Operation | Contract |
| --- | --- |
| `fromArray(values)` | Readonly int[] input, validate all entries, produce `Result<Bytes, ByteError>`; report first invalid index, no partial buffer escapes. |
| `alloc(length, fill = 0)` | Checked nonnegative length and byte fill; zero initialized by default; typed size/allocation error where recoverable. |
| `length(bytes)` | Readonly borrow, int byte count, no allocation or mutation. |
| `at(bytes, index)` | Readonly borrow, checked nonnegative index, `Result<int, ByteError>`; no negative wrapping. |
| `set(bytes, index, value)` | Exclusive borrow; validate bounds/value before mutation; `Result<void, ByteError>`. |
| `append(bytes, values)` / `resize(bytes, length, fill = 0)` | Exclusive borrow; checked growth, no resize while any view is live; preserve original contents on validation/reservation failure. |
| `view(bytes, start, end)` | Readonly half-open zero-copy window with checked bounds; explicit source-owner provenance. Empty end views allowed. |
| `viewMut(bytes, start, end)` | Exclusive half-open window; mutations affect owner; overlapping views rejected conservatively unless disjointness is proved. |
| `copy(bytes, start, end)` | Explicit independent owned copy; checked half-open bounds. |
| `fromUtf8(text)` / `toUtf8(bytes)` | Explicit encoding/decoding. UTF-8 bytes, strict invalid-sequence errors on decode; no lossy replacement unless separately named. |

`ByteError` is a public nominal error record with `code: string`, `message: string`
and `offset?: int`; no native layout or credentials are exposed. Precise codes and
messages are fixed by each operation's acceptance witnesses. Structured record and
Result signatures must use arena identity, not display text.

No arbitrary memory limit is invented. Reject values that cannot fit native length
or source int, use checked arithmetic and fallible reservation where possible, and
document allocation failures that remain process-fatal. Do not promise universal
OOM recovery from ordinary runtime allocations. Encoding, endian numeric access,
base64 and streaming extend this foundation in separate coherent increments.

## Accepted lifecycle decision

The owner chose consuming close. Explicit `close(handle): Result<void, Error>`
consumes an owned resource when its backend can report failure. A second close or
later use is a source ownership error. Normal scope exit drops an unclosed resource
once without throwing. Failure of explicit close does not resurrect an owner.
Ordinary byte buffers need no close; their allocations release on scope exit.

An OS close is not automatically a durability barrier. Backends must document flush,
sync and close separately. Rust File drop ignores closing errors, so wrapping Drop
cannot honestly produce a close error Result. Establish reporting semantics with
the concrete backend before exposing them; destructor fallback remains nonthrowing.

## Implementation gates and acceptance criteria

### Gate 187: typed native declaration and signature foundation

Add a small native type/declaration registry independent of callable metadata.
Use stable native declaration IDs and recursive signature descriptions for owned
opaque types, exported records, arrays and Result. Resolve them through the existing
arena; cache by identity and checked arguments. Module loading/linking must separate
native type imports from value imports, including aliases, re-exports, private names,
unknown exports and incomplete project buffers. User records cannot forge native
identity. Keep old scalar binding metadata accepted through a compatibility adapter
while migrating call sites explicitly; do not scatter special Bytes spellings through
parsers and editor switches.

Acceptance: focused red Bytes import/Result witness becomes checked and native;
scalar path/os/testing controls still pass; generic/record/array/Result storage and
nominal identity witnesses pass; imported type hover/navigation/signatures work.
Register the first minimal matched byte construction/length workload, oracle,
contracts, hashes, driver and untimed verification in this gate. No timing yet.

### Gate 188: owned byte storage and checked operations

Implement Vec<u8>-backed storage, construction, length, checked at/set, allocation,
append/resize and explicit independent cloning/copying. Give native values capability
metadata for clone, inspection, equality, key eligibility and later thread transfer.
Unsupported capabilities get source diagnostics rather than invalid generated Rust.
Check moves, generic parameters, records, arrays, Maps, nullable/Result payloads,
closure captures and cleanup through return/throw/finally. No implicit handle clone.

Acceptance: empty/boundary/invalid sizes and values, one argument evaluation, mutation
atomicity, O0/O3 exact disposal, allocation restoration and source diagnostics; full
catalog behavior and executable examples; expanded matched retained observations.

### Gate 189: scoped byte views and strict UTF-8

Add readonly and exclusive view identities with source-parameter return provenance.
Reuse the loan system for owners, offsets and scopes; do not emit raw lifetime-erased
pointers. Check nested calls, returned views, storage/capture escape, overlapping
mutation, resizing and owner transfer. Views preserve zero-copy behavior with native
allocation controls. Implement strict UTF-8 encode/decode and error offsets, including
ASCII, multibyte scalars, NUL, invalid/truncated sequences and empty buffers.

Acceptance: valid views execute and mutate owners when exclusive; illegal lifetime
programs get source/editor diagnostics; release loans on all exits; matched Node
Uint8Array and TextEncoder/fatal TextDecoder counterparts retain equivalent results.
Measure no sharing/clone behavior that differs from the documented contract.

### Gate 190: opaque resource lifecycle

Apply the owner's close decision to the common resource capability and ABI model.
Prove deterministic native destruction with a non-public instrumented backend, then
with the first useful OS resource. Do not publish a fake testing handle as a standard
library API. A concrete synchronous file open/read/close slice can supply this proof
and is recorded as an S09 increment, without accepting the wider host family.
Define structured operation errors, close/flush distinction and failure ownership.

Acceptance: exactly one release on normal exit, early return, replacement, thrown
callback, Result failure and closure disposal; double close/use after close follows
the chosen rule; no implicit clone/compare/hash/thread sharing of noncapable handles.
Editor lifecycle documentation and diagnostics match compiler behavior. Filesystem
fixtures use isolated temp directories and remove them; no owner files are touched.

### Family milestone: S07 acceptance

Finish all preceding compiler/native/editor/docs/benchmark support first. Run complete
compiler and editor regressions with exit-code, abort and missing-suite detection.
Preserve completed suites and original failures. Measure only accumulated new/affected
workloads with original protocols, compare comparable performance and allocations,
then optimize justified regressions. Review typed native bytes against direct Rust
where wrapper overhead is the question, as well as the matched Node workload.
Import exact records into SQLite, read back and remove loose records. Update the
ledger, coverage/update.mjs, roadmap, acceptance map, guide and handoff. Format the
entire project before each source checkpoint. Refresh five consumers atomically only
from verified source; prune obsolete outputs after evidence closes.

## Module boundaries

- `native_types.rs`: declaration identity, capability metadata and recursive signature
  descriptors; no AST mutation or backend code generation.
- `native_type_resolution.rs`: arena interning/substitution and checked signature
  projection; no source-name-based identity.
- Native module export support in focused project loader/linker helpers; preserve
  current function resolution and source spans.
- `stdlib/bytes/{bindings,types,runtime}.rs`: byte contracts and safe generated Rust;
  operation analysis in a dedicated helper, not expanded recursive dispatch frames.
- `native_resources.rs` and a focused native lifecycle emitter: ownership capabilities
  and destruction/close protocols, shared by future files/sockets/databases.
- Editor native type projection shares compiler identity/public metadata. Catalog
  prose remains in builtins.json; runtime versions and evidence stay in developer docs.
- Separate compiler integration tests, native fixtures, project editor tests and
  independent benchmark source/oracle/contract modules for each coherent slice.

Names are intended boundaries, not a requirement to create empty scaffolding files.
Keep normal worker stacks and rerun depth canaries after dispatch changes.

## Research and constraints

Rust Vec stores initialized contiguous elements and distinguishes length from capacity;
use safe Vec<u8> APIs, not transmuting int[] storage or constructing raw parts:
https://doc.rust-lang.org/std/vec/struct.Vec.html

Rust slices are borrowed views. Ownership and lifetime constraints must be represented
in the checked source model before native emission:
https://doc.rust-lang.org/std/primitive.slice.html

The bytes crate offers shared cheaply cloned/sliced ownership. That is useful future
explicit sharing, but it differs from Tsrust's independent clone contract; it is not
an invisible replacement for the initial owned buffer:
https://docs.rs/bytes/latest/bytes/struct.Bytes.html

File drop ignores close errors and sync_all is a separate durability operation:
https://doc.rust-lang.org/std/fs/struct.File.html

These are design references, not claims of implemented APIs. No new package install
is required for the initial Vec/slice foundation. Tokio, HTTP/TLS, database wrappers,
queues and async cancellation remain downstream families.

## Construction witness

`tests/std_bytes.rs::native_byte_type_imports_and_result_payloads_execute` formats
its fixture idempotently and runs it through the project CLI with std:testing.
The original red failed on unknown std:bytes, then on missing fromArray. It now
passes with empty input, reusable borrowed arrays, both byte bounds and first-invalid
offset checks. The initial failures were new-feature witnesses, not B10 regressions. Exact evidence is retained
in SQLite at `runtime/results/native-owned-bytes-2026-10-10-red.json`. Native byte source integration is implemented; gate acceptance still needs its
matched workload and remaining alias/storage coverage. Installed consumers remain the accepted B10 build.

## Continuation prompt

Continue B11/S07 from docs/handoff-2026-10-10-gate187-accepted.md. Gate 187 is accepted:
native identities/imports/aliases, recursive callable signatures, fromArray/length,
native Result and aggregate storage, editor help and the matched workload all have
focused evidence. Do not redo them. Gate 188 at/set access and mutation are implemented with focused evidence. Next
implement alloc, append, resize, independent copy/clone and the capability/cleanup
matrix, then gates 189 views/UTF-8 and 190 resources. Consuming close
rejects later use/double close even after failure. Keep bounded checks and complete
matched support as operations land. Full regressions and timing belong to the completed
S07 family milestone. Preserve the existing declaration-only match inference limit.

## Gate 187 signature metadata increment

`native_signatures.rs` defines identity-bearing Named, Array and Result metadata,
primitive signatures and explicit private intrinsic signatures. Rendering requires
the caller to supply visible native aliases; an unresolved identity returns an
error instead of borrowing an unrelated source name. Rendering is iterative and
bounded to 128 nesting levels, 4,096 nodes and 65,536 output bytes. Primitive and
existing intrinsic names return borrowed static text, preserving the old editor
path without extra string allocations. These are compiler metadata identities,
not newly published source exports.

`NativeType::signature` adapts existing registry types to this shared description.
Project-native editor imports use it rather than maintaining a separate exhaustive
type-name switch. Four focused metadata controls and five native project editor
controls pass, including alias identity, depth/size failures, unchanged scalar and
callback help, stored/factory native values and optional signatures. Exact evidence
is stored in SQLite at `runtime/results/native-signatures-2026-10-10-checks.json`.
No full regressions, timing or consumer refresh were performed for this increment.

Next: register native declarations and separate type/value exports, then resolve
these descriptions into checked arena types and native storage. Do not mistake
rendered metadata for checked Bytes support: std_bytes.rs is still intentionally
red. Resource close semantics are decided: consuming close, with static rejection
of later use and double close.

## Gate 187 declaration metadata increment

`native_types.rs` now owns native declaration lookup by stable identity and exact
module/export name. Bytes has opaque representation with no public fields and no
source construction capability. ByteError has public code/message string fields
and an optional int offset. Explicit independent cloning is declaration metadata;
it does not authorize implicit copying. The compiler and emitter must enforce these
capabilities when the declarations are integrated into the checked arena.

Two focused tests failed first because the registry did not exist, then passed.
They cover declaration identity, module/name isolation, opaque construction metadata
and the exact error field contract. Project formatting changed zero files. Evidence
is in SQLite at `runtime/results/native-declarations-2026-10-10-checks.json`.
This metadata does not yet publish std:bytes imports or implement byte operations.
Next work remains loader/linker type exports, arena resolution and native storage.

## Gate 187 native export resolution increment

`native_exports.rs` distinguishes function exports from native type declarations.
Strict and tolerant project loading share this lookup and preserve imported aliases
without loading native modules from disk. Runtime feature selection validates type
exports without treating them as callable imports or selecting unrelated runtime code.
Existing scalar function lookup remains available for callable analysis.

Five declaration/export tests pass, including alias preservation, unknown module and
export errors, and feature selection. The existing project-native import control and
five editor-native controls are recorded in SQLite at
`runtime/results/native-export-resolution-2026-10-10-checks.json`. Project formatting
changed zero files. The integration witness still fails intentionally, now at the
missing `fromArray` export rather than the Bytes import. No native byte operation,
checked opaque storage or imported type help is claimed by this increment.

Next: resolve declaration IDs into checked arena types and link native type names
separately from callable symbols, including type aliases and re-exports. Then connect
fromArray/length, their catalog help and the matched workload. Consumers remain B10.

## Gate 187 arena projection increment

Native arena records now carry an explicit `native_type` declaration ID. Source
record, interface, tuple, enum, collection and Result constructors set it to None;
field shape and a source spelling cannot manufacture native identity.
`native_type_resolution.rs` interns native declarations by ID, projects recursive
Named/Array/Result signatures into structured annotations and preserves optional
fields as unions with undefined. Repeated imports share the arena declaration.
Projection is bounded to depth 128 and 4,096 visited nodes. Failed projection removes
new declarations and retains previously interned declarations.

This module is isolated from project linking and emission for now. Native Bytes
must not enter source checking until opaque construction, capability checks and
native storage emission are connected together. ByteError remains a nominal public
record; Bytes remains opaque, never an empty source record representation.

Focused evidence is stored in SQLite at
`runtime/results/native-arena-projection-2026-10-10-checks.json`. Continue by wiring
native imports into the project linker's early alias visibility and final name map,
then inject interned declarations into the linked program. Add source witnesses for
aliases, re-exports, nominal distinctions, opaque construction rejection and native
storage before claiming checked Bytes support. Gate 187 and S07 remain open.

## Gate 187 checked import increment

The project linker now separates native type symbols from callable symbols. It
interns imported declarations in the linked arena and includes native identities in
early alias visibility. Stable native names use the existing linked-name namespace
with unspellable separators, while declaration IDs remain authoritative.
ByteError aliases now check and execute with required fields and omitted or present
offsets. Bytes parameters check, native storage emits a Vec<u8> field, and source
record literals receive a native-opaque diagnostic rather than constructing Bytes.

The native fixture exposed a prior parser defect: wrapped parameters such as
ReadOnly<Alias> did not expand local aliases. Wrapped parameter parsing now applies
the same alias expansion as ordinary annotations. Nine focused declaration/import
checks pass, including the formatted native error fixture. All ten depth canaries,
the type-alias suite and five existing native editor controls pass. Exact evidence
is in SQLite at `runtime/results/native-checked-imports-2026-10-10-checks.json`.

Gate 187 is still not accepted. Native type hover/completion, re-export witnesses,
opaque observation/capability guards, fromArray/length and their documented matched
workload remain. Do not publish Bytes construction or claim S07 acceptance from this
checkpoint. Installed consumers remain the accepted B10 build.

## Gate 187 native type editor increment

Native declarations now supply type hover, alias spelling, public field help and
import completion. Type completion uses the struct item kind rather than the
function item kind. `native_type_intelligence.rs` owns metadata-to-symbol projection;
private storage fields never become editor properties. The catalog documents Bytes,
ByteError and each public error field, with purpose, examples and ownership notes.
Bytes documentation explicitly states that construction operations are not available
at this checkpoint.

Checked project facts now carry `ProjectType::Native(NativeTypeId)` rather than
misidentifying the first import span as a source declaration. Checked receivers keep
local native aliases. Tolerant borrowed receivers strip their access wrapper only
when the inner name resolves to a registered native type alias. Tests cover checked
and declaration-only help, a truncated body, public field completion and native
import completion. Existing native function help remains unchanged.

Nine native declaration tests, one existing nominal project fact test and six native
editor tests pass. All five new catalog code examples format idempotently and execute
natively with exact expected stdout. Evidence is in SQLite at
`runtime/results/native-type-editor-2026-10-10-checks.json`; the editor module extraction
is supplemented at `runtime/results/native-type-editor-module-2026-10-10-checks.json`.
Project formatting changed zero files. No full regression, timing or consumer refresh
was performed. Gate 187 remains open: capability guards, re-export coverage, byte
constructors/length, signature help and the matched workload still need implementation.

## Gate 187 opaque capability increment

Native declarations now explicitly list inspection, string conversion, property
reflection, object spread and value comparison capabilities. Bytes currently grants
none of these; ByteError retains ordinary public-record behavior. The independent
clone flag remains separate and never permits implicit copying.
`native_capabilities.rs` checks identities with an iterative walk through nested
records, arrays, unions and collection payloads. Unsupported operations receive a
native-capability source diagnostic at the affected argument. Console inspection,
String, template interpolation, Object keys/values/entries/hasOwn, object spread and
Object.is use the relevant guard. Emitter inspection support follows the same metadata.

A focused red test demonstrated that Bytes console inspection was previously accepted
as an empty record. The audit also found that spread shape caching could reuse the
empty native declaration and manufacture a Bytes owner from ordinary empty records.
Native declarations are excluded from inferred spread shapes, and opaque expected
spread targets reject construction. Fixed-storage layout inference no longer derives
an opaque owner's size from its empty public field list.

Eleven focused native declaration tests and the opaque layout unit pass. Rejection
witnesses include direct, array, union, Map and nested-record inspection, conversions,
reflection, spread and comparison. The native ByteError fixture also proves that
public reflection and explicit spread remain supported. Editor conversion verifies
the native diagnostic code and exact argument span. Bounded existing-operation and
depth controls are retained in SQLite at
`runtime/results/native-capabilities-2026-10-10-checks.json`.

Gate 187 remains in progress. Next work is native type re-export coverage and first
fromArray/length bindings with checked recursive signatures, Result payload storage,
full catalog help and the minimal matched workload. Gate 188 still owns the wider
byte operation, clone, aggregate-storage and cleanup acceptance matrix. No full
regression, timing or consumer refresh is due for this isolated increment.

## Gate 187 exported native alias increment

Native identity now survives direct exported aliases in editor declaration indices.
`refresh_native_aliases` promotes a direct native alias in place, retaining its source
extent and user documentation, and supplies native fields and catalog help. Imported
native aliases copy the identity with their local spelling. Navigation stays on the
written alias declaration in the exporting source module, rather than pretending a
native declaration has a source file. Ordinary records keep their existing import path.

A focused editor witness first returned null for the exported Buffer alias, then
passed for checked and declaration-only indices. The native execution witness covers
renamed native imports, exported direct and array aliases, readonly error access,
optional offsets and shared Bytes identity across modules. Private native import names
remain private, and constructing an exported opaque alias with a record literal is
rejected. Ordinary compiler/editor alias controls and imported Result help also pass.
Evidence is in SQLite at `runtime/results/native-reexports-2026-10-10-checks.json`.

This increment covers direct native alias editor projection. Structured exported
aliases that hide a native element still need editor materialization when no direct
native type is visible; include that case with the first native Result binding help.
Next implement fromArray/length and their checked recursive binding signatures,
Result ABI, catalog examples and matched workload. Gate 187 remains open.


## Gate 187 first byte operations increment

Native callable metadata can now reference structured signatures. Private schema
fields resolve these signatures through ordinary generic specialization, reusing
checked Result alternatives and nominal native payload identities. No second union
representation or dynamic native handle was introduced. Direct and forwarded calls
resolve these prepared types in the program arena.

The std:bytes fromArray binding borrows int[] and validates all elements before
allocation/copy. It returns a new Vec<u8> owner or a ByteError in Result. The first
invalid element reports byte-range with its array offset. Length overflow reports
length-range; failed reservation reports allocation. The implementation does not
promise recovery from every process-wide memory exhaustion. The length binding
borrows Bytes and returns its checked int byte count. Both bindings have explicit
nonthrowing effect contracts; Result failure is distinct from thrown failure.

The native bridge uses the existing checked Result alternatives and ByteError
optional-offset union. Public source cannot construct the private Vec field. The
editor renders recursive signatures using visible aliases, materializes hidden
native payload identities under private collision-free names, and displays public
names in help. Checked match error fields have catalog documentation and completion.
Declaration-only match bindings retain the existing lack of payload inference;
function help remains available without checked facts.

Focused native, ownership/arity/type rejection, runtime independence, editor and ten
depth controls pass. The two new catalog examples execute with their expected output
and stable formatting. Exact evidence is retained in SQLite at
runtime/results/native-byte-construction-2026-10-10-checks.json. Gate 187 remains open:
matched workload support, structured exported-alias editor coverage and remaining
storage/callable witnesses are next. No timing, full regression or installed consumer
refresh was performed for this increment.


## Gate 187 closing increment

Stored native constructors/observers and callback forwarding execute with generic
identity, Bytes record/array storage and Result record/array storage. Structured
exported array and Result aliases now keep hidden native field help, public display
names and source navigation. A local record named ByteError cannot capture the native
identity. Internal editor symbols are excluded from global completion. Closing aliases
also found and repaired built-in Result qualification in exported compiler aliases;
both primitive and native Result aliases execute afterward.

The minimal native_bytes workload and independent TypeScript/oracle pair are registered
with source guards, runner/profile/report hooks and six retained observations per
iteration. Five native/Node-after-tsc cases pass untimed. The closing focused batch
has 68 passes, with exact SQLite evidence and byte-for-byte readback. Gate 187 is
accepted; S07 is not. See docs/milestones/187-native-byte-foundation.md. Next gate is
188; do not repeat the completed foundation or retime unaffected workloads.


## Gate 188 checked access and mutation increment

The at binding borrows Bytes and returns Result<int, ByteError>. Negative, end and
oversized indices return index-range with the requested index as offset. The set
binding borrows exclusively and returns Result<void, ByteError>. It checks index
before value, accepts only 0..255, and changes nothing on either failure. The shared
native bridge now projects arbitrary checked Result payloads and empty void success
records; error conversion is shared. The backend remains safe Vec/slice code.

Native witnesses cover endpoints, empty buffers, negative/maximum indices, both
invalid byte bounds, failure precedence and atomicity, stored native functions,
readonly/exclusive argument contracts and one evaluation of each index/value argument.
The callback used to count evaluations lives in a nested scope so its capture loan
ends before inspecting the counter. Compiler/editor type diagnostics preserve the
invalid value range; stored function help preserves Mutable/ReadOnly and Result
signatures. Complete at/set catalog entries and two native output examples pass.

The native_byte_access workload retains twelve observations per iteration and has an
independent validated Uint8Array counterpart and arithmetic oracle. The negative
requested index is encoded as offset + 1 for the existing nonnegative observer,
without changing the observer protocol. Source guards, fingerprints, registration,
runner/profile/report hooks and five native/Node-after-tsc cases are ready and verified
untimed. The existing construction workload also passes five cases after the bridge
refactor. No timing or full regression was performed. Original failed checks and their
focused repairs are retained separately in SQLite.

Gate 188 remains in progress. Allocation/growth/copy/clone, broader capability
acceptance and exact cleanup remain next. Installed consumers still represent gate
187; this increment is a source checkpoint, not a consumer publication.


## Gate 188 allocation and resizing increment

The alloc and resize bindings use optional int fill arguments, defaulting to zero,
and the shared structured Result bridge. Length validation precedes fill validation,
even for empty, unchanged or shrinking requests. Resizing preserves prefixes and
validates before mutation; fallible growth reservation precedes filling. Shrinking
retains the existing Vec capacity and does not expose it publicly. Errors omit offset
because length and fill refer to arguments rather than an individual buffer element.

Focused source/native witnesses cover default fill, explicit fill endpoints, empty
allocation, shrinking/regrowth, stored functions, validation precedence and unchanged
contents after errors. A private runtime reservation seam verifies failure leaves
contents, length, pointer and capacity unchanged without attempting huge allocations.
Static witnesses reject wrong types/arity, readonly resizing and use after transfer.
Editor help preserves optional fill parameters and exclusive access. Both catalog
examples execute and print 3. The native_byte_growth matched workload retains eleven
observations with an independent arithmetic oracle and validated Uint8Array counterpart.

Gate 188 remains open for append, independent half-open copy, explicit clone and
capability/aggregate/capture/exact cleanup acceptance. Consumers remain at gate 187.
The consuming close decision remains unchanged: close transfers the resource even on
Err; later use and double close are static errors, with nonthrowing drop fallback.


## Gate 188 append and copy increment

append exclusively borrows the destination and readonly borrows an int array. It
checks combined length, validates every byte in input order, reserves fallibly and
only then appends. Empty input requires no reservation. Invalid values report their
input-array offset. copy readonly borrows Bytes and produces an independent owner
for a strict half-open range, including empty end ranges. Start is checked before
end; invalid or reversed end reports end. There is no negative wrapping or clamping.

Native witnesses cover successful/empty/invalid append, unchanged contents after
failure, aliases stored as callable values, strict ranges and independence after
mutation of either owner. Private runtime seams force reservation failure and prove
unchanged destination contents, pointer and capacity without huge allocations.
Editor/catalog help documents access modes, errors, ranges and ownership. Matched
native_byte_copy_append support retains thirteen observations with independent
arithmetic expectations and a validated Uint8Array counterpart. Explicit clone and
broader capability/aggregate/capture/evaluation/cleanup matrices remain to close gate
188. Views and UTF-8 remain gate 189; consuming resource lifecycle remains gate 190.


## Gate 188 ownership and cleanup acceptance

The ordinary clone operation already supports independent Bytes and aggregate copies;
new acceptance witnesses prove direct, array-record, Map, nullable and Result cloning,
generic storage, escaped reusable/Once captures, unused captures, callback failure and
finally behavior. A shared State counter proves allocation, resize, append and copy
arguments evaluate once. Existing source diagnostics reject opaque collection keys
and equality rather than emitting invalid backend code.

Native declaration metadata now explicitly records collection-key and owned native
thread-transfer eligibility. Bytes and ByteError are not supported collection keys;
their owned safe Rust representations can transfer across a future native thread
boundary. This metadata does not implement a source thread API. Cloneability honors
the native independent-clone capability and propagates rejection through aggregates.

A separate cleanup fixture owns exactly sixteen byte records per invocation across
normal/early return, replacement, Result error, throw, finally, unused/throwing/Once
callbacks and aggregate disposal. A private generated-code Drop observer checks all
sixteen releases; a global allocator observer requires live bytes to return exactly
to baseline after each of thirty-two invocations. O0 and O3 both pass. No public fake
resource API is used. The first cleanup fixture needed a named record annotation;
its original failure and affected repair are retained in evidence.

The native_byte_clone matched workload retains seven values proving direct and
array-record independence after writes to both source and copy. It has explicit deep
Uint8Array copies, an arithmetic oracle, source contracts and complete harness hooks.
Global clone documentation is now in the shared catalog with an executable byte
example. Gate 188 closes with focused evidence; S07 still needs gates 189–190 and the
family regression/performance review. No byte workload is measured.


## Gate 189 strict UTF-8 increment and view architecture

fromUtf8 and toUtf8 are implemented through the accepted owned native Result ABI.
The separate utf8_runtime.rs backend checks valid text once, reserves fallibly and
copies into independent output. Encoding preserves exact source bytes; strict decoding
reports the first invalid sequence start. Empty, ASCII, multibyte, NUL, BOM, overlong,
surrogate, out-of-range and truncated cases have native/reference coverage. Private
reservation seams prove errors expose no partial output, and malformed input does
not reserve. UTF-8 backend source is emitted only for selected UTF-8 operations.

Both operations have complete catalog examples and stored-function editor help.
The native_byte_utf8 matched scenario retains byte lengths, exact round trips,
malformed/truncated error codes/offsets and unchanged buffer length. Node uses fatal
TextDecoder with ignoreBOM true to preserve U+FEFF, and an independent byte scanner
supplies offsets after decoder errors. This mixed scenario is not an isolated encoding
throughput claim. The allocation profiler now uses the execution runner's recursive
compiler-source collector; previous profile metadata omitted nested standard-library
files. Historical records remain unchanged. Both harnesses fingerprint the collector.

Views remain pending. The current callback-oriented record views and ViewUnion do not
represent returned native slice lifetimes. docs/native-byte-views-implementation.md
records the integration route: native return provenance, source loan propagation,
lifetime-bearing record/union ABI, typed readable/mutable byte contracts, safe return/
storage/capture rules, zero-copy controls and matched view workloads. The retained
native-byte-views/read.tsr CLI probe fails because view is not exported. It is not an
accepted executable fixture. Gate 189 and S07 remain open; no timing was performed.

### Gate 189 returned-borrow groundwork

NativeBinding now carries a validated returned-borrow source/access/payload contract.
Projection fails before interning unsupported borrowed results, so adding metadata
alone cannot publish an unsound owned ABI. Existing native bindings remain owned.
Safe slice helpers share copy's range validation and support readonly/exclusive
nested reborrows. They are backend groundwork, not source-language view acceptance.
See native-byte-views-implementation.md for the remaining provenance, representation,
loan and editor integration. Gate 189 and the S07 milestone remain open.

### Gate 189 borrowed identities

ByteView and ByteViewMut have private identity-bearing declarations with readonly
and exclusive borrowed representations. Contracts validate payload representation,
access and the absence of untracked Result error borrows. Bounded shape traversal
finds borrowed storage in nested signatures; observers cannot bypass the pending
provenance/ABI guard. Recursive identity projection and native fixed-storage controls
are verified. Source exports and factories remain unpublished. Continue with source
loan propagation and safe lifetime-bearing emission; this is not gate acceptance.

## Gate 189 focused acceptance

The approved byte-view and recursive-storage scope is accepted at source checkpoint
`bd4b0d02`, with requirement mapping in `milestones/189-native-byte-views.md`.
Earlier pending callback-composition statements describe historical checkpoints.
Gate 190 and S07 complete regressions, matched timing, performance review and final
consumer refresh remain open. No measurement acceptance is implied.

## Gate 190 backend increment

The private `stdlib/resource.rs` owner shares explicit close and fallback release,
with an instrumented backend and O0/O3 lifecycle witnesses. Source handle identity,
capability metadata, file operations and source-level cleanup acceptance remain
pending. This increment publishes no fake resource API.

The concrete backend must report the actual close operation. Rust's
[File documentation](https://doc.rust-lang.org/std/fs/struct.File.html) confirms
that Drop ignores closing errors and that synchronization is a separate operation.
Apple's [close reference](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/close.2.html)
documents an interrupted-call error; platform ownership behavior must be established
before choosing a retry policy. Do not implement fallible close by merely dropping
a Rust File or silently substituting sync_all.

### Concrete read-only file backend

The private `stdlib/fs.rs` backend opens existing paths read-only, reads at most a
requested byte count from the current cursor and consumes its resource on close.
Read lengths are bounded to 0 through 67,108,864 bytes per call. Validation and
fallible buffer reservation precede I/O; zero length performs no I/O. Short reads
and empty EOF results succeed. Interrupted reads retry; other read errors retain
the handle for later close. This is a bounded read operation, not read-to-end.

Errors carry operation, stable category, OS message and optional raw OS code.
Explicit close transfers the Rust File descriptor and invokes native close once;
failure does not restore ownership. It does not flush a user-space buffer or sync
file contents. Fallback uses the same release path and discards reported errors.

Platform analysis supports that policy on macOS and Linux. Apple's current
[XNU descriptor implementation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_descrip.c)
removes the descriptor before returning the file backend's close result.
The [Linux close manual](https://man7.org/linux/man-pages/man2/close.2.html)
likewise explains early descriptor release and the danger of retrying close.
This is a platform-specific policy, not a claim about every POSIX implementation.
Native execution in this checkpoint is macOS-only; Linux execution remains unverified.

Isolated temporary-file tests cover cursor advancement, limits, NUL/missing paths,
directory read errors, success/EOF and explicit/fallback close. O0/O3 subprocesses
observe closed descriptors without attempting to close them again. An injected
post-release Interrupted error proves a single close attempt and a usable later
file; it does not claim to reproduce a hardware close failure. Test directories are
removed. Source exports, registry/ABI integration, editor documentation and matched
benchmark support remain the next gate 190 work. The backend is still test-only.

### Source file API integration

The preceding private-backend boundary is superseded: File/FsError identities and
open/read/close bindings now reach source checking, generated native storage and
Result bridges. File has no implicit clone, comparison, key or thread-transfer
capability. All three functions return operational errors in Result. Mutable read
borrows its handle; close consumes it even when returning Err.

Five complete catalog entries and their executable examples are verified at O0/O3.
The public read-close fixture also executes through stored native function values.
Source errors reject forged/cloned handles, double close, use after close including
the Err arm, inspection and comparison. Editor help and import completion match.
This is a focused S09 file increment within gate 190, not synchronous-host family
acceptance. Generated cleanup instrumentation, remaining editor diagnostics and
matched benchmark readiness are the next work; S07 remains unmeasured.
