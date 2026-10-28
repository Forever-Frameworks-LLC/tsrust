# Statement bodies without braces

The body of `if`, `else`, `while`, `do`, classic `for` and `for...of` can be one statement without braces. The statement forms a block of its own.

```tsrust
import { strictEqual } from "std:testing";
function sign(value: int): string {
    if (value < 0) return "negative";
    else if (value === 0) return "zero";
    else return "positive";
}
function main() {
    strictEqual(sign(-2), "negative");
    let total = 0;
    for (let i = 0; i < 4; i++) total += i;
    for (const value of [10, 20]) total += value;
    while (total > 30) total--;
    strictEqual(total, 30);
}
```

- The body is any single statement: an expression or assignment, `return`, `break`, `continue`, or another `if` or loop. Bodies nest: `for (...) for (...) if (...) work();`.
- An `else` belongs to the nearest `if` that has no `else` yet. `if (a) if (b) x(); else y();` runs `y()` when `a` is true and `b` is false. Use braces to attach the `else` to the outer `if`.
- A `const` or `let` declaration cannot be a body without braces, because nothing could use the name. Put it in a block.
- An empty statement is not a body. `if (ready) ;` is a `syntax` error; write `{}` for an empty body.
- A body without braces counts toward the nesting limit of 128, like a block.

Names declared in a loop header are scoped to the loop whether or not its body has braces. A destructuring assignment can be a body: `if (a < b) [a, b] = [b, a];`.
