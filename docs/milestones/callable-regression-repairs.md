# Callable and depth regression repairs

Focused repairs made after milestone 100 integration, before further gates. Targeted suites only; no full regression or timing was run.

## Why

Running the callable, closure, nullable and operator suites on the tree as it stood before call spread was integrated showed failures that the deferred checkpoint had not yet exercised. They came from three sources: the owned-by-default callable change, the contextual arrow and logical value work, and frame growth in shared recursive compiler functions during gates 97–99.

## Compiler defects fixed

- **Forwarded `mutate` callbacks.** A closure method that receives a `mutate` callback from its enclosing function was marked context-dependent. It was then neither emitted nor counted, so generated Rust called a missing `call0` method and the 128 instantiation limit was not enforced. `callback_needs_context` again applies to owned contracts only. Context-dependent methods are still counted once per caller witness vector as leaf edges.
- **Conditional initializer of a nullable callable.** `const action: (...) | undefined = flag ? make() : undefined` left a union with an unresolved callable member that reached native type emission and panicked. Such unions now use the generic callable payload layout.
- **Owned nullable data capture.** A closure capturing an owned `string | null` stored the payload type instead of the tagged union, producing invalid Rust. The capture field keeps the union type.
- **Annotated arrow parameters.** `(x: void) => ...` and access on a scalar such as `(n: mutate int) => ...` were accepted after the parameter-default work. Both are rejected again. Access on an owned data parameter, such as `(source: read int[]) => ...`, is an intended form used by the documented array callback examples and stays valid.
- **Nullable callback arguments.** A callback type with a nullable callback argument, such as `read (inner: (read () => int) | null, value: int) => int`, was not treated as a caller-specialized contract, so invoking it generated Rust with an undetermined type parameter. It now takes the same specialized path as a present callback argument.
- **Normal-stack depth.** One hundred nested conditionals overflowed the analyzer and one hundred nested unary or binary operators overflowed the emitter on the normal test stack. `checked_expression` keeps a thin retained frame with prelude and result helpers; conditional checking is split into predicate, plan, between-arm and join phases; unary and binary lowering have their own emitter frames. No stack size was raised.

## Fixtures migrated

These tests encoded the earlier borrowed default or earlier diagnostics. Each change states the intended contract explicitly.

- `capture_replacement`, `declared_function_values`, `conditional_expressions`, `recursion`: scoped callback parameters that must borrow now say `mutate`.
- `closures`, `closure_arguments`: concise arrow bodies and defaulted arrow parameters are valid and are now executed as positive cases; an unannotated arrow parameter without context reports `parameter-inference`.
- `operators`: `1 && true` and `true || 0` report `logical-result` at the operator.
- `nullable_callables`: new native test for conditional factory initializers.

## Evidence

- `target/editor/gate100-defect-repairs.log`: 66 callable, closure, nullable, parameter, spread, conditional and operator suites. After the repairs listed above every suite in this set passes.
- `target/editor/gate100-remaining-suites.log`: the other 71 compiler suites, run once because the frame changes affect every program: 389 tests passed, 18 failed. Three `string_addition` failures came from the first version of the emitter frame split and are fixed. Five `string_concat_integer_fusion` tests need an artifact directory variable that the run did not set. The other ten are the unmigrated fixtures below.
- `target/editor/gate100-final-confirm.log`: 23 suites rerun after the final emitter change; 182 passed, and the 2 failures are unmigrated fixtures below.
- Emission equivalence: a reference compiler built from the analyzer and emitter as they were before the frame splits produces byte-identical Rust for 887 sources (all registered workloads, compiler fixtures and guide examples) and identical rejections for 30 more; no source differs.

## Fixtures still assuming borrowed callable defaults

These ten tests still fail. Each passes a bare callable type where the test intends scoped borrowing, so it needs an explicit borrowing annotation. They were left untouched because the callable access spelling is being redesigned in the [function type migration plan](../function-type-migration-plan.md); migrate them there.

| Suite | Test | Symptom |
|---|---|---|
| `array_fields` | `captures_returns_and_callbacks_preserve_field_storage_access` | `use.f` is Consume, expected Mutate |
| `array_removal` | `callback_and_recursive_access_contracts_include_removal` | `use.f` is Consume, expected Mutate |
| `field_moves` | `callable_owned_argument_access_matches_partial_field_consumption` | `use.f` is Consume, expected Mutate |
| `nested_records` | `callback_contracts_and_retained_owned_children_preserve_access` | `use.f` is Consume, expected Mutate |
| `control_flow_native` | `lexical_blocks_release_capture_loans_before_loop_tails_and_preserve_factories` | `loop-transfer` on a callable passed in a loop update |
| `numbers` | `number_parameters_callbacks_reject_implicit_int_conversion_and_wrong_coordinates` | `closure-consumed` on a second pass of one callable |
| `owned_arrays` | `captures_and_owned_callback_contracts_preserve_element_access` | `callback-escape` for a borrowed capture |
| `primitive_identity` | `receiver_once_contracts_captures_temporaries_and_fixed_storage_are_preserved` | `callback-escape` for a borrowed capture |
| `string_concat_padding` | `operands_run_once_in_order_and_selected_temporaries_survive_borrowed_consumers` | `callback-escape` for a borrowed capture |
| `string_substring` | `once_only_order_borrowed_roots_and_selected_temporaries_cover_all_consumers` | `callback-escape` for a borrowed capture |

Outstanding items for call spread itself are recorded in [milestone 100](./100-call-spread.md).
