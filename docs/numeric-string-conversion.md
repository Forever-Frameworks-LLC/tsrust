# Whole numeric string conversion

`Number(text)` and unary `+text` read a whole numeric string and return a number.
Leading and trailing numeric whitespace are ignored; empty text returns zero.
Accepted syntax includes signed decimals and exponents, signed Infinity, and
unsigned `0b`, `0o` and `0x` integers with either prefix case.

```tsrust
console.log(Number("0x10"), +"0b11", Number("0o17")); // 16 3 15
console.log(Number(" .5e2 "), Number("")); // 50 0
console.log(Number.isNaN(Number("12tail"))); // true
```

Unlike `parseFloat` and `parseInt`, conversion rejects a partial numeric prefix.
A sign before a radix prefix, missing radix digits, invalid digits, separators,
BigInt suffixes and trailing text produce NaN. Rust-only inf/infinity spellings
are invalid. Numeric whitespace includes BOM and nonbreaking space, but excludes
U+0085, U+180E and U+200B. Native UTF-8 strings cannot represent lone surrogates.

Results use 64-bit floating-point precision. Radix integers beyond the exact range
round to the nearest representable value, ties to even. Large values can become
Infinity; small decimals can become signed zero. Negative zero is preserved, so
`Object.is(Number("-0"), -0.0)` is true.

Direct strings, string enum values and scalar union payloads use the same rules.
Conversion evaluates its source once and borrows it without a parsing buffer or
implicit owner transfer. Prefix parsing and int arithmetic retain their existing
contracts. Custom objects and boxed-number construction remain unsupported.
