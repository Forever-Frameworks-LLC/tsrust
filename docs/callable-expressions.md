# Calling callable expressions

A call may follow an expression that produces a callable value. This includes a factory result, a parenthesized arrow or function expression, a record field and an indexed callable array element. The callee is evaluated once before the arguments, which are evaluated from left to right.

```tsrust
import { strictEqual } from "std:testing";
function make(base: int): Function<int, [value: int]> {
    return (value: int): int => base + value;
}
function main() {
    strictEqual(make(10)(2), 12);
    strictEqual(((value) => value + 1)(3), 4);
}
```

An immediately invoked arrow can infer a required parameter's type from an argument with an already known type. When the argument has no available type hint, such as a fresh record literal, write the parameter annotation. For spread arguments, inference requires all possible sources for that parameter position to have the same known type; otherwise write an annotation. Defaults, rest parameters and spread calls retain their ordinary call contracts. Missing required arguments, incompatible argument types and non-callable callee results are errors.

A conditional callee retains the existing callable-origin restriction. Two results of the same factory may be selected; unrelated arrow declarations do not become interchangeable solely because their written function signatures agree.

Calling a fresh expression does not implicitly clone its captures or arguments. A consumed owner remains transferred. Temporary capture loans end when an ordinary result cannot retain a callable; returned callables and aggregates containing them retain the corresponding capture restrictions. Once and reusable invocation rules remain in force.
