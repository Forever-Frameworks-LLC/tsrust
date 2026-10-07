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
A nonrecursive declaration can omit its result annotation. The compiler infers
its result from checked return expressions, including scalar, generic, captured and
array values. All return paths must agree on a type, and a function returning a
value must return on every reaching path. Write an explicit result type when a
recursive value-returning function needs a checking boundary. A body without
value returns remains void; main remains the void program entry.
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
- A function can transfer a captured owner, for example by returning a captured
  record or array. The original binding is unavailable after the call. A callable
  value that consumes its captured owner can be invoked only once.

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
arrow form to write instead: a generic function, and a function with rest parameters that uses variables of
its enclosing function.

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

### Returning a captured owner

Each factory invocation retains its own captured value. Calling the returned
function transfers that value; calling the same consuming function again is an error.

```tsr
type User = { name: string; };
function make(user: User): () => User {
    function take(): User { return user; }
    return take;
}
function main() {
    const take = make({ name: "Pat" });
    const user = take();
    console.log(user.name); // Pat
}
```

Transferring a capture cannot overlap a borrowed argument that uses the same value.
Use `clone()` explicitly when two independently owned copies are intended. The
focused factory fixture verifies independent returned owners. Focused retained storage, disposal, failure and recursive owner cases are verified.
Owned-escape gate172 is accepted; full signature coverage remains gate173.

### Retained callback arrays

A nonempty array can infer owned callable storage from its first callback. Every
following callback must have a compatible parameter, result and invocation contract.
Each factory invocation retains separate state.

```tsr
function make(start: int): () => int {
    let value = start;
    function next(): int { value += 1; return value; }
    return next;
}
function main() {
    const actions = [make(10), make(20)];
    console.log(actions[0](), actions[1](), actions[0]()); // 11 21 12
}
```

A callback that consumes its captures produces once-callable storage. Remove such a
callback from its array to own and invoke it. Borrowing an indexed element does not
transfer its environment. Empty arrays still require a type or call/return context.
Borrowed callback environments cannot become owned merely by entering an array.

Declaring an unused inner function does not capture its inputs for an outer call.
The compiler still checks the unused body using the completed lexical scope.
Calling or retaining that inner function makes its captures live and requires
initialization before the call or value use. An unused declaration therefore adds
no borrow to an outer call and cannot block ownership transfer of its argument.

### Captured values with optional and default parameters

A captured named function can be assigned, passed or returned with optional or
default parameters. Omitting an argument or supplying undefined runs a default
exactly once in the original function. A supplied argument skips that default,
including its side effects and possible failure. Without a default, an optional
parameter receives undefined. Existing arity and type checks remain in effect.

The evaluation fixture creates a function value with a default that increments a
counter. Calls with omission, a supplied 2 and undefined return 13, 15 and 18; the
default counter reaches 2. Array callbacks supply their normal value/index/source
arguments, so an index supplied to a second parameter overrides its default.

The wrapper forwards omission instead of evaluating defaults itself. Borrowed
captures retain their normal loan scope. A default that consumes a captured owner
can make the value once-callable; a borrowed default may remain reusable. No
implicit clone is introduced.

## Captured rest function values

A captured named function with a rest parameter can be stored or returned as a
callable. Each invocation builds its supplied rest pack in order. An empty call
passes an empty pack; call spread forwards the elements rather than an array as
one argument. Fixed parameters retain optional/default behavior, so omitted or
undefined fixed slots evaluate the original declaration's default once.

```tsr
const base = 10;
function sum(...values: int[]): int {
    let total = base;
    for (const value of values) { total += value; }
    return total;
}
const action = sum;
console.log(action(), action(2, 3)); // 10 15
const values = [4, 5];
console.log(action(...values)); // 19
```

Rest element types apply to every supplied element. Array callbacks supply the
value, index and source array; a rest parameter of int[] cannot receive that
heterogeneous tuple. A compatible fixed callback contract can adapt a rest value.
Returned values retain independent captured state. Captures and supplied owned
values follow the existing ownership rules; consuming a capture still permits only
one invocation. No implicit clone is introduced. Generic declaration values remain
pending.

## Generic values with a concrete callable context

A fixed callable annotation specializes a generic declaration when it becomes a
value. The annotation supplies parameter and result types; the original template
still checks their relationship. This works for captured declarations, returned
values with an explicit callable result, and compatible array callback contexts.

```tsr
const base = 3;
function choose<T>(value: T): T {
    console.log(base);
    return value;
}
const numberValue: (value: int) => int = choose;
console.log(numberValue(7)); // prints 3, then 7
const stringValue: (value: Owned<string>) => string = choose;
console.log(stringValue("text")); // prints 3, then text
```

An owning return consumes its string argument, so its fixed contract states
Owned<string>. A function that removes a scalar from an array mutates that array;
its contract states Mutable<int[]> and the caller observes the shorter array.
Result context can also infer a generic variable with no argument, such as an empty
array factory assigned to () => int[]. Different typed values are separate callable
values; captured local borrows keep the existing scope rules, while explicitly
returned fixed values retain their independent owned factory environments.

A capture-free generic declaration can also be stored without a fixed callable
annotation and called with different argument types. Each invocation checks the
original generic constraints and preserves argument ownership. Generic values with
captures retain one environment across their concrete calls. A concrete annotation specializes the
value; it does not provide a callable accepting every future argument type.

Concrete contexts also retain default and rest arguments. A default runs in the
original function when its argument is omitted or undefined. Null is a supplied
value and must satisfy the declared type.

```tsr
function offset<T>(value: T, index: int = 0): T { return value; }
function first<T>(...values: T[]): T { return values.remove(0); }
const choose: (value: int, index?: int) => int = offset;
const chosen = choose(7, undefined); // 7
const pick: (...values: int[]) => int = first;
const picked = pick(3, 4); // 3
```

The rest pack belongs to the invocation. It does not copy an existing caller
array implicitly. These examples have an executable assertion counterpart in
context-default-rest.tsr.

```tsr
function identity<T>(value: T): T { return value; }
const action = identity;
const one = action(7); // int
const text = action("text"); // string
const values = action([3, 4]); // int[]
```

Default arguments and homogeneous rest packs also work for these capture-free
values. An owning argument is transferred; a read-only array argument remains
available to its caller. Expected result annotations can infer result-only generic types through an
unannotated value. Polymorphic callback adaptation and complete generic alias
parameter help remain unsupported.

A captured generic declaration can also be stored without a fixed annotation.
Local borrowed captures retain their existing scope rules. Returned factories
retain their own environment; two factory calls have independent state, while
calls with different argument types use the same state within each value.

```tsr
function make(start: int) {
    let calls = start;
    function select<T>(value: T): T {
        calls += 1;
        console.log(calls);
        return value;
    }
    return select;
}
function main() {
    const first = make(10);
    const second = make(20);
    first(1);       // prints 11
    second("x");   // prints 21
    first("y");    // prints 12
}
```

An environment that returns an owned capture consumes that owner and cannot be
called again. Arguments retain their declared access; changing the argument type
does not clone the environment or its owners. Defaults run in the original function
once when omitted or undefined. Generic rest packs belong to each invocation.
The generic_function_values and generic_value_signatures fixtures check these
behaviors, including exact native cleanup after success and caught failures.

An expected result type also supplies generic inference when a stored function
has no typed arguments. Different result types select different invocation methods
while retaining the same captured environment.

```tsr
function empty<T>(): T[] { return []; }
function main() {
    const action = empty;
    const numbers: int[] = action();
    const strings: string[] = action();
    numbers.push(7);
    strings.push("text");
}
```

The result-context fixture also checks a captured factory with two calls. Result
context does not relax parameter types or required argument counts. A result-only
call with neither argument nor expected result information still needs type context.

Generic function values retain their source generic signature in editor hover
and completion, including values imported from another module or returned from
a captured factory. Parameter help at a checked call shows concrete argument
types and ownership access. For example, an identity call with a string reports
`value: Owned<string>` because returning that value transfers its ownership.

An existing generic function value can also be assigned to a concrete callable
annotation. A `Mutable<Function<...>>` view borrows its environment until the view
leaves scope. A default owned `Function<...>` value transfers that environment;
the original value then becomes unavailable. Adapters preserve captured state
rather than constructing another factory instance. The executable examples in
crates/tsrust-compiler/tests/fixtures/generic-value-adapters cover concrete contracts, map callbacks and
record/array storage. Some other callback and nullable contexts remain unsupported.

A concrete owned callback can select between factories returning different generic
function environments. For example, `const action: Function<int, [value: int]> =
(flag ? firstFactory : secondFactory)();` constructs only the selected environment.
Subsequent calls use that environment's retained state. Array traversal also
accepts such selected factory calls when their generic signatures match its input
and result requirements. A borrowed facade cannot outlive a fresh temporary owner.
The selected-factories and selected-traversal fixtures demonstrate these cases.
