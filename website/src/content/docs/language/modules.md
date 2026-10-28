---
title: Import and export modules
description: Use named and default exports, source imports and checked data modules.
---

A source module can expose named declarations and one default function. Source
imports accept explicit `.tsr` paths, extensionless files and directory `index.tsr`
files. JSON and JSONC data imports require an explicit extension.

```typescript
export function greeting(): string {
  return "Hello";
}

export default greeting;
```

Import the default under any local name, optionally alongside named imports:

```typescript
import greet, { greeting } from "./greeting.tsr";

function main() {
  console.log(greet()); // Hello
  console.log(greeting()); // Hello
}
```

A default function can also be declared directly, with or without a local name:

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


## Project aliases

Add `paths` to `tsrust.config.json` to map imports relative to that configuration:

```json
{ "paths": { "@/*": "./src/*" } }
```

`import greet from "@/components"` selects `src/components.tsr` when present,
otherwise `src/components/index.tsr`. Exact mappings take priority over wildcard
mappings; the longest matching wildcard prefix wins. Invalid or ambiguous mappings
are diagnosed. Alias and relative paths to the same file retain one module identity.

## Typed JSON and JSONC imports

A data file has one default export. Its properties and element types are inferred
and checked at build time. JSON is strict; JSONC accepts comments and trailing
commas. Named data-property imports are unsupported; access properties through
the default import.

For `settings.jsonc` containing `{ "host": "local", "port": 8080 }`:

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

Each canonical data file has one immutable value. Reads borrow it without copying.
Moving or mutating imported owned data requires an explicit `clone`; scalar reads
copy normally. Data is embedded in the executable, so execution does not reopen
the original file. `dev` rebuilds after data changes and keeps the previous program
running when changed data fails validation.

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
