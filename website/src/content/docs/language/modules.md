---
title: Import and export modules
description: Use named and default exports, source imports and checked data modules.
---

A source module can expose named declarations and one default function. Source
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

Source default exports currently accept function declarations or a local function name.
Arbitrary expressions, exported global variables and re-export statements remain
unsupported. Each module permits one default export. Missing defaults, duplicate
exports and conflicting local import names are diagnosed.

Imports resolve to the original function declaration. Default aliases introduce no
wrapper, allocation, implicit cloning or change to ownership, generic inference,
or entrypoint selection. Only the selected entry module's main runs automatically.

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
