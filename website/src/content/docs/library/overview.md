---
title: Find a standard-library API
description: Choose a built-in module and use compiler-backed editor help for its exact contract.
---

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
[modules and imports](../../language/modules/) for project files and aliases.

## Choose a module

| Task | Module |
| --- | --- |
| Read arguments and environment variables | [`std:env`](../../reference/library/env/) |
| Work with paths, files and bytes | [`std:path`](../../reference/library/path/), [`std:fs`](../../reference/library/fs/), [`std:bytes`](../../reference/library/bytes/) |
| Parse JSON and validate shapes | [`std:json`](../../reference/library/json/), [`std:schema`](../../reference/library/schema/) |
| Serve or call HTTP APIs | [`std:http`](../../reference/library/http/) |
| Work with SQLite or pooled databases | [`std:sqlite`](../../reference/library/sqlite/), [`std:sql`](../../reference/library/sql/) |
| Schedule async work or use threads | [`std:futures`](../../reference/library/futures/), [`std:threads`](../../reference/library/threads/) |
| Represent recoverable results | [`std:result`](../../reference/library/result/) |
| Test native behavior | [`std:testing`](../../reference/library/testing/) |
| Use native collections and caches | [`std:collections`](../../reference/library/collections/), [`std:cache`](../../reference/library/cache/) |
| Work with clocks and calendar values | [`std:time`](../../reference/library/time/), [`std:date`](../../reference/library/date/) |
| Use numeric, decimal, UUID or cryptographic operations | [`std:numeric`](../../reference/library/numeric/), [`std:decimal`](../../reference/library/decimal/), [`std:uuid`](../../reference/library/uuid/), [`std:crypto`](../../reference/library/crypto/) |
| Inspect the host and handle process signals | [`std:os`](../../reference/library/os/), [`std:signals`](../../reference/library/signals/) |

Strings, arrays, maps, sets and other built-in types also expose methods without
module imports: see [strings](../../reference/builtins/strings/),
[arrays and tuples](../../reference/builtins/arrays/),
[numbers](../../reference/builtins/numbers/), [Map and Set](../../reference/builtins/map-and-set/)
and [globals, `Object`, `JSON` and `console`](../../reference/builtins/globals/).
Supported method names and argument contracts are explicit; JavaScript's prototype
and runtime coercion behavior are not general fallbacks.

Start with [JSON parsing and inspection](../json/), [typed JSON](../typed-json/)
or [JSONC and JSON Lines](../json-files/).

## Read the exact contract

Use completion, hover and parameter help in [VS Code](../../start/editor/). The
language server shares compiler analysis; its API descriptions explain arguments,
defaults, errors, return values, mutation and ownership. Check the behavior of the
specific operation before assuming a byte offset is a character index, an input is
copied, or a returned value can be reused after transfer.

Each module page lists every public operation and type with its signature,
parameters, behavior, errors and ownership. The pages are generated from the same
catalog as editor help, so the two always agree for a given compiler version.

## Learn by building

Start with a [backend application](../../backend/applications/) or run the
repository's [scoped examples](https://github.com/Forever-Frameworks-LLC/tsrust/tree/main/examples).
Examples may require databases, environment variables or native tools; read their
local setup instructions before running them.
