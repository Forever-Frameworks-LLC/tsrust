# Gate 124: destructuring and borrowed const bindings

Status: focused compiler/native, editor, documentation and benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior

`const` and `let` declarations and `for...of` headers accept array and record patterns, with nesting, skipped positions and renamed fields. The user guide is [Destructuring](../guides/language/destructuring.md).

The owner chose the ownership rule: a pattern follows the access of its source, and nothing is cloned.

- Copied positions are copied; the source stays usable.
- Owned positions of an owned source move out, as a hand-written `const label = point.label;` does. A named source keeps its other fields.
- A temporary is held once in a hidden binding.
- Positions of a `ReadOnly` parameter and of a loop element are readonly borrows.

Rest elements, defaults, parameter patterns and patterns in a classic `for` initializer are rejected with `unsupported`. Arrays of owned elements cannot be destructured by position, because indexing an array borrows; this is the existing indexing rule.

## Borrowed const bindings

A `const` whose value is a loop element, a `ReadOnly` parameter, or a field or position of one, is now a readonly borrow for its scope. Before this gate `for (const b of a) { const s = b.name; }` was a `borrowed-element` error. Transferring such a binding is a `borrowed-place` error. A borrowed owned union is held as its readonly view. This uses the existing scoped borrow mechanism for assignment results.

## Implementation

The parser lowers a pattern to ordinary declarations in the same block: one hidden binding for a temporary (`pattern#N`, which source text cannot spell), then one declaration per name that indexes or reads a field. A named source is used directly without a hidden binding. A `for...of` pattern iterates a hidden element name and declares its names first in the body. No later compiler pass needed a new statement kind.

Each generated declaration's value carries the span of its name. The checked type facts report the type at that span, and the editor uses it for hover, completion and member access on destructured names. For incomplete source the editor falls back to the written tuple, array or record type of the whole value for first-level names.

The formatter keeps a pattern on one line: `const { x, y: z } = value;` and `for (const [key, value] of pairs) {`. It also now writes `return [1];` with a space.

## Repairs

- `numbers ?? [1, 2]` with a `number[] | null` left side produced `number[] | int[]` since gate 120, so `.length` on the result failed. An int array literal beside a number array payload is a number array again.
- Three older tests expected a `borrowed-element` error for a const taken from a loop element, and one expected `unsupported` for `x ?? true`; they now check the new behavior.

## Not changed

`acceptance::missing_backend_reports_setup_without_fallback_or_output` still fails in this environment (see gate 123).

## Focused evidence

- `crates/tsrust-compiler/tests/destructuring.rs`: four tests; three Node-matched fixtures in `tests/fixtures/destructuring/` and nine static rejections.
- 27 of 31 affected suites passed in one run; the four failures were the three stale expectations and the coalescing regression above, plus the environment test. After repair these passed: `destructuring`, `formatting`, `owned_arrays`, `field_moves`, `nullish_coalescing`, `data_unions`, `nullable_unions`. The other suites in the run were `for_of`, `loops`, `control_flow`, `closures`, `nullish_assignment`, `nullish_assignment_aliases`, `optional_chaining`, `logical_values`, `callable_fields`, `array_for_each`, `nullable_narrowing`, `nullable_callables`, `object_values_entries`, `union_array_elements`, `array_entries`, `tuple_types`, `callable_ownership_defaults`, `optional_parameters`, `rest_parameters`, `nested_records`, `function_types`, `contextual_arrows` and the unit tests.
- All 183 editor tests passed, including `destructured_names_have_hover_types_completion_and_references`.
- Benchmark support: `destructuring_patterns` with an independent arithmetic oracle, source contract and registration test. Untimed native and Node-after-tsc verification passed. All 328 harness checks passed in one run.

The byte-exact baseline is `archive/destructuring-start`. No full compiler regression, timing or allocation profiling was run.
