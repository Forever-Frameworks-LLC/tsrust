# Math and Number namespace audit

Status: all 58 namespace members reviewed (44 Math, 14 Number). The global Number
conversion function is a separate surface; this slice does not accept it or the
remaining Object/console/receiver audit.

## Placement and shared contracts

Retain familiar global Math and Number namespaces. These numeric operations have
no resource owner or external service dependency that would justify imported
synonyms. Existing global parseInt/parseFloat/isNaN/isFinite and constants retain
the compatibility disposition already recorded in the numeric-global review.
No public rename or semantic change is proposed by this audit.

All rows below are static namespace members. Constants are readonly number
values. Math and predicate arguments copy numeric scalars; inputs never mutate.
Parsing borrows its string and copies radix. Results are owned scalar copies.
There is no resource lifecycle or recoverable allocation Result. Numeric domain
errors produce IEEE NaN/infinity as described, rather than throwing. Wrong arity,
wrong argument types and constant writes are static diagnostics. Callers must
explicitly convert non-number values; familiar names do not imply string/boolean
coercion. Contextually typed numeric literals remain supported.

Math.random is the effectful exception: it advances thread-local generator state
and seeds from host hash randomization on first use. It is not suitable for
security decisions. Transcendental functions use native f64 operations; final bits
may depend on platform math implementation. Numerical agreement is not a promise
of bit-identical JavaScript results. Exact edge guards, signed zero, constants and
integer conversion are tested separately from approximate values.

Compiler dispatch recognizes numeric namespace operations before ordinary method
emission but retains name-resolution checks. The focused rejection tests cover
shadowing/static namespace use, arity, argument types, capture loans and constant
writes. Builtin methods are not general first-class namespace objects; explicit
callback wrappers remain the supported way to pass these operations.

## Per-symbol review

All rows retain their current global namespace placement. The shared access/error
rules above apply to each row. Arity follows the signature: min/max/hypot permit
zero or more arguments, random none, atan2/pow/imul two, other Math methods one.
Parsing accepts only its documented string and optional radix parameters.

| Symbol | Signature | Purpose and edge behavior | Source/backend |
|---|---|---|---|
| `Math.E` | `E: number` | Euler’s number, approximately 2.718281828459045. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.LN2` | `LN2: number` | Natural logarithm of 2, approximately 0.6931471805599453. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.LN10` | `LN10: number` | Natural logarithm of 10, approximately 2.302585092994046. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.LOG2E` | `LOG2E: number` | Base-2 logarithm of e, approximately 1.4426950408889634. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.LOG10E` | `LOG10E: number` | Base-10 logarithm of e, approximately 0.4342944819032518. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.PI` | `PI: number` | Ratio of circumference to diameter, approximately 3.141592653589793. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.SQRT1_2` | `SQRT1_2: number` | Square root of one half, approximately 0.7071067811865476. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.SQRT2` | `SQRT2: number` | Square root of 2, approximately 1.4142135623730951. Read-only number constant, usable in numeric expressions. | NumericConstant::lookup/rust |
| `Math.abs` | `abs(value: number): number` | Returns the nonnegative magnitude; negative zero becomes positive zero. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.floor` | `floor(value: number): number` | Rounds toward negative infinity. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.ceil` | `ceil(value: number): number` | Rounds toward positive infinity. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.trunc` | `trunc(value: number): number` | Discards the fractional part toward zero. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.sqrt` | `sqrt(value: number): number` | Returns the nonnegative square root; negative finite inputs return NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.round` | `round(value: number): number` | Rounds to the nearest integer; halfway ties go toward positive infinity. Negative values near zero preserve negative zero. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.sign` | `sign(value: number): number` | Returns -1 for negative values and 1 for positive values; preserves signed zero and NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.fround` | `fround(value: number): number` | Nearest IEEE single-precision value returned as number; ties to even; overflow gives signed infinity; signed zero preserved. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.f16round` | `f16round(value: number): number` | Nearest IEEE half-precision value returned as number; ties to even; signed zero preserved; magnitude at least 65520 overflows to signed infinity. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.random` | `random(): number` | Pseudo-random number in [0,1); thread-local xorshift state seeded on first use. Not cryptographic; no public seed or reproducible-sequence guarantee. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.clz32` | `clz32(value: number): number` | Counts leading zero bits after finite truncation and modulo-2^32 conversion; zero has 32 leading zeros. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.acos` | `acos(value: number): number` | Returns inverse cosine in radians; values outside [-1,1] return NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.asin` | `asin(value: number): number` | Returns inverse sine in radians; values outside [-1,1] return NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.atan` | `atan(value: number): number` | Returns inverse tangent in radians. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.cos` | `cos(value: number): number` | Returns cosine of an angle in radians. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.sin` | `sin(value: number): number` | Returns sine of an angle in radians. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.tan` | `tan(value: number): number` | Returns tangent of an angle in radians. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.acosh` | `acosh(value: number): number` | Returns inverse hyperbolic cosine; values below 1 return NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.asinh` | `asinh(value: number): number` | Returns inverse hyperbolic sine. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.atanh` | `atanh(value: number): number` | Returns inverse hyperbolic tangent; +/-1 produce infinities, values outside [-1,1] return NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.cosh` | `cosh(value: number): number` | Returns hyperbolic cosine. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.sinh` | `sinh(value: number): number` | Returns hyperbolic sine. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.tanh` | `tanh(value: number): number` | Returns hyperbolic tangent. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.exp` | `exp(value: number): number` | Returns e raised to value. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.expm1` | `expm1(value: number): number` | Returns e raised to value minus one, retaining accuracy near zero. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.log` | `log(value: number): number` | Returns natural logarithm; zero gives negative infinity and negative inputs give NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.log1p` | `log1p(value: number): number` | Returns natural logarithm of 1+value, retaining accuracy near zero; -1 gives negative infinity. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.log2` | `log2(value: number): number` | Returns base-2 logarithm. Zero gives negative infinity; negative inputs give NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.log10` | `log10(value: number): number` | Returns base-10 logarithm. Zero gives negative infinity; negative inputs give NaN. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.cbrt` | `cbrt(value: number): number` | Returns the real cube root, including negative inputs. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.min` | `min(...values: number): number` | Minimum; empty returns +infinity; NaN propagates; -0 wins zero ties. All arguments evaluate before reduction. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.max` | `max(...values: number): number` | Maximum; empty returns -infinity; NaN propagates; +0 wins zero ties. All arguments evaluate before reduction. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.hypot` | `hypot(...values: number): number` | Euclidean norm of zero or more numbers; empty returns +0; infinity takes precedence over NaN; sequential scaled hypot avoids intermediate squaring overflow. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.atan2` | `atan2(y: number, x: number): number` | Returns the angle in radians for point (x,y), using y as the first argument and selecting the quadrant. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.pow` | `pow(base: number, exponent: number): number` | Real power; exponent zero returns 1, including NaN base; NaN exponent or absolute base 1 with infinite exponent yields NaN; remaining domains follow f64 powf. | math_target/emitter; runtime_math.rs helpers where needed |
| `Math.imul` | `imul(a: number, b: number): number` | Returns signed 32-bit wrapping multiplication after converting both numbers modulo 2^32. Result remains number. Arguments and result use number; the inputs remain unchanged. | math_target/emitter; runtime_math.rs helpers where needed |
| `Number.parseFloat` | `parseFloat(text: string): number` | Parses the longest decimal prefix after leading ECMAScript whitespace. Accepts one string, optional sign, decimal fraction/exponent or Infinity. Incomplete exponents stop before the exponent; invalid starts return NaN. | NumberParseFloat/tsr_number_parse_float |
| `Number.parseInt` | `parseInt(text: string, radix?: int): number` | Parses signed integer prefix after whitespace; omitted/zero radix uses decimal or 0x hexadecimal; radix 2–36; invalid radix/no digits yields NaN. Stops at first invalid digit. | NumberParseInt/tsr_number_parse_int |
| `Number.isFinite` | `isFinite(value: number): boolean` | Tests for a finite number, excluding infinities and NaN. Requires one number argument; performs no string/boolean coercion. | number_predicate_target/f64 predicate |
| `Number.isNaN` | `isNaN(value: number): boolean` | Tests for the NaN value; ordinary numbers and infinities return false. Requires one number argument; performs no string/boolean coercion. | number_predicate_target/f64 predicate |
| `Number.isInteger` | `isInteger(value: number): boolean` | Tests whether a finite number has no fractional part. Requires one number argument; performs no string/boolean coercion. | number_predicate_target/f64 predicate |
| `Number.isSafeInteger` | `isSafeInteger(value: number): boolean` | Tests whether a finite integral number lies between -9007199254740991 and 9007199254740991. Requires one number argument; performs no string/boolean coercion. | number_predicate_target/f64 predicate |
| `Number.EPSILON` | `EPSILON: number` | Gap between 1 and the next larger f64 value (2^-52). Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |
| `Number.MAX_VALUE` | `MAX_VALUE: number` | Largest finite positive f64 value, approximately 1.7976931348623157e308. Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |
| `Number.MIN_VALUE` | `MIN_VALUE: number` | Smallest positive f64 subnormal, approximately 5e-324; not the smallest normal value. Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |
| `Number.MAX_SAFE_INTEGER` | `MAX_SAFE_INTEGER: number` | Largest integer exactly safe for f64 integer arithmetic: 9007199254740991. Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |
| `Number.MIN_SAFE_INTEGER` | `MIN_SAFE_INTEGER: number` | Smallest integer exactly safe for f64 integer arithmetic: -9007199254740991. Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |
| `Number.POSITIVE_INFINITY` | `POSITIVE_INFINITY: number` | Positive infinite f64 value. Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |
| `Number.NEGATIVE_INFINITY` | `NEGATIVE_INFINITY: number` | Negative infinite f64 value. Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |
| `Number.NaN` | `NaN: number` | Not-a-number value. Equality with itself is false; use Number.isNaN to test it. Read-only number constant; takes no arguments. | NumericConstant::lookup/rust |

## Independent reconciliation and dependencies

Source anchors are under crates/tsrust-compiler/src:

- analysis.rs::math_target: 36 recognized method names, independently reconciled
  against the Math catalog. math_arguments checks counts and numeric types.
- syntax.rs::NumericConstant::lookup/rust: eight Math and eight Number constants,
  independently reconciled against catalog names and emitted f64 values.
- analysis.rs::number_predicate_target/number_predicate: four checked predicates;
  emit.rs tests finite/fractional status and the inclusive 2^53−1 safe bound.
- analysis.rs::number_parse_int/number_parse_float and runtime_strings.rs provide
  the existing checked string parsing contracts; invalid input returns NaN.
- emit.rs::math_exact_call/math_approx_call and the direct unary targets select
  native operations. runtime_math.rs provides round/sign/half rounding/random,
  32-bit conversion, min/max, pow guards and stable hypot reduction.
- emit.rs includes runtime::MATH_EXACT when generated output uses tsr_math_ helpers.
  Constants and simple direct f64 methods require no separate helper call. Helper
  inclusion is family-level; no per-method binary-elimination claim is made.
  No user linker flag or additional imported package is required.

The current catalog names equal the independently extracted compiler target and
constant sets. Behavioral review checks the emitter/runtime as well as catalog
text; agreement of names alone is not acceptance. The global Number conversion
entry, numeric receiver formatting methods and other utility namespaces still
need their own audit rows.

## Acceptance evidence

SQLite records `tsrust-api-math-number-*.log` retain focused native IEEE/domain,
argument evaluation, ownership, shadowing and editor checks. Every Math/Number
catalog executable example is combined into one native program with isolated
blocks, formatted twice to verify idempotence, and compared against concatenated
expected output. The installed compiler used for examples has the same compiler
source as this documentation checkpoint. Whole-project formatting also runs before
commit. No timing run or full-family regression is claimed.
