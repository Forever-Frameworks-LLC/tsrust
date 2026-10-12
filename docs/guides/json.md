# JSON values and errors

Use the global JSON namespace for bounded parsing and serialization. Parsed data
is an owned `JsonValue`, not an unchecked record or dynamically typed object.

```tsrust
function main() {
  const value = JSON.parse('{"answer":42}');
  console.log(JSON.stringify(value)); // {"answer":42}
  try {
    JSON.parse("null secret");
  } catch (error) {
    console.log(error.code, error.offset); // trailing-data 5
  }
}
```

`JSON.parse(text, limits?)` and `JSON.stringify(value, maxBytes?)` throw structured
`JsonError`. Errors retain `code`, `message` and a UTF-8 byte `offset`. Normal typed
propagation, callbacks and `finally` apply.

For explicit error handling, `JSON.tryParse` and `JSON.tryStringify` return
`Result<..., JsonError>` with the same backend rules and budgets:

```tsrust
function main() {
  match (JSON.tryParse("null secret")) {
    Ok(value) => { console.log(JSON.stringify(value)); },
    Err(error) => { console.log(error.code); } // trailing-data
  }
}
```

Parsing borrows the input string and optional `JsonLimits`; its result owns all
parsed storage. Serialization borrows its value and returns an independent string.
Import `JsonValue`, `JsonError` or `JsonLimits` from `std:json` when explicit type
annotations are useful. Existing `std:json.parse` and `std:json.stringify` continue
to return Results.

Default parsing budgets are 16777216 input bytes, 64 container levels, 1000000
entries per collection and 1000000 total values. A limits record must supply all
four nonnegative integer fields: `maxBytes`, `maxDepth`, `maxEntries`, `maxNodes`.
Depth cannot exceed 64. Serialization defaults to 16777216 output bytes; a
negative output limit fails with `invalid-limits`.

Parsing is strict JSON. Duplicate keys keep the last value while counting every
entry toward limits. Serialization sorts object keys and emits compact JSON.
No reviver, replacer or unchecked generic decoding is supported. JSONC and JSONL
operations remain explicit imports from `std:json`. Typed JSON/JSONC file imports
are separate build-time data modules; see [modules](modules.md).

Local variables named JSON shadow the global namespace. Runtime dependencies are
selected automatically from checked usage; no per-build linking flags are needed.

## Extracting scalar values

Import `asBoolean`, `asInt`, `asNumber` and `takeString` from `std:json`.
Each returns `Result<scalar, JsonError>` without coercion. The first three borrow
an unchanged value. `takeString` consumes it, moving string storage into the
result; the input is consumed even when its kind is wrong.

`asInt` requires an integral value within the signed 64-bit range. `asNumber`
rejects integer-backed values that cannot be represented exactly as floating point.
It preserves floating negative zero; integer extraction converts negative zero to
zero. Extraction cannot restore precision already lost while parsing a decimal or
an integer beyond the backend's 64-bit range. Extraction errors have offset zero.

```tsrust
import {asInt} from "std:json";
function main() {
  const data = JSON.parse("42");
  match (asInt(data)) {
    Ok(value) => { console.log(value); },
    Err(error) => { throw error.message; }
  }
}
```

This prints `42`. `asInt` reports `expected-number`, `integer-fractional` or
`integer-range`; `asNumber` reports `expected-number` or `number-precision`.
The other extractors report `expected-boolean` and `expected-string` respectively.
