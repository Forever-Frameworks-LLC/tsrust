# Gate 143: checked `as` and `satisfies`

Status: accepted with focused checks and matched untimed verification. No timing or full regression was run.

## Behavior

`value as Type` supplies a compatible target type and context for empty arrays,
named record literals and callable expressions. A scalar `int as number` emits
an explicit floating-point conversion, including when the operand is a variable
or a call. Compatible unions receive the existing checked tag representation.
Unsafe union narrowing and incompatible assertions are rejected; the diagnostic
recommends `typeof` narrowing. There is no runtime tag check or cast failure.

`satisfies` checks compatibility and retains the source type. The witness
`3 satisfies number` remains an integer accepted by an `int` parameter. Empty
arrays, record literals and callable expressions receive required annotation
context. Assertions preserve moves and lexical borrows, including readonly
union views. No implicit clone is introduced.

The parser uses relational precedence for both operators. Primitive annotations
do not consume a following comparison as a generic type application. Assertion
chains retain the existing 128-level expression limit; the ordinary-depth
witness checks a 100-assertion chain. Explicit integer assertions also anchor
integer power/bitwise inference; `as number` supplies a number anchor.

## Implementation and repairs

The parser creates a boxed annotation and operand. Annotation substitution,
record materialization, project linking, generic-default scope checks and
checked-expression walks visit the new syntax. A dedicated analyzer helper
keeps unrelated temporaries out of the recursive expression dispatcher.

Checked assertions preserve source bindings and callable origins. Native
emission delegates ordinary assertions to the operand; integer widening emits
a numeric conversion. Transparent dispatch prevents duplicate borrows and
duplicate union-view extraction. Borrow-source collection preserves source
loans through the assertion and permits borrowed widening without constructing
an owning copy.

A positive borrowed read found a gap that rejection-only coverage had missed:
asserted element bindings originally attempted a move. After borrow inference
was repaired, native execution exposed double references and duplicate view
extraction. Positive scalar, nullable and widened borrowed-union cases now
exercise the complete compiler and backend path.

Exact originals are retained in `archive/type-assertions-*-start/`. Existing
exception/editor work and prior benchmark manifest rows are preserved.

## Evidence

Compiler red logs include
`/private/tmp/type-assertions-contract-red-current.log`, numeric and contextual
native failures, `/private/tmp/type-assertions-comparison-red-cli.log`, and
`/private/tmp/type-assertions-borrowed-union-red.log`. A power-inference
witness failed in `/private/tmp/type-assertions-power-red.log` before repair. The intermediate native
borrow failure is in `/private/tmp/type-assertions-borrow-final.log`.

Focused compiler suites cover assertions, type syntax, closure values,
nullish coalescing, for-of, owned arrays and generic defaults. The assertion
fixture uses `std:testing.strictEqual`; its independent Node counterpart only
replaces the testing import. Native and Node executions produce no output when
all assertions pass. Editor checks cover checked hover types, formatting
idempotence, syntax highlighting, and preservation of exception editor behavior.

- Final assertion suite: 6/6 (`target/editor/type-assertions-power-final.log`).
  This includes native assertions for arrays, records, generic records/functions,
  callables, integer/number power, moves, borrowed strings and borrowed unions.
- Regression canaries: closures 14, for-of 10, higher-order function values 6,
  nullish coalescing 4, owned arrays 12, type syntax 8, generic-default built-ins 2,
  generic-default scope 3. Combined with the six new tests, 65 focused compiler
  tests have passing evidence. Ordinary canaries passed earlier in the gate;
  final borrow/emission/power repairs affect only the new assertion variant and
  were verified by the final assertion suite. The failed intermediate assertion
  runs are retained rather than described as passing batches.
- Focused editor: 4/4 (`target/editor/type-assertions-accepted-editor.log`).
  These checks precede the final assertion-only power hint change; final editor
  consumers were rebuilt afterward. No complete editor run was performed.
- Final fixture passes with pinned Node v24.11.1
  (`target/editor/type-assertions-pinned-node.log`).
- Five consumers refreshed atomically; previous bytes are preserved in
  `archive/type-assertions-consumers-start`. Current hashes match
  `target/editor/type-assertions-consumers.log` after benchmark verification.
- Runtime harness: 369/369 (`target/editor/type-assertions-harness.log`), using
  login-shell Node v26.3.0. After the workload ABI signature repair, the affected
  registration check passes separately (`target/editor/type-assertions-abi-registration.log`).
- Pinned Node v24.11.1 native/node-tsc verification: five cases per engine,
  `benchmarks/runtime/results/2026-10-06T04-41-45-933Z-verify.json`.
  The initial Node v26 launch was rejected by the pin guard. The next attempt
  exposed the workload header's required ABI spacing; the source, counterpart,
  fingerprints and source-contract guard were repaired. Both failures are retained.
- No startup, compilation or execution timing rows were produced; existing
  SQLite measurements are unchanged.

## Benchmark readiness

`type_assertions` retains an array element, a half-integer result of widening,
a record field and dynamic input text. Matched TypeScript source uses identical
operations. The independent oracle uses arithmetic without executing assertions.
Source contracts guard hashes, scenario expressions and the operation denominator;
registration tests preserve prior rows and guard mutated sources. Runtime,
profiling, oracle and reporting hooks are registered. The workload is unmeasured.
