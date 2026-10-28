---
title: JSONC and JSON Lines
description: Read commented JSON and stream bounded records with explicit errors and cleanup.
---

## Decode typed JSONC

Import `parseJsonc` from `std:json` and supply a target type to decode directly:

```tsrust
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

```tsrust
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
