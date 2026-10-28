---
title: Find a standard-library API
description: Choose a built-in module and use compiler-backed editor help for its exact contract.
---

Tsrust provides built-in modules for native programs. Import supported declarations
using a `std:` module name. Built-in availability does not imply compatibility with
Node packages or every JavaScript API of the same name.

## Choose a module

| Task | Module |
| --- | --- |
| Read arguments and environment variables | `std:env` |
| Work with paths, files and bytes | `std:path`, `std:fs`, `std:bytes` |
| Parse JSON and validate shapes | `std:json`, `std:schema` |
| Serve or call HTTP APIs | `std:http` |
| Work with SQLite or pooled databases | `std:sqlite`, `std:sql` |
| Schedule async work or use threads | `std:futures`, `std:threads` |
| Represent recoverable results | `std:result` |
| Test native behavior | `std:testing` |
| Use native collections and caches | `std:collections`, `std:cache` |
| Work with clocks and calendar values | `std:time`, `std:date` |
| Use numeric, decimal, UUID or cryptographic operations | `std:numeric`, `std:decimal`, `std:uuid`, `std:crypto` |
| Inspect the host and handle process signals | `std:os`, `std:signals` |

Strings, arrays, maps, sets and other built-in types also expose methods without
module imports. Supported method names and argument contracts are explicit;
JavaScript's prototype and runtime coercion behavior are not general fallbacks.

Start with [JSON parsing and inspection](../json/), [typed JSON](../typed-json/)
or [JSONC and JSON Lines](../json-files/).

## Read the exact contract

Use completion, hover and parameter help in [VS Code](../../start/editor/). The
language server shares compiler analysis; its API descriptions explain arguments,
defaults, errors, return values, mutation and ownership. Check the behavior of the
specific operation before assuming a byte offset is a character index, an input is
copied, or a returned value can be reused after transfer.

The compiler-owned IntelliSense catalog is maintained with implementation changes.
Public reference pages will expand by module from that authoritative data rather
than maintaining a competing handwritten API catalog.

## Learn by building

Start with a [backend application](../../backend/applications/) or run the
repository's [scoped examples](https://github.com/Forever-Frameworks-LLC/tsrust/tree/main/examples).
Examples may require databases, environment variables or native tools; read their
local setup instructions before running them.
