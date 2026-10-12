# JSON symbol placement review

Status: the twelve current `std:json` exports have been reviewed against compiled
binding metadata, native type declarations, bridge emission and editor catalog.
This review does not close the full globals/imports audit or implement JSON globals.

## Per-symbol decisions

All current names remain supported imports with ordinary local aliases. Imports
make the specialized format and resource lifecycle visible. The approved global
namespace adds convenience only after the remaining cross-library audit; no
existing Result contract changes to throwing behavior.

| Symbol | Purpose, inputs and result | Failure and ownership | Runtime support | Disposition |
|---|---|---|---|---|
| `parse` | `(text: string, limits?: JsonLimits): Result<JsonValue, JsonError>`; strict JSON decoding | Borrows text/limits; returns independent owned value or structured error. Syntax and limit failures are Err | JSON parser | Retain import; preferred global checked spelling will be `JSON.tryParse`; throwing `JSON.parse` is a separate entry point |
| `parseJsonc` | `(text: string, allowTrailingComma?: boolean, limits?: JsonLimits): Result<JsonValue, JsonError>`; comments accepted, trailing commas default false | Borrows text/limits, copies flag; original UTF-8 offsets and structured Err, including unterminated comments | JSON parser and comment handling | Keep imported specialized format; do not silently relax strict parse |
| `stringify` | `(value: JsonValue, maxBytes?: int): Result<string, JsonError>`; compact encoding, default 16777216 output bytes | Borrows value, copies limit; owned output or Err for invalid/output limit | JSON serializer | Retain import; approved `JSON.tryStringify` preserves Result; `JSON.stringify` will throw |
| `kind` | `(value: JsonValue): string`; null, boolean, number, string, array or object tag | Borrows value; static text result; no recoverable error | JSON value inspection | Keep compatible; prefer the existing typeof keyword for ordinary categories. Its six-way JSON tags remain distinct from typeof. |
| `openJsonl` | `(path: string, limits?: JsonLimits, capacity?: int): Result<JsonlReader, JsonlError>`; capacity default 8192, valid 1–1048576 bytes | Borrows path/limits; validates configuration before opening; owned reader or structured I/O/configuration error | JSON and buffered filesystem | Keep import; resource creation must remain explicit |
| `readJsonl` | `(reader: Mutable<JsonlReader>): Result<JsonValue \| undefined, JsonlError>`; one strict JSON record, undefined at EOF | Mutably borrows reader; returned value owns its data. EOF/error releases input; terminal errors repeat. JSON null remains a present value | JSON and buffered filesystem | Keep import; preserve explicit mutation and EOF distinction |
| `closeJsonl` | `(reader: Owned<JsonlReader>): Result<void, JsonlError>`; explicit close | Consumes reader on success or failure. Already released input is not closed twice; cleanup also occurs on drop | JSON and buffered filesystem | Keep import; never hide consuming close behind a pure-looking utility |
| `JsonValue` | Opaque decoded JSON value; produced by parsing/reading | Owned and explicitly independently clonable; no source record construction or public backing fields | JSON value representation | Keep imported type; global parse must return this checked type, never any or unchecked generic records |
| `JsonError` | Record `{code: string, message: string, offset: int}` | Ordinary owned record with explicit clone; offset identifies original UTF-8 input; no raw input echoed by bridge | Ordinary record representation | Keep imported structured error; throwing wrappers must retain fields |
| `JsonLimits` | Record `{maxBytes: int, maxDepth: int, maxEntries: int, maxNodes: int}` | Ordinary record borrowed by operations; negative bounds and depth above 64 are runtime Err, not valid budgets | Ordinary record representation | Keep imported configuration type; same defaults and checks for both global error styles |
| `JsonlReader` | Opaque buffered reader created by openJsonl | Unique owner; cannot clone or forge; read mutates, close consumes, drop releases | JSON and buffered filesystem | Keep imported resource type |
| `JsonlError` | Record `{code: string, message: string, operation: string, line: int, offset: int, osCode?: int}` | Ordinary owned error; one-based line and absolute byte offset for input failures; close uses zero location. Optional OS code | Ordinary record representation | Keep imported error type; preserve useful I/O and parse context |

Parsing defaults are 16777216 bytes, depth 64, 1000000 entries per container and
1000000 total nodes. Parse input remains borrowed; decoded values do not borrow
the input. Typed JSON/JSONC file imports are a separate build-time representation,
not a JsonValue shortcut. The catalog describes original-source offsets, defaults,
errors and ownership; aliased imports retain hover and parameter help.

## Source and behavior evidence

- `crates/tsrust-compiler/src/stdlib/json/bindings.rs`: all seven functions,
  argument modes, optional arguments, Result shapes and selected features.
- `crates/tsrust-compiler/src/native_types.rs`: all five type declarations,
  fields, opaque/record distinction and clone capabilities.
- `crates/tsrust-compiler/src/stdlib/json/emit.rs`: parse/encode/kind bridges,
  defaults and structured error conversion without input ownership transfer.
- `crates/tsrust-compiler/src/stdlib/json/lines_emit.rs` and `lines.rs`: range
  checks, filesystem integration, terminal errors, EOF and consuming close.
- `crates/tsrust-compiler/src/emit.rs`: native JSON representation selects the
  JSON runtime; actual native calls emit their bridges. A structured JsonError
  alone does not require the parser. This is representation-based selection,
  not a claim of dead-code elimination for every unused imported declaration.
- `crates/tsrust-lsp/src/builtins.json`, group `std:json`: all twelve exports;
  `project_intelligence.rs` JSON tests verify import completion, aliases, parameter
  help, ownership descriptions and error locations.

Focused audit witnesses check plain programs, a user-owned record named `JSON`,
a JsonError-only program and an actual parse call for dependency selection. The
existing resource witness verifies release on EOF, malformed input and early
scope exit. Original logs remain in SQLite under `tsrust-api-json-*`; broad family
regressions and timing remain outstanding.

No new placement decision is needed for this slice: the approved namespace policy
fits the existing ownership and error distinctions. Remaining globals, receiver
methods, host modules and testing bridges must still be reviewed before adding
that namespace.
