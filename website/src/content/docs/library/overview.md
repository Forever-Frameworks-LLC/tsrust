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
| Read and write asynchronous streams | [`std:io`](/tsrust/reference/library/io/) |
| Launch child processes and own their pipes | [`std:process`](/tsrust/reference/library/process/) |
| Resolve names or use TCP and UDP | [`std:dns`](/tsrust/reference/library/dns/), [`std:net`](/tsrust/reference/library/net/) |
| Compress bytes and streams | [`std:compression`](/tsrust/reference/library/compression/) |
| Work with paths, files and bytes | [`std:path`](/tsrust/reference/library/path/), [`std:fs`](/tsrust/reference/library/fs/), [`std:bytes`](/tsrust/reference/library/bytes/) |
| Parse JSON and validate shapes | [`std:json`](/tsrust/reference/library/json/), [`std:schema`](/tsrust/reference/library/schema/) |
| Serve or call HTTP APIs | [`std:http`](/tsrust/reference/library/http/) |
| Stream server events or exchange WebSocket messages | [`std:http` realtime APIs](/tsrust/reference/library/http/#sseevent) |
| Write structured logs | [`std:log`](/tsrust/reference/library/log/) |
| Match regular expressions or format explicit locales | [`std:regex`](/tsrust/reference/library/regex/), [`std:locale`](/tsrust/reference/library/locale/) |
| Parse URLs and ordered query parameters | [`std:url`](/tsrust/reference/library/url/), [URL guide](/tsrust/library/urls/) |
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

## More library operations

Use the module reference for each signature, error and ownership rule:

- [Collections](/tsrust/reference/library/collections/) provide ordered-set
  snapshots and iteration, ordered-map endpoint keys, front-to-back deque
  snapshots and ascending heap snapshots. Returned snapshots are independently
  owned; heap snapshots preserve the heap.
- [Numeric conversion and arithmetic](/tsrust/reference/library/numeric/) include
  whole-input `parseNumber`, strict `parseInt` with an optional radix from 2 to 36,
  and `checkedAdd`, `checkedSubtract`, `checkedMultiply`, `checkedDivide` and
  `checkedRemainder`. These integer operations return recoverable `Result` errors
  instead of changing the behavior of ordinary arithmetic operators.
- [Calendar and time values](/tsrust/reference/library/date/) include weekday and
  year boundaries, plus hour, minute, second and nanosecond components. `getDay`
  uses Sunday 0 through Saturday 6; these accessors do not introduce mutable
  JavaScript dates or implicit time zones.
- [Path operations](/tsrust/reference/library/path/) parse and format components,
  resolve against the current directory, and expose the host separator and path
  delimiter. [Environment snapshots](/tsrust/reference/library/env/) and
  [host information](/tsrust/reference/library/os/) report fallible OS results
  explicitly. Path text operations follow the host platform's rules.
- [Task groups and timers](/tsrust/language/async/#scheduling-and-channels) include
  ordered `all`, repeating intervals, cooperative cancellation and bounded
  channels. Thread pools also expose lazy asynchronous CPU submission.
- [Testing](/tsrust/tools/testing/) includes structural assertions, checked async
  rejection assertions, name filtering, newline-delimited JSON reports and
  explicit async-body deadlines.

The source tree also contains an experimental
[owned iterator reference](/tsrust/reference/library/iterator/). Its pipelines
remain under validation; their presence does not establish completed acceptance.
See [supported scope](/tsrust/project/status/) when choosing a release or source
revision.

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
