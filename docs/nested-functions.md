# Nested named functions

A named function may be declared inside a function or an explicit block. Its name
is available throughout that block, including before the declaration. Nested
blocks may declare the same name without changing the enclosing declaration.
A declaration does not leak outside its block. A function named main inside another
function is an ordinary local function; only the file entry function runs implicitly.

```tsr
function main() {
    console.log(double(3)); // 6

    function double(value: int): int {
        return value + value;
    }
}
```

Capture-free declarations support self recursion, mutual recursion and use as
callable values. Parameter and result annotations use ordinary declaration syntax.
Basic generic, default and rest signatures share the top-level implementation.
Write the result annotation when returning a value: an omitted named-declaration
result currently means void. Nonrecursive result inference remains planned.
The native fixtures in tests/fixtures/nested-function-declarations verify forward
calls, factorial, mutually recursive even/odd functions, block shadowing, Array.map
callbacks and generic/default/rest signatures with assertions.

A declaration conflicts with another declaration of the same name in its block.
A function-body declaration also cannot replace a parameter of that function.
A closure parameter or a declaration in a nested block may shadow the name.
Function declarations require a braced body and cannot be a lone unbraced if/loop
body. Named function expressions are still a separate unsupported syntax feature.

## Variables of the enclosing function

A nested function can read and change the variables of the functions around it.
It works on the variables themselves, not on copies: a change made inside the
function is visible after the call, and the function sees the current value at
each call.

```tsr
import { strictEqual } from "std:testing";

function main() {
    let total = 0;
    const history: int[] = [];

    function add(amount: int): void {
        total += amount;
        history.push(amount);
    }
    function addTwice(amount: int): void {
        add(amount);
        add(amount);
    }

    addTwice(2);
    add(1);
    strictEqual(total, 5);
    strictEqual(history.join(","), "2,2,1");
}
```

Each call borrows the variables the function uses, directly or through another
nested function it calls, and returns them when the call ends. The ordinary
ownership rules decide what a call may do:

- A variable that the function changes must be changeable: declare a number,
  boolean or string with `let`. Changing a `const` scalar is rejected.
- The variable must have a value on every path that reaches the call. A function
  may be declared before the variable receives its value; the check is made where
  the function is called or used as a value, and the diagnostic links the variable.
- An argument cannot lend a variable in a way that conflicts with what the function
  does to it. `grow(counts)` is rejected when `grow` also changes `counts` itself,
  while `tick(counts.length)` is fine because the argument is only a copied number.
- A view of an element, such as `const first = names[0]`, cannot stay in use across
  a call that changes `names`.
- A function cannot take ownership of a captured variable, for example by returning
  a captured array or pushing a captured string into another array. Copy it with
  `clone()` or pass the value as a parameter.

A function declared in a loop body or block uses the variables of that iteration
or block entry. Recursive and mutually recursive functions share the variables of
their enclosing function, and every call of a function has its own locals for the
functions nested inside it.

Parameters with defaults, rest parameters, callback parameters and type parameters
work the same way as on top-level functions. A default value may use a captured
variable.

A nested function that no call reaches is still type checked.

### Using a nested function as a value

A nested function can be passed or stored where a callable is expected. The value
behaves exactly like the arrow function `(parameters) => name(parameters)`.

```tsr
import { strictEqual } from "std:testing";

function main() {
    const base = [10];
    function shifted(value: int): int {
        return value + base[0];
    }
    strictEqual([1, 2].map(shifted).join(","), "11,12");

    const counts = [0];
    function tick(): int {
        counts[0] += 1;
        return counts[0];
    }
    {
        const run = tick;
        strictEqual(run(), 1);
        strictEqual(run(), 2);
    }
    strictEqual(counts[0], 2);
}
```

While such a value is alive it holds the variables its function uses, as an arrow
function would: in the example, `counts` cannot be read until the block that
declares `run` ends. A callable handed to a callback parameter of a declared
function, or returned from the enclosing function, takes ownership of the variables
it needs; they are no longer available afterwards.

Two forms cannot be used as values yet and report a diagnostic that names the
arrow form to write instead: a generic function, and a function with optional,
default or rest parameters that uses variables of its enclosing function.

Completion includes hoisted declarations throughout their block. Hover and signature
help show source names and checked types, including imported nominal results.
Definitions and references respect shadowing; the document outline nests local
functions under their enclosing function. A later local binding masks an outer name
without being offered before its declaration. Formatting retains block structure and
blank lines. Inside a nested function, hover, definition, references and completion
resolve the variables of the enclosing functions.

Inner block variables can shadow outer variables, and leaving the block restores
the outer binding. Arrow parameters can shadow names in their enclosing scope.
A local in a function's own body still cannot redeclare one of its parameters.
These scope rules preserve each binding's identity and ownership independently.
