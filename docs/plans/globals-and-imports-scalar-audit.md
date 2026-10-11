# Scalar conversion and formatting audit

Status: 14 APIs reviewed: three global conversions and eleven int/number/boolean
receiver members. Retain familiar globals and receiver methods; no imported
synonyms or silent semantics changes are proposed.

## Shared contracts

Global conversions borrow noncopy data while inspecting it; scalar inputs copy.
String conversion creates independent text (static literals may remain static).
Numeric/boolean results copy. Receiver methods copy their scalar and return either
the same scalar or independent string values. No boxed objects, resource handles,
receiver mutation or explicit disposal are introduced.

Static checking enforces receiver/argument types and method counts. Number takes
at most one argument; Boolean/String evaluate additional supported arguments even
though only the first determines the result. Earlier borrowed arguments remain
protected during later argument evaluation. Radix/digit arguments are int, not
implicitly converted number/string values. Invalid runtime ranges terminate with
diagnostics rather than returning Result or promising a catchable typed error.
Invalid Number text is instead a successful NaN result.

`unknown` in global signatures describes checked supported input shapes, not an
unchecked dynamic data type. Native capability checks still protect opaque types.
The global names follow ordinary shadowing resolution; matching their spelling
does not override a user declaration. Public placement is unchanged.

## Per-symbol review

| Symbol | Signature | Purpose, defaults, results and errors |
|---|---|---|
| `Number` | `Number(value?: int \| number \| boolean \| string \| null \| undefined): number` | Zero or one scalar/scalar-union argument; omission/null 0, undefined NaN, booleans 0/1. Whole trimmed numeric string, empty 0, invalid NaN; supports unsigned binary/octal/hex and signed decimal/Infinity. Signed zero preserved. Aggregates/functions rejected; no implicit narrowing needed for scalar unions. |
| `Boolean` | `Boolean(value?: unknown): boolean` | Truthiness conversion; omission/null/undefined/zero/NaN/empty string false; nonempty strings and supported aggregate values true. Converts active union alternative. Additional arguments evaluate but do not change result. No boxed object. |
| `String` | `String(value?: unknown): string` | Omission empty; null/undefined names; primitive decimal/text values, -0 as 0; arrays/tuples comma-join recursively, null/undefined elements empty; records [object Object], Map [object Map], Set [object Set]. Active data unions supported; function-source conversion rejected. Additional arguments evaluate but are ignored. |
| `int.valueOf` | `valueOf(): int` | Copies unchanged native signed 64-bit scalar; no boxing. |
| `int.toString` | `toString(radix?: int): string` | Exact lowercase signed i64 digits; radix default 10, valid 2..36; invalid radix terminates; no loss of integer precision. |
| `int.toLocaleString` | `toLocaleString(): string` | Exact full-range decimal i64 with comma groups; no locale/options arguments; fixed separators, no exponent notation. |
| `number.valueOf` | `valueOf(): number` | Copies unchanged f64 scalar, including signed zero and NaN; no boxing. |
| `number.toFixed` | `toFixed(fractionDigits?: int): string` | Fraction digits 0..100, default 0; checked before nonfinite classification. Ties-to-even. Exact -0 becomes 0; NaN/Infinity spellings; magnitude at least 1e21 uses shortest notation. Invalid digit count terminates. |
| `number.toExponential` | `toExponential(fractionDigits?: int): string` | Fraction digits 0..100; omitted uses shortest scientific digits. Nonfinite values return their spelling before digit range validation. Finite invalid count terminates. Ties-to-even, signed exponent, exact -0 normalized. |
| `number.toPrecision` | `toPrecision(precision?: int): string` | Significant digits 1..100; omitted shortest decimal. Nonfinite values return spelling before range validation. Finite invalid count terminates. Ties-to-even; decimal/scientific layout chosen by magnitude. |
| `number.toString` | `toString(radix?: int): string` | Radix default 10, valid 2..36 checked before special values. Shortest round-trippable decimal; nondecimal lowercase digits and fractions. NaN/Infinity spellings, -0 becomes 0. Invalid radix terminates. |
| `number.toLocaleString` | `toLocaleString(): string` | Fixed comma grouping and period separator; no locale/options arguments. At most three fraction digits, halfway away from zero, trailing zeros removed. Full decimal finite text, NaN/±∞ spellings; signed zero retained. |
| `boolean.valueOf` | `valueOf(): boolean` | Copies unchanged boolean scalar; no boxing. |
| `boolean.toString` | `toString(): string` | Returns true or false text; unchanged receiver. |

## Source reconciliation and dependencies

Compiler anchors under crates/tsrust-compiler/src:

- analysis.rs Number call dispatch checks zero/one scalar input and scalar unions;
  value_builtin_arguments checks String/Boolean observations and preserves loans.
- emit.rs::value_builtin_call and union_observation implement truthiness, scalar
  text, aggregate text and active union alternatives without ownership transfer.
- analysis.rs receiver dispatch recognizes valueOf, toString, toFixed,
  toExponential, toPrecision and toLocaleString, including method-specific access
  and rejection of locale/options arguments.
- runtime_numeric_string.rs provides shared whole-input numeric parsing and
  numeric whitespace rules. This is distinct from prefix parseInt/parseFloat.
- runtime_strings.rs::tsr_int_to_string/tsr_number_to_string implement radix
  formatting; tsr_number_fixed/exponential/precision preserve the reviewed range
  priority and native rounding. tsr_int_locale_string/tsr_number_locale_string
  implement deterministic grouping and locale-independent output.
- runtime_join.rs and emitted tuple helpers support aggregate String conversion.
  Native helpers are selected through existing runtime emission; no external
  package, namespace import or user linker flag is required. Family helper
  inclusion is not evidence of per-method binary elimination.

Every catalog signature above was reconciled with checker/emitter/runtime code.
Precision methods intentionally differ from JavaScript at some halfway values;
fixed locale formatting intentionally does not negotiate locale. Keep these
limitations in user documentation rather than implying compatibility from names.

## Focused acceptance

SQLite `tsrust-api-scalar-*.log` retains numeric formatting, radix, whole-string
conversion, ownership/range checks, editor help checks and combined native catalog
examples. Examples are isolated, formatted twice and compared to expected output.
Whole-project formatting runs before checkpoint. No new timing or complete family
regression is claimed. Primitive type spellings and remaining array/callable APIs
still require their separate final audit coverage.
