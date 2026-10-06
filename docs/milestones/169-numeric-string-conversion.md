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

## Implementation and evidence

The shared helper is runtime_numeric_string.rs. It scans borrowed UTF-8 slices,
validates whole decimal syntax, and converts radix digits with a 53-bit mantissa,
guard and sticky bits. Integer overflow and invalid tails are checked without a
parsing buffer. Decimal parsing delegates to f64 after syntax validation. Emission
routes all string and enum/union conversion paths to the same helper.

Acceptance also found an existing exponential hint traversal: a member receiver
was inferred separately for Map, Set and primitive methods. One cached receiver
hint replaces repeated traversal in expression_type_hint. The existing 48-member
diagnostic witness timed out twice before this repair; the focused identity-chain
witness and depth suite now pass without raising the timeout or stack size.

Final affected compiler/native checks and all eight depth canaries pass (80).
The whole editor crate passes 267 checks. The Number constructor catalog example
pair executes with identical native and Node output. Grammar fixtures cover radix
rounding ties, maximum finite overflow, long leading zeros, invalid tails after
overflow, excluded whitespace, signed zero, unions and enums. Old Rust-parser
expectations were replaced with independent explicit numeric values.

The benchmark family retains four observations over ten grammar cases. Its oracle
uses a fixed expected-value table, without parsing the text under test. ABI and
source guards, fingerprints and runner/profile/report hooks are registered; the
registration/report batch passes 17 checks. Input copying is explicit so the
ordinary borrowed-input ABI remains unchanged. Five matched cases are verified
untimed, with no new timing claim. Records and closed logs are retained in SQLite
with exact read-back hashes; loose verification copies and generated outputs are
removed before handoff.
