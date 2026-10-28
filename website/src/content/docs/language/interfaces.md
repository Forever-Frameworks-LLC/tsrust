---
title: Define interfaces
description: Declare record types, inherit fields and use generic bases.
---

`interface Name { ... }` declares a record type. It is the same kind of type as `type Name = { ... };`: values are written as record literals, and either form can declare fields or serve as an inheritance base. Each declaration still defines a distinct named type.

After [native setup](/tsrust/tools/rust-setup/), run each complete example
separately as `main.tsr` with `tsrust run main.tsr`. Assertions finish without
output; console examples show their expected output.

```tsrust
import { strictEqual } from "std:testing";

interface Named {
  name: string;
}

interface Positioned {
  x: int;
  y?: int;
}

interface Player extends Named, Positioned {
  score: int;
}

function describe(player: Player): string {
  return player.name + ":" + String(player.x + (player.y ?? 0)) + ":" + String(player.score);
}

function main() {
  const player: Player = {
    name: "Pat", x: 1, score: 3
  };
  strictEqual(describe(player), "Pat:1:3");
  const named: Named = {
    name: "solo"
  };
  strictEqual(named.name, "solo");
}
```

## Fields

- A field is `name: Type` or, for a field that a literal may omit, `name?: Type` (see [optional record fields](/tsrust/language/optional-fields/)).
- Fields end with `;` or `,`. The separator after the last field is optional, and so is a `;` after the closing brace.
- An interface can have type parameters: `interface Box<T> { value: T; }`.

## Inherit fields

`interface Player extends Named, Positioned { ... }` gives `Player` the fields of each named type, in the written order, followed by its own fields. The named types are record types: interfaces, or types declared with a body. They can be declared later in the file or imported from another file.

- A field name can appear only once among the inherited and own fields (`duplicate`).
- An interface cannot extend itself, directly or through other interfaces (`recursive-interface`).
- The name after `extends` must be a record type (`type` error otherwise). Supply the base's type arguments; missing or extra arguments are a `type-arguments` error.

## Extend a generic base

A derived interface substitutes its base's type arguments into inherited fields.
It may pass its own type parameter to a base, or supply a concrete type. Aliases
that resolve to named record types can also be bases, including aliases with
default type arguments. Exported bases and aliases may come from another module.

```tsrust
import { strictEqual } from "std:testing";

interface Box<T> {
  value: T;
}

interface NamedBox<T> extends Box<T> {
  name: string;
}

type TextBox = NamedBox<string>;
interface Message extends TextBox {
  urgent?: boolean;
}

function main() {
  const message: Message = {
    value: "Hello", name: "Ada"
  };
  strictEqual(message.value + " " + message.name, "Hello Ada");
  strictEqual(message.urgent === undefined, true);
}
```

Inheritance preserves field types and ownership. Moving an inherited owned field
makes that field unavailable just as moving a directly declared field does.

## Each interface is its own type

Record types are distinct by name. `Player` has the fields of `Named`, but a `Player` value is not a `Named` value: passing it where `Named` is expected is a `type` error. TypeScript accepts that call because it compares shapes. Write a function for the type it needs, or build the smaller record from the fields:

This fragment assumes a `player: Player` from the first example. `slice(0)`
creates a separate string so `player.name` remains available:

```tsrust
const named: Named = { name: player.name.slice(0) };
```

## Unsupported declarations

- Method signatures such as `run(): int;` (`unsupported`). Declare a function-typed field: `run: Function<int>;`.
- Declaring an interface inside a function (`syntax`).
- Declaring the same interface name twice. TypeScript merges such declarations; here it is a `duplicate` error.

## Quoted property names

Interfaces, record type declarations and constructor literals accept quoted keys.
Use bracket access for names that are not identifiers. Names retain their exact
identity through inheritance, spread, moves and JSON serialization:

```tsrust
import { strictEqual } from "std:testing";

interface Accessible {
  'aria-label': string;
}

interface Props extends Accessible {
  'data-slot'?: string;
}

function name(props: ReadOnly<Props>): string {
  return clone(props['aria-label']);
}

function main() {
  const props: Props = {
    'aria-label': "Account"
  };
  strictEqual(name(props), "Account");
  strictEqual(props['data-slot'] === undefined, true);
}
```

A key such as `'a.b'` is one field, distinct from a nested `a.b` path. Moving it
invalidates that exact field without moving its sibling. Keys containing NUL are
explicitly unsupported; quoted strings are not shorthand property declarations.
Editor completion inserts bracket access for quoted names that are not identifiers.

## Related guides

- [Pattern defaults and rest](/tsrust/language/pattern-defaults-rest/)
- [Record spread and pattern assignment](/tsrust/language/record-spread-assignment/)
