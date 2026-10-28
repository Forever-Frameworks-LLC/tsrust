# Milestone 98: optional and default parameters

Status: focused implementation accepted. Full regression and accumulated performance comparisons remain queued for checkpoint 101.

The [optional and default parameter guide](../../website/src/content/docs/language/optional-default-parameters.md) documents examples verified with the integrated compiler and native execution. [Acceptance evidence](../../benchmarks/runtime/diagnostics/gate98-optional-default-implementation.json) binds the compiler/editor integration, focused checks, exact native outputs and retained original failures.

## Required behavior

Support optional parameters (`value?: T`) and expression defaults (`value: T = expression`) throughout the currently supported parameter domain. Cover primitive values, owned and borrowed strings, arrays, records, supported nullable data, declared functions, concrete closures, read/mutate/once/owning callable contracts and finite stored callable families. Keep existing access modes, origin restrictions, generic bounds, capture lifetimes and move checks.

An optional parameter accepts omission or explicit `undefined`. Its body type includes `undefined`, so ordinary access requires the existing narrowing or nullish operations. A default parameter evaluates its initializer when the supplied value is omitted or `undefined`; a supplied `null` remains a supplied value when the declared type accepts it. Do not silently introduce truthiness or additional coercions.

Evaluate the callee and supplied arguments once, in source order, before entering parameter initialization. Evaluate reached defaults once, in parameter order. Earlier initialized parameters are available to later defaults under the existing ownership rules. Later parameters are not available during an earlier default. A reached default may perform ordinary supported calls and construct owned values; a skipped default performs no work and creates no owner.

Arity must account for required positions rather than simply counting required parameters. A default before a required parameter still needs an argument slot, which may contain explicit `undefined`. Reject an optional parameter followed by a required parameter and reject simultaneous optional-marker/default syntax. Preserve strict rejection of extra arguments. Empty calls are valid only when every required position is satisfied.

## Shared contracts and native representation

Omission belongs in the checked callable signature, including source callable annotations, generic substitutions, contextual arrow typing, imported signatures and stored families. Preserve it in hover, completion and parameter help. Callable compatibility must compare accepted argument positions and checked argument/result contracts; do not discard optionality when a function becomes a value.

A default before a required parameter accepts explicit `undefined` but retains its argument slot. Its canonical callable signature must therefore show a required nullable input such as `(first: int | undefined, required: int) => int`, rather than an invalid optional parameter before a required parameter. Trailing default parameters can render as optional inputs. Compare normalized supplied types and argument bounds, so equivalent accepted call contracts do not differ merely because one declaration initializes a default and another receives an explicitly nullable input.

Apply the same call rules to direct declarations, generic specializations, concrete closures, borrowed and owning callbacks, once calls, callable record fields, conditional selections, optional invocation and finite stored families. Existing array callbacks and iterator consumers must retain their documented supplied argument lists. Default behavior belongs to the invoked callable; a forwarding wrapper must not eagerly evaluate another callable's default.

Use the existing typed absent-value machinery and standard Rust ownership. Omission must not require trait-object boxing, hidden clones, erased callback origins, new Copy permissions or unsafe production code. Distinguish a caller's supplied type from the callee's initialized body type when a default removes `undefined`.

## Acceptance evidence

- Check omitted, explicitly undefined, present and accepted-null values for every current parameter kind and supported access mode. Include generic/imported signatures, nested callable parameters/results and stored-family dispatch.
- Check required-position arity, invalid ordering, duplicate syntax, annotation errors, default scope and mismatched supplied/default types with precise source diagnostics.
- Execute argument/default ordering, skipped side effects, earlier-parameter dependencies and once-only evaluation at O0 and O3. Include a default before a required position and multiple dependent defaults.
- Check owning defaults, borrowed inputs, closure captures, once consumption, normal disposal and failure cleanup. Preserve existing rejection of incompatible origins, escaping borrows and repeated use after a move.
- Check higher-order and array-callback invocation paths so omission cannot disappear during specialization or adapter generation. Verify editor signatures and active parameter help for incomplete calls as well as complete source.
- Publish concise executable language examples only after their compiler and native checks pass. Any new built-in method or property still needs complete verified documentation in `crates/tsrust-lsp/src/builtins.json`; parameter syntax alone introduces no built-in name.
- Register matched TypeScript/Node scenario comparisons for runtime behavior and queue them for checkpoint 101. Report compilation, diagnostics and ownership-only guarantees separately when they lack a runtime counterpart. Keep individual samples and original run records in canonical SQLite analytics.

Rest parameters, destructuring, heterogeneous unions and nullable array elements remain separate features. They do not justify omitting currently supported parameter kinds from this gate.

## Reference behavior prerequisite

[Untimed reference evidence](../../benchmarks/runtime/diagnostics/gate98-default-reference/manifest.json) retains seven literal expected output lines and five precise diagnostic cases against pinned TypeScript 5.9.3 and Node 24.11.1. The witness checks callee and supplied-argument effects before defaults, lazy dependent defaults, omission and explicit undefined versus supplied null, defaults before required positions and inferred default arrows. Diagnostic witnesses cover later-parameter and body-only scope, invalid optional ordering, optional-plus-default syntax and missing required positions. These checks establish the reference behavior only; they do not establish native compiler or editor acceptance.

A separate [required-slot forwarding reference](../../benchmarks/runtime/diagnostics/gate98-default-forwarding-reference/manifest.json) verifies that a function with a default before a required parameter can be assigned to a callable taking `int | undefined` in that required slot. Direct, forwarded and optional invocation retain three exact output lines. The corresponding native fixture passes at O0 and O3, with those outputs retained in the acceptance evidence.

## Focused acceptance

The current data and callable parameter domain is accepted, including omission, explicit undefined, nullable defaults, lazy initializer effects, earlier-parameter dependencies, imported and generic signatures, owning captures and mutable borrowed callbacks. Generic default validation checks initializer scope and known built-in return shapes before specialization. Parameter rebinding remains prohibited under existing rules.

The final compiler snapshot checks and emits all 46 retained native fixtures. Thirty-six emitted programs remain byte-identical to their accepted native records. Changed programs pass affected O0/O3 execution; the unchanged passing data supplement is retained. Thirteen matched benchmark sources have correctness acceptance and are registered for checkpoint 101. Twenty-four normal, skipped-default and panic cleanup scenarios retain their original resource evidence. No new timing samples are claimed.

Fourteen array callback catalog entries have consistent argument lists and executable default-tail examples. The exact examples pass native execution; hover, completion, parameter help and parameter-initializer visibility retain their focused protocol checks. No new built-in name is introduced.
