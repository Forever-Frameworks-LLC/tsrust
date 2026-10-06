# Scalar unary plus

Unary `+` converts a string, boolean, null, undefined or scalar union to a
`number`, just as `Number(value)` does. The operand evaluates once. A string is
borrowed and stays reusable.

```tsrust
const input = "12.5";
const parsed = +input; // number, 12.5
const enabled = +true; // number, 1
const empty = +null; // number, 0
console.log(parsed, enabled, empty); // 12.5 1 0
```

String conversion trims whitespace and reads the whole decimal value or Infinity.
Empty text converts to zero. Invalid text and undefined convert to NaN; use
`Number.isNaN` rather than equality to test NaN. Signed zero is preserved. The
existing Number conversion does not accept radix-prefixed text or custom valueOf
objects. Conversion uses 64-bit floating-point precision, so large integers may
round when converted to number.

```tsrust
const input: string | undefined = "7";
console.log(+input, Number(input)); // 7 7
```

A scalar union converts its active alternative, without narrowing the declared
source type or transferring its owner. Arrays, records and callable values are
type errors; extract a scalar first. Numeric operands retain existing numeric
rules: an int stays int unless a number context requests numeric literal widening,
and a number stays number. This change does not introduce string subtraction or
coercive equality.
