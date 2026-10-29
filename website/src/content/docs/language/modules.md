---
title: Import and export modules
description: Use named and default exports, source imports and checked data modules.
---

A source module can expose named declarations and one default export. Source
imports accept explicit `.tsr` paths, extensionless files and directory `index.tsr`
files. JSON and JSONC data imports require an explicit extension.

## Share a function

Save this as `greeting.tsr`:

```typescript
export function greeting(): string {
  return "Hello";
}

export default greeting;
```

Save the following as `main.tsr` in the same directory. Import the default under
any local name, optionally alongside named imports:

```typescript
import greet, { greeting } from "./greeting.tsr";

function main() {
  console.log(greet()); // Hello
  console.log(greeting()); // Hello
}
```

Run `tsrust run main.tsr`; it prints `Hello` twice.

## Choose a default export

Alternatively, replace `greeting.tsr` with one of these declarations. Each exports
only a default function, so its consumer must use only a default import:

```typescript
export default function greeting(): string {
  return "Hello";
}
```

```typescript
export default function(): string {
  return "Hello";
}
```

A named default declaration keeps its name inside its module. It does not also
publish a named export; use the first form when both exports are needed.
`import { default as greet } from "./greeting.tsr";` is also supported.

Source default exports accept function declarations or the name of a local function
or module binding. Arbitrary expressions and re-export statements remain unsupported. Each module permits one default export. Missing defaults, duplicate
exports and conflicting local import names are diagnosed.

Function imports resolve to the original declaration. Default aliases introduce no
wrapper, allocation, implicit cloning or change to ownership, generic inference,
or entrypoint selection. Only the selected entry module's main runs automatically.

## Share module state

Module variables initialize once, after their module dependencies, in declaration
order. Imported bindings remain live: callers observe later changes made by the
exporting module. An importer cannot reassign the imported binding itself.

Save this as `state.tsr`:

```tsrust
export const settings = { retries: 3 };
export let requests = 0;

export function recordRequest() {
  requests++;
}

export default settings;
```

Replace `main.tsr` with:

```tsrust
import settings, { requests, recordRequest } from './state.tsr';

function main() {
  recordRequest();
  console.log(requests);
  const local = clone(settings);
  local.retries = 9;
  console.log(settings.retries);
}
```

Run `tsrust run main.tsr`; it prints `1`, then `3`. Both imports refer to the
original module storage. `const` prevents rebinding, not field mutation; `let`
permits rebinding within its declaring module. Neither creates an implicit clone.
Use `clone` when you need an independent owned value.

### Initialization and access

An initializer may call functions, but they cannot read a later binding before it
initializes. An initialization failure prevents `main` from running. Initialization
also runs for an entry file without `main`.

Functions borrow module storage according to their access, including indirect
access through other functions. Active loans prevent conflicting mutations or
consumption. A function cannot leave program-owned storage consumed.
Module variables do not automatically acquire locks or permission to cross threads.

### Callbacks retain live state

A callback accessing module storage borrows the live owner; it does not take a
private copy of scalar globals. Replace `main.tsr` with this standalone example:

```tsrust
let completed = 0;

function apply(action: Mutable<Function<void, []>>) {
  action();
}

function main() {
  apply(() => { completed++; });
  console.log(completed);
}
```

This prints `1`. The borrowing callback contract permits access to the module
owner. An owned `Function` contract rejects this borrowed capture. Choose the
contract according to how the callback is used; do not silently snapshot state.

## Resolve source paths

For an extensionless import such as `./greeting`, the compiler tries these paths
in order:

1. `greeting.tsr`
2. `greeting.tsrx`
3. `greeting/index.tsr`
4. `greeting/index.tsrx`

The `.tsrx` forms support retained UI source; backend projects normally use `.tsr`.
Use an explicit extension when the intended file could otherwise be ambiguous.

## Project aliases

Add `paths` to `tsrust.config.json` to map imports relative to that configuration:

```json
{ "paths": { "@/*": "./src/*" } }
```

Using the first greeting module and consumer above, move `greeting.tsr` into
`src/` and replace the consumer's import with
`import greet, { greeting } from "@/greeting";`. Both calls remain unchanged.
The alias resolves under `src/` using the
candidate order above. Exact mappings take priority over wildcard
mappings; the longest matching wildcard prefix wins. Invalid or ambiguous mappings
are diagnosed. Alias and relative paths to the same file retain one module identity.

## Typed JSON and JSONC imports

A data file has one default export. Its properties and element types are inferred
and checked at build time. JSON is strict; JSONC accepts comments and trailing
commas. Named data-property imports are unsupported; access properties through
the default import.

Create `settings.jsonc` beside `main.tsr`:

```jsonc
{ "host": "local", "port": 8080 }
```

Replace `main.tsr` with:

```typescript
import settings from "./settings.jsonc";

function main(): void {
  console.log(settings.host); // local
  const owned = clone(settings);
  owned.port = 9;
  console.log(owned.port); // 9
  console.log(settings.port); // 8080
}
```

Run `tsrust run main.tsr`; it prints `local`, `9` and `8080` on separate lines.

### Ownership and rebuilds

Each canonical data file has one immutable value. Reads borrow it without copying.
Moving or mutating imported owned data requires an explicit `clone`; scalar reads
copy normally. Data is embedded in the executable, so execution does not reopen
the original file. `dev` rebuilds after data changes and keeps the previous program
running when changed data fails validation.

### Inferred types and input limits

Nested records, mixed data unions, nullable values and arrays retain checked types.
Record arrays mark properties missing from some elements optional. Empty arrays
can infer an element type from nonempty siblings; an unconstrained empty array
reports an error. Unknown properties are errors. A literal bracket key accesses
escaped names, for example `settings["a-b"]`.

Integer spellings must fit signed 64-bit `int`. Decimal/exponent spellings and
negative zero use `number`. Mixing integers with `number` requires each integer
to remain exactly representable. Parsing is bounded to 16 MiB, 64 nesting levels,
one million entries per collection and one million value nodes. Duplicate keys
keep the last value. Diagnostics point to the original data file.

## Text imports

An explicit `.txt` import embeds a file's UTF-8 text as a read-only default value.
The application does not execute the file or reopen it at startup.

### Import and read text

After [setting up native execution](/tsrust/tools/rust-setup/), create two files
in the same directory. Put this line in `message.txt`:

```text
Hello from a text file
```

Save this program as `main.tsr`:

```tsrust
import message from "./message.txt";

function main() {
  console.log(message.trim());
  const editableCopy = clone(message);
  console.log(editableCopy.startsWith("Hello"));
}
```

Run `tsrust run main.tsr`. It prints:

```text
Hello from a text file
true
```

The `.txt` extension must be explicit in the import. A text module has one default
export and no named exports. Read the imported string directly; use `clone` when
an owned value is needed. The call to `trim()` above changes the printed result,
not the embedded text.

### Preserve exact text

Imports preserve UTF-8 content, including newlines, a byte-order mark and embedded
null characters. Invalid UTF-8 produces a `module-load` error instead of replacing
invalid bytes. See [strings](/tsrust/language/strings/) for byte offsets and Unicode
scalar positions.

Text files are build inputs. Changing an imported file invalidates the native
cache; rebuilding embeds the new contents. An already-built executable keeps
its embedded version. Inside a [configured project](/tsrust/start/project-structure/), use
`tsrust dev` to rebuild after file changes. The watcher uses the configured build
entry; it does not accept a filename.

### Choose text or structured data

| File | Imported value |
| --- | --- |
| `.txt`, including `.json.txt` | Exact read-only text |
| `.json` | Checked structured data |
| `.jsonc` | Checked structured data with comments and trailing commas allowed |

Importing `.json.txt` does not parse its JSON contents. To decode text at runtime,
use [`JSON.parse<T>`](/tsrust/learn/data/#decode-and-encode-typed-json). It returns an owned value after
checking the requested shape and throws for malformed JSON or incompatible data.
For build-time data checking, use [JSON/JSONC imports](/tsrust/language/modules/#typed-json-and-jsonc-imports).

### Work in the editor

Imported text offers string completion and hover documentation. Go to definition
opens the text file. Unsaved text edits are available to editor analysis; save the
file before relying on CLI execution or a native rebuild.
