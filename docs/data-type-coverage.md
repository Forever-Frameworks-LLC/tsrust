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

## Required domain and boundary gaps

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
