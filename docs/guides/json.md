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

## Runtime type inspection

`typeof value` inspects the contents of a JsonValue or JsonView, evaluating once without
consuming it. JSON strings, numbers and booleans report `"string"`, `"number"` and
`"boolean"`. JSON null, arrays and objects report `"object"`, matching the keyword's
ordinary categories. Optional JSON values report `"undefined"` when absent.

A stable `typeof value === "boolean"` guard permits boolean access, including
copying the payload into a local or returning it. The original JSON storage stays
unchanged. String and numeric guards still require checked scalar extraction;
there is no implicit rounding of exact JSON integers. `Array.isArray(value)` inspects whether the contained JSON value is an array.
`value === null` (and reversed/negated comparisons) recognizes contained JSON null
without confusing it with undefined. Both operations borrow their inputs. Richer
safe refinement remains tracked in the S10 plan. The existing
imported kind function remains available for its six distinct JSON categories.


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
loan; a present JSON null still retains its owner. Boolean guards permit boolean payload access. Other category checks do not yet
expose direct property, index or scalar operations on the wrapper.


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
