# Gate 175: named function expressions

Status: accepted. Requirement B1 of
[the compiler completion ledger](../plans/compiler-completion-plan.md).

## Behavior

`function name(...) { ... }` in expression position is a function value. The name
is visible only inside that function, so the function can call itself. Nothing is
declared in the surrounding block. Parameters, defaults, rest, type parameters,
result annotation or inference, and the rules for variables of the enclosing
function are those of a nested function declaration used as a value. The
[guide](https://forever-frameworks-llc.github.io/tsrust/language/function-expressions/#give-an-expression-a-private-recursive-name) describes it.

## Implementation

- The parser turns the expression into a nested declaration that it queues for the
  enclosing block, under a name that source cannot spell
  (`written#expression<offset>`), and leaves a reference to that name in place of the
  expression. An arrow with an expression body keeps the declaration inside the arrow.
- The lexical resolver strips the marker for the source name and makes the written
  name visible in the function's own body only (`local_functions::written_name`).
  Capture planning, value wrapping, generic values and escape rules are unchanged.
- The resolver now removes the empty block that a lifted declaration left behind. A
  declaration after `return` was reported as an unreachable statement before; that
  also affected ordinary nested declarations.
- Assigning to a function name reported the internal emitted name. It now reports
  `Function "name" cannot be assigned`.
- The editor scanner scopes the name of a function expression to its body, and treats
  `const f = function name(...)` as a callable binding.

## Limit

A named function expression cannot be a parameter default: a default is evaluated
before the body block that would hold the private declaration. The diagnostic names
the arrow form.

## Evidence

- `named_function_expressions`: native fixture (recursion, private name, captures
  with block release, argument and array callback use, parameter shadowing, factory
  with default, rest, generic, arrow body, mutual private names, immediate call) and
  seven rejection rows. The fixture is formatter-stable and passes the pinned Node
  reference.
- Bounded compiler batch with unit tests, all eight depth canaries, formatting and
  the nested function suites: 692 pass, 0 fail.
- Whole editor crate: 363 pass, including hover, definition, signature help,
  completion inside and outside the function, and the `name` diagnostic for use
  outside.
- Matched `named_function_expressions` workload appended at the end of the manifest
  with oracle, token-based source contract, registration test and run, profile,
  oracle and report hooks. Native and Node-after-tsc agree on five untimed cases. No
  timing.
- The guide example runs natively and passes the Node reference.
