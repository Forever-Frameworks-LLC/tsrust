---
title: "Functions"
description: "Declare functions, arrow and function expressions, parameters, spread arguments, nested functions and captures."
---

Functions have checked parameter and result types. A parameter that receives owned data says how the function uses it (see [ownership wrappers](../../library/globals/#ownership-wrappers)). Arrow functions and function expressions create callable values; nested function declarations keep helpers close to their use and can capture enclosing bindings. Optional, default and rest parameters and spread arguments are checked against the declaration. Storing and passing function values is covered in [function values](../function-values/); generic functions in [generics](../generics/).

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/) and run `tsrust run main.tsr`. Examples that use `strictEqual` finish without output when their assertions pass; others show expected output in comments.

## Arrow functions

An arrow expression creates a callable value. Methods such as `map` supply checked
parameter context, so their callbacks can omit annotations. Without that context,
annotate the parameters; Tsrust does not silently introduce `any`.

### Infer callback parameters from context

Array callbacks receive the method's checked element and index types. The result
is inferred from the arrow body when its checked returns establish a compatible
type.

```tsrust
function main() {
  const users = ["Alice", "Bob", "Charlie"];
  const upper = users.map(user => user.toUpperCase());
  console.log(upper.join("|")); // ALICE|BOB|CHARLIE
}
```

A declared callable contract also supplies parameter context. Inference preserves
the contract's argument access and result requirements.

### Choose an expression or block body

Parentheses are optional for one unannotated parameter. Use them for multiple
parameters or type annotations. An expression body returns its value automatically;
a block body uses explicit `return` statements.

```tsrust
function main() {
  const values = [1, 2];
  const expression = values.map(value => value + 1);
  const block = values.map((value) => {
    return value + 1;
  });
  const add = (value: int) => value + 1;
  console.log(expression.join(","), block.join(","), add(4)); // 2,3 2,3 5
}
```

A result annotation is optional when the checker can infer a consistent result.
A function that returns a value must return on every reaching path. Parenthesize
an object expression body to distinguish it from a statement block.

### Use a declared contract or return a record

```tsrust
type Row = {
  value: int;
};

function apply(action: (input: ReadOnly<string>) => string): string {
  return action("Pat");
}

function main() {
  console.log(apply(value => value.toUpperCase())); // PAT
  const make = (): Row => ({
    value: 7
  });
  console.log(make().value); // 7
}
```

### Keep ownership and call rules explicit

Arrow syntax preserves the method's parameter count, argument access and receiver
loans. This applies to mapping, filtering, predicates, iteration, sorting and seeded
reductions. Captured owners, scoped borrows and single-use invocation follow the
same contracts as explicitly annotated block arrows.

Defaults, rest parameters and destructuring use their normal checked rules. See
[optional and default parameters](/tsrust/language/functions/#optional-and-default-parameters),
[rest parameters](/tsrust/language/functions/#rest-parameters) and
[local callable ownership](/tsrust/language/function-values/#local-function-values).

Inferred parameter and result types feed editor hover, completion and signature
help. The formatter preserves source tokens and makes arrow layout consistent.
For the block-bodied `function` spelling and private recursive names, see
[function expressions](/tsrust/language/functions/#function-expressions).

## Function expressions

`function (...) { ... }` in a value position creates an anonymous callable. It uses
the same checked parameter, result, capture and ownership contracts as a block
[arrow function](/tsrust/language/functions/#arrow-functions).

### Create an anonymous callable

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
[nested declarations](/tsrust/language/functions/#nested-functions).

### Give an expression a private recursive name

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

### Respect the enclosing capture scope

A live function value retains its captured owners or capture loans. In the
example, leaving the `walk` block releases its mutable capture of `seen` before
the final read. Ownership, conflicting access and capture
initialization checks apply to recursive calls too.

Named expressions use the nested declaration rules for defaults, rest parameters,
type parameters and result inference. An omitted result needs consistent checked
returns; annotate a recursive result when inference cannot establish it.

### Current name and default rules

The private function name cannot be assigned inside the body. A parameter may
reuse its spelling, and the private name is unavailable in sibling expressions or
after the function expression.

A named function expression is not accepted as a parameter default. Use an arrow
there, or declare the function separately and pass its name. Other default
expressions retain their ordinary scope and call rules.

## Optional and default parameters

### Optional arguments

`value?: T` accepts a value of type `T`, omission, or explicit `undefined`. Inside the function, the value has type `T | undefined`. Narrow it or use a nullish operation before an operation that requires `T`.

```tsrust
function show(value?: int): string {
  return String(value ?? 7);
}

function main() {
  console.log(show(), show(undefined), show(0)); // 7 7 0
}
```

An optional marker does not permit `null` unless the declared type includes it. Optional parameters cannot precede required parameters. Extra arguments remain errors.

### Defaults and null

`value: T = expression` evaluates the default only when the argument is omitted or explicitly `undefined`. A supplied `null`, zero, false, or empty string does not trigger a default. The initializer must satisfy the declared type.

```tsrust
function pick(value: int | null = 5): int | null {
  return value;
}

function main() {
  console.log(String(pick()), String(pick(undefined)), String(pick(null))); // 5 5 null
}
```

The accepted call type includes `undefined`, but the initialized body type excludes it when neither the declared type nor the default retains it. An explicit `T | undefined` declaration retains that union; flow analysis may narrow an initialized occurrence when its value is known to be present. Defaults can call functions and construct strings, arrays, records, or supported callable values. Existing access and origin checks still apply.

Callable type annotations describe omission with `?`. Initializer expressions belong to executable function or arrow declarations, not callable type annotations.

### Evaluation order and scope

The callee and supplied arguments evaluate once, in source order, before defaults. Reached defaults then evaluate once, in parameter order. Earlier initialized parameters are available to later defaults. The current parameter, later parameters, and body-local declarations are unavailable in a default. Existing duplicate and reserved binding-name restrictions still apply.

```tsrust
function mark(trace: int[], value: int): int {
  trace.push(value);
  return value;
}

function add(trace: int[], left: int = mark(trace, 1), right: int = mark(trace, left + 1)): int {
  return left + right;
}

function main() {
  const trace: int[] = [];
  console.log(add(trace)); // 3
  console.log(add(trace, 10)); // 21
  console.log(trace.join(",")); // 1,2,11
}
```

The second call skips the `left` initializer. Its `right` initializer sees the supplied value `10`.

### Defaults before required arguments

A default may precede a required parameter, but its position still needs an argument. Pass `undefined` to select that default.

```tsrust
function choose(first: int = 5, required: int): int {
  return first + required;
}

function main() {
  console.log(choose(undefined, 2)); // 7
}
```

`choose(2)` is an arity error. A callable value representing this function has a required first slot accepting `int | undefined`, rather than an optional slot before a required slot. Forwarding invokes the selected function's default; the wrapper does not initialize that default itself.

### Callbacks and ownership

Array methods retain their existing callback arguments. For data-element callbacks, `map` supplies the value, index, and source array. An additional optional or defaulted trailing callback parameter receives `undefined`; a required extra parameter is rejected. Mutable callable-element visitors retain their existing slot limits and source-array loan restrictions. Initializers run only when the callback is actually invoked, so an empty traversal creates no default value.

Defaults preserve ordinary ownership rules. A skipped initializer creates no owner. A reached owned default is local to that invocation and is disposed when its owner leaves scope, including unwinding. Borrowed defaults do not permit an escaping borrow, duplicate mutable loans, or consumption through a read contract. Once callables still require consuming invocation, and existing optional-view restrictions remain in force.

### Current boundaries

Defaults do not add JavaScript coercions or change native string, integer, or collection behavior. Strings retain UTF-8 semantics and ordinary native integer limits apply. Parameter rebinding remains prohibited; mutable access permits supported receiver mutation, not assignment to the parameter binding. Rest parameters, destructuring patterns and nullable array elements are supported under their own type and ownership rules. Defaults do not relax those rules. See [rest parameters](/tsrust/language/functions/#rest-parameters), [pattern defaults](/tsrust/language/patterns/#defaults-and-rest) and [nullish assignment](/tsrust/language/optional-values/#nullish-assignment).

## Rest parameters

### Collect remaining arguments

A terminal `...values: T[]` parameter collects the remaining supplied arguments into a fresh owning array, in order. No remaining arguments produces an empty array. The annotation must be a supported array type. Its element type can include checked union alternatives, including null or undefined. Every supplied tail argument must fit that element type.

```tsrust
function sum(...values: int[]): int {
  return values.reduce((total: int, value: int): int => {
    return total + value;
  }, 0);
}

function main() {
  console.log(sum(), sum(4), sum(1, 2, 3)); // 0 4 6
}
```

A rest parameter must be last. It cannot have `?`, a default initializer or a read/mutate access modifier. Its array owns the collected values. Callable annotations use the same rest syntax, such as `ReadOnly<Function<int, [...values: int[]]>>`.

### Fixed parameters and supplied values

Fixed parameters retain their required, optional and default behavior. All supplied arguments evaluate once, left to right, before reached fixed defaults run. A default does not add an element to the rest array. An explicit `undefined` in the tail is an actual element and must satisfy its element type.

```tsrust
function absent(...values: undefined[]): int {
  return values.length;
}

function main() {
  console.log(absent(), absent(undefined)); // 0 1
}
```

Calling an `int[]` rest parameter with `undefined` is a type error. Empty generic rest calls need enough type information to infer the element type; an explicit type argument can supply it.

### Ownership

Scalar leaves copy into the pack. Owned strings, records, nested arrays and stored callable environments transfer into it, following array-literal rules. Reading the array in the function does not clone those arguments. Explicitly clone an owner before passing it when an independent retained owner is needed. This differs from JavaScript object-reference sharing.

Changing the rest array changes its own collection. Fixed-prefix borrowing and mutation contracts remain unchanged. Strings retain native UTF-8 behavior; `string.length` counts bytes, and native integer limits remain in force. Parameter rebinding remains prohibited.

### Array callbacks

Array methods supply their existing argument tuples. A rest suffix receives only the supplied positions beyond its fixed prefix. Ordinary value visitors supply value, index and source array; reductions also supply the accumulator. Mutable callable-element visitors retain their shorter tuples and source-array loan restrictions. Rest packs cannot transfer a borrowed owner out of those tuples.

```tsrust
function main() {
  const result = [3, 1, 2].toSorted((...pair: int[]): int => {
    return pair[0] - pair[1];
  });
  console.log(result.join(",")); // 1,2,3
}
```

An optional or defaulted fixed callback slot beyond the supplied tuple remains omitted. Its padding never becomes a rest element. Callback defaults and pack creation happen only on actual invocation.

### Current boundaries

[Call spread](/tsrust/language/functions/#spread-arguments) expands arrays into rest and fixed parameters. Array-literal spread and supported iterable expansion have their own [spread rules](/tsrust/language/arrays/#array-spread). Rest parameters collect arguments into an array rather than inferring a heterogeneous tuple. Mixed and nullable elements follow the checked array element type. Borrowed callable environments cannot become owning rest-array elements.

### Collect a union element type

A rest array can contain different allowed alternatives. This example prints
`1,two,`: `join` renders its null element as empty text.

```tsrust
function show(...values: (int | string | null)[]): string {
  return values.join(",");
}

function main() {
  console.log(show(1, "two", null));
}
```

## Spread arguments

### Expand an iterable into arguments

`...values` in a call argument list expands a dense array, string, Set, Map, Deque, Heap, OrderedSet or OrderedMap into individual arguments, in iteration order. A call can mix ordinary arguments and any number of spread arguments. An empty source supplies no arguments.

```tsrust
function total(first: int = 5, ...tail: int[]): int {
  let result = first;
  for (const value of tail) {
    result += value;
  }

  return result;
}

function main() {
  const values = [1, 2];
  console.log(total(0, ...values, 3), total(...values), total(...[])); // 6 3 5
}
```

The expanded values feed the target's existing fixed, optional, default and rest parameters. A default runs when its position receives no argument or receives explicit `undefined`: `total(...[])` supplies nothing, so `first` takes its default and the rest array is empty. Padding for an omitted parameter never becomes a rest element.

### Argument counts

An array literal has a known length, so the compiler checks the expanded count against the target. Other arrays, strings and collections have an expanded length checked when the program runs. The compiler then checks every position that the source could reach, and the program checks the actual count before the call.

```tsrust
function area(width: int, height: int): int {
  return width * height;
}

function main() {
  const size = [3, 4];
  console.log(area(...size)); // 12
}
```

`area(...[3, 4, 5])` is rejected with an `arity` error. Calling `area(...short)` with a one-element source throws a structured TypeError with code `call-arity`, message `expanded call arity outside checked contract` and the source byte range of the call. A catch handler can recover, and finally runs normally.

The count check happens after all arguments evaluate and before the target, its defaults or its body run. Extra arguments are never dropped.

Every element that could reach a parameter must satisfy that parameter's type. A source that could reach parameters of incompatible types is rejected at the spread expression.

### Evaluation order

The callee or method receiver evaluates once, before any argument. Arguments and spread sources then evaluate once each, left to right. A spread reads its elements when it is reached, and all supplied values are held before the target runs.

```tsrust
function trace(log: int[], value: int): int {
  log.push(value);
  return value;
}

function pair(log: int[]): int[] {
  log.push(2);
  return [20, 21];
}

function sum(...values: int[]): int {
  let total = 0;
  for (const value of values) {
    total += value;
  }

  return total;
}

function main() {
  const log: int[] = [];
  console.log(sum(trace(log, 1), ...pair(log), trace(log, 3)), log.join(",")); // 45 1,2,3
    let optional: (ReadOnly<Function<int, [...values: int[]]>>) | null = null;
  console.log(String(optional?.(...pair(log))), log.length); // undefined 3
}
```

An optional call with an absent callee skips its arguments, including spread sources: `pair` does not run in the last line.

### Ownership

Scalar elements are copied. A parameter that reads its argument borrows the element in place, and a parameter that changes its argument changes the source element.

```tsrust
function grow(values: int[]): int {
  values.push(9);
  return values.length;
}

function main() {
  const rows: int[][] = [[1]];
  console.log(grow(...rows), rows[0].join(",")); // 2 1,9
}
```

Owned elements such as strings, records, nested arrays and stored callables transfer into a parameter that takes ownership, including a rest array. The source array is then no longer available. Clone the source first when it must stay usable.

```tsrust
function words(...values: string[]): string[] {
  return values;
}

function main() {
  const source = ["é".repeat(1), "x".repeat(1)];
  const kept = words(...clone(source));
  console.log(kept.join("|"), source.join("|")); // é|x é|x
    const moved = words(...source);
  console.log(moved.length); // 2
}
```

Using `source` after `words(...source)` is a `transferred` error. Nothing is cloned implicitly, which differs from JavaScript reference sharing. A borrowed element cannot move into an owning rest array, and a later argument cannot change an array that an earlier spread still borrows.

### Built-in calls

Spread works with built-in functions and methods that accept the expanded arguments under their existing contracts.

```tsrust
function main() {
  const numbers = [1.5, 3.0, 2.0];
  console.log(Math.max(...numbers), Math.min(0.5, ...numbers)); // 3 0.5
    console.log("a".concat(...["b", "c"])); // abc
    const list = [3];
  list.unshift(...[1, 2]);
  console.log(list.join(","), list.concat(...[[4], [5]]).join(","), Array.of<int>(...list, 4).join(",")); // 1,2,3 1,2,3,4,5 1,2,3,4
}
```

A built-in with a fixed argument count keeps that count: `list.push(...[2, 3])` is an `arity` error because `push` accepts one value. Console calls accept spread sources of one element type.

### Callable values

Spread calls work through named functions, generic and imported functions, closures, callable parameters, record fields holding callables, and nullable callables. Use `?.(...)` for an optional call, or narrow a nullable callable with `!== null` or `!== undefined` before calling it. A `Once<Function<...>>` callable cannot be invoked through `?.`; narrow it first.

### Supported targets and boundaries

Supported sources are dense arrays, strings, Sets, Maps, Deques, Heaps, OrderedSets and OrderedMaps. Named functions, methods and callable expressions keep their existing checked contracts. Generic inference uses expanded element types rather than the source collection type. Sparse slots and arbitrary iterator objects are unsupported. Heterogeneous tuple arrays and declared union elements retain ordinary parameter compatibility rules.

### String arguments

Strings construct owning Unicode scalar character strings, including supplementary
characters. The source stays available, even when a parameter consumes its
character argument or captures it in a returned function. Combining marks remain
separate characters. Native string indexing and length still use UTF-8 bytes.

```tsrust
function join(a: string, b: string, c: string): string {
  return a + "|" + b + "|" + c;
}

function main() {
  const text = String("Aé🙂");
  console.log(join(...text), text); // prints A|é|🙂 Aé🙂
}
```

### Collection arguments

Sets supply insertion-ordered keys. Maps supply borrowed stored key/value tuples
for readonly parameters, including tuples with noncloneable values. A consuming
parameter or owning rest array transfers owning collection elements. Scalars copy.
The aggregate access required by reachable parameters governs the whole source,
as with array call spread. No implicit clone is added.

Mutable borrowing of stored Set, Heap or OrderedSet keys and Map or OrderedMap
entry tuples is rejected because changing keys would invalidate lookup or ordering. Explicitly copy to an array
first when a detached mutable snapshot is wanted and its elements can be copied.
Readonly owning inputs cannot transfer; readonly scalar Sets can supply copies.

#### Iteration order

Deque supplies front-to-back values. OrderedSet supplies ascending keys and
OrderedMap supplies entries in ascending key order. Heap spread order is
unspecified; use repeated `pop()` when descending order is needed. OrderedMap
entries follow the same borrowing and transfer rules as Map entries.

#### Built-in and contextual arguments

Expected parameter types can supply types for empty collection constructors.
`String.raw` additionally accepts string and `Set<string>` spread substitutions; it
keeps its existing explicit output-copy behavior and preserves source values.
`String.fromCharCode`, `String.fromCodePoint` and variadic Math operations accept
compatible Set elements.
Optional calls with absent targets skip sources; source failures skip later
arguments, and completion follows ordinary catch/finally rules.

### Supply undefined to a defaulted parameter

An explicit `undefined` remains a supplied argument for count checking. Once the
call is valid, the default initializes that slot. This example prints `9`:

```tsrust
function choose(value: int = 9): int {
  return value;
}

function main() {
  console.log(choose(...[undefined]));
}
```

This does not make `undefined` acceptable for a required `int` parameter without
a default. Nor does a skipped default create an extra rest element.

See [array-literal spread](/tsrust/language/arrays/#array-spread) for building an owning array and [rest parameters](/tsrust/language/functions/#rest-parameters) for collecting arguments.

## Calling an expression

A call may follow an expression that produces a callable: a factory result, a
parenthesized arrow or function expression, a record field or an indexed callable
array element. The callee evaluates once before the arguments. Arguments then
evaluate from left to right.

### Invoke a factory result or immediate function

```tsrust
function make(base: int): Function<int, [value: int]> {
  return (value: int): int => base + value;
}

function main() {
  console.log(make(10)(2), ((value) => value + 1)(3)); // 12 4
}
```

An immediately invoked arrow can infer a required parameter type from an argument
whose type is already known. Missing required arguments, incompatible argument
types and non-callable results are errors.

### Evaluate the callee before its arguments

```tsrust
function factory(trace: int[]): Function<int, [value: int]> {
  trace.push(1);
  return (value: int): int => value + 10;
}

function mark(trace: int[], value: int): int {
  trace.push(value);
  return value;
}

function main() {
  const trace: int[] = [];
  const result = factory(trace)(mark(trace, 2));
  console.log(result, trace.join(",")); // 12 1,2
}
```

### Supply types when argument hints are insufficient

A fresh record literal does not supply the required parameter hint. Annotate its
parameter. For spread calls, inference needs the same known type from every source
that could reach a parameter position; otherwise annotate that parameter too.

```tsrust
type Point = {
  x: int;
};

function main() {
  console.log(((point: Point): int => point.x)({
    x: 1
  })); // 1
  const args = [2, 3];
  console.log(((a, b) => a + b)(...args)); // 5
}
```

Defaults, rest parameters and spread keep their ordinary call contracts. See
[call spread](/tsrust/language/functions/#spread-arguments) for argument counts and ownership.

### Select compatible callable alternatives

A conditional may select compatible owned functions with different concrete
origins. The chosen environment retains its own captures; the selection does not
invoke it or clone its owners. The checker validates the argument, access and result contracts of every
reachable alternative. Scoped borrowed alternatives retain their capture lifetime and
invocation restrictions.

```tsrust
function main() {
  const flag = false;
  const first = (input: int): int => input + 1;
  const second = (input: int): int => input * 2;
  console.log((flag ? first : second)(4)); // 8
}
```

This selection differs from reassigning an annotated local to a different origin;
see [local callable annotations](/tsrust/language/function-values/#replace-and-select-environments).

### Call a named assignment result

Assigning a named callable and invoking that result replaces its storage before
argument evaluation. Self-assignment keeps the existing environment. A replacement
uses the new environment's ordinary read, mutable or single-use call contract.

```tsrust
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
  let action = make(10);
  (action = action) < int >(7); // 11
  (action = make(20)) < string >("text"); // 21
}
```

Replacing that receiver again while evaluating an argument is rejected when the
invocation borrows it. A consuming invocation still transfers its captures once;
self-assignment does not restore them.

This call form does not create an independent owned copy of an assignment result. Other destinations and escaping assignment
results retain their [assignment ownership rules](/tsrust/language/declarations/#assignment-values).

### Preserve captured ownership and borrows

Calling a fresh expression does not implicitly clone captures or arguments. A
consumed owner remains transferred. Temporary capture loans can end after a call
whose result cannot retain a callable. Returned callables and aggregates containing
them retain the corresponding capture restrictions. Once and reusable invocation
rules still apply.

[Callable record fields](/tsrust/language/function-values/#owned-callable-fields) and
[callable arrays](/tsrust/language/function-values/#callable-arrays) retain their own borrowing and
extraction rules when used as expression callees.

## Nested functions

Declare a named function inside another function or a braced block to keep a
helper near the code that uses it. The declaration is available throughout its
block, including before its source position.

### Call a helper before its declaration

```tsrust
function main() {
  console.log(double(3)); // 6

  function double(value: int): int {
    return value + value;
  }
}
```

Parameters use the same annotations as top-level functions, including
[optional and default parameters](/tsrust/language/functions/#optional-and-default-parameters)
and [rest parameters](/tsrust/language/functions/#rest-parameters). A function declaration
needs a braced body. To declare one inside an `if` or loop, put it inside that
statement's braced block.

### Keep names inside their block

A nested block can declare the same name independently. Leaving the block restores
the outer binding; its inner declaration is unavailable outside that block.

```tsrust
function main() {
  function value(): int {
    return 10;
  }

  {
    console.log(value()); // 20
    function value(): int {
      return 20;
    }
  }

  console.log(value()); // 10
}
```

Two declarations cannot share a name in the same block. A function-body declaration
also cannot redeclare a parameter of that function. An arrow parameter or a binding
in a nested block can shadow an enclosing name.

A nested function named `main` is an ordinary local helper. Only the file's entry
function runs implicitly. Use a [named function expression](/tsrust/language/functions/#give-an-expression-a-private-recursive-name)
when you need a callable value with a private recursive name instead.

### Infer a result or annotate recursion

A nonrecursive helper can omit its result annotation when checked return expressions
establish the result. This includes results derived from parameters, captures and
arrays.

Value-returning paths must agree on a type, and every reaching path must return a
value. A function with no value returns has result `void`; the entry function
`main` must have result `void`.

```tsrust
function main() {
  function double(value: int) {
    return value + value;
  }

  function factorial(value: int): int {
    if (value <= 1) {
      return 1;
    }

    return value * factorial(value - 1);
  }

  console.log(double(3), factorial(5)); // 6 120
}
```

Nested declarations support self recursion and mutual recursion. An explicit result
annotation makes a recursive contract clear. Without one, inference needs a return
value whose type is known independently of the recursive call, such as:

- A literal base case, such as `return 1`.
- One known branch of a conditional expression.
- An arithmetic operand with a known type.

Write the result type when recursion supplies the only evidence or the independent
evidence is available only through a local variable.

### Find and use helpers in the editor

Completion offers hoisted declarations throughout their block. Hover and signature
help show checked source signatures. Definitions and references follow the selected
binding when a name is shadowed, and the document outline nests local functions
under their enclosing function.

Continue with [captured state and callback factories](/tsrust/language/functions/#captures)
for enclosing bindings, stored callback loans and returned environments.

## Captures

A [nested declaration](/tsrust/language/functions/#nested-functions) can read or change
bindings from its enclosing functions. Direct calls use those bindings, so a change
is visible to later calls and to the enclosing code.

### Share state across direct calls

```tsrust
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
  console.log(total, history.join(",")); // 5 2,2,1
}
```

Reassigning a captured scalar requires `let`; assigning to a `const` scalar is an
error. A `const` array binding can still permit element mutation and methods such
as `push`, subject to ownership checks.

A direct call borrows each capture with the access its body needs, including
captures used by other nested functions it calls. Loans end after the call unless
its result retains them. Transferring a captured owner makes the original binding
unavailable; ending the call does not restore an owner that was transferred.

### Initialize captures before use

The function may be declared before a captured binding. That binding must have a
value on every path reaching a call or a use of the function as a value.

```tsrust
function main() {
  function read(): int {
    return value;
  }

  const value = 3;
  console.log(read()); // 3
}
```

Moving `read()` before `const value = 3` produces an uninitialized-binding error.
Initializing a shadowing variable in an inner block does not initialize this binding.
A failing statement can also leave a capture uninitialized on a `catch` path.

An unused nested declaration is still type checked, but its presence alone does
not borrow its inputs or require them to be initialized for an outer call. Calling
or retaining it makes those capture requirements live.

### Keep stored callback loans in scope

Using a captured declaration as a value follows the capture rules of an
[arrow callback](/tsrust/language/functions/#arrow-functions). A stored callback can keep a
capture loan alive beyond an individual invocation.

```tsrust
function main() {
  const base = [10];
  function shifted(value: int): int {
    return value + base[0];
  }

  console.log([1, 2].map(shifted).join(",")); // 11,12

  const counts = [0];
  function tick(): int {
    counts[0] += 1;
    return counts[0];
  }

  {
    const run = tick;
    console.log(run(), run()); // 1 2
  }

  console.log(counts[0]); // 2
}
```

Here `run` holds mutable access to `counts` until its block ends. Reading `counts`
while that callback remains in scope conflicts with its capture.

Passing a callback does not always transfer its captures: the receiving contract and whether the
callback is retained determine the checked access. See
[borrowed local callable contracts](/tsrust/language/function-values/#local-function-values).

Arguments and live views also matter:

- Passing `counts` to a function that also mutates its captured `counts` creates
  conflicting access. Passing a copied scalar such as `counts.length` can be valid.
- Keeping an array-element view alive across a call that changes the array creates
  a view conflict.
- Transferring an owner cannot overlap an argument that borrows that same owner.

### Return an owner from a factory callback

An escaping owned callback retains the owners its body needs. A callback that
consumes a captured owner can be invoked only once.

```tsrust
type User = {
  name: string;
};

function make(user: User): () => User {
  function take(): User {
    return user;
  }

  return take;
}

function main() {
  const take = make({
    name: "Pat"
  });
  const user = take();
  console.log(user.name); // Pat
}
```

Calling `take()` again is a consumed-callback error. A borrowed `ReadOnly<User>`
parameter cannot be retained as an owned capture by this factory. Pass an owned
value, or explicitly `clone()` when you need two independent owners; callbacks do
not implicitly clone captures.

### Retain independent reusable state

Retaining an owner does not make every callback single-use. A callback that updates
its environment without consuming it can be called repeatedly. Each factory call
creates a separate environment.

```tsrust
function make(start: int): () => int {
  let value = start;
  function next(): int {
    value += 1;
    return value;
  }

  return next;
}

function main() {
  const actions = [make(10), make(20)];
  const first = actions[0]();
  const second = actions[1]();
  const again = actions[0]();
  console.log(first, second, again); // 11 21 12
}
```

Nonempty callable arrays infer a contract from the first element and check the
remaining elements against it. A once-callable must be removed from its array
before invocation; indexing borrows the element instead of transferring its
environment. Empty arrays need a type or suitable context. See
[callable array storage](/tsrust/language/function-values/#callable-arrays) for extraction and
ownership examples.

Declarations inside loop bodies or blocks use the bindings of that iteration or
block entry. Recursive peers share their enclosing environment; each invocation
has its own locals for functions nested inside it.

Editor hover, definitions, references and completion follow the captured binding,
including shadowed names.

## Parameters of captured functions

Storing, passing or returning a captured named function preserves its checked
parameter contract. Its defaults run in the original declaration, with access to
that declaration's captures.

### Evaluate defaults once per omitted argument

Omitting an argument or passing `undefined` evaluates its default once. Passing a
non-undefined value skips the initializer, including its side effects and possible
failure. An optional parameter without a default receives `undefined`.

```tsrust
function main() {
  let hits = 0;
  function bump(): int {
    hits += 1;
    return 3;
  }

  let total = 10;
  function next(step: int = bump()): int {
    total += step;
    return total;
  }

  {
    const action = next;
    console.log(action()); // 13
    console.log(action(2)); // 15
    console.log(action(undefined)); // 18
  }

  console.log(hits); // 2
}
```

Borrowed captures retain their normal loan scope. A default that consumes a
captured owner can make the callback single-use, even though some calls supply
an argument. No implicit copy of an owned capture is introduced. See
[capture ownership](/tsrust/language/functions/#captures).

### Account for arguments supplied by array methods

Array callbacks receive the method's arguments. A supplied index is a real second
argument, so it overrides a default on that parameter.

```tsrust
function main() {
  const base = [10];
  function pick(value: int, extra: int = base[0]): int {
    return value + extra;
  }

  console.log([1, 2].map(pick).join(",")); // 1,3

  function optional(value: int, extra?: int): int {
    return value + (extra ?? base[0]);
  }

  const action = optional;
  console.log(action(1), action(1, 2)); // 11 3
}
```

Parameter types and argument counts are still checked. See
[optional and default parameters](/tsrust/language/functions/#optional-and-default-parameters)
for declaration and call rules.

### Forward empty, explicit and spread rest arguments

A stored rest function receives its arguments in order. An empty call builds an
empty rest pack. Spreading an array supplies its elements as arguments.

```tsrust
function main() {
  const base = 10;
  function sum(...values: int[]): int {
    let total = base;
    for (const value of values) {
      total += value;
    }

    return total;
  }

  const action = sum;
  console.log(action(), action(2, 3)); // 10 15
  const values = [4, 5];
  console.log(action(...values)); // 19
}
```

Every rest element must satisfy the declared element type. An array method's full
callback tuple can be incompatible: an `int[]` rest parameter cannot receive a
source-array argument along with integer values and indexes. A compatible fixed
callable contract can adapt a rest function.

Fixed parameters before the rest parameter keep their optional/default behavior.
Omitted or `undefined` fixed slots evaluate their defaults once. Returned factories
retain separate captured state; consuming a captured owner still limits invocation
to once. See [rest parameters](/tsrust/language/functions/#rest-parameters) and
[call spread](/tsrust/language/functions/#spread-arguments) for argument and ownership details.
