---
title: Find a standard-library API
description: Choose a built-in module and use compiler-backed editor help for its exact contract.
---

Start with the [global functions and types](/tsrust/library/globals/), such as `clone`, `Result`,
`unwrap` and `Task`; they need no import and appear in almost every file.

Tsrust provides built-in modules for native programs. Import supported declarations
using a `std:` module name. Built-in availability does not imply compatibility with
Node packages or every JavaScript API of the same name.

## Import an operation

```tsrust
import {basename} from 'std:path';
function main() {
  console.log(basename('/reports/today.json')); // today.json
}
```

Use named imports for the declarations you need. See
[modules and imports](/tsrust/language/modules/) for project files and aliases.

## Choose a module

| Task | Module |
| --- | --- |
| Read arguments and environment variables | [`std:env`](/tsrust/reference/library/env/) |
| Work with paths, files and bytes | [`std:path`](/tsrust/reference/library/path/), [`std:fs`](/tsrust/reference/library/fs/), [`std:bytes`](/tsrust/reference/library/bytes/) |
| Parse JSON and validate shapes | [`std:json`](/tsrust/reference/library/json/), [`std:schema`](/tsrust/reference/library/schema/) |
| Serve or call HTTP APIs | [`std:http`](/tsrust/reference/library/http/) |
| Work with SQLite or pooled databases | [`std:sqlite`](/tsrust/reference/library/sqlite/), [`std:sql`](/tsrust/reference/library/sql/) |
| Schedule async work or use threads | [`std:futures`](/tsrust/reference/library/futures/), [`std:threads`](/tsrust/reference/library/threads/) |
| Represent recoverable results | Global [`Result`, `Ok`, `Err` and `unwrap`](/tsrust/library/globals/#result-ok-and-err) — no import |
| Test native behavior | [`std:testing`](/tsrust/reference/library/testing/) |
| Use native collections and caches | [`std:collections`](/tsrust/reference/library/collections/), [`std:cache`](/tsrust/reference/library/cache/) |
| Work with clocks and calendar values | [`std:time`](/tsrust/reference/library/time/), [`std:date`](/tsrust/reference/library/date/) |
| Use numeric, decimal, UUID or cryptographic operations | [`std:numeric`](/tsrust/reference/library/numeric/), [`std:decimal`](/tsrust/reference/library/decimal/), [`std:uuid`](/tsrust/reference/library/uuid/), [`std:crypto`](/tsrust/reference/library/crypto/) |
| Inspect the host and handle process signals | [`std:os`](/tsrust/reference/library/os/), [`std:signals`](/tsrust/reference/library/signals/) |

Strings, arrays, maps, sets and other built-in types also expose methods without
module imports: see [strings](/tsrust/reference/builtins/strings/),
[arrays and tuples](/tsrust/reference/builtins/arrays/),
[numbers](/tsrust/reference/builtins/numbers/), [Map and Set](/tsrust/reference/builtins/map-and-set/)
and the [globals](/tsrust/reference/globals/functions/): global functions and types,
`console`, `JSON`, `Math`, `Number` and `Object`.
Supported method names and argument contracts are explicit; JavaScript's prototype
and runtime coercion behavior are not general fallbacks.

Start with [JSON parsing and inspection](/tsrust/learn/data/#parse-and-inspect-json), [typed JSON](/tsrust/learn/data/#decode-and-encode-typed-json)
or [JSONC and JSON Lines](/tsrust/learn/data/#jsonc-and-json-lines).

## Read the exact contract

Use completion, hover and parameter help in [VS Code](/tsrust/start/editor/). The
language server shares compiler analysis; its API descriptions explain arguments,
defaults, errors, return values, mutation and ownership. Check the behavior of the
specific operation before assuming a byte offset is a character index, an input is
copied, or a returned value can be reused after transfer.

Each module page lists every public operation and type with its signature,
parameters, behavior, errors and ownership. The pages are generated from the same
catalog as editor help, so the two always agree for a given compiler version.

## Learn by building

Start with a [backend application](/tsrust/learn/first-service/#create-the-service) or run the
repository's [scoped examples](https://github.com/Forever-Frameworks-LLC/tsrust/tree/main/examples).
Examples may require databases, environment variables or native tools; read their
local setup instructions before running them.
