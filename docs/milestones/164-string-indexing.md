# Gate 164: string indexing

`text[position]` reads the character at a position of a string as a new one-character
string. Positions are `int` values that count whole Unicode characters, the unit of
`charAt`, `at`, `codePointAt` and `for...of`. The result type is `string`.

A negative position or a position at or after the end is a range error, with the same
transport as an array index out of bounds: catchable inside `try`, and outside any
`try` the program stops with `string index out of range at source bytes a..b`. This
follows the owner's direction to stay consistent with arrays; JavaScript gives
`undefined` there. A string cannot be assigned through a position (`unsupported`).
See docs/string-indexing.md.

## Implementation

- Analysis: `Analyzer::index_expression` sends a string-typed receiver to
  `string_index`, which reads the text, holds a read loan on it while the position is
  evaluated, and requires an `int` position. The index type hint reports `string`.
  `reject_string_position_write` rejects assignment, compound assignment, logical
  assignment and update of a string position.
- Emission: `Emitter::string_index` binds both operands in a match scrutinee, converts
  the position with `usize::try_from`, selects with `chars().nth` and creates an owned
  string. The failure uses `runtime_failure_exit("RangeError", "bounds", ...)`.
- The receiver diagnostic now reads "Indexing requires an array, a tuple or a string."

## Optional positions

`text?.[position]` on a `string | undefined` or `string | null` value gives
`undefined` when the text is absent and checks the position when it is present. It
was rejected in the first acceptance of this gate and completed in the follow-up
commit. Optional-chain analysis and emission classified every index body as a
borrowed place of the receiver; a string position is a new value. `string_position`
in the analyzer and the string arm of `borrowed_place_expression` in the emitter
mark it as owned, so bindings, comparisons, fields, `??` and further optional hops
use the owned-result path that optional method calls use. Evidence:
target/editor/string-indexing-optional.log.

## Defects found and repaired

- An optional selection on a statically absent receiver, compared with `undefined`
  (`values?.[0] === undefined` where `values` is known to be `undefined`), emitted
  Rust that could not infer the payload type of the result view. The dead present arm
  now keeps the selected expression after its unreachable check. This failed before
  this gate for arrays and method calls. Witness:
  `absent_optional_selections_compare_with_undefined`.
- Editor: a binding initialized by one comparison, such as `const same = n === 1`,
  had no hover type. The scanner splits comparison operators into single symbols;
  `after_comparison` recognizes them and the binding is `boolean`.
- Editor: `map_group_by_catalog_example_checks` required the fixture to contain the
  catalog example text verbatim and failed with a formatted fixture. It compares
  without whitespace now.

## Focused evidence

- Red: target/editor/string-indexing-red.log (four failing compiler checks) and
  target/editor/string-indexing-editor-red.log.
- Compiler: `string_indexing` 5 checks: std:testing fixture with literal, variable,
  field, call-result and nested-array receivers, evaluation order, loops and caught
  range errors; Unicode positions; the uncaught report; twelve static rejections; the
  absent optional selection witness. Affected batch: 18 suites, 151 checks, including
  the depth canaries, `type_syntax`, `higher_order_function_values`, `optional_chain`
  and `exception_handling` (target/editor/string-indexing-final-compiler.log).
- The fixture and the guide example pass the pinned Node reference. The fixture's
  out-of-range blocks also end in their catch clause under Node, because the
  assertion on `undefined` throws there. Guide statements are checked in
  target/editor/string-indexing-guide-verification.log.
- Editor: all suites pass, 262 checks (target/editor/string-indexing-final-editor.log),
  including `string_positions_are_strings_in_the_editor` and the updated completion
  assertion for a comparison initializer.
- Harness 391/391 on the refreshed consumers (target/editor/string-indexing-harness.log).
- Five matched native/node-tsc cases pass untimed; the record
  runtime/results/2026-10-06T14-33-30-958Z-verify.json is imported into SQLite and its
  loose copy removed: string_indexing, type_aliases, numeric_literals, comma_contexts,
  exception_completion.
- Benchmark support: workload pair, independent oracle (charAt and a length test),
  source contract, registration test, manifest row (category strings) and
  oracle/run/profile/report hooks. The workload contains `try`, so its native entry
  returns a checked result; the `string-indexing` family is added to the result-aware
  adapter list. The TypeScript counterpart states the range check explicitly, because
  the platform returns `undefined` for a missing position.
- Five consumers rebuilt and replaced by atomic rename; hashes in
  target/editor/string-indexing-consumers.log.

## Follow-up: optional positions and dispatcher frame

- Optional positions: see the section above. `string_indexing` passes 5 checks with
  the extended fixture and the bounds witness.
- The depth canary `nullish_assignment` overflowed the test stack (not caused by this
  gate: `coalesce_assignment` 9.9 KB and `expression_kind` 7.7 KB per level). The
  synthesized wrapper arms of `Analyzer::expression_kind` (callable store, once, rest
  adapter, parameter stores, selected wrappers, projection, injection) moved to
  `wrapper_expression`. The dispatcher frame is 1.7 KB. A first version sent every
  unary expression through the helper and made `higher_order_function_values` and
  `number_operators` overflow; only the synthesized operators use the helper now.
  `nullish_coalescing` passes at a 1.7 MB stack.
- Batch after the follow-up: 37 compiler suites, 341 checks, 0 failures
  (target/editor/string-indexing-followup-compiler.log), including all depth canaries
  (`closures`, `for_of`, `owned_arrays`, `nullish_coalescing`, `nullish_assignment`,
  `higher_order_function_values`, `number_operators`, `type_syntax`).
- Consumers rebuilt and replaced again; five matched cases verified untimed again
  (record 2026-10-06T14-44-52-198Z-verify.json, imported into SQLite).

No full regression or timing was performed. The workload is unmeasured.
