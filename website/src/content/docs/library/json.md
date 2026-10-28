---
title: Parse and inspect JSON
description: Parse bounded JSON, handle errors and borrow nested values.
---

Use the global JSON namespace for bounded parsing and serialization. Parsed data
is an owned `JsonValue` by default. Supply an explicit type argument for
[checked typed decoding](../typed-json/), including source-backed `JsonRaw` fields.

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

## Handle parsing errors

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

## Set resource limits

The optional `JsonLimits` record must supply all four fields:

| Field | Default | Meaning |
| --- | --- | --- |
| `maxBytes` | 16777216 | Maximum input size in UTF-8 bytes. |
| `maxDepth` | 64 | Maximum nested container levels; cannot exceed 64. |
| `maxEntries` | 1000000 | Maximum entries in one collection. |
| `maxNodes` | 1000000 | Maximum total values. |

All fields must be nonnegative integers. Serialization has a separate output
limit of 16777216 bytes by default; a negative limit reports `invalid-limits`.

## Know the parsing rules

Parsing is strict JSON. Duplicate keys keep the last value while counting every
entry toward limits. Serialization sorts ordinary object keys and emits compact JSON.
[Raw JSON fragments](../typed-json/#preserve-raw-json-fields) retain their original text.
No reviver, replacer or unchecked generic decoding is supported. JSONC and JSONL
operations remain explicit imports from `std:json`. Typed JSON/JSONC file imports
are separate build-time data modules; see [modules](../../language/modules/).

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
zero. Decimal and exponent tokens use floating-point parsing; extraction cannot
restore precision lost there. Integer tokens outside signed or unsigned 64-bit
JSON storage are rejected with `integer-range`, rather than rounded. Extraction errors have offset zero.

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

## Runtime type inspection

`typeof value` inspects the contents of a JsonValue or JsonView, evaluating once without
consuming it. JSON strings, numbers and booleans report `"string"`, `"number"` and
`"boolean"`. JSON null, arrays and objects report `"object"`, matching the keyword's
ordinary categories. Optional JSON values report `"undefined"` when absent.

A stable `typeof value === "boolean"` guard permits boolean access, including
copying the payload into a local or returning it. The original JSON storage stays
unchanged. String guards enable borrowed `StringView` locals and returns as described below.
Numeric values still require checked scalar extraction;
there is no implicit rounding of exact JSON integers. `Array.isArray(value)` inspects whether the contained JSON value is an array.
`value === null` (and reversed/negated comparisons) recognizes contained JSON null
without confusing it with undefined. Both operations borrow their inputs. The imported `kind` function distinguishes all six JSON categories.

## Reading nested values

Import `get` for object properties and `at` for array elements from `std:json`.
Both accept an owner or view and return `Result<JsonView | undefined, JsonError>`.
The view refers to existing storage without copying it. A missing key or an
index beyond the array returns `Ok(undefined)`; JSON null returns a present view.
A wrong container returns `expected-object` or `expected-array`. Negative indices
return `index-range` for arrays; they do not count from the end. These errors use
offset zero and do not expose keys or input data.

```tsrust
import {get} from "std:json";
function main() {
  const owner = JSON.parse('{"name":"Ada"}');
  match (get(owner, "name")) {
    Ok(value) => {
      if (value !== undefined) {
        console.log(typeof value); // string
      }
    },
    Err(error) => { throw error.message; }
  }
}
```

Pass a view to `get` or `at` again for nested access. `typeof`, `Array.isArray`,
null comparisons, `kind`, `asBoolean`, `asInt`, `asNumber` and serialization accept
views. Serialization returns an independent string. `takeString` requires an
owned JsonValue.

A retained view prevents moving or replacing its owner and cannot outlive it.
Views cannot be constructed or cloned. Missing and error branches carry no child
loan; a present JSON null still retains its owner. Boolean guards permit boolean payload access. String guards permit a borrowed `StringView`. Other category checks do not expose
direct property or index operations on the wrapper.

```tsrust
function readFlag(text: string): boolean {
  const value = JSON.parse(text);
  if (typeof value === "boolean") {
    return value;
  }
  return false;
}
```

`readFlag("true")` returns true. Refinement applies to stable bindings and record
fields, including optional JsonValue and JsonView. Replacing or mutating the
source invalidates the proof. A loop must recheck the category after a change;
a proof from a path that does not reach the use cannot justify access.

## Borrowed string reads after a guard

After `typeof value === "string"`, an owned JsonValue or borrowed JsonView supports
`length`, `startsWith`, `endsWith`, `includes`, `indexOf` and `lastIndexOf`. The
search argument can be an ordinary string or another JSON value guarded as a
string. Both operands remain borrowed through evaluation of later arguments.

```tsrust
function main() {
  const value = JSON.parse('"hello"');
  if (typeof value === "string") {
    console.log(value.length, value.includes("ell")); // 5 true
  }
}
```

Reads use the same UTF-8 byte coordinates, clamping, defaults and invalid-boundary
errors as ordinary Tsrust strings. Length is read-only. Missing optional values do
not satisfy the string guard. Replacing the source invalidates the proof; loops
must recheck after replacement. No text buffer is copied for these reads.

Inside a string guard, assigning the value to a local infers a borrowed
`StringView`. A function borrowing a `JsonValue` can return that view while the
caller retains its owner. Import `asString` for explicit checked extraction:
it returns `Result<StringView, JsonError>` and reports `expected-string` on a
wrong kind. Views cannot be cloned or outlive their JSON owner. A live view prevents
moving or replacing the owner.

`String(view)` creates an independent owned string. Alternatively, `takeString`
consumes an owned JsonValue and moves its string storage; a borrowed view cannot
be consumed by that operation. Neither a guard nor a view enables every ordinary
string method or implicit conversion to an owned string.

See [typed decoding and encoding](../typed-json/) or [JSONC and JSON Lines](../json-files/).
