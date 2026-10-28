---
title: Name types with aliases
description: Reuse type expressions, supply generic defaults and export shared names.
---

`type Name = Type;` gives an existing type another name. A value accepted by the
alias is accepted by its expanded type, and vice versa. An alias does not create a
new wrapper, convert values or copy resources.

## Name a reusable type

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

## Supply type arguments and defaults

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

## Export an alias

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

## Diagnose invalid aliases

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

## Inspect aliases in the editor

Hover shows the alias declaration; go to definition navigates to it. Aliases
appear in the outline. Values annotated with aliases offer the members of the
expanded type, including aliases imported from another file.
