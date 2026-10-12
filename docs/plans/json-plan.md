# S10 JSON

Status: S10 remains open. Bounded dynamic JSON/JSONC, owned JSONL readers,
standalone offline package export, typed JSON/JSONC imports, project aliases, and
the approved JSON namespace have focused acceptance. Installed CLI/editor
consumers were most recently refreshed through `5682a032`, including raw-field decoding, corrected lifetime hover and borrowed JSONL adapters, alias-reference repair, bounded encoding optimization, and typed encoding/decoding, owned JSONL writers, complete JSON import hover, child views, boolean refinement,
guarded string reads, scalar extraction, typeof,
array/null inspection and reporting-only diagnostic suppression. Checked scalar extraction has [focused source acceptance](../milestones/json-scalars.md);
its consumer refresh is verified. Borrowed child operations now have [focused source acceptance](../milestones/json-views.md). Scalar and container construction have focused source acceptance.
Direct typed JSON decoding and encoding now have focused source acceptance.
Typed JSONC and JSONL reads have focused source acceptance. Owned and borrowed JSONL readers/writers have focused source acceptance. Allocation evidence covers numeric, Unicode, nested/selective, malformed/deep decoding, numeric/Unicode/object/deep encoding and owned/borrowed JSONL reading/writing: 69 cases and 621 fresh-process samples. The [implemented-feature regression checkpoint](../milestones/s10-json-regressions.md) passes 3,395 compiler and 576 editor tests after focused repairs. Public raw validation and typed-field borrowing have focused acceptance and numeric/Unicode/deep/rejection/selective allocation, timing and RSS evidence. Raw serialization and final S10 acceptance remain open. The initial 20-workload native/Node execution baseline is preserved.
S09 is accepted.

The roadmap requires both dynamic owned JSON values and direct typed decoding.
Typed decoding must not build a dynamic tree first. Schema fusion follows S11;
S10 must retain the typed decoder boundary and allocation evidence needed for it.

The approved global JSON namespace has [focused acceptance](../milestones/json-namespace.md),
including editor documentation and untimed matched workload checks. The required
[compiler source organization checkpoint](../milestones/compiler-source-organization.md)
is accepted; continue the remaining runtime APIs and S10 acceptance below.

See the [requirement-by-requirement contract audit](../milestones/s10-json-contract-audit.md) for the implemented evidence and the explicit raw-fragment gap.

## Current acceptance gaps

The implemented-feature regression checkpoint and measured baseline matrices are
preserved in SQLite. Original runs and focused repairs remain separate records.

| Requirement | Current evidence | Remaining acceptance |
| --- | --- | --- |
| Dynamic/typed decoding | Allocation matrix; 36 timing and 36 RSS cases versus direct Rust/Node, including failures and 32-level nesting | Final contract audit; retain documented validation overhead |
| Encoding | Allocation matrix; 21 timing and 21 RSS cases; measured bounded-buffer optimization | Final contract audit; large Unicode overhead remains documented |
| JSONL streaming | Owned/borrowed reader/writer allocation, timing and RSS matrices; exact writer bytes, bounded storage and installed smokes | Final contract audit |
| Raw fragments | JsonRaw/borrowRaw validation and record storage preserve source lifetimes in native/editor checks; numeric/Unicode/deep/rejection allocation, timing and RSS evidence retained | Owner decision on embedded output spelling, serialization implementation and acceptance |
| Release consumers | Installed through 5682a032; raw native cases, selective fixture and corrected LSP lifetime hover pass | Refresh after raw serialization implementation |

All timing matrices retain three fresh processes, eight warmup batches and 31
measured batches. First-operation costs are separate from warmed batches; they do
not establish filesystem cold-cache latency. Peak RSS includes process/runtime
and observer overhead. Native requested allocations and Node post-GC retained heap
are separate metrics. Selective typed and full dynamic trees retain different data;
each is compared against a direct baseline with matching output retention.

The accepted encoding optimization removes three growth reallocations per fixed
JSONL record. Its original and updated evidence, pre-review budget, tiny-output
memory tradeoff and complete disposal checks are retained below. Larger decoding
and Unicode encoding costs remain visible; checks were not weakened to improve
ratios. No blanket efficiency or completed S10 claim is made.

## Public naming review

The owner approved a thorough globals/imports audit and the familiar JSON
namespace: `JSON.parse` throws typed errors; `JSON.tryParse` returns a checked
Result. Existing imported APIs retain their behavior. See
[the approved policy and mandatory audit acceptance](globals-and-imports-policy.md).
Complete the per-symbol audit before introducing the convention. Alias and typed
file-import work remains first; global JSON follows before S10 acceptance. No
naming decision remains open for the approved parsing pair.

## Immediate priority: typed imports and aliases

Owner approved JSON/JSONC file imports and project path aliases on 2026-10-11,
including compiler-derived data types in editor completion, hover and diagnostics.
Implement shared alias/extension/index resolution, then typed data modules and
editor/dev integration before further runtime API or benchmark expansion. See
[the implementation and acceptance plan](typed-data-imports-and-aliases-plan.md).
Existing JSON support and evidence remain accepted within their documented scope;
the additions and remaining S10 work are not yet accepted.

## Implementation sequence

1. Establish strict decoding and resource limits against the locked serde_json
   backend, with malformed UTF-8, escapes, numbers, depth and partial-failure cases.
2. Package native Rust dependencies automatically through the compiler-owned build
   path. A compiler dependency does not make that crate available to generated
   programs. Build/run/test/dev must resolve, cache and link support automatically;
   users must not pass `--extern` or locate Rust libraries manually. Verify installed
   execution outside the repository and reuse after an application-only edit.
3. Publish dynamic JsonValue ownership, inspection/construction and encoding, then
   direct typed record/array decoding and encoding. Finish each public batch with
   compiler rejection cases, native execution, editor documentation/examples and
   matched benchmark support. Do not expose an unchecked generic cast as decoding.
4. Add explicit opt-in borrowed/raw fragments and bounded incremental input/output.
   Preserve input lifetimes and handle chunk boundaries inside UTF-8 and escapes.
5. Complete the roadmap's tiny/large/numeric/Unicode/deep/selective/invalid/streaming
   measurement matrix, including direct Rust baselines, allocation/retention and
   wrapper overhead. Preserve baseline protocols before optimization. Finish full
   regressions, performance review and installed acceptance at the S10 milestone.

## Current private backend

`stdlib/json/runtime.rs` supplies both the emitted native runtime and backend tests. It uses a DeserializeSeed visitor so
input bytes are checked before parsing, values are counted before decoding, and
array/object entry budgets stop the next value/key before construction. Duplicate
object entries count against the budget even when replacing an earlier value.
Container nesting has a maximum of 64; Serde's independent recursion guard stays
enabled. Scalar values do not add container depth. The public parse defaults are 16 MiB of UTF-8 input, 64 container levels, one million
entries per collection and one million total values; a complete JsonLimits record
can override them.

The backend currently follows serde_json's owned Value representation: duplicate
keys replace previous values, object storage sorts keys, i64/u64 integer values
remain exact, floating negative zero survives, and nonfinite input numbers fail.
The public dynamic slice now covers checked extraction and scalar/container construction.
Typed decoding and public typed encoding have focused source acceptance. Null is a JSON
value; missing properties and Tsrust undefined must remain distinct at that boundary.
Errors now cross the public Result boundary as JsonError records containing code,
message and a byte offset. Their editor documentation and examples are checked.

Reference: [locked serde_json Deserializer](https://docs.rs/serde_json/1.0.151/serde_json/struct.Deserializer.html).
The decoder checks `end()` after one value and rejects trailing non-whitespace.
No relaxed syntax, unsafe lifetime erasure, hidden shared ownership, SIMD claim or
public API acceptance is introduced by this backend slice.

## Backend evidence

The missing-limit witness fails before implementation, then all four JSON tests
pass. The affected standard-library batch passes 68 tests with zero failures,
including the explicitly typed buffered-file empty-result assertion. Importing
serde_json Value exposed ambiguity in the former empty-array comparison; the
replacement `is_empty()` assertion preserves its behavior. Whole-project formatting
changes zero files and zero source fingerprints. SQLite retains the original red,
green, focused-batch and formatting logs under `tsrust-s10-json-*`. No S10 timing,
public binding, editor catalog or installed JSON acceptance is claimed.

## Native dependency bundle foundation

The compiler build now prepares an embedded file table from the independently
locked `crates/tsrust-compiler/native-dependencies` package. Cargo vendors exact
source versions, checksums and license files into the shared build output; no
registry paths are required when the file table is materialized elsewhere.
Build dependencies resolve the complete bundled source closure before the build
script runs, including Serde's optional derive dependencies retained in the lockfile.
Ordinary Cargo dependency preparation therefore supplies everything needed by the
vendoring step. Vendoring itself runs with `--offline --locked`; it never silently
fetches during the build script. Both lockfiles must stay aligned when upgrading
these compiler-owned dependencies.

The focused integration fixture materializes sources into a fresh private
directory, runs the pinned Cargo backend outside the repository with an empty
Cargo home, and executes a JSON assertion. A second check changes a dependency
package checksum and requires rejection against the lockfile. Cargo treats registry
sources as immutable after compilation; source edits alone do not invalidate its
existing artifacts. Future cache integration must preserve private immutable bundles
and include bundle identity in cache keys. Materialization refuses to replace
an existing destination. The shared CLI build preparation now selects this bundle for the exact compiler-owned
JSON runtime module, links its libraries automatically, and caches the complete
library set. Public JSON emission and installed consumer acceptance remain required
before publishing JSON. Existing runtime-only builds retain their current rustc path.

SQLite retains the focused red/green, cache-assumption failure, corrected lockfile
integrity check, final build and formatting logs under `tsrust-s10-bundle-*`.
The final focused test passes; whole-project formatting changes zero files and
zero fingerprints. No installed JSON capability or S10 measurement is claimed.


## Automatic runtime build and artifact reuse

The shared build preparation used by build/run/test/dev recognizes the exact bundled
JSON module. It materializes locked sources into the disposable native build
directory and invokes pinned Cargo with offline, locked dependencies. Cargo uses a
private empty home, incremental compilation stays disabled, and inherited compiler
wrappers, Rust flags, target settings and profile overrides are removed. Existing
dev process-group supervision owns this command, including its compiler children.

The application links the resulting runtime and dependency search directory
automatically. The existing bounded, atomic native cache stores the complete rlib
set as one artifact, with the runtime source, dependency manifest and lockfile,
backend identity, host platform and optimization profile in its identity. The
application cache also includes dependency identity. Cache restoration validates
member names, lengths and the required runtime library before creating output;
it cannot write parent or absolute paths. The runtime cache retains its existing
32-entry/256-MiB limit. Temporary Cargo output is deleted with the native build
directory, not retained as a new compiler-development target.

Focused native fixtures build JSON with an empty Cargo home, link and execute it,
delete the entire original build directory, edit application code, then link and
execute again using the restored runtime with no Cargo compilation command. Dev's
nonblocking cache path is covered by the second build. Backend and profile changes
miss the cache. Invalid artifact member paths and truncated lengths reject before
writing. Existing native cache/runtime checks pass (11 tests, one existing ignore).
The original failing fixture, focused green run, affected batch, final checks and
formatting evidence are retained in SQLite under `tsrust-s10-link-*`.

This is private backend integration acceptance. Public bindings, editor examples,
JSON-aware emit distribution, installed JSON execution, matched benchmark support
and S10 family measurements remain open.


## Private structured errors

Decoding failures now carry `code`, `message` and `offset` without retaining input.
Messages are fixed text; Display and Debug expose no document values or keys.
Codes distinguish invalid JSON, invalid UTF-8, unexpected end, trailing data,
invalid limit settings, and byte/depth/entry/node limits. Budget failures set an
explicit failure code before returning through Serde; no backend-message matching
or input-dependent strings are used to classify them.

Offsets count UTF-8 bytes from zero. Unexpected end points one past the last byte;
invalid UTF-8 points to the first invalid sequence. A byte-limit failure points to
the first excluded byte, and invalid limit settings point to zero. Other failures
convert the backend's one-based line/byte-column into a bounded input offset. For
resource limits, this is the parser checkpoint where the budget stopped decoding,
which may be a delimiter before the next value. Input-size and limit-setting checks
precede UTF-8 validation; syntax decoding follows. UTF-8 validation currently scans
the bounded input before decoding; its cost belongs in the S10 measurements.

Focused tests cover empty and whitespace-only input, CRLF, non-ASCII bytes before
an error, incomplete strings, trailing content, invalid UTF-8, every resource-limit
code, and Display/Debug redaction. The generated native runtime also verifies an
error's code, byte offset and redacted message before and after artifact-cache reuse.
SQLite retains red, green, boundary, native and formatting evidence under
`tsrust-s10-errors-*`. This is private backend acceptance, not public API acceptance.


## First public dynamic slice: implementation, acceptance still open

The source compiler now exposes parse, stringify and kind, plus JsonValue,
JsonError and JsonLimits. Parse accepts an optional read-only limits record;
stringify accepts an optional output-byte cap (16 MiB by default). Both return
Result rather than throwing validation failures. Kind returns one of six category
strings. Values own their trees, explicit clone copies them deeply, and opaque
construction, ordinary field inspection and implicit extraction are unavailable.

Focused native checks cover a returned value outliving its input, a saved parse
callable, cloning, duplicate-key replacement, sorted encoding, Unicode, errors and
limits in debug and release. Invalid arity/types and fabricated JsonValue records
reject. All six catalog examples execute through the CLI. Editor checks cover
aliased hover, optional parameter help and every module export in completion.
The native error-effect registry now explicitly classifies these Result-returning
operations as nonthrowing; the first public witness exposed the missing entry.
Encoding tests additionally check exact UTF-8/escape byte budgets and numeric
boundaries. Whole-project formatting and logs use `tsrust-s10-public-*` in SQLite.

Before accepting this slice, finish matched TypeScript sources, independent
oracles, ABI/driver support, source guards/hashes and quick untimed comparisons.
The existing benchmark driver compiles emitted Rust directly and therefore needs
the same automatic dependency packaging as CLI builds. Implement a self-contained
package export for emitted JSON programs rather than asking users or benchmark
callers to locate Rust libraries. Then verify installed consumers outside the
repository. Dynamic child inspection/construction, typed decoding/encoding,
borrowed/raw fragments, streaming and family measurement acceptance remain open.

## JSONC, JSON Lines and convenient file streams

Owner requested JSONC/JSONL as part of S10, including convenient file streaming.
These are required before S10 acceptance, not an unspecified later extension.
Ordinary parse remains strict; format selection is explicit.

- JSONC: support line and block comments outside strings. Preserve original byte
  offsets and reject unterminated comments. Trailing commas require an explicit
  option, disabled by default. Decoding produces the same owned JsonValue/typed
  outputs as strict JSON; encoding produces strict JSON. References:
  [JSONC draft specification](https://jsonc.org/) and its default trailing-comma rule.
- JSON Lines (`.jsonl` / newline-delimited JSON): decode one strict JSON value per
  line. Accept LF/CRLF and a final record without a newline. Blank lines are errors
  by default; null is a valid record. Reject a BOM. Stream errors identify the
  record/line and absolute input byte offset without disclosing input. Encoding
  writes one compact value plus LF per record. Reference:
  [JSON Lines format](https://jsonlines.org/).
- File integration: build on the accepted File/BufferedReader/BufferedWriter
  resource layer. Provide a synchronous pull interface for owned decoded records,
  plus convenient path-based reading/writing. Distinguish EOF from a JSON null
  value. Do not require reading an entire file or collecting every record.
  Typed records must use the direct decoder when that decoder lands.
- Ownership: a path-based helper owns and deterministically closes its handle on
  EOF, error or early drop. An adapter over a caller-provided reader borrows it and
  leaves explicit close to the caller. Writers expose flush/close errors; Drop
  cleanup cannot report them. Async adapters follow the existing async roadmap.
- Bounds: enforce maximum record bytes, buffered bytes, depth, collection entries
  and value count. Pull consumption must not read arbitrarily ahead. Define record
  failure as terminal by default; do not silently skip malformed records.

Acceptance includes one-byte chunk boundaries through UTF-8, escapes, comments and
CRLF; oversized unterminated lines; empty files; blank records; missing final LF;
reader errors; early stop; explicit close and drop; output failures and bounded
retention. Add compiler ownership/rejection tests, editor documentation, executable
file examples, matched Node references and streaming allocation evidence as these
APIs land. Final API names and records must follow the existing std:json/std:fs
style; the above is the required behavior, not a claim of implemented exports.


## JSONC implementation slice

`parseJsonc(text, allowTrailingComma?, limits?)` is now implemented in source.
The flag defaults to false and the optional limits record shares strict parse's
budgets and defaults. Comments count toward the original input byte limit.
Unterminated block comments produce unterminated-comment at the opening slash;
other decoder offsets continue to address the original document.

The backend validates original UTF-8, then uses one bounded input-sized scratch
buffer. It masks comments with same-length whitespace while preserving CR/LF,
leaves string contents untouched, and optionally masks a trailing comma before a
closing array/object delimiter. It rejects empty collections containing commas,
repeated commas and malformed syntax. Ordinary parse still receives the original
bytes and remains strict. Encoding a parsed JSONC value produces strict JSON.
This is an initial correctness implementation; scratch allocation and repeated
validation cost must be included in the variant measurements before optimization.

Focused tests cover comment delimiters inside strings, line comments ending at
CR, block comments, multibyte comment text before an error, original offsets,
unterminated comments, invalid UTF-8 inside comments, limits and invalid comma
placements. Native checks cover direct and saved callable use, strict rejection,
default/explicit comma behavior, explicit limits and debug/release execution.
The catalog includes a native executable example, aliased hover, three-parameter
signature help and import completion. Exact logs use `tsrust-s10-jsonc-*` in SQLite.

JSONC implementation is not complete family acceptance: matched JSONC reference
and benchmark registration remain open alongside package export and installed
checks. JSONL, bounded file adapters, typed paths and the other S10 requirements
remain active; none is replaced by this whole-document JSONC decoder.

## Private JSONL pull-reader foundation

`stdlib/json/lines.rs` now provides a test-only pull reader over Rust BufRead.
Each call returns one owned value, EOF, or a terminal structured failure. JSON
null remains a value distinct from EOF. The reader accepts LF/CRLF and a final
record without LF; blank records and BOMs reject through strict JSON decoding.
Errors carry a one-based record line and an absolute zero-based input byte offset.
I/O failures retain kind/OS code but never retain the backend's message or input.
Interrupted reads retry. Calls after failure return the same error without more I/O.

Only the current record is accumulated. Its capacity grows geometrically but
never beyond the record byte limit. Scanning inspects at most one byte beyond the
remaining budget, even when the underlying buffer is larger. The LF delimiter is
excluded from the record byte budget; other JSON whitespace, including CR, counts.
Read-ahead storage belongs to the supplied buffered reader, whose capacity the file
adapter must bound. EOF/error releases owned readers and scratch storage; early
drop also releases the reader. A borrowed adapter consumes only the current record,
so unread buffered bytes remain available to the caller after early drop. A failing
witness caught the original extra-buffer design discarding those bytes; the BufRead
implementation repairs it rather than weakening the expected behavior.

The `next_with` decoder boundary accepts direct typed output without creating a
JsonValue first. Its decoder is responsible for syntax and non-byte limits; the
stream core enforces record bytes and positions. A scalar test proves this boundary,
not the still-pending public typed decoder. Position arithmetic is checked.

Focused tests cover every buffer size across UTF-8/escape/CRLF boundaries,
missing final LF, empty files, null versus EOF, malformed/blank/BOM records,
absolute offsets, terminal failure, interrupted reads, redacted I/O errors,
oversized unterminated input, bounded reads/capacity, early stop and exactly-once
owned cleanup. SQLite evidence uses `tsrust-s10-jsonl-*`, including both original
failing witnesses. Public native file adapters, explicit close/flush behavior,
writer support, editor/catalog examples and matched streaming benchmarks remain
required before JSONL acceptance. This core does not publish a Tsrust export yet.

The existing bounded file reader now implements Rust `Read`/`BufRead` through
its unique resource owner. JSONL can consume that owner or borrow it without
allocating another read-ahead buffer. Ordinary `buffered_read` shares the same
refill/consume path and preserves its allocation-before-I/O, zero-length and
short-read behavior. File integration witnesses alternate JSONL and byte reads,
then resume JSONL without losing buffered bytes. Descriptor checks cover owned
EOF, malformed input and early drop. Existing file lifecycle/partial-I/O tests
also execute at native optimization levels 0 and 3. Evidence is stored under
`tsrust-s10-jsonl-file-*`. This is private backend integration; public JSONL
bindings, explicit close and writer APIs remain pending.

## Private JSONL writer foundation

The JSONL writer stages one compact record through the bounded encoder, then
writes its bytes and LF to the existing buffered file resource. The record byte
limit excludes LF, matching the reader. Encoding rejection writes none of that
record; a failed output operation can have written a prefix and provides no
rollback guarantee. Any record or flush failure is terminal for subsequent
write/flush calls. The writer never silently skips or retries a failed record.
Interrupted/partial writes use the existing buffered-file behavior.

Owned output is flushed and consumed by explicit close or fallback Drop. A
borrowed adapter flushes when closed/dropped but leaves the caller's file owner
usable. Explicit close attempts cleanup even after a prior failure; a cleanup
failure takes precedence, otherwise the earlier terminal failure is returned.
Drop cannot report errors. Error records contain a stable code, operation,
record counter and optional OS code, with a fixed message and no input/output
contents. The counter denotes the current record during write and the next
record during flush/close; it does not falsely identify which buffered record
caused a flush failure.

This remains a private test-only backend. Public native bindings, compiler
ownership checks, editor examples and matched streaming benchmarks are still
required. Writer evidence is retained under `tsrust-s10-jsonl-writer-*`.

## Public owned JSONL reader

`std:json` now declares `JsonlReader`, `JsonlError`, `openJsonl(path, limits?,
capacity?)`, `readJsonl(reader)` and consuming `closeJsonl(reader)`. The reader
owns an existing bounded file buffer. Open validates limits and buffer capacity
before opening; defaults match JSON parsing and the buffered file API. Native
optional parameters currently permit trailing omission, so callers supply limits
when selecting capacity instead of passing explicit undefined.

Read returns `Result<JsonValue | undefined, JsonlError>`: undefined is EOF,
whereas JSON null is an owned JsonValue. Records survive subsequent reads and
reader disposal. Failures are terminal and redact input/path/backend messages.
Error fields document operation, record line, absolute input offset and optional
OS code. Open uses line 1/offset 0; close uses 0/0 because it has no input location.
Close consumes the wrapper even after EOF/error already released its file, and
reports an explicit early-close failure without retrying a released descriptor.
The type cannot be forged, cloned or used after close. Stream positions stop at
the public integer limit with position-limit rather than wrapping or clamping.

Focused compiler/native checks, editor help and executable catalog examples use
`tsrust-s10-jsonl-public-*` evidence in SQLite. The initial fixture corrections are
retained alongside the repaired evidence. This is source implementation progress,
not S10 acceptance or timing evidence. Borrowed-reader public adapters, public
writer APIs, direct typed streaming and matched benchmark registration remain
pending, along with the other S10 requirements above. The current reader APIs
now pass installed CLI/editor smoke checks from `d3aedae3`.


## Standalone native source export

`tsrust emit input.tsr --package -o new-directory` exports a relocatable Cargo
package. It includes generated Rust, the exact lockfile, Rust 1.96.1 selection,
checked-overflow profiles and, for JSON programs, the embedded source dependency
closure with checksums/licenses and relative offline Cargo configuration. Plain
programs do not receive unused vendored dependencies. Normal emit remains useful
for inspecting Rust; package export is the distribution path for JSON programs.

Export performs no build or network request. Existing directories/files/symlinks
are refused before mutation; failures after a successful directory creation clean
up only the newly created tree. A recipient runs Cargo from the package directory
with the pinned toolchain already installed. No repository, original Tsrust source,
registry cache or manual extern/library paths are required.

Focused package evidence (`tsrust-s10-package-*` in SQLite) relocates the export,
removes the original source and compiles/runs both debug and release using an empty
Cargo home and offline/locked Cargo. It also covers optional main, dependency-free
packages, protected existing output and symlink destinations, invalid CLI options,
and the existing bundled-dependency checks. Benchmark harness integration remains
pending. Installed CLI and LSP consumer smoke evidence is under
`tsrust-s10-package-consumers-*`. A first pipelined hover request returned no result
before document checking completed; the protocol-aware follow-up waits for checked
diagnostics and then verifies the hover. CLI/package checks already passed in the
first run and were preserved. Both repository and test-project editor servers were
restarted; the test project reports No Problems and the open repository fixture
reports zero errors (five pre-existing workspace warnings remain).

## Benchmark dependency preparation

The runtime harness now prepares JSON dependencies from the compiler's offline
source-package export. Cargo artifact messages identify the exact serde and
serde_json libraries; the harness supplies linkage automatically. Preparation
uses the shared target, an isolated Cargo home, locked vendored sources and the
pinned Rust toolchain. Exported source must match the benchmark emission.
Temporary package sources and Cargo homes are removed after preparation.

Dependency preparation is untimed setup. The existing optimized rustc command
still compiles each instrumented application, including every requested compile
sample; it is not replaced with a cached Cargo application build. Records include
library hashes, manifest/lock/config hashes and the preparation command. Existing
workloads without JSON retain their original compilation path and sample counts.

Four focused checks pass, including offline dependency preparation followed by
separate instrumented native compilation/execution and source-drift rejection.
Evidence: `tsrust-s10-benchmark-dependencies-{red,green,final,format}.log` in SQLite.
Matched JSON workloads, allocation integration and family timing remain open.

## First matched JSON workload

`native_json_roundtrip` parses and compactly encodes a fixed ASCII object once per
iteration. Both implementations compare the entire encoded output to an explicit
expected literal before accumulating its length. The independent oracle uses the
literal's 31-byte length; it does not call either JSON implementation. Source
contracts guard operation placement inside the loop, exact fingerprints, input,
iteration count and denominator. Sorted keys and small integer values make this a
scoped common semantic case, not a general encoding-equivalence claim.

Native and Node-after-tsc pass all five correctness cases, including 2,000
iterations. The original correctness-only record is
`runtime/results/2026-10-11T21-42-54-126Z-verify.json` in SQLite. It contains zero
execution/compilation measurement samples and records automatic offline dependency
identities. The workload remains explicitly unmeasured. Focused registration,
adapter and report checks are retained under `tsrust-s10-json-benchmark-*`.
Allocation profiling, direct Rust comparison and the remaining JSON workload
matrix still need implementation before family measurement acceptance.

## Private container selection foundation

The backend now distinguishes absent children from present JSON null, reports
wrong container kinds explicitly, and rejects negative array indices. Positive
out-of-bounds indices return absence, including indices beyond the host address
space. Object keys use exact UTF-8 strings with no coercion. Selection errors use
zero offsets and fixed messages without source contents.

Read access returns a reference tied to the container, independently of the key
argument's lifetime. Consuming access moves only the selected child and drops the
remaining container, including on absence or error. Array selection does not shift
siblings that will immediately be dropped. Tests verify that selected string
allocations retain their original addresses in both borrowed and consuming paths.
No clone, shared owner, unsafe code or lifetime erasure is introduced.

This is private backend acceptance only. The native return contract now also recognizes an optional borrowed payload
inside Result. Its metadata validation, checked type projection and conditional
loan paths have focused acceptance; a public JSON view still requires native
representation, emission and end-to-end ownership acceptance. Do not substitute an owned clone or collapse absence into JSON null to
avoid this integration. Public binding, editor, lifetime rejection and matched
workload acceptance remain required before exposing child APIs.

SQLite evidence: tsrust-json-containers-red.log, the initial compile repair in
tsrust-json-containers-green.log,
tsrust-json-containers-green-2.log, tsrust-json-containers-final.log and
tsrust-json-containers-format.log.
The green batch runs ten container/backend and ten scalar/backend checks; the
seven shared parser tests run in both harnesses. The final affected harness passes
ten checks. No benchmark timing or installed-consumer acceptance is claimed.

## Optional borrowed Result integration

`ReturnSlot::ResultOkOptional` describes exactly `Result<View | undefined, Error>`
with a declared native borrowed View. Validation rejects a missing Optional wrapper,
an optional return outside Result, mismatched payload identities and borrowed error
payloads. Existing source-parameter and exclusive-access constraints remain intact.

The analyzer records both the Ok variant identity and the present view identity.
Loan projections attach separate guards to the outer Result and its payload.
Narrowing to Err or undefined removes that loan; narrowing to the present view
preserves its original owner provenance. No owner or payload is cloned.

Focused evidence in SQLite: tsrust-json-optional-loan-red.log establishes the
missing contract; tsrust-json-optional-loan-shape.log covers all three new contract,
checked nullable-shape and conditional-projection witnesses. Existing contract
and projection suites pass in tsrust-json-optional-contracts.log and
tsrust-json-optional-projections.log. Five executable public byte-view fixtures
remain covered by tsrust-json-optional-byte-regression.log. Whole-project format
reports no changes in tsrust-json-optional-format.log.

This accepts compiler infrastructure only. No public optional JSON view factory
is exposed yet, and no end-to-end JSON view lifetime or timing acceptance is claimed.
The next integration must publish the actual borrowed native representation and
verify caller escape, owner mutation/move, missing-child branches, nested reborrows,
editor documentation and matched workloads together.

## Owner priority: familiar JSON inspection syntax

The owner prefers the existing typeof keyword over importing kind. Prioritize
runtime inspection through typeof, Array.isArray and equality with null, then safe
flow refinement. Keep the imported kind operation compatible. typeof must inspect
JsonValue contents (also through borrowed/optional values), evaluate its operand
once and borrow rather than consume storage. Preserve familiar labels: numbers,
strings and booleans use their primitive labels; arrays, objects and null report
object. Use Array.isArray and null comparison to distinguish those cases.

Guard analysis must retain the JsonValue alternative wherever its runtime content
can satisfy a guard, including the false branch. Do not narrow the opaque wrapper
to a Rust scalar without implementing the corresponding checked representation
and native access. Numeric narrowing must not silently round exact 64-bit JSON
integers into floating-point numbers. This work is part of S10's public inspection
acceptance, including editor documentation and executable examples; the keyword
alone does not accept the full guard/refinement family.

### Runtime typeof source acceptance

The keyword now reads JsonValue contents directly, including optional/borrowed
unions, and preserves single evaluation and nonconsuming access. Union guards keep
JsonValue on each feasible branch; they do not falsely replace the wrapper with a
scalar storage type. The public kind import remains compatible. Editor hover and
an executable catalog example explain the current refinement boundary.

SQLite records tsrust-json-typeof-guards.log (JSON witness plus 16 existing union
regressions), tsrust-json-typeof-editor.log, tsrust-json-typeof-example.log and
tsrust-json-typeof-benchmark-contract.log retain focused acceptance. The original
fixture corrections and failing runtime assertion remain retained as red evidence.
The matched native_json_typeof workload includes exact hashes, operation guards,
ABI/report registration and an independent six-character checksum oracle.
tsrust-json-typeof-smoke.log proves agreement in five native/Node-after-tsc cases,
with zero timing samples. tsrust-json-typeof-format.log records whole-project
formatting. Installed consumers now include this slice; see the refresh below.

Array.isArray over JsonValue, equality with JSON null and safe scalar/collection
refinement remain required follow-up work. This checkpoint does not accept that
entire inspection family or S10.

### Array and null inspection source acceptance

Array.isArray now inspects contained JSON data, including optional unions. Strict
comparison with null recognizes contained JSON null in either operand position;
undefined remains distinct. Inputs are borrowed, expressions evaluate once, and
union guard analysis retains the JsonValue alternative on all feasible branches.
Existing array/tuple behavior and kind imports remain compatible. These observations
do not yet grant unchecked scalar or child access; richer refinement remains open.

SQLite evidence: tsrust-json-inspection-red.log, tsrust-json-inspection-green.log
(native JSON fixture plus four existing nullable-union regressions),
tsrust-json-inspection-example.log, tsrust-json-inspection-editor.log,
tsrust-json-inspection-benchmark-contract.log and tsrust-json-inspection-smoke.log.
The matched workload has source hashes, operation guards, an independent checksum,
driver support and report registration. Five native/Node-after-tsc cases agree;
there are zero timing samples. Whole-project formatting is retained in
tsrust-json-inspection-format.log. Installed refresh is tracked separately below.

### Installed inspection refresh

Release CLI and LSP binaries from bdedda8a were atomically installed in ~/.cargo/bin
and target/editor/debug. Installed CLI execution outside the repository verifies
typeof, Array.isArray, equality with null and asInt. The installed LSP reports no
diagnostics for that fixture and exposes updated typeof documentation. The active
VS Code server was restarted; the stale asBoolean export error disappeared from
native_json_scalars.tsr. The workspace still shows 11 errors from negative closure
fixtures and 16 warnings; this refresh does not claim a clean repository Problems
panel. Existing open JSON documents show no Tsrust errors.

SQLite retains tsrust-json-inspection-consumer-build.log, consumer-install.json,
consumer-smoke.py and consumer-smoke.log with the same prefix. Binary hashes are
recorded in the install record. Family measurement and richer refinement remain open.


### Borrowed child views source acceptance

`std:json.get` and `at` now return `Result<JsonView | undefined, JsonError>`.
Both owners and views support nested reads without copying child storage. Present
children retain the original owner lifetime; missing and error branches release
the child loan. JSON null remains a present child. Views support typeof,
Array.isArray, null comparisons, checked scalar readers and both serialization
error styles. They cannot be cloned or passed to consuming takeString.

See [focused acceptance](../milestones/json-views.md) and
[placement audit](globals-and-imports-json-views-audit.md). These source changes
are included in the installed refresh below. Further scalar refinement, borrowed string
projections and dynamic construction remain open. The matched child workload is
ready and passed untimed parity checks; no timing acceptance is claimed.


### Boolean refinement source acceptance

A stable typeof boolean guard now permits native boolean payload access from
JsonValue and JsonView, including optional carriers. Access borrows the wrapper
and copies a boolean. The proof uses ordinary place invalidation, branch joins
and loop back-edge validation. Explicit owning wrapper transport remains possible
when its declared type is expected. Strings and numbers still require checked
extraction; borrowed string projection and exact numeric refinement remain open.

[Focused acceptance](../milestones/json-boolean-refinement.md) records native,
editor, regression and untimed matched-workload checks. Installed consumers still
predate child views and refinement. This does not close S10 or its measurements.


### Borrowed text reader preparation

The native length and five search helpers now accept ordinary borrowed UTF-8
slices instead of requiring the owned/static string carrier. Existing generated
string calls retain their behavior through dereference coercion. No public API
or projection is added by this preparatory change. A local borrowed slice cannot
be substituted into the static owned-string representation; public refinement
must still retain its actual source lifetime or perform a scoped read.

SQLite records tsrust-borrowed-string-readers-red.log and green.log retain the
initial native signature rejection and nine passing checks: a local-slice,
zero-allocation witness plus eight existing search/last-index regressions. These
cover Unicode byte boundaries, extreme positions, empty/missing needles,
evaluation order and ownership. tsrust-borrowed-string-readers-format.log records
whole-project formatting. No new workload semantics or timing are introduced.


### Guarded string read integration

The current source batch connects stable typeof string proofs to length and five
search operations on JsonValue/JsonView, including optional views and guarded
JSON search arguments. Native reads use scoped UTF-8 borrows; no string carrier is
created. Ordinary mutation invalidation and loop proof checks apply. This is not
first-class borrowed string projection: string locals/returns, remaining string
methods and numeric refinement remain pending.

Editor facts retain proof spans without relabeling native storage. Completion
uses only the six accepted reads; incomplete trailing members are checked through
a readonly project probe with original import identities and header masking.
The matched native_json_string_reads workload uses ASCII so byte and UTF-16
coordinates coincide. It includes parsing and does not isolate search cost.


Focused source acceptance evidence is retained in benchmarks/records.sqlite:

- tsrust-json-string-refinement-red.log and green.log: initial rejection, then
  native owner/optional-view length and five searches pass.
- tsrust-json-string-refinement-safety.log: two new string checks, two boolean
  refinement checks and eight existing search regressions pass. Missing/wrong
  guards, reassignment, length writes and invalidated loop proofs reject.
- tsrust-json-string-refinement-needle.log and final.log: guarded JSON needles,
  updated typeof catalog example and existing inspection execution pass.
- tsrust-json-string-refinement-editor-final.log: both complete-source hover,
  signatures/completion and incomplete-member recovery pass. Earlier failed
  probes remain retained; recovery now keeps string facts during readonly probes.
- tsrust-json-string-refinement-contract.log: eight workload contracts pass.
  tsrust-json-string-refinement-smoke.log: five native/Node-after-tsc cases agree,
  with zero timing samples.
- tsrust-json-string-refinement-format.log and format-final.log: whole-project
  token-preserving formatting and source fingerprint refresh.

Installed consumers now include this batch (818459f3). S10 acceptance and measurements
remain open; the scoped string reads do not substitute for first-class borrowed
string storage or exact numeric refinement.


### Installed guarded-read refresh

CLI and LSP consumers were rebuilt from 818459f3 and atomically replaced in both
~/.cargo/bin and target/editor/debug. Independent installed checks prove borrowed
child traversal, boolean refinement, guarded string length/search and missing/null
behavior outside the repository. The LSP publishes no diagnostics for that source
and supplies updated guard documentation. Active VS Code was restarted; stale
missing-get and guarded-search diagnostics cleared without editing source files.
After workspace rechecking, 11 existing negative-fixture errors and 18 Rust warnings
remain; this does not claim a clean repository-wide Problems panel.

The first command selected only the CLI target despite naming both packages. The
initial CLI smoke passed; its LSP failure is preserved. The server was then built
with its explicit binary target, replaced and its failed smoke rerun successfully.
The development workflow now documents the Cargo target-selection trap.

SQLite retains tsrust-json-guard-consumer-build.log, consumer-install.json,
consumer-smoke.py/log, consumer-lsp-build.log, consumer-lsp-install.json,
consumer-lsp-smoke.py/log, consumer-editor.json and consumer-workloads.log (all with the
`tsrust-json-guard-` prefix). Binary hashes and source identity are recorded.
This installed acceptance does not close S10 or add timing measurements.

### Borrowed JSON strings

`std:json.asString` now returns `Result<StringView, JsonError>` for named JSON
owners or child views. Successful values retain the source lifetime through
locals and function returns. Wrong kinds report `expected-string` with offset
zero; the error branch retains no source loan. `String(view)` explicitly makes
an independent owned copy. Moving or replacing a retained source, escaping a
local owner and cloning the view are rejected.

Views support UTF-8 byte length and the five existing search methods, including
borrowed search arguments. `typeof` reports `string`, including optional views,
and narrows the optional alternative. `Boolean(view)` tests whether text is
nonempty; console output displays the text. Other string methods still require
an explicit owned conversion. Automatic projection of guarded JSON into string
locals and exact numeric refinement remain pending.

The gate also corrects string-array searches accepting guarded JSON needles:
those alternatives belong only to string receivers. Console helper declarations
now retain generic payload types, and opaque StringView output bypasses ordinary
record-field inspection.

Focused native, ownership, catalog, editor and matched untimed checks are retained
under `tsrust-json-string-views-*` in SQLite. The new workload parses once outside
its loop and compares checked borrowed extraction and three reads with Node string
references. It has independent checksums, source guards, current fingerprints and
ABI support; no timing samples are claimed. The reviewed API inventory is 413.
Installed consumers remain at 818459f3 until a later explicit consumer refresh.

### Guarded string projection

A stable JSON string guard now permits `const text = value` and returning `value`
as a borrowed StringView. The compiler interns the view identity for global JSON
usage without requiring an import solely for inferred locals. Read-only JSON
parameters remain available to JSON APIs; no owned string is implicitly allocated.
Explicit `String(view)` is still required for independent string storage.

The projection carries the original owner's loan through optional JsonView
children, locals and returns. Local-owner escape, retained-owner replacement,
invalidated proofs, unproven joins and loop backedges are rejection witnesses.
Editor checked facts report the inferred StringView and its supported read methods.
The typeof catalog example exercises an inferred borrowed local.

Focused evidence uses `tsrust-json-string-projection-*` in SQLite: compiler/native
and ownership checks, editor facts, catalog execution, matched native/Node checks
and whole-project formatting. The new benchmark parses once, checks a string guard,
projects a local and performs length plus two searches. Five untimed cases agree;
no family measurement or installed-consumer acceptance is claimed here.

The owner accepted preserving exact JSON storage after a number guard. Arithmetic
requires checked asInt/asNumber extraction; asNumber rejects precision loss.
Implicit numeric projection and a new exact numeric type are not required for
this S10 inspection contract. Existing conversion behavior remains unchanged.


### Scalar JSON construction

Imported nullValue, fromBoolean, fromInt, fromNumber and fromString construct owned
JsonValue values without parsing. Integer construction preserves signed 64-bit
values exactly. Number construction preserves finite floats and negative zero;
NaN and either infinity return number-nonfinite with offset zero. String
construction consumes its argument, transferring an owned buffer and allocating
owned text for literal-backed strings. Explicit clone preserves a caller's source.

Focused compiler/native checks cover values, errors and ownership transfer.
Catalog examples execute through the CLI; editor hover and parameter help are
verified. The reviewed public inventory is 418. Native and Node-after-tsc agree
on five untimed construction/encoding workload cases with an independent checksum.
The workload includes the explicit native string copy needed to reuse its input;
it does not isolate constructor costs. Evidence is stored under
`tsrust-json-construction-*` in SQLite. No family timing or consumer refresh is
claimed. Container construction, typed conversion, streams and integrated S10
acceptance remain open.


### Owned container backend and numeric contract

The owner confirmed that typeof-number guards preserve JSON storage. A native
witness retains 9007199254740993 exactly, extracts it with asInt, rejects lossy
asNumber, and still serializes the original value. A direct number assignment
after the guard is rejected. This resolves the numeric-refinement decision;
historical statements about pending exact numeric projection are superseded.

Private owned-container helpers validate depth, per-container entries and total
values before constructing a new root. The root counts as one value and one
container level. Nesting is capped at 64. Object input budgets count duplicate
entries and all their values before last-key replacement. Objects retain the
existing sorted-key representation. Array storage and child string allocations
transfer without cloning. Existing caller storage is consumed on error.

Focused backend tests cover retained allocation identity, exact integer payloads,
duplicate replacement, key order, nested entry budgets, depth/node boundaries and
invalid limits. These helpers are infrastructure only: no public container API,
editor support, workload registration or installed acceptance is claimed yet.
Public array/object constructors and their complete acceptance remain the next
construction step. Evidence uses tsrust-json-containers-backend-* and
tsrust-json-number-contract-* in SQLite.


### Public owned container construction

fromArray consumes JsonValue[]; fromEntries consumes JsonEntry[] with owned key
and value fields. Both return Result<JsonValue, JsonError> and accept a borrowed
optional JsonBuildLimits record. Defaults are depth 64, one million entries per
collection and one million total values. Limits are nonnegative; depth cannot
exceed 64. The root counts, and duplicate entries count before replacement. Root
limits reject before allocating converted wrapper storage; descendants are then
validated by the owned backend. Children transfer without deep copies; container
storage and literal-backed keys may allocate.

Native witnesses cover nested array/object values, sorted output, last duplicate
wins, empty arrays, explicit limits and invalid limits. Both constructors consume
inputs on success and failure, and borrowed JsonView children are rejected. Four
catalog entries share two executed examples. Editor hover and signature help
expose ownership and limits. The reviewed inventory is 422.

Matched native/Node-after-tsc construction and encoding agree in five untimed
cases. The independent checksum counts 34 ASCII bytes for the fixed nested value;
source guards, fingerprints and ABI/report registration are complete. Evidence
uses tsrust-json-containers-* in SQLite. This accepts public dynamic construction;
installed consumers remain at 818459f3. Typed runtime conversion, remaining streams,
allocation baselines and S10 family measurements remain pending.

### Direct typed decoder backend

A private typed decoder boundary now validates the complete input under the JSON
byte/depth/entry/value budgets, then deserializes directly into its requested
native type. Validation retains no JsonValue tree. This initial implementation
makes two passes over the bounded input; its CPU and allocation costs still need
S10 measurement. Schema fusion remains S11 work, not a claim of this boundary.

The validation pass counts values in ignored fields and duplicate object entries,
checks complete input and UTF-8, and keeps the independent backend recursion guard.
Typed shape failures are redacted shape-mismatch errors with byte offsets.
Generated floating-point fields must use the ExactNumber adapter, which rejects
integer precision loss and preserves floating negative zero; ordinary Serde f64
coercion is not the approved Tsrust numeric contract. Generated int fields use
ExactInt with the existing asInt range/fraction checks, including integral floats
and normalization of negative zero.

Private witnesses cover typed nested arrays/maps, a field-by-field record visitor,
last duplicate replacement, missing fields, skipped-field limits, malformed input,
exact signed/unsigned integer boundaries and partial-result destruction on error.
The support module is included in the automatically packaged native JSON runtime.
Evidence uses tsrust-json-typed-backend-*-e55e0647 in SQLite. This is backend
infrastructure only: public typed syntax, compiler-generated visitors, editor
support, matched workloads and direct typed encoding remain pending.


### Checked typed JSON decoding

`JSON.parse<T>` and `JSON.tryParse<T>` now specialize checked compiler-owned
adapters. Ordinary calls without explicit type arguments remain dynamic. The
requested type drives direct visitors for owned scalars, records, tuples, arrays,
nullable/optional values and recursive records. No JsonValue tree is built before
conversion. Both paths borrow input and limits; results own their storage.

Required fields reject omission; optional fields default to undefined. Unknown
fields are validated under all input budgets and then skipped. Every known-field
occurrence must match its type; valid duplicate fields replace previous values.
Typed records use declaration field order. Tuples reject extra elements. Numeric
visitors preserve the checked ExactInt/ExactNumber contracts described above.
Multi-shape unions, native resources, views and functions are diagnosed as
unsupported targets. This batch does not claim schema-driven union selection.

Focused acceptance covers generated native output, recursive arrays and nullable
links, optional fields, tuples, nested arrays, generic wrappers, borrowed input,
throwing and Result failures, numeric boundaries, skipped-field limits, duplicate
fields, unsupported targets and executable catalog examples. Invalid target
errors reference the user type argument rather than synthetic adapter source.
Editor hover, signature help and member completion show concrete requested types;
completion recovery also handles a trailing member before closing braces.

The native_json_typed workload and checked Node-after-tsc counterpart have an
independent checksum, source fingerprints, ABI/operation guards and five untimed
matched cases. Node validates the fixed input shape after JSON.parse; the native
path validates budgets before direct decoding. Those costs are disclosed, not
claimed identical. SQLite records use tsrust-json-typed-*-1a08e3af, retaining
failed witnesses and repaired checks. No timing or allocation measurements are
claimed. Installed consumers remain at 818459f3 until the next refresh.

Next: direct typed encoding, typed JSONC/JSONL and bounded writer integration,
opt-in raw/borrowed fragments, then the complete S10 allocation/performance and
regression milestone. The broader S11–S21 roadmap is unchanged.

### Direct typed encoder backend

A private serialization boundary now writes borrowed typed fields directly to the
same bounded UTF-8 output sink used by dynamic stringify. It does not construct a
JsonValue tree. The sink reports output-limit without returning partial output;
unknown serializer errors retain only a redacted invalid-json error and byte offset.

Generated number visitors must use FiniteNumber: ordinary Serde floating-point
serialization substitutes null for nonfinite numbers, which is not the approved
checked contract. This adapter preserves finite values and negative zero and
returns the existing number-nonfinite code for NaN or infinity. Generated container
visitors must call encoding_container before visiting children; the root starts
at depth zero and the 65th container returns depth-limit. Scalars do not consume
a container level. The generic backend cannot enforce these compiler visitor
obligations by itself; public integration and its rejection tests remain required.

Focused private tests cover borrowed records and integer arrays, signed 64-bit
boundaries, Unicode and escaped byte counts, exact output caps, negative zero,
nonfinite values, redacted failures and the 64/65-container boundary. The existing
dynamic output-budget test and a generated native namespace/limits witness pass.
Evidence uses tsrust-json-encode-*-9493d256 in SQLite. This is backend acceptance
only, not public typed stringify, editor, workload or measurement acceptance.

Public integration is next. The owner approved omitting undefined object fields
and rejecting undefined array entries or root values. No implicit null replacement
is allowed. Generated object visitors skip absent fields; array/root visitors use
the private rejection adapter and report undefined-value. This decision is settled.

Imported-data hover repair: selecting the default import now resolves the binding
rather than a field sharing its import span. Binding hover expands the validated
record shape with readonly fields (bounded to eight levels); member hover and
JSON source navigation remain intact. The reported settings fixture is covered
by a failing-then-passing editor witness. All 32 import-focused project editor
tests pass. Whole-project formatting changes no sources or fingerprints.
Evidence: tsrust-json-hover-*-c2322741 in SQLite. Installed editor refresh remains
pending with the JSON family consumer refresh; no measurements were run.


### Direct typed encoding integration

JSON.stringify and JSON.tryStringify now specialize borrowed serializers for
scalars, named records, tuples, arrays, nullable unions and recursive records.
JsonValue, JsonView and StringView remain supported. No intermediate JSON tree
or input clone is introduced. Aggregate visitors use the ordinary native ABI;
fixed-storage specialization is disabled where that ABI is required. The owned
input remains reusable after encoding. Result wrappers must be matched to obtain
the payload before encoding; callbacks, Maps, Sets and resources are rejected.

Undefined object fields are omitted. Root and array undefined fail with
undefined-value; nonfinite numbers fail with number-nonfinite. Every container,
including dynamic subtrees embedded in typed data, shares the 64-level budget.
The existing bounded output sink enforces maxBytes. Keys are sorted and integer
bits remain exact. Synthetic adapter diagnostics point back to user calls.

Focused acceptance covers three compiler/native tests, five encoder runtime
tests, nine namespace tests including catalog examples, and three editor tests.
The native_json_encoding workload has a TypeScript counterpart, independent
47-byte ASCII oracle, source guards and five untimed native/Node cases. Fourteen
JSON workload guards pass. Evidence uses tsrust-json-encoding-*-fc30a5f9;
preceding failures and the dynamic-depth witness are retained in SQLite. No
measurements or installed-consumer refresh are claimed. JSONC/JSONL typed
adapters, bounded writer integration and the S10 family milestone remain open.

### Typed JSONC and JSONL backend adapters

The private typed JSONC boundary reuses the dynamic parser's bounded comment
normalizer, then constructs the requested owned type with direct visitors. It
allocates one input-sized normalization buffer; it does not construct a dynamic
JSON tree. Same-length whitespace preserves original byte offsets. Strict JSON
stays strict, and trailing commas remain opt-in. Original input bytes count
against the limit, including comments; malformed comments fail before decoding.

The existing JSONL next_with boundary now has a checked typed-decoder witness:
owned strings survive later reads, CRLF contributes to absolute byte offsets,
shape failures are terminal, and the underlying reader is released exactly once.
This is backend readiness, not public generic parseJsonc/readJsonl acceptance.
Public specialization, editor documentation, examples and matched workloads are
still required. JSONC retains normalization plus the existing two-pass typed
validation/decoding cost; no single-pass performance claim is made.

Acceptance: 37 JSON backend tests and the existing JSONC native fixture in debug
and release modes. Whole-project formatting changed no sources or fingerprints.
Original red/repair checks are retained as tsrust-json*-e00c40d0 records in SQLite.
No timing or consumer refresh was performed.

### Public typed JSONC decoding

The existing std:json parseJsonc import now accepts explicit type arguments,
including aliases. parseJsonc<T>(text, allowTrailingComma?, limits?) returns
Result<T, JsonError>. Calls without a type argument keep the dynamic contract.
No new global or import identity is introduced. Owned target validation and
visitors are shared with strict typed JSON; normalization preserves byte offsets.

Focused acceptance includes the complete seven-test typed-decoder fixture,
including both JSONC catalog examples, plus project hover/signature help through
an import alias. Native and Node-after-tsc agree in five untimed cases for
native_jsonc_typed; all 15 JSON workload guards pass. The Node counterpart strips
comments and checks parsed fields. Its shared benchmark domain is ASCII JSONC
without trailing commas; native budget rules remain a disclosed difference.
Evidence: tsrust-jsonc-public-*-4da590c1 in SQLite. Public typed JSONL, bounded
writers and S10 family measurements/consumer refresh remain pending.

### Public typed JSONL reads

readJsonl<T> extends the existing std:json import, including import aliases,
with Result<T | undefined, JsonlError>. The reader mutably borrows per call;
direct decoding produces owned fields independent of later reads and reader
lifetime. EOF is undefined and decoded null remains null. Shape errors retain
line/absolute-byte metadata and the existing terminal cleanup behavior.

Focused evidence covers the five JSONL behavior tests, seven strict/JSONC typed
decoder tests, executable dynamic/typed readJsonl catalog examples and typed
editor hover through an alias. Native and Node-after-tsc agree in five untimed
native_jsonl_typed cases; all 16 JSON workload guards pass. The registered
streaming fixture is fingerprinted and created outside measured work. Both
workloads include open, two checked record reads, EOF and close, using 8192-byte
read-ahead. The independent scalar oracle is 84 per iteration. Native budgets
and numeric policies differ outside the fixed ASCII shared input domain.

Evidence: tsrust-jsonl-typed-*-0374c3d0 in SQLite. Bounded writer integration,
remaining borrowed adapters, allocation/performance acceptance and consumer
refresh remain open. No timing samples were collected.

### Typed writer staging boundary

The private JSONL writer now accepts generated serde serializers through
write_typed, sharing the bounded direct encoder. It stages one complete record
before writing any bytes, borrows the input and appends exactly one LF after a
successful encoding. Failed encoding leaves that record unwritten, becomes
terminal and is repeated by subsequent writes. Close still attempts cleanup
and reports its existing error precedence. Generated serializers retain their
finite-number, undefined and shared-depth obligations from direct stringify.

Record counters now stop before exceeding the signed 64-bit public integer
range, including flush/close's next-record counter. Eight writer tests pass,
covering typed borrowing, encoding rejection after a staged prefix, exact byte
limits, partial I/O, flush/close failures, borrowed output and overflow. Evidence:
tsrust-json-writer-typed-*-f4177484 in SQLite. This is private backend acceptance;
public writer bindings, catalog/editor checks and matched workloads remain next.
No timing or installed-consumer refresh was performed.

### Public owned JSONL writers

JsonlWriter and createJsonl/writeJsonl/flushJsonl/closeJsonlWriter now expose the
bounded writer through std:json. Creation validates options before truncation.
Writes infer ordinary typed values and borrow them through direct serializers;
flush borrows the resource and close consumes it. Errors retain JsonlError's
stable fields, with offset zero explicitly documented for output operations.
The native module extractor recognizes exact writer/no-writer runtime variants,
so JSON-only programs do not acquire file support from writer declarations.

Focused evidence covers four writer native tests (including all five catalog
entries through their shared example), three direct-encoding regressions and
editor hover/ownership documentation. The five new identities are reconciled
in the 427-row global/import audit. The native_jsonl_writer workload has a
TypeScript counterpart, fingerprinted temporary fixture, independent completion
count and exact final-file verification outside timing. Native and Node-after-tsc
run five untimed cases. Seventeen JSON guards plus seven file guards pass.
Evidence: tsrust-json-writer-public-*-32748fc8 in SQLite. Borrowed file adapters,
remaining borrowed JSON representations and the full S10 allocation/performance
milestone remain open; no timing or installed-consumer refresh is claimed.

### Raw fragment validation foundation

A private source-backed Fragment now validates strict JSON with the existing
bounded visitor and retains only a real string borrow. It does not construct a
dynamic tree or retain a replacement source buffer. Validation can still use
Serde's temporary escape scratch. Pointer/length checks verify source retention;
rejection cases cover bytes, nesting, entries, nodes, trailing data, JSONC input
and malformed Unicode escapes. The module is included in emitted JSON support.

Public output semantics remain awaiting the owner's choice: preserve validated
raw spelling when embedded, or normalize it. This foundation does not implement
or silently choose either encoding policy. Public lifetime diagnostics, catalog
entries and benchmark support remain required before exposing raw fragments.
Evidence: tsrust-json-raw-*-1c093f36 in SQLite; two backend tests and one generated
native namespace regression. No measurements or consumer refresh are claimed.

### Installed JSON consumer refresh

CLI and LSP consumers were rebuilt together from ee1e2717 and atomically installed
in ~/.cargo/bin and target/editor/debug. All four installed files match the shared
target binaries by SHA-256. The active VS Code language server was restarted.
The already open native_json_import.tsr now visibly shows both readonly fields
in the settings binding hover. Its installed CLI check passes; an installed LSP
stdio check verifies clean document diagnostics and the complete binding shape.
An installed CLI native typed JSON encode/decode smoke verifies emitted output.

A first protocol harness incorrectly waited for an empty diagnostic publication
from a workspace scan, which publishes error buckets rather than every clean
file. It timed out; the corrected document-only protocol check passed. The
workspace editor check was independently verified through the visible hover.
After workspace analysis settled, the editor showed 14 errors and 41 warnings;
this refresh does not claim those remaining diagnostics are resolved.
Evidence: tsrust-json-consumers-*-ee1e2717 in SQLite. This is a focused consumer
refresh, not S10 family regression or measurement acceptance.


### JSON Lines receiver dispatch foundation

Native readJsonl and closeJsonl bridges now dispatch through a statically resolved
reader capability. The existing owned representation implements that capability;
its native layout and close behavior are unchanged. Borrowed implementations can
retain their actual source lifetime without trait objects, shared owners, or
lifetime conversion. The private borrowed witness verifies early close, unread
bytes, terminal errors, owner reuse and a fresh adapter after failure.

This prepares the native receiver boundary. Public borrowed reader declarations,
returned-borrow contracts, typed-call receiver selection, editor documentation and
matched workloads remain required before borrowed file adapters are accepted.
Evidence: tsrust-json-reader-dispatch-*-d3426aff in SQLite. No timing acceptance.


### Public borrowed JSON Lines readers

borrowJsonl takes an exclusive borrow of an existing BufferedReader and returns
Result<JsonlReaderView, JsonlError>. It validates per-record limits before reading,
uses the owner's existing buffer and starts line/byte positions at 1/0 relative
to adapter creation. It neither reopens nor seeks. Existing readJsonl and
readJsonl<T> accept owned and borrowed readers. Decoded values remain owned;
undefined remains EOF and null remains data. closeJsonl consumes the view without
closing its owner. Early close/drop preserves unread buffered bytes. Failures
remain terminal for the adapter; the owner can be reused after releasing it.

The native representation stores Lines<&mut BufferedReader> with a real lifetime.
Typed calls defer receiver selection until argument types are known, choosing
private owned/borrowed adapters without changing the public single type argument.
Import aliases work. Borrowed-parameter forwarding works; local-owner escape,
owner read/close conflicts, clone, fabrication and post-close use are rejected.

Eleven focused compiler/native tests and four JSONL editor checks pass, including
catalog execution, existing owned reader regressions, relative error positions,
invalid limits without consumption and terminal failure recovery via the owner.
Twenty-five JSON/file benchmark guards pass. native_jsonl_borrowed includes open,
buffering, borrowed adapter creation, two typed reads, EOF, adapter release and
owner close; five untimed native/Node cases agree with the independent checksum.
The public naming audit reconciles all 429 inventory identities exactly once.
Evidence: tsrust-json-borrowed-*-3cb97e60 in SQLite. No timing was performed.
Installed consumers remain at ee1e2717. Borrowed writers, public raw fragments,
allocation baselines and S10 family acceptance remain open.


### Public borrowed JSON Lines writers

borrowJsonlWriter exclusively borrows an existing BufferedWriter and returns
Result<JsonlWriterView, JsonlError>. Its optional maxBytes bounds each staged
record excluding LF, defaults to 16777216, and validates before writing or
flushing. The existing buffer and file position remain in use. writeJsonl accepts
inferred and explicit typed values through owned or borrowed adapters, including
import aliases. flushJsonl and closeJsonlWriter use static receiver dispatch.

The emitted representation holds Writer<&mut BufferedWriter> with a real source
lifetime. Closing or dropping the adapter flushes output but leaves the file
owner open. Explicit close exposes flush failures and otherwise preserves any
prior terminal error. Dropping cannot report failure. Encoding rejection emits
no prefix of that record; I/O failures retain the existing partial-write policy.
Owner conflicts, clone, fabrication, use after adapter close, and local-owner
escape are rejected; forwarding a borrowed parameter is accepted.

Eight focused native/compiler tests pass, including existing owned writers,
borrowed owner reuse, typed aliases, invalid limits, terminal errors, drop flush
visibility, and the catalog's exact output file. Backend writer and returned-borrow
contract checks pass. Two writer editor checks and 26 JSON/file guards pass.
native_jsonl_borrowed_writer compares file creation, buffered owner and adapter,
typed output, adapter flush/close and owner close with Node's matched fixed ASCII
output. Five untimed cases agree, with final file bytes checked outside timing.
The public audit covers all 431 discovered identities exactly once.
Evidence: tsrust-json-borrowed-writer-*-829dd771 in SQLite. Installed consumers
remain at ee1e2717. Public raw fragments, allocation baselines and S10 family
regression/performance acceptance remain open; no timing acceptance is claimed.

### Numeric-array allocation baselines

benchmarks/runtime/json-allocations.mjs now measures generated dynamic and typed
JSON decoding against direct serde_json Rust and Node-after-tsc counterparts. It
reuses the established allocation drivers and three fresh-process samples per
case. Sizes are 0, 1 and 10000 exact integers, with independently constructed
inputs and an arithmetic checksum. The existing input/output disposal accounting
is unchanged. Input allocation is outside operation counters; output is retained
until counters are sampled, then dropped. Every Rust sample verifies zero live
bytes after output disposal and after input disposal. Node reports post-GC heap
deltas, which are not comparable to Rust allocation counts or requested bytes.

The harness requires an explicit verified TSRUST binary and the shared absolute
CARGO_TARGET_DIR. It uses the pinned Rust/Node/TypeScript versions, prepares native
dependencies automatically, preserves source/emission/instrumentation/dependency
hashes and compile commands, and removes its temporary build directory. Run with
those environment variables and `node benchmarks/runtime/json-allocations.mjs`;
stdout is the immutable JSON record and stderr reports progress. No timing samples
are collected. The older general allocation runner is not used by this harness.

The first run at compiler checkpoint 7dbf303d has six cases and 54 process samples
(three engines, three samples each). Requested-byte allocation counts are stable
across all three Rust samples. Empty arrays allocate nothing. At 10000 values,
generated typed output and direct Vec<i64> each retain 131072 requested bytes;
generated dynamic output and direct serde_json::Value each retain 524288 bytes.
The single-element native typed output fits inline and allocates no heap storage;
the direct Vec baseline retains 32 bytes. Inline storage still occupies memory.
These observations concern these successful numeric arrays, not general JSON or
process RSS. They do not measure the CPU cost of native limit validation or typed
decoding's two passes. The direct Rust baseline has no native resource limits.

Four adapter/accounting tests pass, and both generated programs and both typed
TypeScript counterparts compile and execute under the measured protocol. Evidence:
tsrust-json-allocation-*-7dbf303d in SQLite, including the original result JSON.
This is partial S10 allocation evidence, not family acceptance. Unicode, nested
and selective objects, malformed/partial-failure inputs, encoding, streaming and
raw-fragment allocation cases remain required, along with full family regressions
and timing. Raw-fragment public output semantics still await the owner's choice.

### Unicode and escaped-string allocation baselines

The allocation harness now covers dynamic and typed string arrays. Each input
unit contains literal Unicode `é🦀` and escaped `line\n\u0061`; decoded output
contains 12 UTF-8 bytes per pair. Sizes 0, 1 and 1000 pairs use three fresh-process
samples for each of generated native, direct serde_json Rust and Node-after-tsc.
The checksum counts decoded UTF-8 bytes, including on Node, rather than UTF-16
code units. Source input retains its escape spelling. The new `--only` selection
allows completed cases to remain untouched; unknown and duplicate kinds fail.
Use `--only=dynamic_strings,typed_strings` to run this batch independently.

All six cases and 54 process samples pass. Both Rust engines release all tracked
output and input bytes, and allocation metrics are stable across three samples.
At 1000 pairs, dynamic native/direct results match: 2002 allocations, nine
reallocations and 77536 retained requested bytes. Typed results retain 61152 bytes
in both engines; native uses 2003 allocations/eight reallocations versus direct
Rust's 2002/nine. At one pair, native typed output retains 12 heap bytes versus
108 for direct Vec<String>, with array storage inline. Native typed decoding has
extra validation scratch allocation; this evidence does not imply lower CPU cost.
Empty arrays allocate nothing. Node post-GC heap deltas remain a separate metric.

Evidence: tsrust-json-unicode-*-e19e66bb in SQLite. The original numeric evidence
is preserved and was not rerun. These are allocation measurements, not timing
acceptance. Nested/selective, invalid, encoding, streaming and raw-fragment
allocation coverage and S10 family acceptance remain open.

### Nested and selective decoding allocation baselines

The same allocation protocol now covers nested integer pairs and records with
selected `items` plus an ignored payload containing escaped strings, arrays and
objects. Sizes are 0, 1 and 1000 pairs, with three fresh processes per engine per
case. `--only=dynamic_nested,typed_nested,dynamic_selective,typed_selective` runs
this batch without repeating earlier measurements. The independent oracle sums
2n consecutive integers starting at 7. Generated record instrumentation verifies
the concrete field ABI before accessing it. Direct selective Rust uses a Serde
map visitor with owned string keys and IgnoredAny for discarded values; Node
validates parsed items and returns only the selected field. Dynamic cases retain
the complete tree, so dynamic-versus-typed differences include distinct retained
output semantics rather than equivalent work.

All 12 cases and 108 process samples pass. Rust allocation counts are stable,
and every Rust sample releases all tracked output and input memory. At 1000
pairs, nested native typed output uses one allocation/seven reallocations and
retains 49152 requested bytes, versus direct Vec<Vec<i64>> at 1001/eight and
56576 bytes. Inner two-element native arrays fit inline. Selective typed output
retains the same amounts; native uses five allocations/seven reallocations and
direct Rust 1004/eight. Validation and ignored-field scratch allocations are
included. Dynamic native and direct Rust counts match for both shapes; full
selective-input trees retain 972180 bytes in each. These results do not establish
CPU performance or general memory superiority; Node heap deltas remain separate.

Evidence: tsrust-json-nested-*-f9ef1461 in SQLite. Prior numeric and Unicode records
remain unchanged. Failure paths, encoding, streaming and raw-fragment allocation
coverage, plus full S10 regression and timing acceptance, remain open.

### Failed-decoding allocation baselines

The allocation harness adds dynamic and typed malformed arrays plus typed arrays
with a late string-to-number mismatch. Inputs contain 0, 1 or 1000 valid Unicode
strings followed by `?` (invalid syntax) or `7` (valid JSON, incompatible with
string[]). Existing source fixtures are reused without altering successful-case
measurements. `--only=dynamic_invalid,typed_invalid,typed_mismatch` runs only these
cases. Native instrumentation observes the generated function's error result;
direct Rust observes Serde rejection, and Node catches parse/validation failure.
Each returns a scalar rejection witness and drops the error before counters are
sampled. Every Rust sample additionally requires zero retained requested bytes.
This measures error construction and disposal together, not retained error size.

All nine cases and 81 fresh-process samples pass; counts are stable across three
samples and all Rust input/output disposal checks pass. At 1000 preceding strings,
dynamic native/direct Rust both use 1003 allocations/eight reallocations and peak
at 38808 requested bytes. Malformed typed native input uses two allocations and
peaks at 80 bytes because syntax validation precedes output construction; direct
Vec<String> uses 1003/eight and peaks at 30616 bytes. The valid JSON type mismatch
exercises partial typed output cleanup: native uses 1008 allocations/12
reallocations versus direct Rust 1004/nine, with both peaking at 30700 bytes.
The native error path has additional allocation overhead to review at the family
performance milestone; no CPU timing or optimization acceptance is claimed.

Evidence: tsrust-json-failure-*-3b36da2c in SQLite, including the failing guard and
eight passing focused checks. Prior successful-case records are unchanged.
Encoding, streaming and raw-fragment allocation coverage, full family regressions,
and S10 timing acceptance remain open.

### Numeric-array encoding allocation baselines

The allocation harness adds `--only=encode_dynamic,encode_typed`, measuring
JSON.stringify over preconstructed dynamic JSON values or typed integer arrays.
Input preparation finishes before operation counters reset; native typed inputs
use the generated array's ordinary push behavior. Direct Rust serializes a Value
or Vec<i64>, and Node serializes equivalent prepared values. The existing driver
retains output until counters are sampled, then verifies exact canonical JSON
bytes and a byte-sum oracle before disposal. Input heap storage is reported
separately; it is not equated with source-text byte length for these cases.

Six cases (0, 1 and 1000 integers for each representation), totaling 54 fresh
process samples, pass exact output, deterministic Rust counts and input/output
disposal checks. Both native representations use one allocation and retain eight
requested bytes for empty/single-value arrays; direct serde_json::to_string
retains 128 bytes. At 1000 values all retain 4096 bytes, while native uses nine
reallocations versus direct Rust's five. The smaller initial native capacity
trades tiny-output memory for additional growth; CPU impact remains unmeasured.
Node post-GC heap deltas remain distinct from requested-byte allocation metrics.

Evidence: tsrust-json-encoding-*-bf30f66b in SQLite. Eight existing focused guards
pass; the new encoding guard passes after repairing its initial test syntax error.
The original failed log is retained. No previous allocation cases were rerun.
Broader encoding shapes, streaming and raw-fragment coverage, full S10 regressions
and timing acceptance remain open. Installed consumers are unchanged.

### JSON Lines reader allocation baselines

The allocation harness reuses the existing typed owned/borrowed JSONL workload
sources with one file pass per measurement. `--only=stream_owned,stream_borrowed`
creates files containing 0, 1 or 10000 fixed ASCII records outside counters. File
open, the 8192-byte buffer, per-record decoding and close are inside counters;
only the input path is retained outside. Direct Rust uses a typed Serde visitor,
owned string field names and a reusable line buffer. Node uses the existing
incremental UTF-8 decoder and buffered file reader. The independent checksum is
7 + 42 * record count; payload bytes and hashes are included separately from the
path. Direct Rust assumes the valid fixture and does not impose native JSON limits.

All six cases and 54 fresh-process samples pass. Ten focused guards pass. Both
Rust implementations retain zero heap bytes after processing and release the
input path afterward. Across all three samples, peak requested bytes are identical
at one and 10000 records: 8459 for both native reader forms and 8283 for direct
Rust. Empty files peak at 8192. At 10000 records native performs 40002 allocations
versus direct Rust's 50002; neither retains the complete file or decoded sequence.
This proves bounded peak heap for this fixed-record fixture, not for arbitrarily
large individual records or Node process RSS. Node post-GC heap remains a separate
metric. Owned and borrowed adapters show the same allocation results here.

Evidence: tsrust-json-stream-*-4a4bf441 in SQLite, including an independent
cross-size peak assertion. Prior measurements remain unchanged. Streaming writer,
broader encoding/raw-fragment coverage and S10 regression/timing acceptance remain
open; no timing or installed-consumer refresh is claimed.

### JSON Lines writer allocation baselines

`--only=stream_write_owned,stream_write_borrowed` measures one open writer over
0, 1 or 10000 typed records, including staging, buffered writes, flush and close.
New fixtures retain one input record and writer for the sequence rather than
reopening/truncating per record. Direct Rust stages each record with Serde before
writing to an 8192-byte BufWriter; Node stages JSON.stringify output and uses an
8192-byte file buffer. Every engine's complete file is compared against an
independent literal-byte oracle immediately after each process sample. Empty
files, record count, canonical key order, LF separators and final flushing are
covered. The returned independent scalar checksum is 7 + record count.

Six cases and 54 fresh-process samples pass, as do eleven focused allocation
checks. All Rust samples release output state and input paths completely. Peak
requested bytes remain unchanged between one and 10000 records: 8256 for both
native writer forms and 8320 for direct Rust. Empty writers peak at 8192. Both
Rust engines perform 10001 allocations at 10000 records, but native staging also
performs 30000 reallocations versus zero in direct Rust. This confirms the small
initial staging capacity tradeoff identified by encoding probes. Prioritize its
CPU impact in the S10 timing review before choosing a capacity/reuse optimization;
retain all pre-optimization records. Bounded peak memory here applies to fixed
small records, not arbitrary record sizes or process RSS.

Evidence: tsrust-json-writer-alloc-*-82e9e8cd in SQLite, including exact-file checks
and the cross-size peak assertion. Prior allocation records are unchanged. Broader
encoding/raw-fragment coverage, full S10 regressions and timing acceptance remain
open; installed consumers are unchanged.

### Deep dynamic decoding allocation baseline

`--only=dynamic_deep` measures 31 singleton array wrappers around a numeric leaf
array, for 32 container levels. Leaf sizes 0, 1 and 1000 share the independent
arithmetic checksum used by numeric decoding; all wrappers are checked after
sampling. The dynamic native program and direct Serde baseline retain the whole
tree. Node checks the same shape and leaf values after its heap measurement.

All three cases and 27 process samples pass, with twelve focused allocation
guards. Native/direct Rust allocation counts match: 31 allocations and 3968
retained requested bytes for an empty leaf; 32/4096 for one value; 32 allocations,
eight reallocations and 36736 bytes for 1000 values. Every Rust sample releases
all tracked output and input memory. This is depth 32, not a boundary-limit test
or a recursive typed-decoder measurement; existing correctness tests cover limits.

Evidence: tsrust-json-deep-*-65a7336a in SQLite. No older cases were rerun. The
current acceptance table distinguishes these 486 accumulated process samples from
still-missing family latency, throughput, RSS, raw-fragment and release evidence.

### Unicode and deep encoding allocation baselines

The encoding adapter now handles owned typed string arrays alongside dynamic
values. `--only=encode_dynamic_strings,encode_typed_strings,encode_dynamic_deep`
measures canonical literal Unicode plus escaped-newline strings, and 32-container
numeric trees. Typed source strings and arrays are constructed outside operation
counters. Exact canonical output is independently assembled as text and checked
byte-for-byte after sampling; the secondary checksum sums UTF-8 bytes rather than
UTF-16 code units. Sizes 0, 1 and 1000 count string pairs or numeric leaf values.

Nine cases and 81 fresh-process samples pass exact output and complete Rust
input/output disposal checks. Fourteen focused guards pass. At 1000 string pairs,
native dynamic and typed encoders each allocate once, reallocate twelve times,
and retain 32768 requested bytes. Direct Serde allocates once, reallocates eight
times and retains the same capacity. At 1000 deep leaf values, native/direct
retain 4096 bytes with nine/five reallocations. These reinforce the existing
initial-capacity tradeoff; they do not establish CPU efficiency. Escaped-source
normalization correctness remains covered by the decoder tests; these encoding
inputs are already canonical so exact output has an independent literal oracle.

Evidence: tsrust-json-encoding-shapes-*-132cbdf5 in SQLite. Prior records were not
rerun. The current allocation matrix totals 63 cases and 567 process samples.
Object encoding, raw-fragment publication/measurement, full family regressions,
latency/throughput/RSS review and final installed acceptance remain open.

### Mixed-field object encoding allocation baselines

`--only=encode_dynamic_object,encode_typed_object` covers records with an active
boolean, nested integer-pair arrays and a Unicode name. Dynamic trees and owned
typed records use equivalent field values and canonical sorted output. Temporary
preparation trees are dropped before counter reset; the native adapter checks the
complete concrete record layout before constructing it. Direct Rust uses an
explicit SerializeStruct implementation, with preparation outside measurement.
The independent output oracle specifies every byte, including key order and UTF-8.

All six cases (0, 1 and 1000 nested pairs) and 54 fresh-process samples pass exact
output, stable Rust counts and complete input/output disposal. Sixteen focused
checks pass across the preserved batch and repaired encoding test file; its
initial syntax failure remains recorded. Empty/single-pair records retain 64
requested bytes natively versus 128 for direct Rust; native uses three growth
operations versus zero. At 1000 pairs both retain 16384 bytes, with eleven/seven
reallocations. Dynamic and typed output allocation metrics match in this fixture.
These are encoding-only allocation results, not parse or CPU measurements.

Evidence: tsrust-json-object-alloc-*-59abe08d in SQLite. Prior records are unchanged.
The current matrix totals 69 cases and 621 process samples. Public raw fragments,
full regression, latency/throughput/RSS and wrapper-cost review, justified
optimization and final installed-consumer acceptance remain required for S10.

### Family regression progress and catalog repair

The complete workspace regression started at 6d40badf, with Cargo metadata listing
355 executable test/doctest targets. The run is still in progress; no full-pass
acceptance is claimed. VS Code build, executable discovery, grammar and isolated
integration checks pass, including cross-file diagnostics, hover/completion,
restart, CLI checking and native execution. Their closed logs are verified in
SQLite under tsrust-s10-extension-*-6d40badf.

The first compiler failure was an obsolete catalog-size assertion: std:json grew
from 12 entries to 39. The runner now requires nonempty entries, unique names and
nonempty examples, then executes every example instead of freezing catalog size.
Failure diagnostics identify the catalog entry. All 41 current examples pass in
the focused repaired test; the three other tests already passed in the original
full run and were not repeated. The original failure remains in the active full
log; the repaired result is tsrust-s10-json-catalog-repair-6d40badf in SQLite.
Remaining targets and the final completeness audit must finish before regression
acceptance. Raw fragments and performance/release acceptance remain open.

### Initial JSON execution baseline

At source checkpoint `4db77a2e`, all 20 JSON workloads (including typed data
imports) passed five independent correctness cases per engine and completed
3 fresh processes × 8 warmups × 31 timed batches for native and Node-after-tsc.
The audit verifies 120 process records and 3,720 positive, finite timed batches,
with matching checksums. Startup and compilation arrays are intentionally empty.

Canonical SQLite records:
- `results/2026-10-12T06-04-29-404Z.json`: immutable commands, inputs and samples.
- `tsrust-s10-runtime-audit-4db77a2e.json`: sample audit and batch p50/p95 values.
- `tsrust-s10-runtime-4db77a2e.log`: completed run log.

Native versus Node median batch milliseconds: typed decoding 0.4885/0.3500,
typed encoding 0.4478/0.2313, container construction 0.9875/0.4474, borrowed
string views 0.0738/0.0222, owned JSONL reading 22.6761/25.2522 and owned JSONL
writing 64.2100/67.3035. These are complete workload costs, not isolated API costs.
Small absolute values and uncontrolled background/thermal load limit tiny-gap
claims. JSONC reference processing and native checked/ownership behavior differ;
measurements establish these authored workloads, not unrestricted API equivalence.

Review priorities are encoding buffer growth, container construction, typed
decoding validation, and borrowed string access. Preserve this baseline before
optimization; direct Rust comparisons must isolate wrapper overhead under matching
retained outputs and checks. This execution batch does not close warm/cold,
peak RSS, direct Rust, full shape-matrix, public raw-fragment or release acceptance.

### Encoding latency and direct Rust baseline

The focused encoding harness reuses checked allocation adapters and their exact
output oracles. Seven shapes × three sizes (0/1/1000) × three engines × three
fresh processes establish 189 process samples, 5,859 batches and 585,900 timed
encodings. Each process records its first encoding separately, then eight warmup
batches and 31 measured batches of 100 encodings. Prepared owned input remains
outside timing; output replacement/disposal is inside, with the final string
retained until timer stop and exact-byte verification. First-operation latency is
not process startup or filesystem cold-cache latency. Node reclamation follows GC;
Rust drops replaced strings immediately. These costs are intentionally retained.

At size 1000, median microseconds per encoding (native/direct Rust/Node):
- Typed integers: 7.95 / 4.33 / 4.49.
- Dynamic integers: 10.05 / 6.16 / 4.50.
- Typed Unicode/escaped strings: 44.39 / 14.50 / 29.41.
- Dynamic Unicode/escaped strings: 47.73 / 20.38 / 28.97.
- Deep dynamic arrays: 10.49 / 6.82 / 5.38.
- Dynamic mixed-field objects: 26.81 / 17.10 / 26.27.
- Typed mixed-field objects: 26.12 / 11.03 / 26.15.

Direct Rust uses Serde on the same valid data and retains the same exact output;
it does not enforce native size/depth/number contracts. Ratios include that
contract cost and must not all be attributed to wrapper overhead. Review bounded
writer/escaping costs and buffer growth before optimization; no safety checks may
be removed to match the baseline. RSS, decoding latency, streaming latency against
direct Rust, public raw fragments and final consumers remain open.

SQLite evidence: `tsrust-json-encoding-timing-8ba777db.json` and matching run log;
focused red/check logs and whole-project formatting log retain implementation
verification. Raw evidence hashes the harness, both drivers, adapters, compiler
inputs, emitted programs, TypeScript counterparts and dependency artifacts.

### Bounded encoding buffer optimization

After preserving the original timing/allocation records, the bounded output writer
now reserves min(maxBytes,128) bytes and implements write_all directly. Byte-limit
checks, offsets, UTF-8 validation, nonfinite/undefined rejection and depth checks
remain intact. The pre-review budget required fewer growth allocations, explicit
bounded small-output storage, and investigation of any >10% native median slowdown.

All 21 timing cases were repeated with the same protocol and exact output checks
(189 process samples, 5,859 batches). No native case exceeded the slowdown budget.
Small Unicode/object medians fell about 40–42%; large cases changed little, so this
accepts a small-output improvement and does not resolve the large Unicode overhead.
Unchanged direct Rust and Node controls remain in both raw records.

The affected allocation rerun covers 27 cases and 243 fresh-process samples across
seven encoding shapes and owned/borrowed JSONL writers. Fixed writer records now
need zero growth reallocations, previously three per record; 10,000 records remove
30,000 reallocations. Large encodings shed four growth reallocations and retain the
same final requested capacity. Small output capacity increases to 128 bytes, capped
by maxBytes. Writer peak requested bytes increase from 8,256 to 8,320, independent
of whether one or 10,000 records are written. Output disposal remains complete.

Validation: 42 JSON runtime tests, three native encoding tests, exact-output timing
and allocation probes, and whole-project formatting pass. No LSP surface changed.
SQLite records use prefix `tsrust-json-encoding-opt-` and suffix `-b162df03`:
`budget` (pre-review thresholds), `timing` (raw JSON/log), `review` (paired ratios),
`alloc` (raw JSON/log), `tests`, `json-tests`, `native-tests`, `build` and `format`.
Original baselines remain immutable. The changed runtime still needs final consumer
refresh; broader S10 performance and public raw-fragment acceptance remain open.

### Encoding process peak RSS

`json-encoding-timing.mjs --rss` reuses the checked source/input adapters with
single-operation drivers, without timing warmups or allocation instrumentation.
Seven shapes at sizes 0/1/10000, native/direct Rust/Node, and three fresh processes
per case produce 189 verified samples. Each fresh Python monitor launches exactly
one child and reads macOS RUSAGE_CHILDREN.ru_maxrss in bytes. No monitor is reused,
so earlier child peaks cannot contaminate later samples. Unsupported hosts fail.
Child failures, unexpected stderr, missing RSS and checksum mismatches fail closed.

The metric includes process startup, prepared input, retained output, exact-output
validation and runtime overhead. Node's UTF-8 checksum observer also allocates;
Rust's byte checksum does not. Neither startup nor observer memory is subtracted.
This is whole-process peak RSS, not encoding-only or incremental heap storage.

At size 10000, median MiB (native/direct Rust/Node): typed integers
2.17/2.16/47.81; dynamic integers 2.38/2.39/47.77; typed Unicode strings
3.64/3.31/51.02; dynamic Unicode strings 3.44/3.45/51.05; deep dynamic arrays
2.39/2.41/48.16; dynamic objects 3.83/3.86/51.08; typed objects 4.23/4.11/51.03.
Small differences need repeated controlled investigation before optimization.
No timing samples were added by this RSS run or merged into prior timing records.

SQLite records: `tsrust-json-rss-0debe317.json`, matching log, and
`tsrust-json-rss-audit-0debe317.json`, plus red/check/format logs. The raw record
includes exact input/source/driver/helper hashes and per-process values. This closes
encoding RSS coverage for these authored shapes; decoding and streaming RSS,
remaining latency coverage, public raw fragments and installed refresh remain open.

### Decoding latency and peak RSS

The decoding harness reuses the ABI-checked allocation adapters and independent
shape/checksum oracles. Twelve kinds cover numeric and Unicode arrays, nested and
selective records, malformed syntax, typed mismatches and 32-level dynamic arrays.
Timing sizes 0/1/1000 use three fresh processes per engine, eight warmup batches,
31 measured batches of 100 operations, plus separate first-operation latency.
The completed matrix contains 324 process samples, 10,044 batches and 1,004,400
measured decodes. Input preparation and output observation remain outside timers;
replacement/disposal and failure-error construction/disposal remain inside.

At size 1000, median microseconds per operation (native/direct Rust/Node):
- Numeric dynamic: 10.56/8.80/5.38; typed: 15.62/6.58/6.14.
- Unicode dynamic: 84.18/68.14/46.17; typed: 121.55/68.74/48.01.
- Nested dynamic: 59.65/49.86/32.67; typed: 54.78/44.92/35.37.
- Selective dynamic: 309.70/259.08/186.08; typed: 192.39/91.60/189.06.
- Malformed dynamic: 40.93/27.47/18.65; typed: 16.76/26.28/18.43.
- Typed mismatch: 46.60/26.42/17.05; deep dynamic: 13.34/10.26/6.86.

Native enforces bounded JSON validation that direct Serde does not. Typed decoding
prevalidates syntax/depth/numbers and directly deserializes without an intermediate
dynamic tree. This costs time on valid input but rejects malformed input without
building the typed result. Review any future fused validation against these exact
error, number and allocation contracts; do not remove checks merely to match Serde.
Selective typed results retain only requested fields; dynamic results retain the
whole tree. Compare each with its matching baseline, not with different retention.

Separate RSS sizes 0/1/10000 provide another 324 process samples, no timed batches.
Whole-process peaks include startup, input setup, retained output and checksum
observers. At size 10000 native/direct Rust peaks in MiB were numeric typed
2.12/2.09, Unicode typed 3.42/3.39, nested typed 2.75/2.83, selective typed
4.20/4.28 and selective dynamic 13.56/13.58. Node process peaks ranged about
44–62 MiB; runtime and observer costs are included, not serializer-only memory.

SQLite evidence uses prefix `tsrust-json-decoding-` and suffix `-ec9c4b41`:
`timing` and `rss` JSON/log pairs, `audit` JSON plus its executable Python source,
and red/check/format logs. Eleven focused adapter/driver tests pass. Source and
input hashes and individual measurements remain immutable. Streaming direct-Rust
latency/RSS, public raw fragments and final installed consumers remain open.

### JSONL streaming latency and peak RSS

Four owned/borrowed reader/writer variants at 0/1/10000 records now have direct
Rust and Node baselines. Each timed batch is one complete file pass, including
open, processing, flush/close; fixture preparation and final file verification are
outside timing. Three processes, eight warmups and 31 batches retain the established
protocol: 108 process samples and 3,348 measured file passes. First-pass latency is
separate; OS page cache is uncontrolled, so no disk cold-cache claim is made.
Writer files match exact expected bytes after every process. Reader checksum
oracles and buffer contracts are reused from accepted allocation probes.

At 10000 records, median milliseconds per file pass (native/direct Rust/Node):
owned read 2.822/1.806/2.340; borrowed read 2.815/1.746/2.385; owned write
1.812/1.163/5.653; borrowed write 1.836/1.189/5.645. Native enforces bounded
record validation; direct baselines assume this valid fixture. All use bounded
buffering; no durability sync is included. Similar owned/borrowed costs provide no
signal requiring an adapter-specific optimization. Native validation/encoding
costs remain visible against direct Rust and must retain safety contracts.

A separate 108-process RSS matrix performs one pass per process. At 10000 records,
native/direct Rust median MiB: owned read 1.88/1.83, borrowed read 1.84/1.83,
owned write 1.78/1.78, borrowed write 1.75/1.77. Node peaks are about 51 MiB.
These include process/runtime overhead. Requested allocation evidence independently
establishes bounded buffer storage and complete disposal; RSS alone is not a leak
or allocation-count proof.

Four focused adapter/protocol tests pass. SQLite evidence uses prefix
`tsrust-json-stream-` and suffix `-ad4f3224`: timing/RSS JSON and log pairs,
audit JSON/Python source, red/check/format logs. Sources, adapters, direct baselines,
inputs and drivers are hashed in the raw records. Prior timing and allocation
records remain unchanged. Public raw fragments, final performance/contract audit,
installed refresh and release acceptance remain open.

### Refreshed measured-feature consumers

Release CLI and LSP were rebuilt from `12a6426e` using the shared target with
incremental compilation disabled. Atomic replacements in ~/.cargo/bin and
target/editor/debug match the release artifacts by SHA-256. Installed CLI smokes
reuse accepted borrowed JSONL reader/writer witnesses, including owner reuse,
exact writer bytes, formatting idempotence and native execution. The installed
CLI checks native_json_import.tsr. An isolated installed LSP session publishes
clean diagnostics and the complete readonly settings shape. VS Code's restart
control was invoked; the visible editor retains the complete settings hover.
This does not claim all workspace Problems are resolved.

Evidence prefix `tsrust-json-consumers-`, suffix `-12a6426e`, includes build,
installation hashes, native and LSP logs plus their executable smoke scripts.
Raw-fragment API work still requires its own affected checks and refresh. The
consumer update accepts the current measured implementation, not complete S10.

### Derived throughput and latency metrics

`python3 benchmarks/runtime/json_metrics.py` reads the accepted optimized encoding,
decoding and streaming timing records directly from SQLite. It produces 207
case/engine rows with p50/p95 batch latency, p50 per-operation latency and median
bytes/sec. Three process identities, 31 batches each, positive finite durations,
consistent operation counts and independent expected checksums are required.
Nearest-rank p95 uses all 93 batches within one source record; no measurements
from different source records are pooled. Two focused tests cover known percentiles,
byte-rate arithmetic, empty payloads and incomplete/invalid sample rejection.

Byte counts mean offered JSON input for decoding, exact encoded output for
encoding, and complete file bytes for streaming. Failed decoding throughput is
explicitly an offered-byte rate, not proof of full consumption. First-operation
samples remain separately available in original records. This derivation runs no
benchmarks and does not alter earlier evidence.

Representative native rates: typed numeric encoding at size 1000 is 480.9 MiB/s,
typed numeric decoding 238.9 MiB/s; 10000-record JSONL reads about 162 MiB/s and
writes 249–253 MiB/s. These fixture-specific rates include the documented operation
and disposal costs; they are not universal parser throughput claims. Canonical
result: `tsrust-json-metrics-e4705935.json`, with red/check/format logs in SQLite.
Public raw fragments and the remaining final contract audit still block S10 closure.

### Public raw-fragment validation and borrowing

`std:json` now exposes opaque `JsonRaw` and `borrowRaw(text, limits?)`. This
validation step is independent of the pending preserve-versus-normalize embedding
decision. Serialization is deliberately not accepted yet, and the catalog states
that limitation. No output-policy decision is inferred from elapsed time.

The compiler registers a readonly Result::Ok borrow of the source string. Generated
storage is a real `Fragment<'a>` and helper signatures preserve the source lifetime;
there is no copied source, hidden owner or erased lifetime. Existing error effects
register the Result-returning constructor as nonthrowing. Defaults and byte/depth/
entry/node budgets match strict JSON; comments and trailing data are rejected.

Three focused native tests pass: successful validation and failed-loan release,
argument-return lifetime preservation, mutation/clone/escape rejection, all four
resource budgets, invalid limits, strict syntax, exact large integer spelling and
unchanged source. One editor hover/signature test passes. Both catalog entries
share a verified formatted/idempotent native example. Intermediate failed test
and missing error-effect/hover-default repairs remain retained in SQLite.

`native_json_raw` registers native and TypeScript sources, hashes, ABI, independent
success-count oracle, source-placement/denominator guards and report category.
All 181 filtered native metadata/view unit tests and twenty JSON benchmark guards
pass; the new workload passes five untimed cases per
engine against Node-after-tsc. Both retain the input and count successful validation;
Node uses JSON.parse and discards its dynamic tree, while native validation creates
no dynamic tree. This difference is explicit; no allocation parity or timed result
is claimed. Record: `results/2026-10-12T06-43-07-171Z-verify.json`.

Focused evidence uses `tsrust-json-raw-*-fa4822b7` in SQLite. Installed consumers
remain at 12a6426e. This is an implementation checkpoint for validation/borrowing;
raw serialization, selective raw fields, complete measurements and the final S10
consumer refresh remain outstanding.

### Raw validation timing and numeric allocation baseline

`native_json_raw` completed the established native/Node execution protocol: five
correctness cases, three fresh processes per engine, eight warmups and 31 measured
batches of 2000 validations. All six process records and 186 batches have the
independent checksum 2007. Median batch milliseconds were native 0.145709 and
Node-after-tsc 0.313417. This fixed ASCII valid-input result does not establish raw
serialization speed or equivalence of parser resource policies.

A separate source-retaining allocation probe covers numeric arrays of sizes
0/1/10000, with three fresh processes per native/direct Rust/Node engine (27
samples). The native adapter verifies the fragment pointer equals the original
source pointer and rejects emitted ABI that substitutes a static lifetime. Native
and direct Rust report zero allocations/reallocations/retained/peak requested
output heap, and complete disposal. Source storage is prepared outside counters;
stack storage and Node retained heap are different metrics. Escaped strings may
need temporary scratch and are not covered by this zero-allocation result.
Direct Rust uses Serde IgnoredAny plus end-of-input validation and returns the
input slice; it does not apply native budgets. Node parses/discards its dynamic
tree and retains the original source. Source-byte checksum arithmetic is
independent of all parsers. Eleven adapter tests pass. After whole-project
formatting changed the new fixture layout, only these three affected cases were
repeated; both original and formatted evidence remain immutable.

SQLite: `results/2026-10-12T06-45-51-174Z.json`, raw timing log, and
`tsrust-json-raw-allocation*-6ea5b855` JSON/log records. The formatted record is the
current source baseline. No RSS, Unicode/deep/raw-field matrix or serialization
acceptance is claimed. Those and the post-raw consumer refresh remain open.

### Raw Unicode and deep allocation coverage

Raw validation allocation coverage now includes escaped Unicode pairs and a
numeric array wrapped to 32 container levels, each at sizes 0, 1 and 1000.
All six cases pass three fresh-process samples per engine (54 samples total).
Native source-pointer identity and independent UTF-8 source-byte checksums pass;
all native/direct Rust output and input disposal checks return to zero.

Nonempty Unicode inputs use one native 8-byte temporary allocation, unchanged
between one and 1000 pairs, and retain no output heap. Direct Serde IgnoredAny
uses no heap for these strings. Deep native validation uses no heap; direct
Serde uses one allocation and two reallocations (56 cumulative requested bytes,
32 peak bytes). These are different validation implementations: the direct
baseline does not enforce native budgets, and Node parses/discards a dynamic
tree. Node post-GC heap deltas are not allocation counts. Source storage is
prepared outside the operation counters; zero heap does not mean zero stack.

Twelve adapter tests pass, including the initial missing-kind failure witness.
Whole-project formatting changes no files or fingerprints. SQLite evidence:
`tsrust-json-raw-shapes-*-e1bf6193` (allocation JSON/log, red, check and format).
No additional timing, RSS, raw-field or serialization acceptance is claimed.
Raw serialization policy still awaits the owner; those remaining checks and
post-raw consumer refresh remain open before S10 acceptance.

### Raw rejection allocation coverage

The raw allocation harness now exercises a syntax error after 0, 1 or 1000
Unicode strings. All three sizes reject in native, direct Serde and Node, with
three fresh processes per engine (27 samples). Native error construction and
throwing-wrapper handling request 80 bytes across two allocations; direct Serde
requests 40 bytes in one allocation. Neither count grows with the preceding
string count. Both release all operation heap storage before the result probe;
subsequent input disposal also returns to zero. These totals include the native
wrapper's conversion to a thrown message and are not equivalent error APIs.
Node post-GC heap deltas remain a separate metric.

Thirteen adapter tests pass, including the prior missing-kind rejection witness.
Whole-project formatting changes no files or fingerprints. SQLite records use
`tsrust-json-raw-failure-*-b9639697` for allocation JSON/log, red, check and format.
This supplements successful raw validation measurements; timing/RSS, raw-field
support, output policy and final consumer acceptance remain outstanding.

### Raw validation timing and peak memory

The decoding harness accepts `--raw` to select raw numeric, Unicode, deep and
malformed inputs without rerunning accepted decoding cases. `--raw --rss` runs
memory independently. Both preserve source hashes, toolchains, independent
checksums and native pointer identity. Four shapes at three sizes and three
processes per engine produce 108 timing processes and 3348 measured batches
(100 operations per batch, eight warmups), plus 108 separate RSS processes.

At size 1000, median microseconds per operation for native/direct Rust/Node:
numeric 8.13/4.10/5.40; Unicode 40.25/22.08/44.04; deep 8.76/4.23/6.31;
malformed 16.11/6.56/18.43. Native is slower than Node for numeric/deep and
faster for these Unicode/rejection cases; direct Rust is faster throughout.
This is an initial raw baseline, not a regression against earlier decoding
semantics. Native performs budgeted validation, direct Serde skips budget checks,
and Node parses/discards a dynamic tree. The timing difference warrants later
profiling; it does not identify a cause or justify removing validation guarantees.

At size 10000, native median peak RSS spans 1.91–2.48 MiB, direct Rust
1.92–2.47 MiB and Node 44.31–46.78 MiB. These are whole-process peaks including
input preparation, observers and runtime overhead, not incremental allocations
or proof of leaks. Separate allocation evidence establishes temporary storage
and disposal. Timing excludes source preparation and result observation.

Fourteen focused adapter/protocol tests pass; whole-project formatting changes
no files or fingerprints. SQLite evidence uses `tsrust-json-raw-timing*-07331fe6`
and `tsrust-json-raw-rss-07331fe6` JSON/log records, including the original
unsupported-flag witness. Existing decoding records remain unchanged. Raw-field
support, output policy, post-raw consumer refresh and S10 closure remain open.

### Raw fragments stored in records

Two compiler tests and one editor test verify existing JsonRaw record storage.
A function returns an Envelope containing a raw fragment borrowed from its input;
native execution verifies the envelope and permits source replacement after its
scope ends. Negative checks reject source replacement while the record remains
live, return of a record borrowing a local source, and cloning the record. Editor
field hover retains the public JsonRaw spelling. Positive and negative sources
exercise formatting round trips. No compiler implementation change was needed.

This proves storage of an already validated fragment, not selective decoding of
raw fields from a larger JSON document. Selective decoding and serialization
remain open. Whole-project formatting changes no files or fingerprints. SQLite
evidence: `tsrust-json-raw-records-*-aac23a39` compiler, editor and format logs.

### Borrowing decoder boundary for selective fields

The internal typed backend now exposes decode_borrowed with Deserialize tied to
the input lifetime. A heap-owned input witness decodes borrowed keys and values,
checks source-pointer identity, rejects escaped text that cannot be represented
as a direct string slice, and retains byte-budget enforcement. This is groundwork
for selective source-backed fields, not a new public string or raw-field API.
The existing owned decode boundary delegates to it; JSONC normalization and
streaming continue to require independently owned results.

The initial borrowed witness failed under DeserializeOwned as expected. Replacing
the existing boundary directly exposed higher-ranked function-item requirements
in streaming; retaining the owned wrapper resolves that compatibility issue.
Both failed attempts are retained in SQLite. All 43 JSON backend tests pass.
Public raw-field visitors, analyzer provenance and editor support remain open;
serialization policy remains awaiting the owner.
The generated-native typed record/array/checked-number regression also passes.
Whole-project formatting changes no files or fingerprints. SQLite evidence:
`tsrust-json-borrowed-decoder-*-57682ac0` red, first check, repaired check,
native and format logs. No new performance measurements are claimed.

### Internal selective raw-field extraction

Fragment now implements lifetime-bound Serde deserialization using borrowed
RawValue. The bounded decode_borrowed entry point validates the complete input
before field extraction. A record-map witness preserves a selected object's
exact whitespace, unsigned 64-bit number spelling and Unicode escape tokens,
checks its source pointer, rejects a malformed surrogate and enforces the node
budget. No dynamic subtree or replacement source is retained. This does not
choose an output serialization policy.

The pinned serde_json raw_value feature is enabled in compiler and bundled
native dependencies. An isolated offline bundle build with an empty Cargo home
verifies borrowed RawValue availability and source identity. The initial missing
Deserialize witness fails as expected; all 44 JSON backend tests, five native
raw tests and the isolated dependency test subsequently pass. Whole-project
formatting changes no files or fingerprints. SQLite evidence uses prefix
`tsrust-json-raw-fields-` and suffix `-937e270a` for red, backend, bundle, native
and formatting logs.

This is internal decoding infrastructure. Public typed raw-field analysis,
owner provenance, lifetime-bearing generated visitors, editor documentation and
matched selective-field measurement support remain required. Existing owned
JSONC/streaming boundaries remain unchanged. Output semantics still await the
owner's decision; S10 is not accepted.

### Public typed raw-field implementation checkpoint

Strict JSON.parse<T> and JSON.tryParse<T> now accept explicit JsonRaw fields.
The analyzer retains the input borrow through the successful Result payload;
generated visitors carry real input lifetimes through raw records, arrays and
optional fields. Whole-result provenance conservatively retains the source for
borrowed projections. JSONC and JSONL decoding continue to reject raw targets
because their normalized or streamed input buffers cannot escape safely.
Serialization remains unsupported pending the owner's policy decision.

Focused native witnesses cover a raw field alongside an integer, raw arrays,
optional raw fields, returns borrowing a caller's source, scope-end owner reuse,
failed-decode loan release, owner invalidation, local-source escape and clone
rejection. Eight raw tests and seven typed-decoding tests pass, including catalog
examples and existing owned recursive/JSONC behavior. One editor test verifies
input-lifetime documentation; JsonRaw and both JSON parse forms have consistent
ownership notes and an executable selective-field example. Whole-project
formatting changes no files or fingerprints.

SQLite evidence: `tsrust-json-public-raw-*-106e74ac` initial rejection, first
native success, ownership, editor, affected native suites and formatting logs.
This is an implementation checkpoint, not selective-field measured acceptance.
Matched selective workload/driver support, allocation/timing/RSS measurements,
additional projection precision review and consumer refresh remain outstanding.

### Recursive and extracted raw-field lifetime checks

A focused native witness now decodes a nullable recursive Link containing raw
payloads and returns an extracted raw field from a function borrowing its input.
Both compile and execute. Two negative cases independently retain a direct field
projection or a function-returned raw field, then attempt to replace the source;
both reject with native-view-conflict. These supplement record-level provenance
checks without changing implementation or loosening borrowing rules.

The first recursive witness incorrectly used strictEqual on a union containing
an opaque raw value; the supported undefined guard replaces that comparison.
The original failure remains in SQLite alongside passing checks. Whole-project
formatting changes no files or fingerprints. Evidence records:
`tsrust-json-raw-recursive-{check,recheck}-4d7bffab`,
`tsrust-json-raw-extracted-check-4d7bffab` and
`tsrust-json-raw-projections-format-4d7bffab`. Selective-field benchmark support,
output policy and consumer refresh remain outstanding.

### Selective raw-field allocation baseline

The selective fixture decodes an envelope, checks its integer tag, retains only
its raw numeric-array payload and validates skipped escaped-string data. Native
output points exactly at the payload's source offset; independent decimal-byte
arithmetic checks its length. Direct Rust uses a borrowed RawValue field visitor.
Node validates the document with JSON.parse, then locates the source slice using
the fixed benchmark envelope. That Node adapter is not a general raw-field parser;
no equivalent allocation or representation claim is made.

Sizes 0, 1 and 1000 pass three processes per engine (27 samples). Native requests
17 bytes in three key allocations for empty input collections, or 25 bytes in
four allocations with escaped ignored strings. Direct Rust requests 17 bytes in
three allocations. Native peak temporary storage is 7/8 bytes respectively;
no output heap remains, and source disposal returns counters to zero. Counts
stay constant between one and 1000 elements. Node post-GC heap is a separate
metric. The current debug compiler emits optimized native benchmark executables;
source, emitted code, adapters, direct baseline and toolchains are fingerprinted.

Fourteen adapter tests pass. Whole-project formatting formats the new fixture
before measurement and changes no manifest fingerprints. SQLite records use
`tsrust-json-raw-selective-*-97cf2811`: initial missing-kind failure, transient
adapter syntax failure, passing checks, formatting and allocation JSON/log.
Selective timing/RSS and public consumer refresh remain open, as does the raw
serialization decision. This does not accept S10.

### Selective raw-field timing and memory

The existing decoding runner accepts --raw-selective, optionally with --rss,
without rerunning the earlier raw or decoding matrices. It records the fixed
Node source-slicing limitation and hashes the direct raw visitor. Three sizes,
three processes and three engines produce 27 timing processes and 837 measured
batches (100 operations each; eight warmups), plus 27 independent RSS processes.
Input preparation and source/checksum verification remain outside timing.

At size 1000, median microseconds per operation are native 50.44, direct Rust
15.18 and Node 56.32. P95 batch milliseconds are 5.5596, 1.6997 and 6.3887;
median offered-input throughput is 395.98, 1315.64 and 354.67 MiB/s respectively.
These derive from 93 retained batches per engine, with nearest-rank p95. Native
is faster than this Node fixture but slower than direct Rust, which omits native
resource budgets. This first selective baseline is not a regression comparison
and does not identify the cause of the gap. Prior allocation measurements prove
bounded temporary storage; timing alone does not.

At size 10000, median whole-process RSS is native 2.44 MiB, direct Rust 2.45 MiB
and Node 46.00 MiB. Startup, input storage and observers are included; these are
not incremental heap figures. Fifteen adapter/protocol tests pass. Whole-project
formatting changes no files or fingerprints. SQLite records use
`tsrust-json-selective-timing*-510cae62` and `tsrust-json-selective-rss-510cae62`
for original measurements, logs, initial unsupported-flag witness and checks.
Raw serialization policy and consumer refresh remain outstanding; S10 stays open.

### Raw consumer refresh and installed-hover repair

Both release executables were rebuilt from cbdbfd7a and atomically installed in
~/.cargo/bin and target/editor/debug, with all hashes verified. Six installed
CLI raw witnesses pass formatting idempotence and native execution; the selective
fixture checks successfully. Installed LSP diagnostics are clean for that fixture.

The installed protocol smoke caught a catalog placement defect: raw-field prose
had been inserted into the text parameter documentation instead of the main
parse/tryParse documentation. Corrected both locations and strengthened the
editor test to inspect each catalog field. The corrected release LSP was rebuilt,
atomically replaced and hash-verified; the installed protocol smoke now verifies
the raw-field input-lifetime explanation. The original failure remains preserved.

VS Code's restart control was invoked after each installation. The settled editor
shows checking enabled, the complete readonly settings import hover, and 11 errors
plus 42 warnings across the workspace. This refresh does not claim those remaining
Problems are fixed. Whole-project formatting changes no files or fingerprints.
SQLite evidence: `tsrust-json-raw-consumers-*-cbdbfd7a` includes builds, original
and corrected installation hashes, executable native/LSP smoke scripts, passing
checks, the original failed hover check, editor regression and formatting logs.
Raw output semantics and corresponding serialization acceptance remain open.

### Raw evidence and naming reconciliation

The metrics exporter now derives 252 case/engine rows from five immutable timing
records, including 45 raw validation/selective-field rows. Two metric tests pass;
all source protocols and independent checksums validate during derivation. No
workloads were rerun or samples pooled. The fresh compiled API inventory exposed
two unreviewed identities, JsonRaw and borrowRaw; their naming/ownership addendum
now reconciles all 433 public identities exactly once. The original missing-review
result remains in SQLite. The S10 contract audit retains its historical table and
adds a current reconciliation; roadmap consumer/raw status is updated accordingly.

Evidence: `tsrust-json-raw-{metrics*,api*,audit-format}-5682a032` in SQLite.
Whole-project formatting changes no files or fingerprints. Raw serialization
still awaits the owner's decision and is not accepted by this reconciliation.

### Generated-output cleanup

Removed 142 older duplicate test executables from the shared Cargo target,
reclaiming 3.71 GiB while retaining the newest executable per target and all
installed consumers. No source or analytics records were removed. Rebuilding an
older test variant may now require compilation. The workspace editor target was
left intact because its active usage could not be established from the sandbox.
The exact removed paths and sizes are retained in SQLite as
`tsrust-obsolete-test-outputs-243b012b.json`. Current S10 overview text is reconciled
with the already recorded raw implementation, measurements and consumer refresh.

## JSON import hover follow-up

Imported binding hovers now expand the validated schema structurally, including
nullable record arrays and empty objects. Internal data-record names no longer
replace those shapes. Array/union precedence remains explicit. Five focused
project-intelligence tests pass, including the existing simple import, field
navigation, completion and optional-array checks. This changes display only;
canonical type identity, readonly access and source navigation remain intact.
SQLite evidence: `tsrust-json-hover-{red,check,format}-dc6d1c73.log`.

