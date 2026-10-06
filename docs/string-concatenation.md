# String concatenation

When either operand of `+` has type `string`, the other may be a string, integer,
number, boolean, null, undefined, or a union containing only these types. Scalar
values convert using the same rules as `String(value)`. The result is an independent
string, and operands evaluate once, left to right.

```ts
const count = 3;
const label = "item " + count; // "item 3"
const reverse = count + " items"; // "3 items"
let message = "ready:";
message += true; // "ready:true"
```

Addition remains left associative: `1 + 2 + "x"` produces `"3x"`, while
`"x" + 1 + 2` produces `"x12"`. Parentheses can request numeric addition first.
Null and undefined become their names; numbers use their normal String formatting,
including NaN, Infinity and negative zero. Integers retain their signed 64-bit
range rather than losing precision through floating-point conversion.

String `+=` supports scalar conversion for a mutable string binding, field or array
position. It remains a standalone statement. The old target stays protected while
the right operand runs, and the right operand runs once.

Arrays, records and functions are not concatenated implicitly. Use explicit
`String(value)` or a template literal for supported data. String subtraction,
coercive equality and numeric operators on strings remain rejected. A string result
annotation alone does not turn numeric addition into concatenation. Sources are
borrowed; concatenation does not transfer them or introduce implicit clones.
