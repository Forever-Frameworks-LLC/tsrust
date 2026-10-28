# Declarations without an initializer

A `let` declaration can leave out its initializer when the value is decided later. The declaration needs a type annotation.

```tsrust
import { strictEqual } from "std:testing";

function grade(score: int): string {
    let label: string;
    if (score >= 90) {
        label = "high";
    } else if (score >= 50) {
        label = "mid";
    } else {
        label = "low";
    }
    return label;
}

function main() {
    strictEqual(grade(95), "high");
    strictEqual(grade(60), "mid");
    strictEqual(grade(10), "low");
    let width: int, height: int = 3;
    width = height * 2;
    strictEqual(width, 6);
    let note: string | undefined;
    strictEqual(note === undefined, true);
}
```

## Assignment before use

The variable has no value until it is assigned. Every use must come after an assignment on every path that reaches it. A use that can be reached without an assignment is an `unassigned` error.

The compiler accepts these forms:

- A plain assignment `name = value;` in the same block or in a nested block.
- An `if` statement in which every branch assigns the variable or leaves with `return`, `break` or `continue`. An `if` without an `else` does not count, because the condition can be false.
- A `switch` statement with a `default` clause in which every clause assigns the variable or leaves.

These do not count as an assignment for the code that follows:

- An assignment inside the body of `while`, `for`, `for...of` or `do...while`. The rest of the same loop iteration can use the variable; code after the loop cannot.
- A compound assignment such as `name += 1`, because it reads the variable first.

Before the variable is assigned, a function value cannot mention it:

```text
let total: int;
const read = (): int => total;   // unassigned
total = 1;
```

Assign the variable first, or give the declaration an initializer.

## Types that include `undefined`

When the annotated type includes `undefined`, the variable starts as `undefined` and can be read at once. `let note: string | undefined;` is the same as `let note: string | undefined = undefined;`.

## Rules

- Only `let` can omit the initializer. `const name: int;` is a `syntax` error.
- The annotation is required: `let name;` is an `annotation-required` error.
- A function-typed variable needs an initializer.
- A destructuring pattern needs an initializer.
- The declaration can be one declarator of a declaration list and the initializer clause of a `for` statement.
- After the first assignment the variable is an ordinary `let` variable, with the ordinary ownership rules.
