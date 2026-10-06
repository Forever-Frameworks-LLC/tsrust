# Gate 169: complete numeric string conversion

## Scope and acceptance plan

Number and scalar unary plus will share the StringToNumber grammar for supported
UTF-8 text. The same helper will serve direct strings, string enums and their union
payloads. Decimal syntax, signed Infinity, empty input, exact ECMAScript whitespace,
unsigned binary/octal/hexadecimal prefixes, invalid tails, signed zero and correctly
rounded radix integers are in scope. Numeric operands and prefix parseFloat/parseInt
remain unchanged. Custom objects, Symbol and BigInt remain outside supported types.

1. Commit red std:testing native witnesses and their independent Node reference.
2. Implement a shared runtime module: borrowed trimming, whole decimal validation,
   and constant-space power-of-two radix conversion with nearest-even rounding.
   Validate all digits, even after overflow; use guard and sticky bits to avoid
   intermediate rounding. Route every string Number path to the helper.
3. Test bounded long inputs, rounding boundaries, whitespace exclusions, scalar,
   union and enum paths. Update catalog documentation and execute its examples;
   run the affected compiler suites and whole editor crate.
4. Add a matched benchmark family with independent fixed expected values, complete
   result observation, source guards/fingerprints, registration and runner/profile/
   report hooks. Verify five native/Node cases untimed; defer measurements.
5. Update coverage and roadmap, import and read back exact records in SQLite,
   refresh five consumers, prune generated outputs and commit accepted work.

The authoritative grammar is ECMA-262 StringToNumber:
https://tc39.es/ecma262/2024/multipage/abstract-operations.html#sec-tonumber-applied-to-the-string-type.
The parser must reject Rust-only inf/infinity spelling and U+0085 whitespace;
ECMAScript admits U+FEFF but excludes U+0085 and U+200B. No allocation is necessary
for trimming, validating or reading radix bits. Decimal conversion delegates to the
standard correctly rounded floating-point parser only after grammar validation.

Status: red witnesses added; implementation and acceptance remain pending.
