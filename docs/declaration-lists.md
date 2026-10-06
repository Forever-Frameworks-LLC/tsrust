# Declaration lists and loop update lists

## Several names in one declaration

A `const` or `let` declaration can introduce several names, separated by commas. Each name has its own initializer and may have its own type annotation.

```tsrust
import { strictEqual } from "std:testing";
function main() {
    let width = 3, height = 4, label: string = "area";
    const area = width * height, text = label + "=" + String(area);
    strictEqual(text, "area=12");
    width = 5;
    strictEqual(width * height, 20);
}
```

- The initializers run in source order, left to right.
- A later initializer can read an earlier name of the same declaration. Reading a later name is a `name` error.
- The keyword applies to every name: all are `const`, or all are `let`.
- Every name needs an initializer. `let a = 1, b;` is a `syntax` error.
- A name can appear once. `let a = 1, a = 2;` is a `duplicate` error.

A declarator can be a destructuring pattern, mixed with plain names:

```tsrust
import { strictEqual } from "std:testing";
type Point = { x: int; y: int; };
function main() {
    const origin: Point = { x: 7, y: 8 };
    const first = 4, [second, third] = [5, 6], { x, y: z } = origin;
    strictEqual(first + second + third + x + z, 30);
}
```

Each declarator follows the ownership rules it has when written as its own declaration. A list does not copy or borrow anything extra. See [Destructuring](destructuring.md) for pattern rules.

## In a classic for loop

The initializer of a classic `for` loop accepts a declaration list, and the update clause accepts several expressions separated by commas.

```tsrust
import { strictEqual } from "std:testing";
function main() {
    let total = 0;
    for (let i = 0, j = 10; i < j; i++, j--) {
        total += j - i;
    }
    strictEqual(total, 30);
}
```

- All names of the initializer are scoped to the loop.
- The update expressions run in source order after each iteration, including after `continue`. `break` skips them.
- Each update is an assignment, an increment or decrement, or a call such as `seen.push(i)`. A declaration in the update clause is a `syntax` error.

## Limits

The comma separates declarators and update expressions only. A comma expression in other positions, such as `x = (a, b);`, is not available. A trailing comma in a declaration list or update list is a `syntax` error.
