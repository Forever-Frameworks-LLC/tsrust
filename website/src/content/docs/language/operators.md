---
title: "Operators and conversions"
description: "Truthiness, logical selection and assignment, non-null assertions, the in operator, unary plus, Number conversion, as and satisfies."
---

Operators never convert a value implicitly except where this page says so. `||`
and `&&` select one of their operands by truthiness, `!` asserts presence, `in`
tests for a key, unary `+` and `Number(...)` convert scalars to `number`, and `as`
and `satisfies` check a type without changing the value.

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/)
and run `tsrust run main.tsr`.

## Long operator chains

A chain of `+`, `-`, `*`, `/`, `%`, `&&` or `||` over strings, ints, numbers or
booleans has no length limit. `a + b + c + ...` builds one string in a single
buffer, and arithmetic or boolean chains update one running value, so a chain of
thousands of operands compiles like a list of statements. Operands still run left
to right, `&&` and `||` still short-circuit, and each `+` still checks int overflow
at its own position.

Other nesting, such as parentheses inside parentheses, nested calls or chains of
`??` and nullable `||` selections, still counts toward the expression nesting
limit of 128.

## Truthiness

`false`, `0`, `NaN`, the empty string, `null` and `undefined` are falsy. Every other
value is truthy, including whitespace strings and empty arrays and records.
Conditions in `if`, loops and `? :` use these rules. `!value` returns the opposite
truth value and `!!value` returns the truth value as a `boolean`.

Testing an owned value borrows it. Testing a nullable variable or fixed record field
also narrows it: inside `if (name)` the name is present.

## Logical selection

`left || right` returns `left` when it is truthy and otherwise evaluates and
returns `right`. `left && right` returns `left` when it is falsy and otherwise
evaluates and returns `right`. The left side runs once; the right side runs only
when selected. `??` differs: it falls back only for `null` and `undefined`.

```tsrust
function main() {
  const users = ["Alice", "Bob"];
  const first = users.shift() || "Guest";
  console.log(first, "" || "Guest", "" ?? "Guest", 0 || 5, 3 && 4);
}
// prints: Alice Guest  5 4
```

The result type joins both operand types. Selecting a borrowed owned value does
not copy it: binding the result keeps the borrow, and a borrowed payload cannot
become an owner without `clone(...)`. Callable operands must share one compatible
payload type; for different callable shapes, use a conditional. Mix `??` with
`&&` or `||` only inside parentheses.

## Logical assignment

`target ||= value` stores `value` only when `target` is falsy. `target &&= value`
stores it only when `target` is truthy. A skipped right side does not run. The
expression returns the target's value after the optional store.

```tsrust
function main() {
  let name = "";
  name ||= "Guest";
  console.log(name);
  name &&= "Member";
  console.log(name, !name, !!name, !![], !!" ");
}
// prints: Guest
// prints: Member false true true true
```

- For a non-boolean target, the target must be a variable or a fixed record field
  path. `names[0] ||= "a"` is `unsupported`: read the element into a variable,
  update it and store it back.
- The value must fit the target's type. `let text = ""; text ||= 1;` is a `type`
  error.
- Binding an owned result transfers the target; read-only use leaves it available.

## Non-null assertion

Postfix `!` selects the present payload of a nullable value and removes `null` and
`undefined` from that expression's type. The operand evaluates once.

```tsrust
function main() {
  const users = ["Alice", "Bob", "Charlie"];
  console.log(users.pop()!.toUpperCase());
  const empty: string[] = [];
  console.log(empty.pop() ?? "Guest");
  try {
    console.log(empty.pop()!);
  } catch (error) {
    console.log("no user");
  }
}
// prints: CHARLIE
// prints: Guest
// prints: no user
```

Unlike TypeScript, the assertion is checked at run time. An absent value throws
`Non-null assertion failed: value is null or undefined`, which `try`/`catch`
receives; uncaught, the program stops with that message and the source range.
`!` does not supply a default (use `??`), clone, or restore a moved value. It works
in chains such as `action!()` and `values![0]`, and on an already nonnullable value
it changes nothing. A value that can only be absent is rejected at compile time.

## The in operator

`key in value` tests whether a record, tuple or array has an own key, like
`Object.hasOwn(value, key)`. Inherited names such as `"toString"` are absent.

```tsrust
type Point = {
  x: int;
  y: int;
};

function main() {
  const point: Point = {
    x: 1, y: 2
  };
  console.log("x" in point, "z" in point, "toString" in point);
  const values = [5, 6, 7];
  console.log(1 in values, 3 in values, "length" in values, "2" in values);
}
// prints: true false false
// prints: true false true true
```

- A record's keys are its field names. Every field of a record type is always
  present, so the answer depends only on the type and the key text.
- An array or tuple has the positions `0` to `length - 1`, as numbers or decimal
  strings, plus `"length"`.
- The key must be a string, `int` or `number`, and the value a record, tuple or
  array; anything else is a `type` error.
- The key evaluates before the value. `in` binds like `<`: write
  `!("x" in point)` with parentheses.

[`for...in`](../control-flow/#forin) visits the same keys.

## Unary plus and Number conversion

Unary `+value` and `Number(value)` convert a string, boolean, `null`, `undefined`
or scalar union to `number`. The operand evaluates once; a string is borrowed and
stays usable.

```tsrust
function main() {
  console.log(+"12.5", +true, +null, Number("0x10"), +"0b11", Number("0o17"));
  console.log(Number(" .5e2 "), Number(""), Number.isNaN(Number("12tail")));
  const input: string | undefined = "7";
  console.log(+input, Number(input), Number.isNaN(+undefined));
}
// prints: 12.5 1 0 16 3 15
// prints: 50 0 true
// prints: 7 7 true
```

- The whole string must be numeric. Leading and trailing whitespace is ignored and
  empty text is `0`. Accepted forms are signed decimals with exponents, signed
  `Infinity`, and unsigned `0b`, `0o` and `0x` integers in either letter case.
- A partial number (`"12tail"`), a sign before a radix prefix (`"-0x10"`),
  separators, BigInt suffixes and Rust spellings such as `inf` give `NaN`. Test
  with `Number.isNaN`, never `===`. `parseInt` and `parseFloat` read a prefix
  instead; see [global functions](../../library/globals/).
- Results are 64-bit floating point: large integers round, very large values become
  `Infinity`, and `-0` is preserved.
- Arrays, records and functions are `type` errors. An `int` stays `int` under unary
  `+`. There is no string subtraction or coercive `==`.

## as and satisfies

`value as Type` gives a compatible expression a target type. It supplies context
to empty arrays, record literals and function expressions, and `1 as number`
converts an integer literal to `number`. `value satisfies Type` checks
compatibility but keeps the expression's own type.

```tsrust
type Point = {
  x: int;
  y: int;
};

function main() {
  const values = [] as int[];
  values.push(7);
  const ratio = 1 as number;
  const count = 3 satisfies number;
  const origin = {
    x: 0, y: 0
  } satisfies Point;
  console.log(values.length, ratio / 2, count / 2, origin.x);
  console.log(([1, 2] as int[]).length);
}
// prints: 1 0.5 1 0
// prints: 2
```

`count` stays an `int`, so `count / 2` is integer division. Neither operator
parses strings, adds a run-time check or narrows a union: for `value: int | string`,
`value as int` is a `type` error; use `typeof` in an `if`. Parenthesize an
assertion before member access, indexing or a call. Assertions do not clone an
owned value or make a borrowed value owned.
