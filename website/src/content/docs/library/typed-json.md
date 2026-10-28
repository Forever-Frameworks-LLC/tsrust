---
title: Decode and encode typed JSON
description: Validate owned shapes and serialize ordinary records without unchecked casts.
---

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
Typed JSONC and JSON Lines use the same checked shapes; see [JSON files](../json-files/).

## Checked literal targets

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

## Encode ordinary typed values

`JSON.stringify(value)` accepts scalars, named records, tuples, arrays, `JsonValue`,
`JsonView` and `StringView`. `JSON.tryStringify(value)` provides the same encoding
through `Result<string, JsonError>`. Dynamic constructors such as `fromEntries`
are useful for assembling a `JsonValue`, but ordinary data does not require them.
Object literals currently require a named record annotation:

```tsrust
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
