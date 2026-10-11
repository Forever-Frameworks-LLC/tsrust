# Gate 176: recursive result inference

Status: accepted. Requirement B2 of
[the compiler completion ledger](../plans/compiler-completion-plan.md).

## Behavior

A function that calls itself, directly or through another function, no longer needs
a result annotation when one of its returns does not depend on the recursive call.
That return fixes the result type; every other return, including the recursive ones,
is then checked against it as usual. Independent values are literals, parameters,
calls of already known results, one branch of a conditional expression and an
arithmetic operand of known type. This holds for top-level functions, nested
functions that use enclosing variables and named function expressions.

When no return is independent, or the independent value comes only from a local
variable, the `result-inference` diagnostic asks for an annotation.

TypeScript in strict mode rejects every one of these functions without an
annotation (TS7023). The accepted behavior is therefore a superset; the matched
TypeScript counterpart states the result types, and its source contract requires
that the native source does not.

## Implementation

- `return_type_inference::provisional_result` walks the returns of the body in source
  order and takes the first value whose type follows from the parameter context
  (`independent_value_hint`). It runs before the body check of a function without a
  result annotation and publishes the type in the signature, so type hints inside
  the body see it.
- When the checker meets a call of a function that is still being checked, it uses
  the provisional result instead of reporting the recursion.
- The body check is unchanged: the inferred result is still taken from the checked
  return expressions, and all returns must agree.
- The editor shows the checked result of a top-level function that has no written
  result type (`CheckedProgram::checked_function_results`). Before, only nested
  functions did.

## Evidence

- `named_result_inference`: native fixture with int, string, boolean and number
  results, mutual recursion, a nested capturing function and a named function
  expression; three rejection rows. The fixture is formatter-stable and passes the
  pinned Node reference.
- Bounded compiler batch with unit tests, all eight depth canaries and the function,
  recursion and generic value suites: 807 pass, 0 fail.
- Whole editor crate: 364 pass, including hover with the inferred result on the
  function and its call values, member completion and the remaining diagnostic.
- Matched `recursive_result_inference` workload at the end of the manifest, with
  oracle, token-based source contract (native without annotations, counterpart with
  five), registration test and hooks. Native and Node-after-tsc agree on five untimed
  cases. No timing.
