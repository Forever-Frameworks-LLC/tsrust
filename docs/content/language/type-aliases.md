<!-- tsrust-docs
{"id":"language-type-aliases","group":"language","order":30,"status":"partial","targets":["web","native"]}
-->
# Type aliases

`type Name = Type;` gives a name to any written type. The name and the type it stands for are the same type: a value of one is a value of the other.

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

## What an alias can name

The right side is any type that can be written in an annotation: a primitive, a union, an array, a tuple, a record or enum name, `Set` and `Map` types, a generic record with arguments, and function types. An alias can use other aliases, declared before or after it.

A record type is still declared with a body, `type Point = { x: int; y: int; };`. That form creates a new named type; an alias does not.

## Rules

- An alias is declared at the top level of a file. A declaration inside a function is a `syntax` error.
- An alias cannot refer to itself, directly or through other aliases (`recursive-alias`). Use a record type for a recursive shape.
- An alias can have type parameters and trailing defaults, such as `type Values<T = string> = T[];`. Omitted arguments use their defaults; missing required or extra arguments are `arity` errors. A default can reference an earlier parameter. Defaults on record bodies, interfaces and functions remain unsupported.
- A name can be declared once: two aliases, or an alias and a record or enum with the same name, are a `duplicate` error. Built-in type names cannot be declared.
- A type error in a use of an alias is reported at the use.
- `export type Name = Type;` exports an alias. An importing file uses it like its own aliases. The alias can name types of its own file that the importing file did not import.

## Generic aliases

```tsrust
type Values<T = string> = T[];
type Pair<A = int, B = Values<A>> = [A, B];
interface Box<T> { value: T; }
type Rows<T = int> = Box<T>[];
```

`Values` means `string[]`; `Values<int>` means `int[]`. `Pair` means
`[int, int[]]`. Parameters shadow other type names inside their alias. Aliases
preserve the expanded type's ownership, borrowing and exact integer semantics;
they do not clone resources or convert numeric values. Exported generic aliases
retain their defaults when imported.

Aliases cannot form recursive types. Expansion is limited to 65536 annotation
nodes and the existing annotation-depth limit; excessive expansion reports an
error instead of exhausting memory. The executable repository example is `examples/generic-type-aliases/`.

## In the editor

Hover on an alias name shows its declaration. Go to definition works from every use. A value whose annotation is an alias has the members of the type it stands for; its hover shows that type. Aliases appear in the outline.
