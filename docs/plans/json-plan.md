# S10 JSON

Status: private bounded decoding, automatic native dependency linking and artifact
reuse pass focused checks. No public `std:json` binding is available yet. S09 is
accepted; S10 remains open.

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

`stdlib/json/mod.rs` is currently test-only. It uses a DeserializeSeed visitor so
input bytes are checked before parsing, values are counted before decoding, and
array/object entry budgets stop the next value/key before construction. Duplicate
object entries count against the budget even when replacing an earlier value.
Container nesting has a maximum of 64; Serde's independent recursion guard stays
enabled. Scalar values do not add container depth. These limits are internal test
parameters, not published default settings.

The backend currently follows serde_json's owned Value representation: duplicate
keys replace previous values, object storage sorts keys, i64/u64 integer values
remain exact, floating negative zero survives, and nonfinite input numbers fail.
These are backend observations; public number conversion and object-order contracts
must be documented and tested explicitly during binding design. Null is a JSON
value; missing properties and Tsrust undefined must remain distinct at that boundary.
The private backend now returns structured codes, byte offsets and fixed redacted
messages. Public error records and their editor documentation remain required before
publication.

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
