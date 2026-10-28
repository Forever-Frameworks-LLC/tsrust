---
title: Write statement bodies
description: Use single-statement bodies, attach else branches and understand loop scope.
---

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


## Choose a single statement

The body can be an expression or assignment, `return`, `break`, `continue`, or another `if` or loop. Bodies nest: `for (...) for (...) if (...) work();`.

## Attach an else branch

An `else` belongs to the nearest `if` that has no `else` yet. `if (a) if (b) x(); else y();` runs `y()` when `a` is true and `b` is false. Use braces to attach the `else` to the outer `if`.

## Use braces for declarations

A `const`, `let` or `function` declaration cannot be a body without braces. Put it in a block.

## Write an empty body

An empty statement is not a body. `if (ready) ;` is a `syntax` error; write `{}` for an empty body.

## Scope and nesting

A body without braces counts toward the nesting limit of 128, like a block.

Names declared in a loop header are scoped to the loop whether or not its body has braces. A destructuring assignment can be a body: `if (a < b) [a, b] = [b, a];`.
