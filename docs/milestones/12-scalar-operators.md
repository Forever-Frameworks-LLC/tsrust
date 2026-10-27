# Milestone 12: scalar operators

[Retained callbacks](./11-retained-callbacks.md) · [Primitive workstream](../guides/language/primitives-and-collections.md) · [Roadmap](../roadmap.md)

**Status:** Complete. This gate supplies numeric updates and boolean conditions before loop ownership analysis. All 162 regression groups pass, including 13 operator groups; formatting, Clippy, and local documentation links are verified.

## Provisional semantics

Integer arithmetic supports binary `+`, `-`, `*`, `/`, `%` and unary `-`. Every operand must have type `int`; results are signed 64-bit integers. There is no string concatenation, numeric coercion, or unary plus. Division truncates toward zero; a nonzero remainder has the dividend's sign.

Overflow, division by zero, and remainder by zero are fatal runtime failures. Both `MIN / -1` and `MIN % -1` fail. Generated Rust uses explicit checked operations followed by a failure message containing the operator's source byte range. Checks remain active with backend overflow checks disabled and under optimization. Validly typed expressions such as `1 / 0` pass frontend checking and fail only if executed; constant evaluation and recoverable arithmetic errors are separate work. This provisional panic behavior does not settle the general error model.

Comparisons `<`, `<=`, `>`, `>=` take integers and return booleans. Strict equality `===` and inequality `!==` require either two integers or two booleans. Strings, records, closures, and mixed operand types are rejected. Coercive `==` and `!=` are unsupported.

Boolean `!`, `&&`, and `||` require boolean operands and return booleans. `&&` evaluates its right operand only when the left is true; `||` evaluates it only when the left is false. There is no truthiness conversion or operand-value return behavior.

## Parsing and evaluation

From highest to lowest precedence:

1. Grouped expressions, field access, and calls.
2. Unary `-` and `!`.
3. `*`, `/`, `%`.
4. `+`, `-`.
5. `<`, `<=`, `>`, `>=`.
6. `===`, `!==`.
7. `&&`.
8. `||`.

Binary operators at the same level associate left. Operand expressions run once in source order. Parentheses override precedence. `1-2` is subtraction; `1 - -2` subtracts a negative integer. `++`, `--`, compound assignment, exponentiation, shifts, bitwise operators, and ternaries remain unsupported. Write separate minus tokens when negating a negative value; adjacent `--` is rejected as an update operator.

The lexer emits the minus sign separately. A directly following integer token is folded into a signed literal before range validation, preserving `-9223372036854775808` and exact range-error spans. Whitespace or comments between the sign and digits are accepted. Other negations use checked runtime negation. Positive out-of-range literals remain errors even inside a negation of a parenthesized expression.

Precedence parsing preserves the existing depth bounds. Unary chains are parsed iteratively; grouping recursion is kept outside the larger record/closure parser frame. Parsing and analysis reject excessive depth rather than overflowing the compiler stack.

## Ownership and effects

Scalar operand access ends once its value is copied. For example, `state.count + increment(state)` copies the old count before the second operand mutates the record. Loans held by an enclosing call or closure remain active across both operands.

The frontend checks both sides of `&&` and `||`, including literal conditions. A transfer that may occur on the right makes that value unavailable afterward. Its access also contributes to parameter and capture contracts. Thus `false && consume(value)` does not preserve later source-level use of `value`, and a closure that may consume a capture remains single-use. The analyzer does not correlate later branches with a particular short-circuit outcome. Runtime evaluation still skips the right operand when required.

Operators work in arguments, record fields, assignment values, conditions, returns, and closure bodies, including retained callbacks and callable-valued factories. Existing whole-binding ownership rules continue to apply.

## Exit evidence

- Exact native output for precedence, associativity, negative literals, all supported operators, evaluation order, short-circuit effects, and closure integration.
- Source diagnostics for operand types, unsupported syntax, malformed expressions, possible transfers, and enclosing loan conflicts.
- Runtime failure checks for every arithmetic failure category, compiled at both optimization level 0 with overflow checks enabled and level 3 with overflow checks disabled.
- Backend-independent frontend checks, exact operator byte spans after Unicode comments, accepted deep expressions, and excessive-depth diagnostics.
- Full regression suite, formatting, Clippy, updated current documentation, and local documentation link validation.

The [operator suite](../../crates/tsrust-compiler/tests/operators.rs) contains 13 groups. A [counter fixture](../../crates/tsrust-compiler/tests/fixtures/operators/counter.tsr) demonstrates a retained mutable environment using arithmetic and comparisons.

Loops, mutable local rebinding, collections, floats, conversions, general value equality, async, modules, and the language server remain separate scopes. [Milestone 13](./13-while-loops.md) subsequently implements while-loop control flow and repeated-iteration ownership.
