# Comma expressions

A comma expression evaluates its operands once, from left to right, and returns the
last operand. Earlier results are discarded before the next operand runs.

```ts
let i = 0, j = 3;
for (i = 0, j = 3; i < 3; i++, j++) {}
// i is 3; j is 6.
const selected = (1, "last");
// selected is a string containing "last".
```

Comma is the lowest-precedence expression operator. Use parentheses when placing a
comma expression in an argument, array element, initializer or arrow body. Commas
in declaration, parameter, argument, array and record lists remain separators.
Return and throw expressions, conditions and computed indexes also allow commas.
A declaration cannot appear as a comma operand.

A thrown operand stops evaluation of the remaining operands. Existing catch and
finally behavior applies without a new function boundary.

The last operand determines the result type and access. Reading and discarding an
owned name does not clone or transfer it; a call with an Owned parameter still
transfers its argument. A borrowed result retains its source loan. Temporary results
of earlier operands are discarded; named borrows keep their normal lexical scope.

Owned replacement and string += are allowed when their result is discarded, such
as an earlier comma operand or a for update. They do not produce an owning result
that can be assigned to another binding. Destructuring assignment retains its
existing statement form.
