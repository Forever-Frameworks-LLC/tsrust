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
- A statement that starts with `function` and a name is a nested function declaration; see [nested functions](nested-functions.md). Without a name it is a `syntax` error, because a function expression is a value.

### Named function expressions

A function expression can have a name. The name is visible only inside that function, so the function can call itself. It does not declare anything in the surrounding block, and a variable with the same spelling outside is untouched.

```tsrust
import { strictEqual } from "std:testing";

function main() {
    const factorial = function fact(value: int): int {
        return value <= 1 ? 1 : value * fact(value - 1);
    };
    strictEqual(factorial(5), 120);

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

    strictEqual(seen.join(","), "3,2,1");
    strictEqual([1, 2, 3].map(function square(value: int): int {
        return value * value;
    }).join(","), "1,4,9");
}
```

- Everything else follows nested function declarations: parameter defaults, rest and type parameters, an omitted result type, and the rules for variables of the enclosing function. A function that calls itself infers its result from a return that does not depend on the call, as described for nested functions.
- While the function value is alive it holds the variables its body uses, exactly like a nested function used as a value. In the example `seen` is free again after the block that declares `walk`.
- Inside the function the name cannot be assigned, and a parameter may reuse it.
- A named function expression cannot be a parameter default yet. Use an arrow function there, or declare the function and write its name.
