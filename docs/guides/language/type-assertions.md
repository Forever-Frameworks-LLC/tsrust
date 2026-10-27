# Checked type assertions

`value as Type` gives a compatible expression a target type. It provides context
for empty arrays, record literals and function expressions. For example,
`const values = [] as int[]; values.push(7);` creates an integer array.
`const value = 1 as number;` produces a floating-point number.

Assertions cannot turn an incompatible value into the target type. In particular,
`value as int` is rejected when `value` has type `int | string`. Use `typeof`
inside an `if` to establish which alternative is present. Assertions do not add
runtime tag checks or catchable cast failures.

`value satisfies Type` checks compatibility and keeps the expression's source
type. `const value = 3 satisfies number;` keeps `value` as `int`. Literals that
need a type, such as `[] satisfies int[]` and a record literal satisfying a named
record type, receive context from the target annotation.

Integer widening with `as number` converts to the language's floating-point
representation. Large integers can lose precision in that conversion, just as in
an ordinary explicit `Number(value)` conversion. `satisfies number` leaves an
integer unchanged. Neither operator parses strings or performs other coercions.

Assertions use ordinary type annotations, including named records, array types,
function types and compatible unions. Put an assertion in parentheses before
accessing a member, indexing it or calling it: `(value as int[]).length`.

Both operators preserve the operand's access and ownership rules. An assertion
does not clone an owned value or make a borrowed value owned. Passing an owning
array through an assertion transfers it when the surrounding expression consumes
it; a borrowed element still prevents mutation of its source while the borrow is
in scope.
