# JSON values and errors

Use the global JSON namespace for bounded parsing and serialization. Parsed data
is an owned `JsonValue` by default. An explicit type argument requests checked
decoding into that owned type.

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
are separate build-time data modules; see [modules](../../website/src/content/docs/language/modules.md).

Local variables named JSON shadow the global namespace. Runtime dependencies are
selected automatically from checked usage; no per-build linking flags are needed.

## Decoding a known shape

```tsrust
type Item = { name: string; count: int; };
function main() {
  const item = JSON.parse<Item>('{"name":"hello","count":42}');
  console.log(item.name, item.count); // hello 42
}
```

`JSON.parse<T>` returns `T` or throws `JsonError`. `JSON.tryParse<T>` returns
`Result<T, JsonError>`. The explicit type drives validation; it is not an unchecked
cast. Without a type argument, both functions retain their dynamic JsonValue
contracts, even when the surrounding declaration has a record annotation.

Supported targets are owned scalar values, records, tuples, arrays and recursive
records with nullable or optional links. Required fields must exist; omitted
optional fields become undefined. Every occurrence of a known field must match
its declared type; valid duplicates replace earlier values. Unknown fields are
skipped but still count toward all input budgets. Tuples require the declared
number of elements. A shape mismatch reports `shape-mismatch` with a byte offset.

`int` rejects fractional and out-of-range values. `number` rejects signed or
unsigned 64-bit integer payloads that would lose precision, and preserves floating
negative zero. Larger integer tokens retain the parser's finite floating-point
rules. Use `int` for exact signed 64-bit arithmetic. The errors are
`integer-fractional`, `integer-range` and `number-precision`.

Decoding borrows text and limits and directly builds independent owned storage.
It does not build a dynamic JSON tree first. Native resources, borrowed views,
functions and unions with multiple non-null value shapes are not decoding targets.
Typed JSONC and JSONL decoding and direct typed encoding remain pending.

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
unchanged. String guards enable the borrowed reads below. First-class string extraction and
numeric guards still require checked scalar extraction;
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

The wrapper still owns or borrows JSON storage. This does not yet permit assigning
the payload to a string local, returning a borrowed string, or invoking other
string methods. Use the explicit consuming `takeString` when independent owned
string storage is needed; borrowed views cannot be consumed by that operation.

## Encode ordinary typed values

`JSON.stringify(value)` accepts scalars, named records, tuples, arrays, `JsonValue`,
`JsonView` and `StringView`. `JSON.tryStringify(value)` provides the same encoding
through `Result<string, JsonError>`. Dynamic constructors such as `fromEntries`
are useful for assembling a `JsonValue`, but ordinary data does not require them.
Object literals currently require a named record annotation:

```ts
 type Item = { name: string; count: int; };
 function main() {
   const item: Item = { name: 'hello', count: 42 };
   console.log(JSON.stringify(item));
 }
```

The result is `{"count":42,"name":"hello"}`. Object keys are sorted. Tuples
encode as arrays. Encoding borrows input and produces an owned string, without
building an intermediate JSON tree or implicitly cloning the input. Recursive
records retain their existing ownership and allocation rules.

Undefined object fields are omitted. Undefined at the root or in an array reports
`undefined-value`; it is never silently replaced with null. Nonfinite numbers
report `number-nonfinite`. Integers retain their exact signed 64-bit values.
The default output limit is 16777216 UTF-8 bytes; pass a nonnegative second
argument to change it. Exceeding this limit reports `output-limit` without
returning partial output. Container nesting above 64 reports `depth-limit`,
including dynamic JSON nested inside typed data. Resource handles, callbacks,
Maps and Sets are not JSON encodable.

## Decode typed JSONC

Import `parseJsonc` from `std:json` and supply a target type to decode directly:

```ts
import {parseJsonc} from 'std:json';
type Item = { name: string; };
function main() {
  match (parseJsonc<Item>('/* note */ {"name":"hello"}')) {
    Ok(item) => { console.log(item.name); },
    Err(error) => { throw error.message; }
  }
}
```

The result is `Result<Item, JsonError>`. Omit the type argument for the existing
`Result<JsonValue, JsonError>` result. The second argument enables trailing
commas (default false); the third supplies `JsonLimits`. Comments count toward
the input byte limit, and error offsets address the original UTF-8 bytes.
Typed field validation follows `JSON.tryParse<T>`, including missing optional
fields, checked numeric conversion and limits on ignored fields. The result owns
its data. JSONC normalization uses bounded scratch storage; input is unchanged.

## Read typed JSON Lines

`readJsonl<T>(reader)` returns `Result<T | undefined, JsonlError>`. Each call
constructs one owned value directly; previous values remain valid after later
reads or after closing the reader. `undefined` means EOF. A decoded JSON null
remains null, including when the requested target is `null`.

```ts
match (readJsonl<Item>(reader)) {
  Ok(item) => {
    if (item !== undefined) { console.log(item.name); }
  },
  Err(error) => { console.log(error.code, error.line, error.offset); }
}
```

The reader mutably borrows for each call. Its opening limits apply per record.
Shape errors, malformed input and I/O errors are terminal: the file is released
and subsequent reads repeat the error without more I/O. Errors retain one-based
line and absolute UTF-8 byte offsets. Omit the type argument for dynamic
`JsonValue` reads. Use `closeJsonl(reader)` to consume an unfinished reader and
report close errors; scope exit also releases it.

## Write bounded JSON Lines

`createJsonl(path, maxBytes?, capacity?)` creates or truncates output after
validating options. The default record limit is 16777216 bytes excluding LF;
the default output buffer is 8192 bytes. Limits must be nonnegative and capacity
must be between 1 and 1048576 bytes. Supply maxBytes when specifying capacity.

Use `writeJsonl(writer, value)` for scalars, records, arrays or dynamic JSON.
It borrows the value and mutably borrows the writer. Encoding uses the same
rules as JSON.stringify and stages the complete bounded record before output.
Encoding rejection writes no prefix of that record; I/O failure can leave a
prefix and offers no rollback. Successful records end with exactly one LF.

`flushJsonl(writer)` reports buffered output failures while retaining ownership.
`closeJsonlWriter(writer)` consumes the writer, flushes and closes it. All calls
return Result with JsonlError. A write/flush failure is terminal for later
write/flush calls. Close still attempts cleanup: a new close failure takes
precedence, otherwise the prior failure is returned again. Drop also closes but
cannot report errors. None of these operations requests durable storage sync.

Writer errors use offset zero, not an output byte location. The line field is
the current record for write and the next-record counter for flush/close; a flush
error can concern any buffered earlier record. Writer resources cannot be cloned
or constructed as ordinary records.


### Checked literal targets

Explicit typed decoders accept scalar literal types and closed unions of one
scalar kind. Nullable/optional values retain their ordinary missing/null rules.
Unknown alternatives reject as `shape-mismatch`; numeric precision checks and
input budgets remain in effect. Mixed value-shape unions remain unsupported.

```tsrust
type Status = 'draft' | 'published';
function main() {
  const status = JSON.parse<Status>('"draft"');
  console.log(status === 'draft', String(status).length); // true 5
}
```

Direct common string-member lookup on a literal union is not yet implemented;
`String(status)` explicitly obtains an owned string without changing its text.
For typed JSONC/JSONL literal results, use ordinary Result matching. Nested generic
inference through `unwrap(parseJsonc<Status>(...))` remains a separate limitation.
