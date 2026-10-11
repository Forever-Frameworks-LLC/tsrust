# Module exports and imports

A module can expose named declarations and one default function. Relative imports
require the explicit `.tsr` extension.

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

Default exports currently accept function declarations or a local function name.
Arbitrary expressions, exported global variables and re-export statements remain
unsupported. Each module permits one default export. Missing defaults, duplicate
exports and conflicting local import names are diagnosed.

Imports resolve to the original function declaration. Default aliases introduce no
wrapper, allocation, implicit cloning or change to ownership, generic inference,
or entrypoint selection. Only the selected entry module's main runs automatically.
