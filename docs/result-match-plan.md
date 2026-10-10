# B10: tagged Result and exhaustive matching

Status: implementation started. The owner accepted Rust-style `Ok` and `Err`
constructors and exhaustive `match` on 2026-10-10. B10 is not accepted.

## Public contract

`Result<T, E>` stores exactly one payload. `Ok(value)` constructs success and
`Err(error)` constructs failure. An expected Result type supplies the other type
argument; a context-free constructor must provide both arguments explicitly as
`Ok<T, E>(value)` or `Err<T, E>(error)`. Missing type information produces a source
diagnostic. Construction transfers owned payloads under the existing rules.

The first increment supports statement matching:

```ts
function divide(a: number, b: number): Result<number, string> {
    if (b === 0) {
        return Err("Cannot divide by zero");
    }
    return Ok(a / b);
}

function main() {
    match (divide(10, 2)) {
        Ok(value) => { console.log(value); },
        Err(error) => { console.error(error); },
    }
}
```

The scrutinee is evaluated once. Only the selected arm executes. Payload bindings
exist only in their arm. Both variants must be covered, or a final `_` arm must
cover the remainder. Duplicate or unreachable arms are diagnosed. There are no
guards in the first increment. Match expressions and nested patterns require
later increments; they must not be silently accepted with partial semantics.

Matching an owned Result transfers it and exposes the active owned payload.
Borrowed matching must preserve the existing read/mutate contracts; it must never
copy a payload implicitly or permit a borrowed payload to escape. Inactive payload
properties are not exposed. This is tagged data, not exception throwing.

Propagation begins with explicit `return Err(error)` in the failure arm. Returning
from every arm satisfies the enclosing function's return requirement. `?` shorthand
is separate work. General discriminated unions remain D2, but the pattern model,
finite-variant coverage checks and arm control flow must be reusable there.

## Implementation chunks and acceptance gates

1. Add executable construction/matching witnesses, then a small finite-variant
   coverage component with exhaustive, missing, duplicate and wildcard tests.
2. Add span-preserving patterns and match statements to syntax and all source
   visitors. Keep recursive dispatch frames small. Check malformed arms, lexical
   binding scope, contextual keyword behavior and the normal depth limits.
3. Intern Result identities by both checked payload types. Add contextual and
   explicit constructor checking, substitution and module identity. No synthetic
   record containing both initialized payloads and no erased dynamic payload.
4. Check match arms using variant payload types and the shared coverage component.
   Join availability, loans, return and exception state across reachable arms.
   Cover replacement, moved results, unused payloads, callbacks and recursion.
5. Emit ordinary native tagged enums and one scrutinee evaluation. Verify owned
   strings, records, containers, callable payloads, nullable payloads and cleanup
   on early return/failure. Add read/mutate matching with explicit ownership tests.
6. Complete editor coloring, completion, hover, parameter help, diagnostics and
   module-aware facts. Document constructors and examples in the built-in catalog.
   Verify formatting, idempotence and formatted native execution.
7. Register matched TypeScript tagged-union workloads, independent oracles,
   contracts, observers, driver hooks and hashes. Run bounded untimed verification.
8. At the completed Result/match family milestone, run complete compiler/editor
   regressions, detect aborted suites explicitly, measure registered workloads and
   review performance. Store and verify evidence in SQLite, update roadmap and
   acceptance records, commit and refresh the five consumers.

## Current evidence

B9 baseline is `fc9ac96a`. The current verified formatter restored 330 pre-existing
formatting changes to committed source. A second project-wide pass changed zero
files and zero benchmark fingerprints. No B10 behavior has yet been accepted.

The construction/match integration witness in `tests/result_match.rs` is red:
the current compiler reports `Expected ";"` at the first match body. Its independent
TypeScript tagged-union reference passes under Node 24.11.1. Four unit tests for
finite-variant coverage pass, covering declaration-ordered missing variants,
duplicates, unreachable arms, wildcards and invalid indices. The coverage module
is currently compiled only in tests until checked match statements consume it;
it is not evidence of executable language support. No full regression or timing
run has been performed for this initial chunk.

The isolated `match_syntax` inspection API now parses the scrutinee with the existing
expression parser and arm blocks with the existing statement parser. It retains
internal expression/body trees and exposes original-source spans for inspection.
Five syntax tests pass: variant/discard/wildcard patterns, Unicode coordinates,
malformed arms, coverage errors deferred to checking, formatter idempotence, and
normal versus excessive body depth. All eight standing depth canaries plus the
recovered rest-parameter and exact-math canaries pass with successful process exit
codes and no aborted target. Evidence is stored in SQLite at
`runtime/results/result-match-parser-2026-10-10.json`, imported and read back exactly;
the loose copy was removed.

This API is not yet wired into program statement dispatch. Nested match statements,
checked Result types, native emission and editor support remain pending. The
end-to-end witness remains red. This is a source checkpoint, not gate acceptance.

## Construction and matching increment

The preceding syntax-only status describes checkpoint `2be14c83`. The current
increment wires statement matching into the ordinary parser. Like switch lowering,
it uses an owned temporary and existing blocks/branches. Compiler-only operations
validate finite coverage before the branches and project only the selected variant.
Extraction first moves the entire private variant to a temporary, then moves its
payload using the existing record ownership rules. No visitor family or cleanup
model is bypassed.

Result type identity uses two distinct private nominal alternatives in the existing
tagged-union arena. Each alternative contains only its own payload; both type
arguments participate in identity. A void alternative has zero fields and its
constructor takes zero arguments. Nested Results, arrays, nominal records and
nullable payloads have focused type evidence. Contextual constructors and nested
matching execute natively, including single evaluation and explicit owned-error
propagation. The original end-to-end witness is now green.

Constructor catalog entries include behavior, arity, context requirements,
ownership and executable examples. Focused hover, signature help and completion
pass; the TextMate grammar recognizes match. All three catalog examples execute
with exact output after idempotent formatting. See `result-match.md` for source
examples and current limits.

Next work is explicit constructor type arguments, generic/callable payload
integration, borrowed matching and scope/drop stress, pattern-binding editor facts,
project assistance and matched benchmark support. Complete the family milestone
before claiming B10 acceptance or refreshing installed consumers.

Current bounded evidence: seven Result integration tests, five match syntax tests
and ten depth canaries pass. The three executable Result witnesses format in memory,
require idempotence and run the formatted source. All twelve compiler commands
exited successfully, with no absent result line or stack abort. Their complete
outputs are stored in SQLite at
`runtime/results/result-match-construction-2026-10-10-checks.json`; exact readback
passed and loose copies were removed. One focused editor test and the real TextMate
tokenization test pass. This is not a whole-crate regression or a timing result.
