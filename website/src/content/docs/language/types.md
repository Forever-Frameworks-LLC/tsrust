---
title: "Type aliases and interfaces"
description: "Name types with aliases, declare record types with interfaces and use optional fields."
---

Tsrust types are checked at compile time and have a fixed native layout. `type` gives an existing type another name without creating a wrapper. `interface` and object-shaped `type` declarations both define record types; records have a fixed set of fields, and optional fields (`name?: T`) may be left out of a literal without changing that shape. Optional values (`null` and `undefined`) are covered in [optional values](../optional-values/).

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/) and run `tsrust run main.tsr`. Examples that use `strictEqual` finish without output when their assertions pass; others show expected output in comments.

## Type aliases

`type Name = Type;` gives an existing type another name. A value accepted by the
alias is accepted by its expanded type, and vice versa. An alias does not create a
new wrapper, convert values or copy resources.

### Name a reusable type

After [setting up native execution](/tsrust/tools/rust-setup/), save this example
as `main.tsr` and run `tsrust run main.tsr`. It finishes without output when all
assertions pass.

```tsrust
import { strictEqual } from "std:testing";

type Id = int | string;

type Ids = Id[];

type Pair = [int, string];

type Scores = Map<string, int>;

function kind(id: Id): string {
  if (typeof id === "string") {
    return "text";
  }

  return "number";
}

function main() {
  const ids: Ids = [7, "seven"];
  strictEqual(kind(ids[0]) + kind(ids[1]), "numbertext");
  const pair: Pair = [2, "two"];
  const [count, label] = pair;
  strictEqual(label + String(count), "two2");
  const scores: Scores = new Map<string, int>();
  scores.set("a", 1);
  strictEqual(scores.size, 1);
}
```

Aliases can name primitives, unions, arrays, tuples, named records and enums,
collections and function types. An alias may use another alias declared before or
after it in the same file.

A record declaration such as `type Point = { x: int; y: int; };` creates a named
record type. An alias such as `type Position = Point;` names that existing type.

### Supply type arguments and defaults

A generic alias substitutes its type arguments into the type expression. A default
fills an omitted argument. Required parameters must come before defaulted ones;
a default may refer to an earlier parameter.

Run this as a separate `main.tsr`; its assertions also finish without output:

```tsrust
import { strictEqual } from "std:testing";

type Values<T = string> = T[];

type Pair<A = int, B = Values<A>> = [A, B];
interface Box<T> {
  value: T;
}

type Rows<T = int> = Box<T>[];

function main() {
  const words: Values = ["one", "two"];
  strictEqual(words.join(","), "one,two");
  const pair: Pair = [7, [2, 3]];
  strictEqual(pair[0] + pair[1][0], 9);
  const rows: Rows = [{
    value: 4
  }];
  strictEqual(rows[0].value, 4);
}
```

Here, `Values` expands to `string[]`, `Pair` to `[int, int[]]`, and `Rows` to
`Box<int>[]`. Writing `Values<int>` replaces the default with `int`.
Parameters shadow other type names inside their alias.

Defaults are supported on aliases, but not directly on record bodies, interfaces
or functions. An alias of a generic record can supply defaults, as `Rows` does.

### Export an alias

Prefix the declaration with `export`, then import its name from another file.
For example, put this declaration in `types.tsr`:

```tsrust
export type Values<T = string> = T[];
```

Use it from `main.tsr`:

```tsrust
import { Values } from "./types.tsr";

function main() {
  const words: Values = ["hello", "world"];
  console.log(words.join(" "));
}
```

`tsrust run main.tsr` prints `hello world`. Exported aliases retain their defaults
and can refer to types in their own module that the importing file did not import.
See [modules](/tsrust/language/modules/) for import paths and visibility.

### Diagnose invalid aliases

| Problem | Diagnostic |
| --- | --- |
| Missing a required type argument or supplying extra arguments | `arity` |
| Referring to the same alias directly or through other aliases | `recursive-alias` |
| Declaring a name twice or replacing a built-in type name | `duplicate` |
| Declaring an alias inside a function | `syntax` |
| Supplying a value incompatible with the expanded type | `type` |

Declare aliases at the top level. For recursive shapes, use a record declaration
instead of an alias cycle. Very large expansions are rejected: the compiler uses
a 65,536-node expansion budget as well as its annotation-depth limit.

Ownership, borrowing and exact integer rules remain those of the expanded type.
An alias does not make an owned resource copyable or permit a lossy conversion.

### Inspect aliases in the editor

Hover shows the alias declaration; go to definition navigates to it. Aliases
appear in the outline. Values annotated with aliases offer the members of the
expanded type, including aliases imported from another file.

## Interfaces

`interface Name { ... }` declares a record type. It is the same kind of type as `type Name = { ... };`: values are written as record literals, and either form can declare fields or serve as an inheritance base. Each declaration still defines a distinct named type.

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

### Fields

- A field is `name: Type` or, for a field that a literal may omit, `name?: Type` (see [optional record fields](/tsrust/language/types/#optional-fields)).
- Fields end with `;` or `,`. The separator after the last field is optional, and so is a `;` after the closing brace.
- An interface can have type parameters: `interface Box<T> { value: T; }`.

### Inherit fields

`interface Player extends Named, Positioned { ... }` gives `Player` the fields of each named type, in the written order, followed by its own fields. The named types are record types: interfaces, or types declared with a body. They can be declared later in the file or imported from another file.

- A field name can appear only once among the inherited and own fields (`duplicate`).
- An interface cannot extend itself, directly or through other interfaces (`recursive-interface`).
- The name after `extends` must be a record type (`type` error otherwise). Supply the base's type arguments; missing or extra arguments are a `type-arguments` error.

### Extend a generic base

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

### Each interface is its own type

Record types are distinct by name. `Player` has the fields of `Named`, but a `Player` value is not a `Named` value: passing it where `Named` is expected is a `type` error. TypeScript accepts that call because it compares shapes. Write a function for the type it needs, or build the smaller record from the fields:

This fragment assumes a `player: Player` from the first example. `slice(0)`
creates a separate string so `player.name` remains available:

```tsrust
const named: Named = { name: player.name.slice(0) };
```

### Unsupported declarations

- Method signatures such as `run(): int;` (`unsupported`). Declare a function-typed field: `run: Function<int>;`.
- Declaring an interface inside a function (`syntax`).
- Declaring the same interface name twice. TypeScript merges such declarations; here it is a `duplicate` error.

### Quoted property names

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

## Optional fields

`name?: Type` declares a record field that a record literal may leave out.

```tsrust
import { strictEqual } from "std:testing";

type Point = {
  x: int;
  y?: int;
};

type User = {
  name: string;
  nickname?: string;
  home?: Point;
};

function width(point: Point): int {
  return point.x + (point.y ?? 0);
}

function main() {
  const flat: Point = {
    x: 1
  };
  strictEqual(flat.y === undefined, true);
  strictEqual(width(flat), 1);
  strictEqual(width({
    x: 1, y: 2
  }), 3);

  const user: User = {
    name: "Pat", home: {
      x: 3
    }
  };
  strictEqual(user.nickname ?? user.name, "Pat");
  strictEqual(user.home?.x === 3, true);

  let moving: Point = {
    x: 0
  };
  moving.y = 5;
  strictEqual(width(moving), 5);
  moving.y = undefined;
  strictEqual(width(moving), 0);
}
```

### Read and update optional fields

- The type of `name?: Type` is `Type | undefined`. Reading the field gives that union; narrow it, or use `??` or `?.`, before using the value.
- A literal may omit the field. The field then holds `undefined`. Writing `name: undefined` means the same.
- The field can be assigned later, also back to `undefined`.
- Required fields are unchanged: a literal without one is a `type` error.
- Optional fields work in generic record types, nested records, arrays of records, destructuring with defaults and object spread.

### A record keeps its shape

A record type has a fixed set of fields. An omitted optional field is still a field of the record:

```tsrust
type Point = {
  x: int;
  y?: int;
};

function main() {
  const point: Point = {
    x: 1
  };
  console.log(point, Object.keys(point).join(","), "y" in point);
    // prints { x: 1, y: undefined } x,y true
}
```

JavaScript leaves an omitted property out of the object, so it prints `{ x: 1 }`, lists only `x`, and `"y" in point` is `false`. Programs that test `point.y === undefined`, or use `??` and `?.`, behave the same in both.

Omitted fields follow the written fields in declaration order when inspecting keys
or printing the record.

### Require a field that can be undefined

A field declared as `name: Type | undefined` is required: every literal must write it, possibly as `undefined`. Only `name?: Type` may be omitted.
