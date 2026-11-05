# Standard library v0.2.0 implementation checklist

Owner direction, 2026-11-04: plan the missing useful standard-library features as
one explicit v0.2.0 checklist; bundle related implementation and acceptance work;
use Sol 6.1 subagents in parallel; design and measure performance from the first
gate. Popular, maintained, well-tested Rust packages are approved backends.

Status: proposed release contract and execution plan, not implemented acceptance
or permission to publish v0.2.0. The current v0.1.2 promotion must finish first.
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

- [ ] Add narrow native signature literal/exported-alias support where required
  for `SeekOrigin` and similar finite contracts. The source language already has
  literals; the native registry currently does not. Do not invent opaque owners
  or silently widen a promised literal union to arbitrary string.
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

- [ ] `std:bytes.toHex/fromHex`, `toBase64/fromBase64`,
  `toBase64Url/fromBase64Url`, explicit `toUtf8Lossy`, `fromUtf16/toUtf16`.
- [ ] `concat`, `copyInto`, in-place `copyWithin`, `equals`, `compare`, `indexOf`.
- [ ] `readInt/writeInt` for widths 1/2/4/8; `readUint/writeUint` for widths
  1/2/4; `readFloat32/writeFloat32`, `readFloat64/writeFloat64`.
- [ ] Explicit offsets and byte order; reject unsupported widths/unrepresentable
  integer values before mutation. Float32 writes use Rust f64→f32 rounding,
  including nonfinite values, rather than the existing exact-conversion helper.
  Full-u64 values await a separately accepted exact type.

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

- [ ] `PathParts`; `std:path.resolve`, `parse`, `format`, `separator`, `delimiter`.
- [ ] `std:env.vars`, `currentDir`, `currentExe`, `tempDir`, `homeDir`.
- [ ] `std:os.availableParallelism`, `hostname`.
- [ ] Windows console interrupt/control-break support through existing signals.

Contract: preserve existing POSIX/Windows lexical path rules; normalization is
not filesystem containment. Resolve can fail on cwd. Environment absence, empty
values and non-Unicode failures remain distinct. No process-global env setters,
env removal or cwd mutation in the multithreaded runtime; B07 supplies per-child
configuration. Parallelism is an estimate. Unsupported signal events error.

Witnesses: drives/UNC/roots/dotfiles, controlled child cwd/environment, optional
home and real Windows interrupt handling. Performance: path corpus and environment
reads; do not fabricate meaningful speedups for constant target identity.

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

### B13 — A useful explicit-locale subset

Backend: selected ICU4X components, not automatic inclusion of the meta-crate.
B00/B02; measure locale-data footprint before API acceptance.

- [ ] `Locale` parsing/canonicalization with explicit locale input.
- [ ] Reusable `Collator.compare`, and `String.localeCompare` using that contract.
- [ ] `Segmenter.segment` for grapheme boundaries with UTF-8 byte ranges.
- [ ] Reusable `NumberFormat.format` for exact int/Decimal inputs; document
  supported rounding/grouping/options and reject unsupported options.
- [ ] Pin selected compiled data and document backend locale fallback/coverage.
  Do not label this full ECMAScript Intl or infer host-user locale silently.

Witnesses: combining text, emoji clusters, collation fixtures, negative/large exact
decimals, unsupported options and data fallback. Performance: construct once versus
format/compare repeatedly, allocation counts, binary size and cold/build memory.
Node ICU and Rust ICU versions/locales are recorded; differing data is not a
compiler defect or permission to combine incomparable timing rows.

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
