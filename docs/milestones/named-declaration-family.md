# Named declaration family milestone

Status: source requirements, measurements and performance review accepted; executable publication and final handoff remain open. Gates 170–173 and the subsequent callable replacement family are accepted. This milestone closes the accumulated declaration workloads without reopening completed language gates or claiming unsupported scoped aggregate composition.

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
| Matched workload readiness and independent oracles | Twelve accumulated workloads listed below, source contracts, registration tests, native ABI and runner/profile/report hooks | Twelve current registrations pass; matched measurements and hashes verified |
| Complete compiler/editor regression reconciliation | Callable-family milestone and affected publication repair: 2,325 compiler passes and 358 editor passes, supplemented by two new compiler tests and one new checked-editor test, eight canaries | Completed unchanged-source evidence retained; context supplements pass |
| Accumulated measurement, performance review and final records | Three original bounded groups and two optimized affected workloads accepted in SQLite | Accepted |

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

Review comparisons within matching semantics and measurement scopes. Investigate actual regressions before optimization; faster native results alone do not justify refactoring. Preserve exact raw records, normalize accepted analytics, verify readback and remove closed loose copies. Measurement status and generated coverage now record the accepted family. Coordinate any stdlib publication through exclusive ownership transfer; one agent edits main at a time.

## Original measurement and allocation review

Three bounded execution-only groups pass all five correctness cases and retain 2,232 original timing samples: 93 samples per engine for each of twelve workloads. Records are runtime/results/2026-10-08T08-16-11-141Z.json, runtime/results/2026-10-08T08-17-02-755Z.json and runtime/results/2026-10-08T08-19-23-107Z.json. Nine allocation samples cover the three slower native workloads. Exact records and normalized analytics are accepted through runtime/diagnostics/named-declaration-family-2026-10-08/baseline-acceptance.json in SQLite. Loose records and logs are removed after exact readback.

Native median execution is faster in nine workloads and slower in nested_function_results, nested_function_rest and generic_function_captures. Results and captured generic calls each allocate 2,049 times for 2,048 scenarios, matching retained output storage. The rest workload allocates 12,289 times: six allocations per scenario plus the outer container, of which five snapshots per scenario are avoidable. The original immutable baseline proof incorrectly says six allocations beyond output storage; this paragraph corrects that review wording without changing original records or measurements. Native rest median is 0.204209 ms versus 0.102083 ms for Node after TypeScript compilation.

A new native allocation-budget witness fails with 2,001 allocations for 1,000 small scalar rest calls. The emitter now stages eager scalar snapshots in the existing bounded inline Array storage, retaining heap spill for larger snapshots. It applies only to ordinary calls with copied int, number or boolean elements. Borrowing/consuming elements and builtin dispatch retain their existing carriers. Eager staging preserves evaluation order when later arguments mutate an earlier spread source.

The budget witness passes at optimization levels 0 and 3. All 47 affected array-literal spread, ordinary/iterable call spread, captured rest and rest-parameter checks pass, including source snapshot order and ownership/failure controls. This optimization is checkpointed in 15af854. Changed generated programs receive fresh correctness, timing and allocation evidence. Unchanged programs retain their original measurements.

The allocation optimization is checkpointed in 15af854. All 359 editor tests pass. Canonical generated-program comparison identifies only nested_function_rest and generic_value_adapters as changed; ten workloads retain exact emitted hashes and their original measurements. The optimized rest profile records 2,049 allocations and eight reallocations, removing 10,240 snapshot allocations from the 2,048-scenario batch. A serial release comparison succeeds after debug and concurrent release checks of the large generic adapter workload hit a 120-second limit. A live stack sample identifies specialization worklist and guard-partition/state joins. These compiler-efficiency observations remain a follow-up; their cause is not yet proven. Preserve the first unnormalized comparison as rejected evidence and use the final trimmed-stdout comparison, matching the benchmark runner. Affected timing is accepted; executable publication and final handoff remain pending.

## Accepted optimized measurements and regression reconciliation

The two changed programs pass all five correctness cases and retain 372 new timing samples under the original protocol. Record runtime/results/2026-10-08T08-44-35-439Z.json and the three-sample allocation record runtime/diagnostics/2026-10-08T08-30-41-367Z-native-allocations.json are bound by optimized-acceptance.json in the same SQLite diagnostic family. Original and optimized samples remain separate. The family retains 2,604 timing samples and twelve allocation samples in total.

Optimized rest native median is 0.162500 ms versus 0.102833 ms for compiled Node: about 20.4% lower native time than the original run, while still slower than Node. Generic adapters record 0.564584 ms native and 1.921541 ms compiled Node. The roughly 2.6% increase in native time is retained as a small cross-run signal, not a performance win or an established regression. Ten programs retain identical generated source and original measurements.

The retained complete compiler milestone has 2,325 unique passing checks. Two context audit tests and one allocation-budget test extend that evidence to 2,328 unique checks; this is reconciled milestone evidence, not a fresh complete compiler run after the emitter change. The changed emitter passes 47 spread/rest checks and the complete affected project and generic adapter groups (89 and 88 respectively). The whole current editor passes 359 checks. Twelve workload registrations pass after replacing a stale last-manifest-row assumption with a unique workload identity check; hash, mutation, oracle and execution-hook controls remain intact.
