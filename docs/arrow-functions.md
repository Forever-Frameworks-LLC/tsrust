# Arrow functions

An arrow expression creates a callable value. Array methods supply the element type and index type, so their callbacks can omit annotations:

```tsrust
const users = ['Alice', 'Bob', 'Charlie'];
const newUsers = users.map(user => user.toUpperCase());
console.log(newUsers.join('|')); // ALICE|BOB|CHARLIE
```

Parentheses are optional for one unannotated parameter. Use parentheses for multiple parameters or type annotations. An expression body returns its value automatically; a block uses explicit return statements:

```tsrust
const upper = users.map((user) => user.toUpperCase());
const same = users.map((user) => { return user.toUpperCase(); });
const add = (value: int) => value + 1;
console.log(add(4)); // 5
```

A declared callback contract also supplies parameter context. Without context, annotate parameters; Tsrust does not silently introduce any. Return annotations are optional when checked returns establish a compatible result type. Parenthesize an object expression body, such as `(): Row => ({ value: 7 })`, to distinguish it from a statement block.

Map, filter, predicates, forEach, sorting and seeded reductions retain their existing callback arity and access rules. Arrow syntax does not relax callable ownership, escape checks or receiver loans. Captured values and single-use environments follow the same rules as explicitly typed block arrows. Formatting preserves the arrow's tokens, and inferred parameter/result types feed editor hover and completion.

## Function expressions

`function (...) { ... }` written where a value is expected is an anonymous function value. It behaves like an arrow function with a block body: the same parameter forms, result inference, captures and ownership rules apply.

```tsrust
import { strictEqual } from "std:testing";
function apply(action: Function<int, [int]>, value: int): int { return action(value); }
function main() {
    const double = function (value: int): int { return value * 2; };
    strictEqual(apply(double, 5), 10);
    const offset = 3;
    const shift = function (value: int) { return value + offset; };
    strictEqual(shift(1), 4);
    const words = ["b", "a"];
    const upper = words.map(function (word) { return word.toUpperCase(); });
    strictEqual(upper.join(""), "BA");
}
```

- Parameters accept annotations, defaults, rest and destructuring patterns. A callback parameter can leave its type to the method that receives it, as with an arrow.
- The result type may be stated or inferred.
- The body is a block. For an expression body, write an arrow function.
- A function expression cannot have a name. Bind it to a `const`, or declare the function at the top level of the file.
- A statement that starts with `function` inside a function body is a `syntax` error, because function declarations belong at the top level.
