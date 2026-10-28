---
title: Call the result of an expression
description: Invoke factories, immediate functions and checked conditional callable values.
---

A call may follow an expression that produces a callable: a factory result, a
parenthesized arrow or function expression, a record field or an indexed callable
array element. The callee evaluates once before the arguments. Arguments then
evaluate from left to right.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Invoke a factory result or immediate function

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

## Evaluate the callee before its arguments

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

## Supply types when argument hints are insufficient

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
[call spread](/tsrust/language/call-spread/) for argument counts and ownership.

## Select compatible callable alternatives

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
see [local callable annotations](/tsrust/language/callable-locals/#replace-and-select-environments).

## Call a named assignment result

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
results retain their [assignment ownership rules](/tsrust/language/assignment-values/).

## Preserve captured ownership and borrows

Calling a fresh expression does not implicitly clone captures or arguments. A
consumed owner remains transferred. Temporary capture loans can end after a call
whose result cannot retain a callable. Returned callables and aggregates containing
them retain the corresponding capture restrictions. Once and reusable invocation
rules still apply.

[Callable record fields](/tsrust/language/callable-fields/) and
[callable arrays](/tsrust/language/callable-arrays/) retain their own borrowing and
extraction rules when used as expression callees.
