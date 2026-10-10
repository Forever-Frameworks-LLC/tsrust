# B11/S07: resources and bytes

Status: gate 187 signature metadata increment implemented; native byte source witness remains red. B10/S06 is accepted at gate 186, publication
commit `1b631986`. This family is not accepted. The owner is being asked to choose
consuming close versus idempotent close; dependent lifecycle implementation must
wait for the answer. Type identity, signatures and owned bytes can progress independently.

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

## Lifecycle decision pending

Recommended: explicit close consumes an owned resource and returns a Result when
its backend can report failure. A second close or later use is a source ownership
error. Normal scope exit drops an unclosed resource once without throwing. Failure
of explicit close does not resurrect an owner. Ordinary byte buffers need no close;
their allocations release on scope exit.

Alternative under owner review: close borrows mutably and is idempotent; the value
remains closed and operations report typed closed-state errors. This requires a
closed-state representation and flow rules distinct from mere ownership transfer.
Do not implement either alternative before the answer arrives.

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

## Current red witness

`tests/std_bytes.rs::native_byte_type_imports_and_result_payloads_execute` formats
its fixture idempotently and runs it through the project CLI with std:testing.
It currently fails with `module-native`: unknown std:bytes. This is an intentional
new-feature witness, not a regression in accepted B10. Exact evidence is retained
in SQLite at `runtime/results/native-owned-bytes-2026-10-10-red.json`. The signature metadata increment below is implemented; native byte source integration
remains unimplemented. Installed consumers remain the accepted B10 build.

## Continuation prompt

Continue B11/S07 from this plan. Read AGENTS.md and the development workflow first.
B10 is accepted at gate 186 (`1b631986` publication, `8d431eed` executable source).
Gate 187 has one intentional red integration test in std_bytes.rs: unknown std:bytes.
Implement typed native declaration identity, recursive signatures and distinct native
type/value imports before the first owned Bytes operation. Preserve Result and editor
parity, no implicit clone and normal worker stacks. Resource close semantics await
the owner response; do not choose that behavior silently. Native type/byte work is
independent and may proceed. Installed consumers remain B10 until verified increments
are published. No full regressions or timing are due for this red checkpoint.

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
red. Resource close semantics still await the owner's answer.
