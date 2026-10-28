---
title: Write function expressions
description: Create anonymous callable values or give an expression a private recursive name.
---

`function (...) { ... }` in a value position creates an anonymous callable. It uses
the same checked parameter, result, capture and ownership contracts as a block
[arrow function](/tsrust/language/arrow-functions/).

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Create an anonymous callable

The result type may be stated or inferred. A receiving callback contract can
supply omitted parameter types. Without context, annotate the parameters.

```tsrust
function apply(action: Function<int, [int]>, value: int): int {
  return action(value);
}

function main() {
  const double = function (value: int): int {
    return value * 2;
  };
  console.log(apply(double, 5)); // 10
  const offset = 3;
  const shift = function (value: int) {
    return value + offset;
  };
  console.log(shift(1)); // 4
  const words = ["b", "a"];
  const upper = words.map(function (word) {
    return word.toUpperCase();
  });
  console.log(upper.join("")); // BA
}
```

Parameters support annotations, defaults, rest and destructuring patterns. The
body must be a block; use an arrow for an expression body. A `function` expression
has no trailing `=>` between its signature and body.

A statement beginning with `function` and a name declares a nested function.
An anonymous `function` at the start of a statement is a `syntax` error: use the
expression where a value is expected. See
[nested declarations](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/docs/guides/language/nested-functions.md).

## Give an expression a private recursive name

A named function expression can call itself through its private name. That name
does not declare a binding in the surrounding block or change an outer binding
with the same spelling.

```tsrust
function main() {
  const factorial = function fact(value: int): int {
    return value <= 1 ? 1 : value * fact(value - 1);
  };
  console.log(factorial(5)); // 120
  const seen: int[] = [];
  {
    const walk = function down(depth: int): void {
      if (depth === 0) {
        return;
      }

      seen.push(depth);
      down(depth - 1);
    };
    walk(3);
  }

  console.log(seen.join(",")); // 3,2,1
  const squares = [1, 2, 3].map(function square(value: int): int {
    return value * value;
  });
  console.log(squares.join(",")); // 1,4,9
}
```

## Respect the enclosing capture scope

A live function value retains its captured owners or capture loans. In the
example, leaving the `walk` block releases its mutable capture of `seen` before
the final read. Ownership, conflicting access and capture
initialization checks apply to recursive calls too.

Named expressions use the nested declaration rules for defaults, rest parameters,
type parameters and result inference. An omitted result needs consistent checked
returns; annotate a recursive result when inference cannot establish it.

## Current name and default rules

The private function name cannot be assigned inside the body. A parameter may
reuse its spelling, and the private name is unavailable in sibling expressions or
after the function expression.

A named function expression is not accepted as a parameter default. Use an arrow
there, or declare the function separately and pass its name. Other default
expressions retain their ordinary scope and call rules.
