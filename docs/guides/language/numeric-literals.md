# Numeric literal forms

Decimal, binary (`0b`), octal (`0o`) and hexadecimal (`0x`) literals can use `_`
between digits for readability. Prefix letters can be uppercase. Separators do
not change the value. Decimal fractions and exponents can each contain separators.

```tsrust
function main() {
    const mask = 0b1010_0011; // int, 163
    const permissions = 0o7_7; // int, 63
    const code = 0xDE_AD; // int, 57005
    const amount: number = 1_234.5_6e+0_1; // number, 12345.6
    console.log(mask, permissions, code, amount);
}
```

A separator cannot begin or end a digit sequence, repeat, touch a prefix, decimal
point, exponent marker or exponent sign. `1__0`, `0x_FF`, `1_.0`, `1e_2` and `0_1`
are syntax errors. A radix prefix requires at least one digit of that radix;
`0b102`, `0o8` and `0xGG` are invalid. Numeric suffixes, including BigInt `n`, remain
unsupported. This changes literal syntax only; underscores inside numeric strings
retain the existing String/Number parsing behavior.

Whole literals default to signed 64-bit `int`, including nondecimal forms. Values
outside the int range are rejected unless the expression has number context.
Decimal fractions and exponent spellings have type `number`. Contextual whole
literals also use number; a value wider than int can then round to the nearest
binary64 value or overflow to infinity. Runtime integers do not gain implicit
conversion from this syntax change. Native negative int minimum is supported;
JavaScript numbers cannot represent every integer near that boundary exactly.

```tsrust
const integer = -0x8000_0000_0000_0000; // exact minimum int
const rounded: number = 0xffff_ffff_ffff_ffff; // rounds to 18446744073709551616
```

Formatter preserves literal spelling, including prefix case and separators.
Diagnostics retain source ranges, and editor types come from checked numeric
expressions. Scanning and radix normalization do not add recursive parser frames.
These literals contain no owners, so no borrowing or cloning is involved.
