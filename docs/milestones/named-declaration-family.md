# Named declaration family milestone

Status: source requirement audit complete; accumulated measurement and final acceptance remain open. Gates 170–173 and the subsequent callable replacement family are accepted. This milestone closes the accumulated declaration workloads without reopening completed language gates or claiming unsupported scoped aggregate composition.

## Contract and source audit

The owner-selected contract uses existing ownership rules: lexical capture borrowing, explicit ownership transfer for escaping callables, scalar copy semantics and no shared JavaScript capture cells. Recursive value results require an explicit annotation; this is an accepted inference boundary. Named function expressions remain a separate unsupported feature.

| Plan requirement | Direct source evidence | Current status |
| --- | --- | --- |
| Lexical declarations, hoisting, shadowing and same-scope conflicts | nested_function_declarations.rs: hoisting, scope conflicts and closure parameter shadowing; hoisting.tsr | Accepted; complete compiler milestone covers these checks |
| Self and mutual recursion, finite native environments | hoisting.tsr, recursive-forwarding.tsr, retained.tsr; recursive request convergence controls | Accepted |
| Capture initialization and later bindings | nested_function_captures.rs: readiness, transitive initialization, shadow initialization and failure paths | Accepted |
| Readonly/mutable captures and fresh loop environments | borrowed-calls.tsr, mutable-block.tsr, fresh-loop.tsr; capture borrowing controls | Accepted |
| Owned escape, aggregate storage, once invocation and disposal | nested_function_owned_escape.rs: independent owners, retained arrays/records, disposal at optimization levels 0 and 3, escape rejection | Accepted for existing owned storage; scoped aggregate extensions remain separate |
| Defaults, optional/rest parameters and parameter patterns | function-value-signatures.tsr; generic_value_signatures.rs; default/rest native witnesses and generic adapters | Accepted |
| Generic values, concrete adapters and inferred results | generic_function_values.rs, generic_value_adapters.rs, inferred-result.tsr; result-inference rejection controls | Accepted |
| Imported nominal types and source identities | nested capture module fixtures; checked_project_local_declarations_keep_source_names_and_imported_result_identity | Accepted |
| Catch/finally lexical declarations and live captures | New nested_declarations_hoist_inside_catch_and_finally_with_live_captures project/native witness | Pass: formatted project/native execution and independent Node assertions |
| Invalid bodies in unreachable catch declarations | New unreachable_catch_declarations_still_check_their_bodies type/name controls | Pass: type/name rejection controls |
| Default worker stack and declaration depth | deep_nested_declarations_check_with_the_original_worker_stack; eight depth canaries | Accepted on current compiler source |
| Editor discovery, hover, definition, references, completion, outline and signature help | Nested declaration/capture intelligence checks and checked project tests; complete editor milestone | Accepted |
| Local main does not become the entry; named expressions remain separate | Scope/local-main check; function_expressions_keep_static_contracts | Accepted |
| Formatting, guide and executable examples | Format round-trip helpers, nested-functions.md and native/project witnesses | Accepted; new context witness explicitly formats and rechecks |
| Matched workload readiness and independent oracles | Twelve accumulated workloads listed below, source contracts, registration tests, native ABI and runner/profile/report hooks | Existing untimed acceptance; current manifest hashes inspected |
| Complete compiler/editor regression reconciliation | Callable-family milestone and affected publication repair: 2,325 compiler passes and 358 editor passes, supplemented by two new compiler tests and one new checked-editor test, eight canaries | Completed unchanged-source evidence retained; context supplements pass |
| Accumulated measurement, performance review and final records | Pending three bounded groups under the retained execution-only protocol | Open |

The dead-catch probe confirms nested declaration bodies already receive type and name checking even when the catch cannot execute. This differs from the broader historical unreachable-statement limitation and does not require a new compiler implementation. A draft assertion of an unrefined caught runtime-error payload reports an internal unsupported type; the accepted context witness compares the payload with its expected value before asserting the boolean result. That broader assertion diagnostic remains a separate issue, not evidence of named-declaration parity.

## Accumulated workload scope

Measure these twelve accepted workloads, preserving five correctness cases, independent oracles, operation denominators, three processes per engine, eight warmups and 31 samples per process:

1. nested_function_declarations
2. nested_function_captures
3. nested_function_owned_escape
4. nested_function_results
5. nested_function_defaults
6. nested_function_rest
7. generic_function_contexts
8. generic_function_deferred
9. generic_function_captures
10. generic_function_results
11. generic_value_adapters
12. scoped_callable_records

Use native and Node after TypeScript compilation, execution-only. Run bounded groups and retain completed groups instead of restarting them. The separately accepted nineteen-observation generic_callable_replacement workload has current verification and reviewed measurement; do not retime it. The older generic_function_record_pipeline and unrelated stdlib workloads are outside this milestone.

Review comparisons within matching semantics and measurement scopes. Investigate actual regressions before optimization; faster native results alone do not justify refactoring. Preserve exact raw records, normalize accepted analytics, verify readback and remove closed loose copies. Final measurement status and coverage updates remain pending. Coordinate any stdlib publication through exclusive ownership transfer; one agent edits main at a time.
