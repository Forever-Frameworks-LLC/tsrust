# S10 JSON

Status: public dynamic JSON/JSONC, owned JSONL readers and standalone offline
package export pass focused compiler/native/editor checks. Installed CLI and LSP
are refreshed from `d3aedae3`; outside-repository JSONC/JSONL execution, relocated
package compilation/execution and checked editor hover pass. Matched benchmark
support, dynamic child operations, typed paths, borrowed adapters, public writers
and remaining S10 requirements are still open. No S10 timing acceptance is claimed.
S09 is accepted; S10 remains open.

The roadmap requires both dynamic owned JSON values and direct typed decoding.
Typed decoding must not build a dynamic tree first. Schema fusion follows S11;
S10 must retain the typed decoder boundary and allocation evidence needed for it.

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
The initial public slice documents these rules; numeric extraction/construction
and typed conversion remain pending. Null is a JSON
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
