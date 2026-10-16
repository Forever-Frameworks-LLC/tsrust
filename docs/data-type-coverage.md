# Practical data-type coverage

This inventory starts from the implemented compiler declarations, native bindings,
editor catalog and accepted fixtures. It is discovery, not new acceptance. Resource
acceptance is complete; implementation of the gaps below follows it.
S18 driver evidence is retained. New SQL conversions will require the explicit
three-driver runner.

## Existing language and container support

| Family | Source representation and behavior | Ownership / boundary status | Existing witnesses |
| --- | --- | --- | --- |
| Boolean | `boolean`; two values, no implicit truthy object conversion at typed boundaries | Copies; Rust `bool`, SQL boolean supported within driver contracts | `tests/numbers.rs`, `tests/rust_bridge.rs`, `tests/async_sql.rs` |
| Signed integers | `int`; signed 64-bit native storage | Copies; Rust `i64`; checked SQL SMALLINT/INTEGER parameter narrowing exists | `tests/numeric_literals.rs`, `tests/fixed_number_storage.rs`, S18 acceptance |
| Floating point | `number`; IEEE 754 binary64, NaN/infinities/signed zero; documented formatting and predicates | Copies; Rust `f64`; SQL rejects nonfinite values. SQL REAL currently checks overflow but can round during f32 narrowing: required gap | `tests/number_predicates.rs`, `tests/number_formatting.rs`, `stdlib/sql/runtime.rs` |
| Other numeric widths | No distinct source i8/i16/i32/u8/u16/u32/u64/i128/u128/f32 types | External Rust bridge currently recognizes only i64/f64. Narrow and unsigned adapters require checked conversion. Full u64/i128/u128 values cannot be represented by int | `project/rust.rs::shape`; new positive/negative boundary witnesses required |
| Strings / Unicode | Owned UTF-8 `string`; documented byte-oriented operations and Unicode families | ReadOnly borrows, owned moves, explicit clone; Rust String/&str and SQL text | Existing string families and `tests/rust_bridge.rs` |
| Binary data / views | `Bytes`, immutable/mutable borrowed views; checked allocation/index/conversion | Owned buffers and checked loans; SQL blobs supported. External Vec<u8>/byte-slice adapter is absent | `tests/native_byte_view_runtime.rs`, S07 resources/bytes, S18 |
| Null and optional | `null`, `undefined`, unions, optional fields; Result for fallible operations | SQL null parameter and `isNull` supported. External Rust Option mapping is absent | `tests/nullable_unions.rs`, `tests/result_match.rs`, S18 |
| Records and arrays | Named/structural records, generic records, owned arrays, borrowed views; recursive finite values within documented guards | Checked move/borrow/mutation; external public plain structs/enums and limited primitive vectors, not arbitrary Rust layouts | `tests/data_unions.rs`, `tests/generic_value_adapters.rs`, `tests/rust_bridge.rs` |
| Tuples | Heterogeneous fixed-length tuples | Checked ownership of contents. External Rust tuple mapping beyond unit is absent | `tests/tuple_types.rs` |
| Collections | Map/Set and accepted collection operations | Owned keys/values, checked returned views, explicit cloning; no blanket external HashMap/BTreeMap mapping | S08 acceptance; `tests/map_iteration.rs`, `tests/set_algebra.rs` |
| Variants/results | Data/discriminated unions and `Result<T,E>` | Payload ownership checked; restricted Rust concrete enum and output Result adapters | `tests/data_unions.rs`, `tests/result_match.rs`, `tests/rust_bridge.rs` |
| JSON/schema | JsonValue, typed parsing, checked schema validation, raw borrowed views, JSONC/JSONL | Owned typed results or explicit raw loans. Signed/unsigned 64-bit integer tokens retained; extraction to int/number is checked. Decimal/exponent tokens use floating point | S10/S11; `tests/json_typed.rs`, `tests/json_scalar_backend.rs`, `tests/schema_decode.rs` |
| Callbacks | Stored callables, checked captured values and access contracts | Ownership is retained; no universal callable clone or Rust closure/trait-object mapping | Existing callback suites; `tests/tracked_resources.rs` |
| Resources/tasks/jobs | Opaque native resources and explicit asynchronous/worker lifecycle | Clone/transport depends on native metadata. `TrackedResource` is thread-affine; local async allowed, workers/started tasks rejected. Projected external Drop remains rejected | S07/S12–S18; `tests/tracked_resources.rs` |

Paths under `tests` refer to `crates/tsrust-compiler/tests`; native paths refer to
`crates/tsrust-compiler/src`. Editor counterparts live in `builtins.json` and the
completion/agreement/consistency suites. The table reuses existing acceptance; it
does not claim every listed suite was rerun during this inventory.

## Initial domain and boundary gaps

| Need | Existing coverage | Required practical completion |
| --- | --- | --- |
| Checked numeric conversions | Checked SQL integer narrowing and JSON extraction; ordinary numeric operations retain their documented semantics | Explicit exact int/number and width conversions; reject range/fraction/precision loss. Fix SQL f32 narrowing. Define full-width unsigned/128-bit alternatives honestly |
| Exact decimals | No dedicated decimal value or arithmetic; strings alone have no validated decimal semantics | Maintained exact-decimal backend with explicit precision/scale bounds, checked parsing/arithmetic/conversion and readable money example. No silent rounding or implicit SQL cast |
| Calendar dates | No calendar type; monotonicNow is elapsed time only | Validated calendar date, invalid-date rejection, parse/format/comparison and date arithmetic with clear bounds |
| Time of day / UTC instant | No dedicated time/instant value | Time-of-day and UTC timestamp contracts, fractional precision and checked serialization/conversion |
| Offset / time zone | No zone model | Fixed offsets and pinned IANA-zone backend; distinguish zone from offset, reject or explicitly resolve DST gaps/ambiguity |
| Durations | Existing task delays accept integer durations; monotonic cache core uses Rust Duration internally | Explicit units, range/overflow, duration versus calendar-period distinction; checked arithmetic |
| UUID | No UUID value or validated conversion | Maintained UUID parser/formatter and byte conversion; generation scope tied to actual service needs |
| Domain SQL mappings | Any-driver adapter currently handles scalar numbers, boolean, text, bytes and null | Explicit per-driver decimal/date/time/UUID/JSON mapping witnesses. Until added, typed retrieval is unsupported; explicit SQL text serialization must be named and validated, never presented as native typed support |
| JSON precision documentation | Parser guards exact integral tokens; some hover prose still says larger integer tokens may round before a later paragraph says they reject | Reconcile prose with existing integer-range behavior and retained tests; no weakening of parser limits |

Arbitrary Rust crates, traits, lifetime-heavy APIs, arbitrary precision arithmetic,
unlimited nesting, ORM behavior and universal database type coverage are not goals.
An unsupported case needs a concrete use case and an explicit safe representation
or rejection. An unchecked cast, hidden clone or string proxy does not close a row.

## Comparison tooling and pins

The local TypeScript package is 5.9.3 at
`benchmarks/runtime/node_modules/typescript`. The installed VS Code Rust analyzer
binary is `rust-analyzer 0.3.3081-standalone (2ad81e72b7 2026-10-11)`.
The rustup toolchain has no rust-analyzer component; the installed standalone binary
is available for bounded comparison runs. The workspace extension remains disabled.

Representative completion/hover/signature discovery now runs with:

```sh
node tools/data-type-inventory.mjs
CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target node tools/rust-type-inventory.mjs
CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p tsrust-compiler --example public_api_inventory
node tools/compare-type-inventories.mjs TYPESCRIPT.json RUST.json TSRUST.json
```

TypeScript discovery covers 15 families, 231 members and 17 constructor/call
overloads. Rust discovery covers 20 families, 2,230 methods and four call signatures;
392 postfix/editor actions are retained separately and are not counted as methods.
All Rust hovers and signatures were populated. The standalone process shuts down
and its temporary project is removed. Rust pins the host rustc/sysroot and standard
library, with Cargo checks, build scripts and proc macros disabled. Selected domain
crate inventories will be added with their implementations.

Tsrust discovery records 815 catalog/binding symbols and 69 native declarations.
The discovery example was repaired to accept comma-separated enum variants and
resolve source aliases through the compiler lookup. The private, non-importable
TestRegistry is explicitly identified without inventing public declaration metadata.
The original inventory failure is preserved. Tsrust checker/editor agreement covers
its receiver contracts; name matches alone do not establish runtime equivalence.

The initial comparison classifies each reference member. A matching catalog name is
`partial`, requiring overload/behavior/ownership evidence. Required absent domain or
width types are `missing`; separate Task/Job and optional-union semantics are
`intentionally different`; the remaining broad reference API surface is `deferred`
unless a concrete UI/service need requires it. These classifications do not claim
that 2,000 Rust methods must be implemented or that every name match is supported.
Per-type native witnesses and the required-gap table remain the acceptance criteria.

Exact raw discovery and comparison records use `tsrust-data-types-*-reference.*`
and `tsrust-data-types-comparison-final.json` in the canonical SQLite store after import.
No runtime or performance parity is inferred from these declaration records.

## Checked numeric conversion section (accepted 2026-10-15)

`std:numeric` adds `toInt`, `toNumber`, `toFloat32`, `checkSigned`,
`checkUnsigned` and strict decimal `parseInt`. All return `Result` with
`NumericError`; no operation rounds to claim success. See
`examples/numeric-boundaries/checked.tsr`. Width checks keep `int` storage;
full unsigned 64-bit and 128-bit values remain unsupported. Existing implicit
numeric expressions retain their language semantics: converting an already
rounded `number` cannot recover its original integer. Use the exact helpers at
boundaries before any widening or arithmetic that may round.

The matched conversion workload is registered in `benchmarks/numeric` and remains
unmeasured. SQL REAL narrowing passes explicit three-driver acceptance.

Acceptance: native fixture and seven executable catalog examples pass; backend
edge-case tests cover syntax, fraction, range, precision, negative zero and width
errors. NumericError checker agreement, all four completion delivery modes,
module import discovery, hover and signature help pass. The initial completion
fixture incorrectly used unsupported namespace imports; the repaired named-import
fixture passes. Original failure evidence is retained. JSON hover prose now agrees
with the existing integer-range rejection contract; parser behavior is unchanged.
The native/Node untimed workload matches its independent checksum and mutation
guards pass. Closed evidence is stored as `tsrust-numeric-*` in SQLite.

## SQL floating boundaries

Binding to driver-declared REAL now rejects finite binary64 values that would
round or underflow in binary32 with `SqlError.code == "precision"`. Overflow and
nonfinite inputs report `range`. Exactly representable values and signed zero
retain their value. The check applies before both pool and transaction execution.

Driver metadata does not universally describe destination column types. Database
implicit casts and SQL expressions remain database behavior; the adapter cannot
promise lossless storage for arbitrary SQL. Validate a binary32 boundary with
`unwrap(toFloat32(value))` before binding when the driver does not expose it.
SQLite REAL is binary64. Decimal/date/time domain mappings remain a separate gap.

A failed transaction operation retains the established automatic-rollback contract:
a precision error makes the transaction inactive. A subsequent commit or rollback
reports `transaction`; scope cleanup still releases it. Acceptance explicitly
checks this behavior instead of changing lifecycle semantics for conversion errors.
The three-driver runner can resume after retained passing groups with
`--resume-from=postgres` or `--resume-from=mysql`; this skips the already-passed
backend group and preceding generated-driver groups, and still runs the examples.

SQL acceptance (2026-10-15): three focused backend tests pass; the explicit
three-driver backend group passes; generated SQLite passes before the fixture
repair, then resumed PostgreSQL/MySQL and all three examples pass. Default SQL
checks pass three tests while database integration remains ignored. Focused editor
parameter help passes. Original failure and resumed evidence are retained as
`tsrust-sql-precision-*`. The first failure was a test attempting a second rollback
after automatic rollback; runtime lifecycle behavior was preserved. The runner
removed its containers, and Docker was stopped after confirming none remained.

## Exact decimals

`std:decimal` uses pinned [BigDecimal 0.4.11](https://docs.rs/bigdecimal/0.4.11/bigdecimal/)
for validated opaque values, with at most 38 significant digits, 38 integer digits
and 28 fractional places after normalization. `parseDecimal` accepts bounded plain decimal
text; exponent syntax and display-scale preservation are intentionally unsupported.
`formatFixed` pads exactly or returns Precision. Division verifies the inverse
before accepting a quotient; nonterminating division never silently rounds.
`toNumber` compares the exact binary64 value against the original decimal.

Examples: `examples/decimal/money.tsr`. JSON/SQL require explicit validated text
serialization; direct native domain mappings remain unsupported. Values are owned,
explicitly clonable and transferable, and expose no writable coefficient fields.
The matched [Decimal.js](https://mikemcl.github.io/decimal.js/) workload lives in
`benchmarks/decimal`; its correctness and timing status are recorded separately.

Focused decimal acceptance on macOS arm64 covers exact arithmetic and conversion
failures, explicit cloning, records/arrays/nullability, worker transfer, rejected
forging/implicit JSON, locked offline export and all 13 catalog examples. Public
parsing is named `parseDecimal`, consistently with `std:numeric.parseInt`; the old
`parse` name is not exported. Decimal and DecimalError checker agreement, decimal
completion rows, hover and signature help pass. The matched native/TypeScript
workload produces the independent checksum 65,247,616; timing remains unmeasured.
Whole-project formatting preserves tokens and a second pass changes zero files.
Evidence is retained under `tsrust-decimal-*` in the canonical SQLite store.

## Calendar dates

The owner prefers date-fns. `std:date` uses named immutable functions with familiar
`addDays`, `addMonths`, `differenceInCalendarDays`, `compareAsc`, `format` and
accessor names. `Date` is a validated Gregorian calendar date, separate from an
instant or time zone. `parseDate` requires YYYY-MM-DD within years 0001..9999;
`formatDate` emits that canonical form. There is no invalid-date sentinel.

`addMonths` clamps to the target month end, matching the
[date-fns 4.1.0 rule](https://github.com/date-fns/date-fns/blob/v4.1.0/src/addMonths/index.ts).
`getMonth` is zero-based, matching date-fns. Negative arithmetic counts subtract;
overflow and out-of-range results return errors. `format` supports documented
numeric date and English month/weekday tokens plus quoted literals. Unsupported
tokens return Format; locale, time, zone and arbitrary parsing patterns are not
claimed. No full date-fns compatibility is implied.

The maintained backend is [Chrono 0.4.45](https://docs.rs/chrono/0.4.45/chrono/),
with default features disabled and only std enabled. Calendar operations do not
consult the system zone or clock. Time of day, UTC instants, durations and IANA
zones remain the next section. Rust/SQL domain mappings remain separate work.
The readable billing example and matched date-fns workload live in
`examples/calendar-dates` and `benchmarks/calendar`; timing is unmeasured.

Calendar compiler/native acceptance on macOS covers the billing example, explicit
clone/record/array/nullability/worker ownership, rejected forged/implicit JSON
values, 15 executable documentation examples and locked offline export. The first
fixture used unsupported inline record syntax and an incorrect spawn form; its
original failure is retained and the repaired fixture passes. Date/DateError
agreement, three completion checks and focused hover/signature help pass. The
native and date-fns workload both produce the independent checksum 40,064.
Timing remains unmeasured. Evidence uses `tsrust-calendar-*` in SQLite.

## Instants, clock times, durations and zones

`std:date` adds separate Time, Instant, Duration and TimeZone values. ISO instants
require an explicit offset and preserve nanoseconds; calendar Date remains a
zone-free date. Time rejects leap seconds and 24:00. Instant UTC years are
0001..9999. Epoch conversion rejects fractional loss. Durations store signed
64-bit nanoseconds (about ±292 years); overflow and inexact millisecond extraction
return errors. Calendar months/days remain distinct from elapsed durations.

Named zones use pinned Chrono-TZ 0.10.4 and its IANA 2025b database. Newer zone-rule
changes are not included until an explicit backend/database upgrade is accepted.
Fixed offsets use signed seconds east of
UTC, strictly within ±86400. `resolveDateTime` requires reject/earlier/later;
repeated local times are explicit and gaps reject under every policy. No process
local zone or silent daylight-saving adjustment is inferred. Wall-clock `now`
can jump and is separate from monotonic elapsed time. Unknown ISO -00:00 offsets,
leap seconds, free-form date parsing and locale formatting remain unsupported.

The appointment example covers a New York fold, gap and nanosecond conversion
rejection. Native witnesses also cover Lord Howe's half-hour fold and Apia's
skipped date. The date-fns workload in `benchmarks/temporal` uses millisecond inputs
only; it does not establish nanosecond or DST-policy parity. Measurements remain
pending the data-type family milestone. Rust/SQL domain mappings remain separate.

Focused temporal acceptance passes on macOS: four compiler/export checks,
31 batched native documentation examples, two calendar/temporal runtime checks,
four receiver-agreement selections, completion and hover/signature checks. The
prior calendar billing example still passes. Native and date-fns millisecond
workloads match checksum 32,640; timing remains unmeasured. Installed CLI/LSP and
the active VS Code workspace pass after refresh. Evidence uses `tsrust-temporal-*`
in SQLite; IANA provenance remains developer evidence, separate from API help.

## UUIDs

`std:uuid` provides opaque exact 128-bit Uuid values, strict `parseUuid`, canonical
`formatUuid`, independent 16-byte conversions, byte-order comparison, raw version
inspection and fallible version-4 generation. All bit patterns can be parsed;
parsing does not assert a recognized UUID version or variant. Standard network
byte order is explicit. Implicit JSON/SQL conversion and UUID collection keys are
unsupported; use checked text/bytes at those boundaries.

The backend pins uuid 1.28.0 and existing getrandom 0.4.3. OS entropy failure
returns Entropy without producing an identifier; v4 collisions remain possible.
The readable identifier example covers byte mutation isolation, nullable records,
arrays, cloning and worker transport. The Node reference uses strict hexadecimal
validation and built-in Buffer operations; random generation is outside the
matched round-trip workload. Timing remains unmeasured.

Focused UUID acceptance passes on macOS: four compiler/export checks, nine native
documentation examples, runtime entropy/byte boundaries, Uuid/UuidError editor
agreement, completion and help. The native and Node workloads match checksum
33,664. Installed CLI/LSP and restarted VS Code pass. Evidence uses `tsrust-uuid-*`
in SQLite. Timing remains unmeasured; Rust/SQL adapters remain separate work.

## Explicit Rust boundary adapters

`examples/rust-data-boundaries` passes native source/compiler and editor acceptance.
It demonstrates checked i8/i16/i32 and u8/u16/u32 conversions in widened storage,
exact f32 conversion, checked real Rust u64/u128 arithmetic using validated decimal
text, explicit Vec<u8> copies, concrete Option enum mapping and owned tuple/record
transport. Direct unsupported widths, Vec<u8>, Option and tuple signatures reject.
Full i128 and direct opaque domain projection remain unsupported. Text-backed u128
is not a source numeric type and does not fit every value into bounded Decimal.
See the example README for ownership, errors and limitations. Existing Rust bridge
performance protocols remain unchanged; no copied-buffer performance claim is made.
Evidence uses `tsrust-rust-boundaries-*` in SQLite. No compiler runtime changed.

## Selected domain reference inventory

`node tools/rust-type-inventory.mjs --domains` probes pinned BigDecimal 0.4.11,
Chrono 0.4.45 and UUID 1.28.0 with the same bounded standalone analyzer. Seven
receiver types expose 269 method declarations and three call signatures.
`node tools/domain-reference-inventory.mjs` reads pinned TypeScript declarations:
525 date-fns 4.1.0 exports and 106 Decimal.js 10.6.0 export/instance declarations.
These counts include types and aliases; they are not distinct implemented functions.
Named counterparts remain partial because precision, errors, overloads and ownership
differ. Other members are deferred unless a concrete application needs them.
Generated chrono-tz declarations are excluded from the analyzer inventory because
build scripts remain disabled; its public zone operations and pinned IANA data
are covered by the accepted temporal native witnesses. No complete parity is claimed.
The original and classified records use `tsrust-data-types-domains-*` in SQLite.

## Explicit SQL domain mappings

`examples/sql-domain-values` supplies checked application encodings for the shared
PostgreSQL/MySQL date range, exact DECIMAL(38,18), microsecond time/UTC timestamps,
UUID text/bytes, named zones, integer nanosecond durations, JSON and SQL null.
SQLite uses exact TEXT/BLOB representations. Typed driver columns use explicit SQL
casts at the scalar Any-driver boundary; direct opaque binding/decoding remains
unsupported. See its README for precision/range rejection and unsupported cases.
Existing S18 lifecycle acceptance remains intact; new mapping acceptance invokes the
explicit three-driver runner. No SQL runtime or task cleanup rules changed.

Focused SQL-domain acceptance passes on macOS: all three source variants check
without database access; SQLite, PostgreSQL and MySQL execute exact domain
round-trips and early rejection witnesses. The existing S18 backend and generated
transaction/cancellation/cleanup suite also passes through the explicit runner.
Editor helper signatures agree with the checker. Initial fixture failures (an
owned-string alias, unused catch payload contract and consumed parameter strings)
are retained with repaired checks; no compiler semantics were relaxed. Evidence
uses `tsrust-sql-domains-*` in SQLite. Containers are removed and Docker stopped.

The bounded data-type implementation section and family regressions are complete.
Numeric/decimal/calendar/temporal/UUID measurements remain the next milestone step
before S19. Unsupported cases remain listed above; none are accepted through
implicit rounding, cloning or native domain projection.

## Data-family regression acceptance

The original consolidated pass recorded 4,151 compiler passes and 721 editor
passes. Its 24 compiler and three editor failures are resolved by targeted reruns,
without restarting passed groups. Original failures and repaired evidence remain
under `tsrust-data-family-*` and `tsrust-data-repair-*` in SQLite.

Repairs explicitly select the HTTP TLS provider when SQL enables another provider,
keep empty record-rest results distinct from opaque native resources, reject
native-resource rest projection, and protect bundled dependency pins during
package management. Nine HTTP runtime checks, ten HTTP source checks, the combined
SQL/HTTPS witness, seven package-management checks, 24 record-rest checks and both
HTTP/HTTPS relocated offline debug/release packages pass. Existing PostgreSQL,
MySQL and SQLite domain/lifecycle acceptance is reused; no SQL runtime or task
cleanup rule changed.

The catalog now checks SQL families and documents SQL/SQLite type/error examples
and the resource observer consistently. Corrected catalog witnesses, resumed native
examples, public export checks and the new record-rest editor witness pass. Identical
complete catalog programs execute once per source/output pair; interrupted runs can
resume standalone programs while retaining the final fragment batch. SQL execution
remains opt-in. Calendar and decimal documentation examples retain all 28 original
assertions while compiling as two programs. New error/resource examples also pass
formatted native execution, and installed-server hover exposes their documentation.

Both doctest drivers compile successfully (neither currently contains runnable
doctests). Nested offline builds can replace shared-target dependency artifacts;
a separate Cargo doctest invocation refreshes those dependencies. Installed CLI and
both LSP paths are hash-verified; the restarted editor reports no problems. Timing
acceptance remains separate from these correctness results.
