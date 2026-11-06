# Standard library v0.2.0 implementation checklist

Owner direction, 2026-11-04: plan the missing useful standard-library features as
one explicit v0.2.0 checklist; bundle related implementation and acceptance work;
use Sol 6.1 subagents in parallel; design and measure performance from the first
gate. Popular, maintained, well-tested Rust packages are approved backends.

Status: finite release contract with implementation in progress, not completed
acceptance or permission to publish v0.2.0. The v0.1.2 main/tag/assets/docs
promotion is verified complete at main8efaaee2.
This plan supersedes historical stdlib scheduling and parallel-worker instructions
where they conflict. `docs/agent-progress.md` remains the only continuation note.
Update this checklist with accepted commits and evidence; do not create worker
handoff files. Source/fixture directories use descriptive feature names, not the
bundle IDs below.

## What completion means

v0.2.0 supplies the finite API surface listed in B01–B19, with B00 foundations and
B20 integration acceptance. It does not promise every Node export, JavaScript
runtime behavior or Rust trait/type. The disposition table below accounts for
the deliberately excluded families. A failed dependency gate does not silently
remove a release checkbox: repair the prerequisite or obtain an explicit scope
decision. API names below are proposed concrete spellings; freeze signatures,
defaults and ownership in B00 before implementing each bundle.

The owner is interested in UI as well. Completing every v0.2.0 checkbox is not a
prerequisite for discussing or prototyping UI. After the first useful utility
wave, review that priority with the owner. UI implementation remains a separate
scope decision; this plan does not revive the frozen renderer/framework work.

## Audited starting point

Read-only audits used `e3fea93666d9cc97e5d0e31190ffd6bd753dadd3`, including actual
native bindings/types/exports, stdlib runtime modules and `builtins.json`.
Reconcile again against the final v0.1.2 main commit before starting code.

| Existing capability | Preserve and extend |
|---|---|
| Arrays, strings, Map/Set, numbers | Rich eager collection/callback operations, Unicode normalization/casing, checked arithmetic/conversions and formatting already exist. Basic indexing/length are implemented despite stale inventory rows. |
| Files/bytes/path | Owned file and buffer handles, read/write/flush, strict UTF-8, borrowed byte views; seven lexical path functions. Public files are currently macOS/Linux only. |
| Data | JSON/JSONC/JSONL, typed schemas, exact decimal, Date/Time/Instant/Duration/TimeZone and UUID. |
| Services | HTTP client/server/TLS, Router/middleware/context/cookies, forms/multipart/typed JSON, static serving, streamed bodies/channels and graceful shutdown. |
| Async/threads | Tasks/Jobs, cancellation/timeout, race/allSettled, bounded channels, scoped/dedicated threads and ThreadPool. |
| Persistence | PostgreSQL/MySQL/SQLite pools, transactions, cursors and migrations; synchronous SQLite prepared statements; local/shared LRU and TTL. |
| Operations/testing | Signals on Unix; console, assertions/matchers, sync/async tests and hooks, source-located reports. |
| Rust packages | Locked package resolution and explicit safe synchronous adapters with concrete supported data; not arbitrary-crate automatic bindings. |

There are 23 public backend std namespaces in the current editor catalog, excluding
the private testing runtime and frozen UI. This is not a coverage percentage.
The 1,993-row ECMAScript/language inventory contains overlapping/stale entries and
is not a Node/Rust host-API ledger. Preserve implemented coverage when reconciling
it; do not manufacture missing work by reading an old `pending` label.

Primary implementation evidence:

- `crates/tsrust-compiler/src/native/{bindings,exports,types,signatures}.rs`.
- `crates/tsrust-compiler/src/stdlib/` and `crates/tsrust-lsp/src/builtins.json`.
- `crates/tsrust-compiler/src/project/rust.rs` and `examples/rust-regex/regex.rs`.
- Both `native-dependencies/Cargo.toml` and `backend-dependencies/Cargo.toml`.

## B00 — Contracts, package admission and baseline infrastructure

- [ ] Reconcile every checklist export against current compiler/catalog/tests.
  Mark already-supported items with actual evidence rather than reimplementing.
- [ ] For each new type/function/method, record the exact signature, receiver
  access, owned/borrowed result, error behavior, defaults, platform conditions,
  callback effects and cancellation/drop contract. Include aliases/stored calls.
- [ ] Match Rust std or the selected Rust backend's ordinary defaults. Audit
  existing artificial ceilings: file reads/writes currently impose 64 MiB;
  HTTP response readers impose 1 MiB and client code fixes a 30-second timeout,
  disables redirects and proxies. HTTP also caps forms at128fields/multipart at
  128parts and rejects configured body sizes over64MiB/deadlines over1hour.
  These are concrete v0.2 migration work, not defaults to copy into every new API.
- [ ] Make Tsrust-added size/cardinality/deadline policies explicit opt-ins through
  options or clearly bounded operations. Preserve the selected backend's own
  defaults and algorithmic/protocol/integer constraints,
  UTF-8 validation and ownership checks. Existing explicitly bounded APIs retain
  their documented contracts except the specifically listed B01/B10 migrations.
  Document observable v0.1→v0.2 default changes.
- [ ] Keep server/application resource policies explicit and source-owned; do
  not remove configured limits or add API/middleware settings to project config.
- [ ] Admit each package using official docs/repository, maintained compatible
  release, license/notices, RustSec/advisory review, MSRV and all four release
  targets. Check maintenance and direct application usage as evidence of fit;
  popularity alone is not a correctness or security guarantee.
- [ ] Prototype the direct Rust operation and its allocation/CPU/build profile
  before fixing the public facade. Prefer std, existing backend packages and
  thin typed adapters. Never implement URL parsers, regex engines, crypto,
  compression or protocol state machines from scratch to avoid a dependency.
- [ ] Pin compatible versions/features in both dependency closures and locks,
  reserved-package inventory, reexports, cache identity and portable exports.
  A transitive dependency or UI-only feature is not public backend availability.
- [ ] Prove feature selection: an unused module adds no unwanted dependency
  closure/runtime initialization. Measure minimal-program build time and size
  before admitting costly data tables or native build requirements.
- [ ] Prepare a reusable bounded test runner/build-slot procedure for every
  execution host. Keep one Cargo target per host, incremental off, jobs=1,
  process-tree memory/time monitoring, and ownership of killed processes clear.

First-wave source preparation identified these concrete shared prerequisites;
they are not completed implementation gates:

- [x] Add narrow native signature literal/exported-alias support where required
  for `SeekOrigin` and similar finite contracts. The source language already has
  literals; the native registry currently does not. Do not invent opaque owners
  or silently widen a promised literal union to arbitrary string.
  Accepted on9d3573ee: both `native_literal_aliases` tests pass, including
  shared exported native types in either import order without selecting an
  unrelated runtime. Homogeneous literal-string operations retain their exact
  type identity in compiler checks; editor phase identity repair remains open.
- [ ] Extend readable/writable byte alternatives by argument position for paired
  inputs; preserve input-only borrowed-shape checks and abstract callable
  contracts. Update the current single-byte-generic emission bridge where two
  independent input storage types are needed. Do not clone views to adapt them.
- [ ] Reuse existing StringView returned-loan machinery for borrowed URL getters;
  add computed property lowering rather than treating native getter properties
  as physical record fields. Prove separate read/write types, receiver/RHS once
  evaluation, optional loans and mutation blocking through aliases/captures.
- [ ] Investigate reported url2.5.8 empty-host setter panic, quadratic authority/
  dot-segment parsing and relative-reference differences before admission:
  [1174](https://github.com/servo/rust-url/issues/1174),
  [1163](https://github.com/servo/rust-url/issues/1163),
  [1164](https://github.com/servo/rust-url/issues/1164). These are upstream
  reports, not locally reproduced defects. Confirm under the watchdog and fix
  real failures; do not hide scaling problems with an arbitrary input cap or
  claim blanket WHATWG conformance from package selection.

Package preparation found url2.5.8 already locked/cached, with a compatible
declared MSRV, but its full closure/advisory/behavior admission remains open.
Base640.23.1 is a candidate with default SIMD features; choose its actual engine
and feature set deliberately, validate scalar fallback/all targets and measure
on a suitable quiet host. Trusted dependency internals may use unsafe; generated
Tsrust Rust must remain safe. Neither candidate is declared admitted here.

## Concrete API checklist

### B01 — Portable files, directories and file utilities

Backend: Rust `std::fs`/`std::io` first; retain the existing error/resource model.
Depends on B00. One owner covers file handles, directory operations and cursors.

- [ ] Port all existing `std:fs` exports and deterministic close/drop behavior to
  Windows; preserve `FsError {code, operation, message, osCode?}` and unique owners.
- [ ] `OpenOptions`; `openFile(path, options)` with read/write/append/create/
  createNew/truncate. Validate contradictory options before filesystem mutation.
- [ ] `FileTime {unixSeconds, nanoseconds}`, `Metadata`, `DirEntry`; `stat`,
  `lstat`, `fileStat`, `readDir`. Normalize pre-epoch times exactly; do not pull
  calendar dependencies or impose Instant's year range on basic filesystem times.
- [ ] `mkdir`, `mkdirAll`, `rename`, `copyFile`, `removeFile`, `removeDir`,
  `removeDirAll`, `canonicalize`, `readLink`, `hardLink`.
- [ ] `SeekOrigin`; `seek`, `position`, `truncate`, `syncAll`, `syncData`.
- [ ] `appendFile`, `readTextFile`, `writeTextFile`, `createTempDir`.
  Use admitted `tempfile::Builder` for exclusive temporary creation; validate a
  filename-only prefix and path conversion before transferring cleanup ownership.
- [ ] Remove arbitrary ordinary file-size ceilings through the B00 migration;
  retain explicit optional `maxBytes`/`maxEntries` controls and exact-bound tests.

Contract: no implicit recursive deletion; stat follows links and lstat does not;
tree deletion does not traverse directory symlinks. Preserve host directory order,
host rename/overwrite/cross-volume semantics and partial I/O. Non-Unicode paths
fail explicitly. Unsupported timestamps are absent, not invented. Positions/sizes
must fit exact public integers. Restrict cursor operations to plain File until
buffer invalidation is implemented. Temporary creation is atomic/exclusive.
Windows explicit close must use a safe backend. If it follows std File drop,
document that OS close errors are not observable there; do not pretend syncAll
is a close-error check or emit an unsafe Win32 call to manufacture parity.

Witnesses: Unicode/link/dangling-link tree, exclusive create, permissions, cursor
and EOF, short I/O, close failures, malformed UTF-8, explicit bounds, Windows
locked handles and exact drop counts. Performance: enumeration/stat, binary copy,
buffered append and random-position reads; same files/cache/durability policy in
Tsrust, handwritten Rust and Node. Report sync-to-storage costs separately.

Read-only dependency review (2026-11-04): `tempfile =3.27.0` with default
`getrandom` is the selected candidate for `createTempDir`; no nightly/wasm/sys_rng
extras. Declared crate MSRV1.63; effective selected native closure requires1.85
from getrandom0.4.3, compatible with pinned1.96.1. MIT OR Apache-2.0; retain
additional LLVM-exception notices where selected rustix/linux-raw-sys offer them.
The official RustSec snapshot965bf9ad083b58e6340d65829d85217b18be9290 has no
matching advisory for the selected locked native versions; once_cell's old
advisory is patched by1.21.4. This review is not runtime/platform admission.
Use Builder's .tmp prefix/six random characters/default collision behavior and
platform permissions/umask; do not promise0700. Validate UTF8 before keep().
Sources: [crate manifest](https://docs.rs/crate/tempfile/3.27.0/source/Cargo.toml),
[Builder](https://docs.rs/tempfile/3.27.0/tempfile/struct.Builder.html),
[RustSec snapshot](https://github.com/RustSec/advisory-db/tree/965bf9ad083b58e6340d65829d85217b18be9290/crates).

### B02 — Byte codecs and binary records

Backend: std for binary/hex/UTF-16; `base64` for base64. Depends on B00.

- [x] `std:bytes.toHex/fromHex`, `toBase64/fromBase64`,
  `toBase64Url/fromBase64Url`, explicit `toUtf8Lossy`, `fromUtf16/toUtf16`.
- [x] `concat`, `copyInto`, in-place `copyWithin`, `equals`, `compare`, `indexOf`.
- [x] `readInt/writeInt` for widths 1/2/4/8; `readUint/writeUint` for widths
  1/2/4; `readFloat32/writeFloat32`, `readFloat64/writeFloat64`.
- [x] Explicit offsets and byte order; reject unsupported widths/unrepresentable
  integer values before mutation. Float32 writes use Rust f64→f32 rounding,
  including nonfinite values, rather than the existing exact-conversion helper.
  Full-u64 values await a separately accepted exact type.

Implementation acceptance (macOS ARM64, worker174218b9): all23 new APIs have
compiler/runtime/catalog/editor/export witnesses accepted in40focused checks,
including actual O0/O3 catalog execution and safe relocated offline exports.
This checks implementation only. All75 matched runtime workloads and Windows/Linux
execution remain pending; shared performance/platform/release gates stay open.

Contract: readable Bytes/views for observers, writable Bytes/mutable views for
mutators. `copyInto` respects distinct-owner loans; same-owner overlap uses one
exclusive `copyWithin` receiver and Rust slice copy_within, without a hidden
temporary or relaxed alias checks. Canonical padded base64 and unpadded base64url output;
strict input alphabet/padding/trailing-bit checks. Hex output lowercase. UTF-16
endianness is explicit; malformed pairs/odd bytes error, without BOM guessing.
Existing strict UTF-8 stays strict; lossy conversion is separately named.

Witnesses: published codec vectors, all byte values, malformed offsets/encodings,
overlapping ranges, endianness, unaligned reads, NaN/signed zero and no partial
writes. Performance: token/4 KiB/1 MiB codecs and packet parsing/writing into reused
buffers, with independent byte checksums and equivalent strict Node validation.

- [x] Count allocations in actual emitted native packet and UTF-16 kernels,
  including inline endian literals and stored callable aliases with a checked
  literal-union order variable. Compare n=0 setup and several iteration counts
  against independent direct-Rust floors; do not infer bridge behavior from a
  zero-allocation direct-Rust kernel.
- [x] Packet field access must not add per-field/per-iteration allocations.
  UTF-16 output buffers are expected allocations; identify extra order/alias
  allocations separately, including mixed Unicode and CJK growth/reallocation.
  Preserve actual borrowed union payloads while profiling. Record actual emitted
  source/binary identity, exact outputs, requested/peak/live/drop counts and
  allocation slopes before considering a literal/storage optimization.

Generated allocation acceptance, 2026-11-05 (macOS ARM64): 104 scoped O0/O3
cases and 1,872 independent generated/direct-Rust samples pass exact outputs,
deterministic counters and complete disposal. Inline endian arguments and stored
aliases add no per-field/per-iteration allocations. Packet kernels keep one
64-byte setup allocation; UTF-16 encoders allocate one output per iteration.
Mixed decoding has no output reallocation; CJK decoding has one per output,
compared with one/two respectively in the direct std floor. This difference
justifies no sizing redesign. Unmodified generated variants compile with unsafe
code forbidden. Matched runtime workload timings remain pending; these untimed
allocation controls do not establish Windows/Linux execution.

Search backend prerequisite (2026-11-04): pin `memchr =2.8.3` with its default
std/alloc features, optional behind `byte-search` in both runtime graphs. Other
byte operations remain std-backed. The version was already locked in compiler
and runtime graphs; only the independent Rust reference graph adds a package.
Offline resolution preserves every previous package/version/checksum and all
default feature selections. Cached archive checksum
`cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98` matches
the lock. The crate declares edition2021/MSRV1.61, Unlicense OR MIT, no required
runtime dependencies. Keep optional logging/libc/core features disabled. Official
RustSec snapshot965bf9ad083b58e6340d65829d85217b18be9290 contains no memchr advisory;
platform execution and actual Tsrust API measurements remain pending.

The direct-Rust prototype supports using `memmem::find` for indexOf: linear time,
constant auxiliary space and portable fallback avoid the naive windows search's
adversarial growth. Token-sized inputs favored windows (10.19ns vs20.55ns), while
4KiB/1MiB tail matches and65KiB repetitive misses favored memchr (107/20,969/
66,063ns vs1,428/465,823/377,313ns). These are prototype samples, not released
Tsrust workload measurements. Do not add a hand-written size hybrid without an
accepted need. Pinned Node24.11.1 independent oracle preflight passed; earlier
Node26 prototype timings are supplementary, not canonical reference controls.
Sources: [pinned memmem find](https://docs.rs/memchr/2.8.3/memchr/memmem/fn.find.html),
[crate manifest](https://docs.rs/crate/memchr/2.8.3/source/Cargo.toml),
[reviewed advisory tree](https://github.com/RustSec/advisory-db/tree/965bf9ad083b58e6340d65829d85217b18be9290/crates).

### B03 — Path, environment, OS and signal completion

Backend: existing lexical path implementation, std and existing Tokio signals;
small `hostname`/platform adapter only where std lacks the operation. B00/B01.

- [x] `PathParts`; `std:path.resolve`, `parse`, `format`, `separator`, `delimiter`.
- [x] `std:env.vars`, `currentDir`, `currentExe`, `tempDir`, `homeDir`.
- [x] `std:os.availableParallelism`, `hostname`.
- [ ] Windows console interrupt/control-break support through existing signals.

Contract: preserve existing POSIX/Windows lexical path rules; normalization is
not filesystem containment. Resolve can fail on cwd. Environment absence, empty
values and non-Unicode failures remain distinct. No process-global env setters,
env removal or cwd mutation in the multithreaded runtime; B07 supplies per-child
configuration. Parallelism is an estimate. Unsupported signal events error.

Witnesses: drives/UNC/roots/dotfiles, controlled child cwd/environment, optional
home and real Windows interrupt handling. Performance: path corpus and environment
reads; do not fabricate meaningful speedups for constant target identity.

Preparation: hostname0.4.2 matches its official non-yanked sparse-index checksum
617aaa3557aef3810a6369d0a99fac8a080891b68bd9f9812a1eeda0c0730cbd,
MIT license and declared MSRV1.74. Locked metadata/fetch/tree admission kept
existing cfg-if1.0.5, libc0.2.190 and windows-link0.2.1 pins unchanged. Default
and optional setter features remain disabled. Advisory and platform admission remain separate from the macOS implementation
acceptance below. Direct-Rust allocation/build/timing prototypes remain distinct
from generated API throughput measurements.

B03 contracts frozen for implementation (2026-11-05; native/platform acceptance
remains separate):

| Module | New public signature or record |
|---|---|
| `std:path` | `PathParts {root:string; dir:string; base:string; ext:string; name:string}`; `PathFormatOptions {root?:string; dir?:string; base?:string; ext?:string; name?:string}`; `PathError {code:string; message:string}` |
| `std:path` | `parse(path:string):PathParts`; `format(parts:ReadOnly<PathFormatOptions>):string`; `resolve(...paths:string[]):Result<string,PathError>`; `separator():string`; `delimiter():string` |
| `std:env` | `EnvVar {name:string; value:string}`; `vars():Result<EnvVar[],EnvError>`; `currentDir():Result<string,EnvError>`; `currentExe():Result<string,EnvError>`; `tempDir():Result<string,EnvError>`; `homeDir():Result<string\|undefined,EnvError>` |
| `std:os` | `OsError {code:string; message:string}`; `availableParallelism():Result<int,OsError>`; `hostname():Result<string,OsError>` |

These are exactly12 functions and5 new ordinary record identities. String input
modes are Read, including resolve's rest inputs; spelling follows existing path
metadata, whose scalar signatures omit redundant access wrappers. Results are
independent values, with no returned-borrow contract or implicit input cloning.
Constants may use the existing static string representation. No public path-style
argument is added: execution-target cfg selects native separators and delimiters.

Parse returns all five fields, preserves lexical root/dir spelling and existing
dotfile/extname behavior; empty input returns five empty strings. Format treats
omitted and empty fields alike: nonempty dir precedes root, nonempty base precedes
name/ext; add a dot only for nonempty undotted ext and a separator only between a
non-root directory and the selected base. Use the measured checked exact-capacity
single-output adapter, ordinary Rust allocation behavior and no arbitrary cap.
Resolve ignores empty segments, stops right-to-left when an absolute device/root
is sufficient, observes cwd/drive context only as needed, and never canonicalizes.
New PathError codes are Io/NotUnicode; new resolve propagates bad-Unicode drive
context only when needed. Keep relative’s public signature and successful lexical
behavior; repair its known invalid-Unicode drive-context fallback through the
existing error channel, with focused demand/error witnesses.

Existing env get/args already preserve strict Unicode and explicit Result errors;
no getter migration is needed. Keep existing EnvError identity and InvalidName/
NotUnicode code spelling; add Io for fallible OS observers. vars converts both
names and values from vars_os into a complete owned unsorted snapshot, accepts
names that get's lookup validator rejects, and returns no partial data or raw
values on Unicode error. homeDir delegates pinned std::env::home_dir, including
its platform fallback and unavailable/unsupported None behavior; returned empty
paths remain empty. tempDir delegates std::env::temp_dir and creates nothing.
Convert all returned paths strictly. availableParallelism queries std each call,
checks positive usize-to-int conversion and never caches, caps or invents1.
Hostname uses selected hostname0.4.2, strict Unicode and no DNS/env fallback.
OsError codes are Io/NotUnicode/Range. New recoverable failures are Result data;
only ordinary unwrap introduces a throwing effect.

Windows extends existing listen/receive exports with Tokio ctrl_c/ctrl_break
and SIGINT/SIGBREAK results; Unix retains SIGINT/SIGTERM. No event selector,
configuration setting, process-global setter, kill API or listener framework.
Actual Windows console/OS acceptance remains required. The22 injected Windows
lexical controls (including demand/error ordering) passed both Rust and pinned
Node locally; they do not prove Windows OS behavior. Baseline/format allocation
profiles and their cached Node controls remain prototype evidence, not measured
Tsrust workloads or SQLite ingestion.

Source ownership: bundle worker owns stdlib path/env/os/signals runtime/bindings/
emission, bounded additive native signature/type/binding and feature predicate
hunks, effects audit, complete catalog/docs/examples/fixtures/editor agreement and
matched benchmark support. Integrator owns dependency manifests/locks/admission,
sourcepack/cache/export and benchmark dependency wiring, goal/plan/sole handoff.
Only hostname selects its optional dependency; all other new observers remain
std-only and unused imports add no runtime/dependency closure. Finish the entire
coherent section before batched affected checks.

Implementation acceptance (macOS ARM64, source72039246 plus fixture repairs):
all51 focused compiler/native/editor tests now pass; preserve46 initial passes
and5 repaired-test rechecks rather than replaying passed scopes. Coverage includes
17 catalog entries/15 unique examples, O0/O3 controlled environment observers,
path allocation and relative controls,17 help cases across four editor phases,
11 completion matrix rows and checker agreement. Five offline dependency/export
integration checks pass, including safe hostname linkage. All seven registered
workloads pass five independent correctness cases in Node24.11.1 and generated
O0/O3; this is untimed acceptance, not throughput measurement. Actual Windows
console signals, Windows/Linux execution, final package admission and performance
gap review remain pending. The official runner subsequently exposed a missing
fallible-family ABI registration; the existing Result-aware path now includes
path-env-os, with actual seven-workload emission and invalid-result guards. Eight
focused adapter/source/oracle tests pass; subsequent official timing is recorded below.
Fixture repairs preserve read-only iteration, use
supported module aliases/factory wrappers, and handle macOS filesystem rejection
of invalid-Unicode names while retaining strict conversion and child-env checks.

B03 first registered measurements (macOS ARM64,2026-11-05T14:40:20.088Z):
all three engines passed five correctness cases/workload, then3 processes ×31
samples after8 warmups. Quiet-load override was false, maximum observed load3.917;
watchdog90.710s/471,530KiB. Ingested21 actual run entries with npm run store:ingest;
verified93 samples per history and SQLite integrity. These are new API baselines,
not pre-feature speedups. Values below are median milliseconds per2,000 iterations
(env_get performs3 lookups per iteration).

| Workload | Native | Handwritten Rust | Node after tsc |
|---|---:|---:|---:|
| path_parse_parts | 0.181375 | 0.326500 | 0.106625 |
| path_format_options | 0.052292 | 0.048500 | 0.148167 |
| path_resolve_parts | 0.337583 | 0.305833 | 0.357708 |
| env_vars_snapshot | 11.734000 | 10.959166 | 101.418958 |
| env_get_snapshot | 0.598416 | 0.577208 | 0.999000 |
| os_parallelism_query | 0.444625 | 0.441541 | 1.008583 |
| os_hostname_read | 0.550625 | 0.549209 | 0.585625 |

Review: parse trails Node while beating the direct std-based Rust reference;
format is1.078× its corrected Rust floor. The first Rust format control omitted
the prefix check observed by native/Node; it measured0.038083ms and remains in
history. Added the same required prefix observation and remeasured ONLY that
changed reference at2026-11-05T14:47:48.571Z (93 samples, quiet max1.874,
watchdog6.757s/325,216KiB), ingesting one additional actual run. Native/Node
samples remain unchanged. Emitted-code review finds four owned nonempty parse
field allocations and a repeated basename scan, while format already uses one
exact-capacity output allocation. These allocation observations are source-derived,
not measured generated allocation counts. Profile any proposed parse scan change
before implementation; do not attribute inherent owned-string costs to the bridge.
Other native/Rust ratios range1.003–1.104. OS identity observations are not broad
compiler speed claims. Reuse these unchanged Node/Rust controls for later
compiler-only candidates. Official preparation also exposed compact handwritten
signatures and duplicate hostname extern selection; references now conform to
the existing ABI, and explicit reference closures replace generated bindings.
Nine focused adapter/oracle/emission/reference guards pass. Preserve first-run
failures as preparation evidence; none produced timing samples or ingested rows.

### B04 — General URL and query parameters

Backend: `url` plus its form-query support; `percent-encoding` if directly needed.
The existing url pin is UI-only and needs backend wiring. Depends on B00 and
existing byte/path capabilities; it does not wait for B02/B03's new public APIs.

- [ ] `Url`, `UrlError`, `parseUrl(text, base?)`, `formatUrl`, `resolveUrl`.
- [ ] Read/write href/protocol/username/password/hostname/port/pathname/search/hash
  and readonly origin, using checked backend component setters.
- [ ] `SearchParams`, `parseQuery`, `formatQuery`; `get`, `getAll`, `has`,
  `append`, `set`, `delete`, `entries`, stable `sort`.
- [ ] Independent `searchParams(url)` snapshot and explicit
  `setSearchParams(url, params)` replacement; no invisible parent alias.
- [ ] `pathToFileUrl`, `fileUrlToPath`, `encodeURIComponent`,
  `decodeURIComponent` with a documented UTF-8 component encoding set.

Contract: WHATWG/url normalization, IDNA/IPv6/default ports/opaque schemes;
duplicate query keys and absence versus empty; form-query plus differs from
component percent decoding. Strict malformed percent/UTF-8 failures where the
explicit decoder promises them. Do not expose URL credentials in errors.

Witnesses/performance: base-relative URL corpus, mutation and snapshot loans,
duplicate-key parse/mutate/serialize, IDNA/IPv6/file URLs and invalid input.
Node URL/URLSearchParams and direct Rust url are matched references.

### B05 — Async byte I/O across resources

Backend: existing Tokio with needed fs/io-util/io-std features. B01/B02/B06.

- [ ] Owned `Reader`, `Writer`, `LineReader`, `IoError` and explicit adapters for
  files/buffer handles, HTTP Body and standard output/error.
- [ ] Separate explicitly synchronous `Stdin`, `stdin`, `readStdin`,
  `readLineStdin` using Rust std blocking semantics. Do not expose an async stdin
  Task until portable interruptible/settlement behavior is proven.
- [ ] `read`, `readExact`, `readAll`, `write`, `writeAll`, `flush`, `copy`.
- [ ] Freeze and implement writable-destination Reader reads, concretely
  `readInto(reader: Mutable<Reader>, destination: Mutable<Bytes | ByteViewMut>):
  Task<Result<int, IoError>>` or the equivalent documented read overload. Return
  bytes written, observe only that range, and specify EOF versus a zero-length
  destination. Reuse caller-owned storage without per-read allocation. Keep both
  the reader and destination exclusively borrowed through actual I/O settlement;
  cancellation/deadline completion must not release a pending loan early,
  including blocking-backed file reads. Reject readonly or aliased destinations.
- [ ] `lines`, `readLine`; UTF-8 split across chunks, LF/CRLF and unterminated
  final-line behavior specified. EOF is distinct from an empty line.
- [ ] Consuming `close`/writer shutdown; report fallible flush/finish errors.
- [ ] Adapter points for B07 child pipes and B08 TCP halves.
- [ ] Optional explicit total-byte/line-byte bounds; ordinary defaults follow
  Rust/Tokio rather than new 64 MiB/1 MiB ceilings. Copy uses bounded working
  buffers and backpressure instead of whole-stream accumulation.

Contract: no blocking file calls on an async executor worker; input borrows live
through completion or inputs move into Tasks. No per-chunk hidden clone/boxing.
Cancellation of socket/body work must settle owned work and release resources;
partial I/O remains observable. Blocking-backed filesystem work may continue
after cancellation: retain its owner/completion handle and wait for actual
completion before claiming cleanup. A deadline is not a hard syscall interrupt.
Tokio stdin is an uncancellable blocking read, so it cannot be smuggled behind
the ordinary async Reader contract. Synchronous stdin blocks its caller as Rust
std does; cancellable interactive/async stdin remains later scope. Do not
introduce a public generic trait system for adapter plumbing.

Witnesses: short/interrupted synthetic I/O, EOF, malformed text, explicit bounds,
flush failure, cancellation, file→HTTP and HTTP→file; controlled blocking-operation
settlement and a process-isolated idle-stdin watchdog witness. Performance: fixed-payload
copy and line processing with chunk boundaries; allocations and peak retained
memory alongside throughput against direct Tokio and Node streams.

B01 allocation signal: the owned-read-contract handwritten seek prototype used
1,024 fresh 64-byte Vec buffers plus one observation Vec, with eight observation
reallocations and 16 KiB of retained rows. Those samples remain labeled as that
prototype, rather than the idiomatic Rust floor. Before actual timing, the public
handwritten reference uses one reusable stack buffer and preallocated observation
storage; Node likewise reuses one Buffer and reserves its result array. B01's
owned `read` remains unchanged, so generated allocation gaps remain observable.
B05's writable-destination contract must address this reuse case; no B01 API was
expanded and no generated allocation improvement is claimed from the prototype.

B05 public integration (2026-11-05): compiler/runtime/LSP/catalog/docs/examples
and three matched workload registrations are integrated. Runtime7, catalog/native
including file→HTTP and offline forbidden-unsafe export, shared editor agreement,
and focused task/SQL/HTTP compatibility scopes passed. Stored/conditional/forwarded
Task calls now retain reader/destination loans through await; positive native and
negative compiler/editor witnesses pass. Final service/literal compatibility repairs
pass their focused scopes. Actual public I/O timing/allocation review remains
pending; child-pipe/TCP adapters depend on their later bundles. This is local
integration evidence, not full release/platform acceptance.

B05 public measurements (2026-11-05, source6f8d3342): all45 correctness
combinations pass. Original3processes×31samples after8warmups for native/directRust/
Node; maximum quiet load3.79834. Nine actual histories ingested and SQLite integrity
verified. Median ms native/Rust/Node: readInto14.366958/14.348959/108.940625;
lines16.219/2.813625/18.351167; filecopy31.995583/24.175333/53.446625.
Line reading remains5.76×the Rust floor; filecopy1.32×. Native allocation profiling completed: readInto1003allocations/7reallocations;
lines5003/11000; filecopy5513/9, at their registered iteration counts. Line-reader
bytewise append/growth is a concrete candidate for chunk-based comparison before
implementation. No claim of optimized parity or full bundle/platform acceptance.

B05 line-reader optimization acceptance (2026-11-05): same-path/flags/library
paired comparison selected chunk scanning/appending over bytewise work, preserving
exact limit-detection consumption and reservation-failure progress. Three runtime
boundaries, eight oracle checks,18 allocation observations and186 timing samples
pass. Paired median15.513792→3.265958ms; full-workload allocations unchanged at
5003/11000reallocations; quiet max3.984375. Fresh libraries differ from the original
public measurement, so this is candidate evidence only. Canonical integration
passes all eight runtime witnesses plus the formatted native example. Fresh public
native timing remains pending, with unchanged Rust/Node controls reused.

B05 direct-Rust admission (2026-11-05): safe owned blocking-reader staging with
one reusable buffer passed short/interrupted reads, EOF/zero destination, retained
partial-error bytes, cancellation/deadline settlement and staged recovery without
rereading. Synthetic retained Bytes frames preserve the unread tail without a Vec
bridge; real HTTP integration remains pending. Three negative Rust witnesses reject
readonly destination, borrowed-destination reuse and reader reuse during a read.
Current futures runtime correctness passed1.558s/312,304KiB.

Optimized profile passed31.172s/300,432KiB, separate allocation builds,3processes×
31samples after8warmups, quiet maximum3.816. Both candidates reuse a64-byte caller
destination; file setup/drop are outside timing but inside allocation counts.

| Reads per operation | Staged median ms | Direct Tokio median ms |
| --- | ---: | ---: |
| 64 | 0.327583 | 0.315083 |
| 1024 | 5.003625 | 4.882750 |
| 8192 | 39.754334 | 38.860583 |

Staging costs2.3–4.0% over this matched backend floor. Ordinary allocation rows are
reads+3 staged versus reads+4 Tokio, principally blocking-task scheduling; reusable
destination storage does not imply allocation-free I/O. One staged1024-read sample
recorded1032allocations/1reallocation and151retained bytes; other retained deltas
were0. Keep this observation visible; no source attribution or zero-retention claim
is inferred. Prepared-source/profiling guards and actual output validate unchanged
inputs. These are admission prototypes, not generated facade or Node measurements;
no rows were ingested into generated workload storage. Full public B05 work remains.

### B06 — Task, timer and channel ergonomics

Backend: existing Tokio; `tokio-util::sync::CancellationToken` where appropriate.
Depends on B00; retain existing Job/Task cancellation and child-drain behavior.

- [ ] `all` for homogeneous Task/Job collections: ordered results, empty input,
  first failure, sibling cancellation and awaited cleanup.
- [ ] `Interval`, `tick`, explicit missed-tick policy; document Tokio's actual
  first-tick/default behavior rather than silently emulating JS setInterval.
- [ ] Cooperative `CancellationToken`: `share`, `child`, `cancel`,
  `isCancelled`, `cancelled`. Child cancellation does not cancel its parent.
- [ ] Async `Sender.close`, `trySend`, `Channel.tryReceive`; full/closed send
  errors retain the unsent owner, matching the synchronous channel contract.
- [ ] `ThreadPool.taskAsync` returning a Task with asynchronous completion rather
  than calling today's blocking Thread.join on an executor worker. Use an
  explicit bounded pool; freeze lazy submission/queue admission, owned captures,
  failure results and completion transport. Reject full/closed admission cleanly.
- [ ] Prove cancellation-before-start drops/reclaims owned inputs; after start,
  signal cooperative cancellation and await noninterruptible CPU work before
  claiming cleanup. B09/B16 use this adapter, with no hidden unbounded global pool.

Witnesses: ordering, failures, cancellation-before-wait, nested child cleanup,
missed ticks and move/drop tests. Performance: direct Tokio task/channel batches
and Node Promise/timer counterparts with equivalent work; timers themselves are
not a CPU speed benchmark. This is not JS Promise.then/catch compatibility.

B06 initial preparation findings, before the accepted unit-array prerequisite: allSettled<void> stores
Result<void,E>[] with empty Ok payloads; it does not establish a void[] mapping.
Task output and generic inference preserve Void, while current value/array type
resolution lacks a Void leaf. Preserve the ordered T[] contract for all, including
void, rather than silently substituting undefined[] or rejecting that case.
Prototype Rust Vec<()> and explicit compiler witnesses must settle this prerequisite.
Existing OwnedPool contains Arc<Pool>; cloning the public close-on-Drop owner or
marking it globally transferable would change shutdown/transfer semantics. A
lazy private submission capability can retain the existing Arc, submit only on
first poll, and observe closed admission if the public owner shuts down first.
Prototype both borrowed and retained transports, .start and awaited CPU cleanup
before fixing implementation. Locked tokio-util0.7.20 exports CancellationToken
without rt/default features; do not enable unrelated features. The preflight and direct-Rust prototype passed. Public B06 APIs are now integrated in source checkpointcb7d5a08, with no
public API acceptance claimed. The
unit-array prerequisite has since passed the focused acceptance below.

B06 prerequisite worker candidate f6f14954 is accepted in focused scope:
unit-array native ownership/callback/alias/TaskJob/mixed-tuple witnesses passed;
actual checked receiver capabilities survive local/imported aliases, incomplete
members and unrelated errors. Nonempty3receiver agreement,4rows×4phases and
editor consistency passed23.895s/79,088KiB; pure4case recovery-span unit passed.
Canonical integration3de00c52 subsequently built successfully517.850s/3,748,787KiB.
Focused integrated native/discard checks, nonempty3receiver agreement and all16
matrix observations passed18.660s/218,240KiB. This completes the prerequisite,
not any public B06 API checklist item.

B06 public source status (2026-11-05): all/Interval/CancellationToken, owner-retaining
async send/trySend/tryReceive/close and lazy ThreadPool.taskAsync are integrated with
compiler/LSP/catalog/examples and six matched workloads. Shared sync/async pool
transports preserve synchronous dependency isolation; cancellation uses exact
optional tokio-util0.7.20 with default features disabled. Root also added mixed
threads/async/cancellation portable export coverage. Benchmark source/oracle/ABI
mutation checks pass; generated workloads and timings remain unrun. Initial
combined no-run build exposed three test-only type/migration errors, repaired in
72ad2f9f together with deterministic cancellation cleanup witnesses. Combined
validation now has preserved passes through118539c4:18 runtime tests (including
cancelled-drain registration and deterministic missed ticks),2 CPU cleanup tests,
HTTP producer disposal, sync thread/channel baselines, taskAsync admission/owned
captures, all ordered/unit/scalar/record/nested-array outputs, catalog examples,
portable relocated offline export at O0/O3, five receiver agreement cases, eight
matrix rows×4phases and async editor consistency. Two final public failures led to
9ba4fa48: select futures runtime for synchronous async-channel operations and
correct the fixture to assert existing ThreadError.error beneath AsyncError.cause.
Their focused rebuild/validation passed all3 affected scopes in6.234s/383,648KiB.
Additional populated-array probes found fictitious ArrayPush failures, unsupported
owned Job-array iteration and skipped task evaluation inside rvalue/named indices.
Repairs reuse owned array storage and evaluate index side effects before token
transfer, preserving failure ordering and diagnostic severity. Source98d63784
passed all42 affected tests (6array,24async,12Tokio),27.490s/785,536KiB.
Named-index tests cover settled tasks/jobs, stores/assignments/indexed receivers,
hidden abandoned jobs and throwing evaluation order. No full regressions claimed.
Benchmark references now have
optional exact regex/Tokio dependencies; task group/channel observations detect
reordering through a matched ordered fold and independent BigInt oracle. Original
sample counts remain3×31after8. Generated workload validation and actual timing have since passed below;
performance-gap review and family/platform acceptance remain pending.

B06 prototype profile (2026-11-05): three processes ×31 samples after8 warmups,
separate allocation builds, quiet load3.448, watchdog8.434s/424,832KiB. Direct
polling all_yield measured48.30ns/input versus1,400.23ns for independently
spawned JoinSet tasks; these have different scheduling costs, so preserve direct
polling rather than claiming equivalent public performance. all_ready measured
21.43ns/input versus2.00ns for a sequential ready-only floor that lacks concurrent
polling/cleanup semantics. The71-allocation direct-polling batch includes64
individual future pins. Source review found no safe existing contiguous projection
for arbitrary !Unpin futures; do not impose Unpin or emit unsafe to remove these
allocations. The locked try_join_all large-collection path can delay a later error
behind an earlier pending future, so it does not preserve this fail-fast contract.
Keep direct polling and existing scoped drop/drain cleanup. Exclusive try-receive measured
47.64ns/operation versus47.46ns direct, both5 allocations/batch. Existing shared
async receiver measured190.87ns versus106.12ns direct and one additional64-byte
allocation/batch; this supports the planned exclusive receiver path.
Both pool capability variants passed lazy submission, start, owner-drop closed
admission and awaited cleanup controls with136-byte futures and zero construction
allocations. Strong retains a closed pool allocation; Weak does not. Select Weak for the lazy
private capability: public owner Drop closes admission either way, and accepted
Completion already retains its pool through execution/cleanup. Construction
5.30/7.50ns and closed polling25.86/30.87ns (Strong/Weak) are small relative to
CPU work. Async completion adds one64-byte allocation/job; observed CPU medians
7,410/7,415ns (Strong/Weak) versus6,945ns blocking join are noisy prototype signals,
not generated API measurements. Dependencies are unoptimized existing exact
rlibs; optimized probe code does not make this release-build performance. No
prototype rows were ingested as accepted API workloads.

B06 generated measurement (2026-11-05, source1c2367cd): all6 workloads passed
native/handwritten-Rust/Node-after-tsc five-case correctness. Actual execution-only
run passed294.179s/589,146KiB,3processes×31samples after8warmups, quiet maximum3.878.
All18 workload/engine groups were ingested;93samples each, exact medians and SQLite
integrity independently verified. No unchanged control was retimed after ingestion.

| Workload | Iterations | Native ms | Hand Rust ms | Node ms |
| --- | ---: | ---: | ---: | ---: |
| task_all_ready | 2000 | 0.845209 | 0.030666 | 0.640917 |
| task_all_yield | 2000 | 0.959041 | 0.076958 | 26.213084 |
| async_channel_try | 2000 | 0.103417 | 0.110916 | 0.242875 |
| async_channel_await | 2000 | 0.335333 | 0.181125 | 0.881125 |
| cancellation_fanout | 2000 | 1.191500 | 1.215875 | 36.341042 |
| thread_pool_async | 100 | 1.798625 | 1.777958 | 17.904041 |

Task groups show material costs against the naked Tokio join floor: about27.6×
ready and12.5×yield. The pre-optimization generated literal path boxed each future for
type erasure, then the generic runtime pinned it in another box; group scope/drain
and owned result storage also differ from the narrower hand control. These are
source observations, not isolated allocation attribution. Profile safe single-pin
and stack-pinned literal candidates while preserving arbitrary !Unpin futures,
borrowed inputs, ordered results, fail-fast drop and exact cancellation/cleanup
scope behavior before optimizing. Async-channel await is1.85×hand Rust and also
needs gap review; other3 rows are close to that backend floor. Full family/platform
acceptance and the measured optimization review remain pending.

B06 literal-all optimization (2026-11-05, source21c71460): safe stack-pinned
fixed inputs replace double-boxed futures and dynamic polling scratch storage.
Eight focused witnesses passed, including borrowed !Unpin futures, construction
order, typed failure, drop order, dynamic-array fallback and catalog execution.
A crate-visibility defect was repaired atc0b3c2bc before native acceptance.
Native-only actual run `2026-11-05T20-40-11-695Z.json` passed373.261s/
2,291,916KiB including release bootstrap and quiet waiting; five correctness cases
per workload,3processes×31samples after8warmups, maximum quiet load3.620.
Ready median0.506292ms versus0.845209ms previously (40.1% lower); yield0.670125ms
versus0.959041ms (30.1% lower). Both93sample histories ingested and independently
verified with SQLite integrity. Existing handwritten-Rust/Node controls remain
unchanged and were not retimed. Ready is still16.5×the narrower Rust join floor;
yield8.7×. Scope registration/draining and owned results remain required work,
so these ratios do not isolate compiler overhead. Four false unused-task warnings
after a named task await remain a separate checker repair; native behavior passes.

### B07 — Subprocesses

Backend: Tokio process, not a new process framework. B03/B05/B06.

- [ ] `CommandOptions`, `Child`, `ExitStatus`, `Output`, `ProcessError`.
- [ ] `spawn(program, args, options?)`, `run(program, args, options?)`.
- [ ] Per-child cwd/env/clearEnv and finite inherit/null/pipe stdio choices.
- [ ] Freeze Rust-style spawn inherited stdio versus run captured stdout/stderr
  and null stdin. Give run a consumed Bytes/Reader input option; reject an
  inaccessible stdin pipe when no payload/reader is supplied, then close input
  after draining it so a child can observe EOF.
- [ ] Child `id`, `takeStdin`, `takeStdout`, `takeStderr`, `wait`, `tryWait`,
  `kill`; pipe endpoints transfer once and use B05.
- [ ] Concurrent stdout/stderr draining and awaited reaping; optional explicit
  deadline/output bound, no arbitrary default 30-second/64-MiB policy.

Contract: no implicit shell interpolation. Nonzero exit is an Output status,
distinct from spawn failure. Cancellation/explicit limits terminate and reap the
owned child. Document direct-child scope and best-effort destructor limitations;
do not promise whole-process-tree termination from ordinary kill-on-drop.

Witnesses: binary echo, pipe-capacity flooding, argument boundaries, cwd/env,
missing executable, nonzero exit, timeout, cancellation, inaccessible run pipe
rejection and closed stdin on all
platforms. Performance: same helper executable for spawn/wait, capture and duplex
I/O; separate OS launch time from adapter work.

B07 direct-Rust admission (2026-11-05): normal and separately instrumented
release builds pass exact source/dependency/binary guards. All19 correctness
records pass, including binary duplex and EOF, flooded outputs, environment/cwd,
nonzero/missing executable, explicit bounds/deadline, lazy unpolled input,
cancellation/reaping and transferred/borrowed child-pipe ownership. The runner
verified1,395 matched timing rows and10 allocation cases after a quiet start3.69238.
These are admission prototypes, not acceptance of public Tsrust APIs or target
platform execution. Public implementation now proceeds in the isolated process tree.

### B08 — TCP/UDP, address parsing and system DNS

Backend: existing Tokio net/system resolver; std address types. B05/B06.

- [ ] `IpAddress`, `SocketAddress`, `NetError`; `parseIp`, `formatIp`,
  `parseAddress`, `formatAddress`, `isIPv4`, `isIPv6`.
- [ ] `TcpListener`, `TcpStream`, read/write halves; `listenTcp`, `connectTcp`,
  `accept`, `localAddress`, `peerAddress`, `split`, `setNoDelay`,
  `shutdownWrite`, `close`.
- [ ] `UdpSocket`, `Datagram`; `bindUdp`, `sendTo`, `receiveFrom`,
  `localAddress`, `close`.
- [ ] `std:dns.lookup(host, family?)` for any/ipv4/ipv6 via the system resolver.

Contract: parsing performs no DNS; bind port zero returns the actual endpoint.
Preserve half-close, datagram boundaries, empty datagrams and underlying receive
truncation semantics explicitly; do not invent a reliable truncation flag. DNS
does not imply TTL/cache/record-query support. Backend timeouts remain opt-in;
timeout does not prove an OS resolver thread was forcibly cancelled.

Witnesses/performance: deterministic loopback IPv4/IPv6, refusal, EOF, partial
traffic, UDP boundaries/receive capacity and controlled resolver fixtures.
Measure fixed-size echo/transfer with equal connection policy; DNS latency is a
separate environmental observation, not a compiler throughput claim.

### B09 — Compression and streaming transforms

Backend: maintained `flate2` with one deliberately selected Rust backend. B02/B05/B06.

- [ ] `CompressionOptions`, `DecompressionOptions`, `CompressionError`.
- [ ] `gzip/gunzip`, `deflate/inflate`, `deflateRaw/inflateRaw`.
- [ ] Reader/writer transform adapters and explicit `finish`/close.
- [ ] Optional explicit decompressed-byte policy and clearly bounded entry
  points for untrusted input; no hidden default expansion ceiling on ordinary
  Rust-equivalent operations. Apply explicit bounds incrementally.

Contract: inherit backend compression defaults; validate level and format options.
Check trailers/checksums/truncation, concatenate gzip members according to the
chosen documented decoder, and define trailing-data rejection. Deterministic
gzip metadata where controllable. CPU-heavy work must not block async I/O workers.

Witnesses: cross-engine decode, corruption, concatenation, explicit expansion
bounds and finish failures. Performance: compressible/incompressible buffers and
streaming chunks at matched levels, recording output size, CPU and memory.

### B10 — HTTP client completeness and source-owned middleware

Backend: existing Reqwest/Axum/Tower stack. B04/B05/B06; compression uses B09.

- [ ] Extend ClientOptions with timeout/connect-timeout, redirect and proxy
  policy and response-body policy; preserve TLS verification and existing ca.
- [ ] Migrate unexplained client default overrides through B00: absent options
  use the selected Rust backend's defaults. Keep explicit disabling/bounds.
- [ ] `requestBytes`, `requestStream`, `requestChannel`; owned binary and
  streaming upload bodies with backpressure, cancellation and exact framing.
- [ ] Reusable client multipart/form construction, including binary parts,
  instead of limiting public client requests to String bodies.
- [ ] Source-installed `cors`, `secureHeaders`, `compress` middleware helpers.
  Defaults/arguments are documented; no helper installs itself globally.
- [ ] Prove supported negotiated HTTP/2 and TLS paths end to end; expose a
  transport mode option only where the existing backend needs explicit selection.

Contract: policy remains application-owned. Test redirect credential stripping,
proxy auth/error redaction, CORS preflight and normal/error/stream responses,
header behavior, disconnects and configured limits. API/client generation remains
source/contract-owned with no new project-config requirements.

Performance: existing text/JSON/SQLite comparisons remain controls; add fixed
binary/stream uploads, pooled requests, middleware and CPU-per-request cases.
Compare direct same-stack Rust and Node; include p50/p99, throughput, CPU/request
and RSS for service scenarios, with the same load generator and concurrency.

### B11 — SSE and WebSocket APIs

Backend: Axum SSE/ws and compatible Tokio Tungstenite client. B05/B06/B10.

- [ ] `SseEvent {data, event?, id?, retryMs?}` and `sse` over a typed producer
  channel, with optional keepalive and correct event framing.
- [ ] Router WebSocket upgrade registration with application auth/origin hook.
- [ ] Owned `WebSocket`, typed text/binary/close messages, `receive`, `send`,
  `close`, and backend ping/pong/close processing.
- [ ] Consuming `split` into owned read/write halves, so send can proceed while
  receive is pending without violating exclusive async receiver loans.
- [ ] `connectWebSocket(url, options?)` using the same socket contract and TLS
  verification, with explicit frame/message/queue policies.

Contract: SSE transport already exists manually; this adds safe framing. Auth
runs before upgrade; no automatic reconnect, unbounded facade queues or hidden
per-message clones. Follow underlying backend defaults and expose bounds/policy.
Disconnect and shutdown settle producer/socket tasks and release owned state.

Witnesses/performance: independent multiline/Unicode SSE byte oracle, slow readers,
authentication rejection, invalid frames, text/binary echo, full-duplex pending
receive plus send, half-drop/cancellation, close races and exact
cleanup. Measure events/bytes, p99 and retained memory against direct backend Rust
and Node WebSocket/SSE implementations with versions recorded.

### B12 — Reusable regular expressions and text entry points

Backend: Rust `regex`; the current user-binding example is not a bundled module.
Depends on B00, independently of generic lazy iterators.

- [ ] `Regex`, `RegexError`, match/capture records; `compile`, `escape`.
- [ ] `isMatch`, `find`, `findAll`, `captures`, `capturesAll`, `replace`,
  `replaceAll`, `split` on a reusable compiled owner.
- [ ] `String.match`, `matchAll`, `search` entry points over compiled Regex,
  with the same explicit result/offset/ownership contracts.
- [ ] UTF-8 byte ranges and optional capture ranges; avoid copying every matched
  substring or capture name into every result. Distinguish absent/empty capture.
- [ ] Ordinary builder limits follow regex defaults; optional explicit compile/
  result limits are documented. Replacement literal/template behavior must be
  named explicitly; never accidentally interpret replacement tokens.

Contract: no repeated compilation in matching loops, mutable lastIndex or silent
JS lookaround/backreference emulation. Explicitly document native regex syntax,
Unicode and zero-width progress, including any eager matchAll snapshot behavior.

Witnesses/performance: invalid/unsupported patterns, Unicode byte offsets, capture
absence, zero-width matches, reuse and resource lifetime. Separate compilation
from reused search/capture/replace workloads; use shared-semantic Node patterns
and independent range oracles. Include allocation counts and adversarial scaling.

B12 implementation status (2026-11-05): source preparation1379a46e and repairs
938f57b2/11ae1144/31738942 cover compiler/runtime/LSP/catalog/examples, optional
backend dependency wiring and13 matched workload registrations. Eight direct
runtime tests, feature selection/hidden exports, all18entry editor consistency,
16matrix observations and5receiver checker agreement passed in their focused
scopes. All13 intrinsic error-effect contracts and the unreviewed-effect guard
passed11ae1144. Native/catalog execution exposed optional-input signature lookup;
portable export exposed missing default feature activation. Both are repaired in
31738942. All18 catalog examples and portable relocation/offline/forbid-unsafe
export now pass at O0/O3. Native ownership exposed a temporary-borrow staging
error for computed optional options; its same-statement conversion fix passed
formatted native ownership/options/stored-call execution at O0/O3 on72ad2f9f
(32.088s). No checklist item is accepted yet;
Generated benchmark correctness and actual timings now pass as recorded below;
remaining platform and feature-family checks stay pending. Prototype evidence
is distinct from generated API performance.

B12 generated workload correctness/timing (2026-11-05, sourcebe086e3a): all13
workloads passed5cases on native, handwritten Rust and Node-after-tsc. Official
execution-only run completed535.232s/526,155KiB with quiet guard enabled,
maxload3.690. Each engine/workload has93samples (3processes×31after8warmups).
`npm run store:ingest` added39 actual histories at2026-11-05T18:26:31.084Z;
exact medians/samplecounts and SQLite integrity were verified. Unchanged Rust/Node
controls are now reusable. Disposable result2026-11-05T18-26-31-084Z.json.

All rows below are milliseconds for2000iterations:

| Workload | Native | Handwritten Rust | Node after tsc |
|---|---:|---:|---:|
| regex_compile | 31.836792 | 31.238375 | 0.114375 |
| regex_escape | 0.126375 | 0.132875 | 0.441833 |
| regex_is_match | 0.048166 | 0.047041 | 0.032958 |
| regex_find | 0.057167 | 0.054958 | 0.059459 |
| regex_find_all | 0.186334 | 0.176667 | 0.584750 |
| regex_captures | 0.234541 | 0.244167 | 0.306375 |
| regex_captures_all | 0.651041 | 0.652542 | 1.046167 |
| regex_replace | 0.399833 | 0.381542 | 0.388208 |
| regex_replace_all | 0.660791 | 0.664000 | 0.346041 |
| regex_split | 0.186541 | 0.178292 | 0.482625 |
| string_regex_match | 0.248500 | 0.232000 | 0.309875 |
| string_regex_match_all | 0.656959 | 0.696042 | 1.046750 |
| string_regex_search | 0.055500 | 0.055625 | 0.037292 |

Native/direct-Rust median ratios span0.944–1.071; no large adapter gap appeared
in this run. Constructor timing measures repeated public construction, not isolated
fresh backend code generation; the large Node/Rust construction difference must
not be represented as compiler bridge overhead. Backend operations and eager
snapshots retain their documented semantics. Platform execution and remaining
feature-family acceptance stay separate; no v0.2 publication approval is implied.

B12 initial admission preparation (2026-11-05; before package wiring):
select regex1.13.1 with default std/perf/unicode and regex-syntax/default features.
The [official index](https://raw.githubusercontent.com/rust-lang/crates.io-index/master/re/ge/regex)
records a non-yanked July15,2026 release, MSRV1.65 and checksum
f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d.
The [selected manifest](https://raw.githubusercontent.com/rust-lang/regex/1.13.1/Cargo.toml)
uses MIT OR Apache-2.0. The [changelog](https://raw.githubusercontent.com/rust-lang/regex/1.13.1/CHANGELOG.md)
fixes incorrect leftmost-first offsets in reverse-suffix/inner optimizations;
do not choose cached1.12.2 solely for availability. Keep the backend builder's
10MiB compiled-NFA and2MiB hybrid-cache defaults. The
[package advisory listing](https://rustsec.org/packages/regex.html) identifies
RUSTSEC-2022-0013 as fixed before this version; resolved-closure advisory review,
archive verification, target execution and optional-feature/export admission
remain pending.

B12 resolved prototype admission (2026-11-05): standalone lock selects
aho-corasick1.1.5 (Unlicense OR MIT, MSRV1.60), memchr2.8.3
(Unlicense OR MIT, MSRV1.61), regex1.13.1, regex-automata0.4.18 and
regex-syntax0.8.11 (MIT OR Apache-2.0, MSRV1.65). All five archive SHA256s
match their lock checksums. Current official RustSec database26fd2d43
(1409 advisories) reports zero vulnerabilities and zero warnings for this closure.
Offline locked optimized prototype build passed13.447s/348,064KiB; direct
correctness/allocation/timing driver passed2.065s/213,408KiB with optimized exact
rlibs, three processes ×31 samples after8 warmups and separate allocation builds.
Ten semantic cases plus invalid-pattern/explicit-limit/escape/owner-lifetime
controls passed independent Node oracles. Quiet guard maxload2.226. Driver
verifies unchanged source/artifact hashes and materializes owned outputs through
black_box before checksum traversal. Evidence `/tmp/tsrust-b12-{artifacts.jsonl,
build.log,profile.json,profile.log,closure.json,audit.json}`. This admits a
prototype, not public APIs, production package wiring, portable export or other
platforms. No prototype timing is ingested as generated API performance.

Profile decision: retain the thin backend owner, no extra cache or mutable scratch.
ASCII/Unicode compile costs11.49/18.27µs versus reused isMatch22.12/16.94ns
with zero allocations. For64matches, findAll ranges take3.161/2.723µs and
1allocation+4reallocations versus copied strings4.384/4.018µs and65+4;
requested bytes1,984 versus3,424/3,232. Streaming floors2.819/2.417µs do not
return equivalent owned snapshots. Captures snapshots132.81/152.34ns allocate2;
capturesAll9.434/9.185µs allocate130+4 for64matches. Mutable reused
CaptureLocations is a non-equivalent floor, not grounds for owner locks/caches.
No-match owned replacement adds one32byte copy by contract. Split ranges use
one output allocation+5reallocations; avoid counting prepasses. Adversarial
`a+z|a` iteration64/256/1024bytes takes backend2.136/15.198/166.750µs and
range snapshots2.354/15.833/166.302µs: potentially quadratic total iteration
work already exists in the backend and must be documented. These are warmed
direct Rust prototype medians, not generated workload acceptance measurements.

Use owned UTF-8 byte-range records for find/capture/split snapshots; borrow regex
and text only during the call. Missing captures remain distinct from empty ranges.
Numbered capture slots cover this finite family without copying names into each
result. Preserve Rust replacement-template expansion with an explicitly named
replacementTemplate argument and documented numbered/named references, braces
and literal-dollar escaping. This is a contract direction, not frozen signatures
or completed APIs. Profile compile/reuse, optional/empty captures, zero-width
progress, range versus substring allocations and adversarial iteration before
implementation; include all three String entry points in the coherent gate.

### B13 — A useful explicit-locale subset

Backend: selected ICU4X components, not automatic inclusion of the meta-crate.
B00/B02; measure locale-data footprint before API acceptance.

- [x] `Locale` parsing/canonicalization with explicit locale input.
- [x] Reusable `Collator.compare`, and `String.localeCompare` using that contract.
- [x] `Segmenter.segment` for grapheme boundaries with UTF-8 byte ranges.
- [x] Reusable `NumberFormat.format` for exact int/Decimal inputs; document
  supported rounding/grouping/options and reject unsupported options.
- [x] Pin selected compiled data and document backend locale fallback/coverage.
  Do not label this full ECMAScript Intl or infer host-user locale silently.

Witnesses: combining text, emoji clusters, collation fixtures, negative/large exact
decimals, unsupported options and data fallback. Performance: construct once versus
format/compare repeatedly, allocation counts, binary size and cold/build memory.
Node ICU and Rust ICU versions/locales are recorded; differing data is not a
compiler defect or permission to combine incomparable timing rows.

B13 focused public acceptance (2026-11-05, source458500b2):3 runtime cases,
7 compiler/native cases (including17catalog examples atO0/O3, exact int/Decimal
union inputs, optional options, aliases/stored calls, negative constructors,
NumberFormat worker transfer/suspension and relocated offline forbid-unsafe
builds) passed. Editor exports/hover/signature help,6completion rows across4modes
and8receiver agreement groups passed. Runtime selection stays optional. Selected
provider sync preserves all existing dependency versions and the previously
measured54active-identity production graph. Required borrowed-union input repair
80795335 adds repeated concrete Decimal/union/forwarding owner reuse atO0/O3 and
preserves contextual record/generic inference and narrowed union projections;
26focused tests accepted after repairing two intermediate projection regressions.
Actual public11workload measurements passed on80795335:165correctness cases,
33workload/engine groups with3processes×31samples after8warmups, quiet maximum
3.83447265625. All33actual histories ingested and independently verified against
checksums,93positive samples, median/rate calculations and SQLite integrity.
Result `benchmarks/runtime/results/2026-11-05T22-00-33-080Z.json`; watchdog851.767s/
2,280,220KiB includes release compilation, preparation and quiet wait. Native/hand
median ratios span0.9855–1.1144; fixed Decimal1.1144 and reusable segmentation1.1013
need allocation review before optimization claims. Node locale formatting uses an
already canonical string and comparison uses its ICU backend; do not attribute those
differences solely to compiler overhead. Allocation/gap review and cross-platform
milestone validation remain pending; these API items do not close those gates.

Allocation review on80795335 passed11workloads×3stable samples, all zero retained
bytes (`diagnostics/2026-11-05T22-09-20-074Z-native-allocations.json`,504.716s/
530,651KiB watchdog). Reused and constructed collators/String.localeCompare allocate
zero; locale strings and integer format outputs allocate once per operation.
Separate unchanged Rust controls passed54samples acrossn0/1/full for segmentation
and both Decimal cases (`/tmp/tsrust-locale-control-allocations.json`,2.590s/259,584KiB).
Per formatting operation, Decimal matches Rust at6allocations/4reallocations/231
requested bytes; fixed Decimal matches at4allocations/0reallocations/28bytes.
Setup-only native differences are2allocations/32bytes for large Decimal and1/5bytes
for fixed formatting. No measured per-iteration allocation penalty explains the
fixed-format timing gap. Segmentation matches Rust at1allocation/64bytes per call,
but generated Array stores this three-range result inline after consuming a backend
Vec. The direct backend iterator-to-owned-array candidate passed owned Unicode
snapshot and allocation controls: zero allocations for0..4graphemes, one at5, and
one plus3reallocations at64, all retained0. Candidate timing was provisional because
shared-target rebuilds changed dependency artifact bytes; it reused exact current
control-library hashes and retained prior timing identities separately.
The selected runtime change keeps the existing emitter's owned collection and
avoids the temporary Vec. Three runtime tests, expanded snapshot thresholds at
O0/O3,17catalog examples and relocated offline forbid-unsafe builds passed6focused
cases (130.575s/425,024KiB); no public API or editor contract changed. Build501.188s/
3,794,563KiB, with the same3unit-harness warnings as the prior accepted builds.
Fresh actual timing for only2changed native workloads passed on86664712:
`results/2026-11-05T22-35-31-237Z.json`,458.599s/2,323,996KiB including compilation
and quiet wait, quietmax3.67236328125. Both93-sample histories were ingested and
independently verified. Reuse0.189833ms and construct0.170250ms are higher than the
prior0.174416/0.167708ms; all observations remain recorded without outlier removal.
Process medians vary substantially (reuse0.213208/0.189833/0.149916ms).
One controlled A/B used identical current libraries, source pathname/crate name and
flags, alternating variant order: all six pairs favored the iterator, but substantial
process variation prevents a stable cross-run speed claim (372samples,quietmax2.515,
4.135s/293,408KiB). No Rust/Node controls were retimed or duplicate histories invented.
The accepted benefit is removal of the temporary allocation. Exact newly measured
sources/library hashes passed18allocation invocations atn0/1/full, all zero allocation,
reallocation and retention (3.642s/304,432KiB). Source remains selected; no further
local retiming solely for presentation. Other locale measurements remain unchanged.

B13 admission/profile status (2026-11-05): isolated exact ICU4X locale/collator/
decimal/segmenter closure passed offline optimized build26.933s/319,104KiB.
Initial32record correctness classification has31 matching Node contracts and one
explicit untimed backend difference: en-u-nu-arab retains English separators while
using Arabic digits, as asserted by the selected ICU4X upstream test. Both raw
outputs are retained; no normalization or comparable timing uses that locale.
Node24.11.1 uses ICU77.1/CLDR47; selected ICU4X compiled data has its own epoch.
Profile78912 passed7.752s/318,880KiB,99rows/process,3×31after8warmups with separate
allocation builds and quiet guard. These are direct Rust prototypes, not public
API or Node throughput measurements; nothing was ingested as generated performance.

Profile supports reusable Collator (roughly5–7× construct+compare cost reduction),
owned grapheme byte ranges instead of substring copies, and an initial exact
normalized BigDecimal-to-fixed_decimal text bridge. The bridge costs about2.9–3.4×
the fixed-input formatting floor, with4/6/5allocations versus1 for the sampled
English/German/Hindi cases; do not hide the bridge cost or add an unmeasured cache.
All measured allocation rows retained0bytes. The2,144,904byte prototype executable
is a combined size, not incremental compiled-data overhead.

Expanded profile1470 passed9.315s/274,640KiB:43 correctness records (36 matching,
one explicit fallback difference,6 backend limit controls),99rows/process and
3×31samples after8warmups; quiet max2.145. Same-source optimized footprint controls
measured baseline609,800bytes; incremental locale167,168,collator1,296,448,
decimal87,456,grapheme17,344,all1,494,368bytes. These are reachable service code
plus compiled data, not pure data sizes or additive costs. All checksum guards pass.
Explicit fractionDigits requests fixed displayed
scale plus half-even rounding, not merely a maximum; absent preserves the normalized
value. Backend checked range is0..32768 (i16 magnitude0..-32768), not an arbitrary
28digit formatting cap derived from Decimal input precision. Large padding edges
and rejection without mutation passed. Public option/type signatures and generated
compiler/runtime/editor/export implementation remain pending.

### B14 — Small collection, numeric and date completion

Backend: existing Rust containers/numeric/date implementations. B00.

- [ ] OrderedSet `values`, `keys`, `entries`, `forEach`, `first`, `last`;
  OrderedMap `firstKey`, `lastKey`; Deque `values`; Heap `toSortedArray`.
- [ ] Keep snapshots independently owned and endpoint borrows scoped; specify
  heap output order and preserve existing total ordering/NaN/zero rules.
- [ ] `std:numeric.parseNumber` with whole-input Rust floating parsing semantics;
  optional radix2..36 on existing strict `parseInt`, default10.
- [ ] Result-returning `checkedAdd`, `checkedSubtract`, `checkedMultiply`,
  `checkedDivide`, `checkedRemainder` for int, without changing existing operators.
- [ ] Date `getDay`, `startOfYear`, `endOfYear`; Time `getHours`, `getMinutes`,
  `getSeconds`, `getNanoseconds`, following current date-fns-style conventions.

Witnesses: empty endpoints, owned values, order, exact int bounds, MIN/-1/zero,
NaN/infinity/negative zero, malformed/radix text, leap-century/year/time bounds.
Performance: endpoint/heap/deque kernels, strict parse/arithmetic and date access;
Node exact integers use BigInt. This is not JS Date mutability/coercion support.

### B15 — Owned lazy iterator pipelines

Backend: typed Rust iterators; dedicated compiler owner. B00/B14 and an explicit
ownership/type-lowering contract before other agents depend on this bundle.

- [ ] `Iterator<T>`, `IteratorResult<T>`, `Iterator.from` consuming an owned array.
- [ ] `next`, `map`, `filter`, `take`, `drop`, `flatMap`.
- [ ] `reduce`, `forEach`, `some`, `every`, `find`, `toArray`.
- [ ] Adaptors consume their input iterator and own captures; next transfers
  yielded values; terminal operations consume/short-circuit and clean up.
- [ ] Preserve eager Array/Map/Set keys/values/entries behavior. Do not quietly
  replace snapshot APIs or claim a public general trait/generator system.

Witnesses: non-Clone owners, partial consumption/early drop, callback order/errors,
capture lifetime, short circuit and zero surplus drops. Performance: direct Rust
map/filter/take/reduce pipelines, allocated bytes, early-exit work, compiler time
and emitted size. Avoid mandatory boxing/allocation per adaptor; preserve safe
generated Rust. Borrowed stored chains and arbitrary user protocols are deferred.

### B16 — Crypto and password primitives

Backend: existing sha2/getrandom/subtle; compatible RustCrypto hmac/hkdf/argon2.
B02/B06. Do not implement algorithms or application authentication frameworks.

- [ ] `sha384`, `sha512`, `createHasher` for sha256/sha384/sha512, `Hasher.update`
  and consuming `finalize`; no post-finalization reuse or hidden state clone.
- [ ] `hmacSha256`, constant-time `verifyHmacSha256`, `hkdfSha256`.
- [ ] Unbiased `randomInt(min, max)` with half-open integer bounds.
- [ ] `PasswordHashOptions`, `hashPassword`, `verifyPassword`, PHC encoding and
  Argon2id. Use backend default parameters/recommended salt generation, recorded
  explicitly after version selection; do not invent a new default cost profile.
- [ ] Explicit bounded PHC verification policy for untrusted encoded parameters;
  `PasswordVerifyOptions` and `verifyPasswordBounded` require caller-selected
  memory/iteration/parallelism limits checked before allocation/expensive work.
  Ordinary and bounded operations are clearly distinguished under B00.
- [ ] Memory-heavy password Tasks use existing bounded CPU work facilities;
  cancellation does not falsely promise immediate interruption of crate code.

Contract: readable byte views accepted, constant-time verification delegated to
the crate, no secret logging, no hidden password normalization or insecure RNG
fallback. Review key copies/backend zeroization without promising caller-owned
memory was wiped. HKDF's algorithmic output limit remains enforced.

Witnesses: SHA/HMAC/HKDF vectors, mismatch/truncation, random-bound failures,
PHC corruption/cost bounds and awaited cleanup. Performance: matched sizes and
exact password parameters, kernel versus scheduling costs separately. Security
does not follow from benchmark speed or passing a statistical randomness test.

### B17 — Structured logging and request context

Backend: tracing plus selected tracing-subscriber JSON formatting. B06/B10.

- [ ] `Logger`, levels, scalar structured fields, `debug/info/warn/error`,
  contextual child logger and explicit sink/setup.
- [ ] Opt-in access-log middleware: request ID, method, route template, status,
  elapsed time; no automatic body/credentials/query logging.
- [ ] Context follows async work without holding entered span guards over awaits;
  concurrent requests cannot inherit each other's fields.
- [ ] Disabled logging avoids message formatting/allocation. Preserve console.

Witnesses: JSON/escaping/filtering, nested/concurrent contexts, error redaction
and bounded sink cleanup. Performance: disabled events, formatting-only and
actual output separately; identical direct tracing configuration for Rust.

### B18 — Everyday testing facilities

Backend: current std:testing and native test runner. B06/B12 for task/regex support.

- [ ] `deepStrictEqual`, `notDeepStrictEqual` for supported structural data,
  following existing scalar equality semantics and ownership.
- [ ] Async `rejects` with checked failure payloads.
- [ ] CLI test-name filtering and machine-readable JSON reporter.
- [ ] Explicit async-test timeout with child drainage and teardown completion;
  blocking callbacks still require the process watchdog.

Witnesses: nested exact values, NaN/signed zero, mismatches/source ranges, rejected
tasks, hooks after timeout, filter and reporter oracles. Measure assertion-heavy
overhead only where meaningful; do not invent a whole-framework speed score.

### B19 — Practical Rust package adapters

Backend: extend existing adapter machinery and use pinned rustdoc JSON for
discovery, with the existing source parser as a defined fallback. B00/B02;
coordinate all shared compiler changes with B15's owner.

- [ ] Support `Option<T>`, `Vec<String>`, `Vec<OwnedRecord>` and nested supported
  owned containers in explicit adapters, preserving exact values and moves.
- [ ] Explicit CLI generation from selected concrete synchronous crate exports:
  adapter source, Tsrust declarations, docs/hover and a skipped-item report.
- [ ] Editable stubs for unsupported signatures, deterministic regeneration and
  preservation of customized adapters; no overwrite by ordinary builds.
- [ ] Pinned-toolchain rustdoc-JSON compatibility fixture and source-fallback
  tests. Explicit generation may prepare inputs; ordinary compilation stays
  locked/offline and does not install/download silently.
- [ ] At least two real package witnesses, including one retained compiled
  stdlib backend through its intentional compiler-owned adapter and one ordinary
  generated concrete-data adapter; test relocated offline export and editor help.

Contract: a bundled compiler-owned resource adapter is not proof that arbitrary
Drop-bearing crate types can cross the general bridge. Generic traits, methods,
async exports, returned borrows and general opaque-owner inference remain named
follow-ups. Generated code contains no unsafe; trusted backend internals remain
the backend's responsibility and are included in dependency review.

Performance: direct same-crate Rust calls versus generated calls, scalar and
owned-container round trips, pointer/capacity/drop witnesses, generation/build
cost separately from runtime. No redundant stringify/parse or field-copy bridge.

## Parallel execution and ownership

Owner-approved model: `gpt-6.1-sol` (Sol 6.1), up to three bundle agents plus the
integrator with the current four-agent capacity. Each agent owns a coherent
compiler/runtime/editor/examples/benchmark slice, not one method at a time.

| Wave | Agent A | Agent B | Agent C |
|---|---|---|---|
| 0 | B00 inventory/contracts + filesystem prototype | B00 codec/regex/crypto backend profiles | B00 async/network/bridge contracts |
| 1 | B01 files/directories/cursors | B02 bytes/codecs/binary | B04 URLs/queries |
| 2 | B03 path/env/OS/signals | B12 regex/text | B06 task/channel/timer extensions |
| 3 | B05 cross-resource I/O | B14 completion, then B15 iterator foundation | B16 crypto/password primitives |
| 4 | B07 subprocess | B08 networking/DNS | B09 compression |
| 5 | B10 HTTP/client/middleware | B13 explicit-locale subset | B19 Rust adapter/generation work |
| 6 | B11 realtime | B17 logging, then B18 testing | B20 examples, package/platform and coverage integration |

These are dependency waves, not mandatory idle barriers. A ready agent may take
another independent bundle after integration. HTTP/realtime share one owner;
bytes/codecs share one owner; futures/cancellation share one owner. Only one
agent owns shared type/ownership/iterator architecture changes at a time.

- [ ] Start agents from an accepted integration commit in isolated managed
  worktrees, one writer per tree. Reuse suitable trees; use the app's managed
  worktree tooling for new ones. Do not share uncommitted source or live tools.
- [ ] The integrator owns central registry/schema layout, manifests/locks,
  catalog/benchmark reconciliation, measurements.sqlite, canonical formatting
  and checkpoints. Agents may prepare additive namespace entries in their own
  trees; shared architecture changes need a recorded ownership lease.
- [ ] Each agent returns a reviewable commit, exact tests/untested scope, source
  identity, changed shared files, independent oracle and benchmark support. Send
  status to the integrator; no extra handoff files or source snapshots.
- [ ] Before each worker source checkpoint, grant a formatter/checkpoint slot:
  format the entire worker tree with the verified compiler, prove token and
  idempotence preservation, then commit its coherent slice. The integrator repeats
  required canonical formatting before integration checkpoints. Worker commits
  do not bypass the repository formatting requirement.
- [ ] Integrate small coherent bundles, resolve shared files once, then recheck
  affected combinations against the merged source. Branch-local results do not
  establish combined acceptance. Preserve completed compatible checks.
- [ ] Share one absolute Cargo target per physical host; jobs=1 and incremental=0.
  Serialize Cargo builds and native-compiling test process trees on this 8-GB Mac.
  Source work, review, pure analysis and lightweight independent tests can run
  concurrently within the aggregate memory budget. True parallel native/platform
  testing uses separate execution hosts/runners, each with its own single target.
- [ ] Use about5GB/30min watchdog bounds per native test tree, test-threads=3;
  monitor aggregate host pressure before concurrent lightweight tests. Kill only
  owned test children. Do not confuse three agents with three simultaneous rustc
  builds on this machine.
- [ ] Timing owns the machine exclusively: no builds/tests/profiling jobs in
  parallel, quiet-load guard enabled. A busy host means defer timing, not override
  the guard. Parallel implementation must not contaminate performance evidence.
- [ ] Only the integrator writes/ingests the measurement DB. Agents supply
  disposable run outputs for verified serial ingestion; never replace the DB
  with a worker copy or create per-worker history archives.

## Performance and acceptance from the first gate

Every B01–B19 bundle must complete this same checklist before it is accepted:

- [ ] Define representative small/medium/large and boundary workloads before
  implementation; write independent expected results and a direct handwritten
  Rust implementation using the selected backend, with the same ownership/work.
- [ ] Establish the direct Rust profile first: allocation/reallocation counts,
  bytes retained, CPU hot paths and compile/size effects. Prefer reusable owners,
  borrowed input and destination buffers where the source contract permits them.
- [ ] Supply matched TypeScript→Node cases. If Node lacks a built-in, identify
  the exact package/algorithm adapter; if no equivalent exists, mark N/A instead
  of inventing parity or timing a different operation.
- [ ] Complete compiler checking/emission, aliases/stored calls, native execution,
  error/loan/drop/cancellation controls, platform behavior, formatted-source
  execution and complete IntelliSense/examples. Invalid boundaries must fail
  correctly; backend package presence alone is not implementation acceptance.
- [ ] Keep workload sources/oracles/driver/ABI/registrations/fingerprints ready
  as the feature lands. Separate successful paths, rejection paths, one-time
  setup and reuse. Do not postpone benchmark implementation until release week.
- [ ] Measure the completed coherent bundle with original registered protocols
  and sample counts. For existing operations, profile and A/B the candidate
  against the previous accepted commit. For new operations, compare against the
  direct Rust floor; do not fabricate a pre-feature speedup.
- [ ] Review emitted code and allocation/scaling gaps against handwritten Rust.
  Investigate repeatable material regressions, including compile time and memory;
  keep an optimization only with repeatable gains and no correctness/ownership
  regression. Remove avoidable copies, wrapper work and repeated parsing now.
- [ ] Do not pad handwritten Rust with generated-code inefficiencies to claim
  parity. An optional adapter-matched diagnostic is labeled separately from the
  direct Rust headline reference. Preserve useful richer semantics explicitly.
- [ ] Measure Node/handwritten Rust references once per unchanged workload,
  backend/configuration/semantic identity. Cache them for later compiler-only
  changes. Revalidate correctness; retime references only for changed inputs,
  backend/toolchain semantics or a diagnosed environmental comparison problem.
- [ ] Add compile-time/scaling canaries and byte/allocation/drop assertions where
  they expose real defects. No clock-threshold assertions in ordinary unit tests.
- [ ] Run `npm run store:ingest` in `benchmarks/runtime`; verify history, original
  references and SQLite integrity before disposing raw outputs. Only actual
  measurements enter `benchmarks/measurements.sqlite`; no logs/source snapshots.
- [ ] Whole-project format, token preservation/idempotence, focused checks and
  documented-example execution pass before the source checkpoint. Commit measured
  numbers and update this checklist plus the single progress note.

Correctness, Rust semantics and safe ownership are release requirements. Faster
than Node on every workload is not. Handwritten-Rust parity remains the direction;
an unresolved avoidable adapter gap is work to investigate, not a marketing win.

## B20 — Feature-family and v0.2.0 release checklist

Accepted compiler inventory repair (2026-11-05): optional readonly finite-string
unions retain their proven string-only residual after an undefined guard. Native
length/index/string-method observations, aliases and repeated owner reuse pass;
missing guards, mixed residuals, mutation and loop invalidation remain rejected.
Fourteen focused compiler/native/editor witnesses pass, including existing tag,
borrow-allocation and disposal controls. This is distinct from literal-argument
context preservation and uses the existing borrowed-subset emitter.


- [ ] After the owned-I/O/async/iterator foundations form one accepted family,
  run the appropriate complete compiler/LSP milestone and combined ownership,
  cancellation, export and editor matrix. Preserve prior passing groups and repair
  only affected failures. Do not run full suites after each method or small gate.
- [ ] Finish B01–B19 and update every item with exact accepted evidence; identify
  all intentional differences and deferrals in public reference docs.
- [ ] Run final complete backend compiler/LSP binaries and both library suites
  under bounded resources, one binary at a time; doctests separately. Zero
  registered doctests are not executable example coverage.
- [ ] Execute native Windows/Linux/macOS acceptance, including file handles,
  signals, child processes, networking, TLS, exports and packaged CLI/LSP.
  Cross-target metadata or helper simulations do not prove native execution.
- [ ] Run the integrated applications: binary/file-transform CLI; persistent
  JSON API; streamed file/proxy upload; SSE/WebSocket service; bounded CPU work;
  generated Rust-package consumer. Include failures, disconnects and cleanup.
- [ ] Refresh affected public micro/HTTP metrics with cached unchanged reference
  histories. Keep Rust/Node comparisons, sample scopes, CPU/RSS/p99 and real
  function examples readable; no before/after or date-heavy presentation needed.
- [ ] Reconcile the existing language inventory and a concrete host export ledger
  against bindings/catalog/tests. Account for every excluded family below;
  never turn a module count into a conformance percentage.
- [ ] Update help, hover/signature/completion, reference, guides, migrations and
  examples; check links/rendered tables and installed consumers.
- [ ] Audit package notices/features, offline cache/export relocation, minimal
  dependency selection, install/update/rollback and all four platform archives.
- [ ] Check live PRs/Dependabot before every push. Batch remote validation by
  meaningful feature families; call CI full only with `full_regressions=true`.
- [ ] Prepare a concrete version and source tree, obtain explicit v0.2.0
  publication approval, then use staged promotion and verify actual main/tag/
  assets/docs deployment. This planning request is not release authorization.
- [ ] Prune closed owned build/probe outputs, preserving shared caches and current
  consumers; record remaining work in docs/agent-progress.md only.

## Explicit family disposition and later scope

This is the exclusion checklist, not forgotten v0.2 implementation. Reopen a row
with a concrete application need and its own acceptance scope.

| Node/Rust family | v0.2 treatment and deliberate boundary |
|---|---|
| Buffer/string decoding/files/path/env/OS/process | B01–B07. Legacy encoding aliases, watchers/glob, mmap, ACL/chown/xattrs, recursive copy, PTY and process-tree supervision remain later. |
| Streams/timers/workers | B05/B06/B15 extend existing foundations. Cancellable interactive/async stdin, JS callback/object-mode stream graphs, generator syntax, arbitrary iterator protocols, async iteration/select, broadcast/watch and generic synchronization APIs remain later. |
| HTTP/HTTPS/HTTP2/TLS/net/UDP/DNS | B08/B10/B11. Low-level HTTP2 sessions, raw TLS sockets, custom DNS record queries/reverse/DNSSEC, multicast, Unix sockets/named pipes and reconnect protocols remain later. |
| URL/query/zlib | B04/B09. Legacy querystring quirks, Brotli/zstd/archive extraction/dictionaries remain later. |
| Crypto/WebCrypto | B16 exact listed primitives. General WebCrypto compatibility, encryption/key exchange, PEM/X509/RSA/ECDSA/JWT/OAuth/session frameworks and legacy security hashes remain later. Application auth policy stays user-owned. |
| Intl/text/RegExp | B12/B13. No JS lone-surrogate/stateful regex emulation; full ECMA Intl, non-Gregorian calendars, currency/plural/list/relative-time families and legacy locale APIs remain later. |
| Console/events/errors/utilities/async context/diagnostics/performance/trace/report | Existing console/typed errors plus B17/B18. General EventEmitter/EventTarget, callbackify/promisify, inspector/trace protocols, OpenTelemetry exporters, metrics registries, process reports and background log rotation remain later. |
| SQLite/SQL/cache/jobs | Preserve existing drivers/pools/cursors/migrations/LRU/TTL. ORM/query builder, Redis/distributed cache, durable broker/cron, cache single-flight and driver-specific advanced types remain later. They are ecosystem additions, not missing Node/Rust std methods. |
| Additional config/serialization formats | Existing JSON/JSONC/schema support remains. YAML/TOML/CSV, XML parsing and generalized serializer plugins are later ecosystem adapters, rather than unlisted requirements for complete Node/Rust std parity. B19 keeps explicit package adapters usable. |
| Tests/readline/TTY/REPL | B18 plus existing runner. Fake clocks, mocks/snapshots/coverage, unrestricted test parallelism/watch, line editor and REPL remain later. |
| Rust adapters/modules/packages/TS loading/SEA | B19 plus existing offline build/install/export. General methods/generics/traits/async/borrowed/Drop-bearing foreign types need separate bridge contracts. Arbitrary JS/npm/CommonJS loaders are not supplied by native imports. |
| Typed arrays/BigInt/weak references/Symbol/prototypes/reflection | Existing owned bytes/views and exact numeric types remain. Full JS object identity, shared/detachable buffers, full-u64/BigInt scalar work and dynamic reflection are separate compiler scope. |
| Rust alloc/ptr/mem/marker/unsafe internals | Use compiler-owned safe representations. Do not expose raw-pointer/unchecked APIs or generate unsafe to reproduce Rust implementation internals. |
| V8/VM/Node addons/C++ embedding/WASI/permissions/cluster/debugger | No native parity claim. Interpreter/sandbox/FFI/target capabilities, multiprocess supervisors and source-level native debugging need separate product/ownership decisions. |
| Domain/punycode/deprecated Node APIs | URL backend handles its declared IDNA behavior; no deprecated-runtime compatibility campaign. |
| Newer Node FFI/VFS/iterable-stream/benchmark APIs beyond pinned index | Track separately when choosing a new baseline, not as silently added v0.2 obligations. |

## Package/source references

Official documentation reviewed for planning, 2026-11-04. These are backend
choices/candidates, not claims of admitted compatible versions or new pins.

- [Node 24.11.1 API index](https://nodejs.org/download/release/v24.11.1/docs/api/)
  and [Rust std](https://doc.rust-lang.org/std/): different scopes; check Rust APIs
  against pinned1.96.1 rather than assuming latest online docs are that version.
- [std files](https://doc.rust-lang.org/std/fs/),
  [Tokio features](https://docs.rs/tokio/1.53.3/tokio/),
  [Tokio process](https://docs.rs/tokio/latest/tokio/process/),
  [Tokio net](https://docs.rs/tokio/latest/tokio/net/),
  [environment mutation safety](https://doc.rust-lang.org/std/env/fn.set_var.html).
- [url](https://docs.rs/url/latest/url/),
  [base64](https://docs.rs/base64/latest/base64/),
  [percent-encoding](https://docs.rs/percent-encoding/latest/percent_encoding/),
  [regex](https://docs.rs/regex/latest/regex/),
  [hostname](https://docs.rs/hostname/latest/hostname/).
- [tempfile Builder](https://docs.rs/tempfile/latest/tempfile/struct.Builder.html)
  and [TempDir](https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html).
- [flate2](https://docs.rs/flate2/latest/flate2/),
  [ICU4X/components](https://docs.rs/icu/latest/icu/),
  [CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html).
- [Axum SSE](https://docs.rs/axum/0.8.9/axum/response/sse/),
  [Axum WebSockets](https://docs.rs/axum/0.8.9/axum/extract/ws/),
  [Tokio Tungstenite](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/).
- [sha2](https://docs.rs/sha2/latest/sha2/),
  [hmac](https://docs.rs/hmac/latest/hmac/),
  [hkdf](https://docs.rs/hkdf/latest/hkdf/),
  [Argon2](https://docs.rs/argon2/latest/argon2/),
  [pinned Node crypto](https://nodejs.org/download/release/v24.11.1/docs/api/crypto.html).
- [tracing](https://docs.rs/tracing/latest/tracing/),
  [tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/),
  [rustdoc JSON](https://doc.rust-lang.org/nightly/rustdoc/unstable-features.html#json-output).

No new API, package version, platform result or performance number is accepted
by writing this plan. Implementation, measurement and release evidence remain
separate checklist states.
