# B10 Result and statement-match acceptance map

This map covers S06's tagged Result foundation. B10/S06 is accepted at gate 186 after
regression, performance and publication checks. Earlier generic and callable
storage acceptance remains valid; this family does not repeat those implementations.

| Requirement | Implementation and evidence |
| --- | --- |
| Typed success/error representation | `result_types.rs` interns both payload types and private variant identities; only the active payload exists. Contextual and explicit constructors have native and diagnostic controls in `tests/result_match.rs`. |
| Exhaustiveness and arm scope | `match_syntax.rs` preserves spans and evaluates the scrutinee once; coverage checking rejects missing, duplicate and unreachable alternatives. Pattern bindings are arm-local and immutable. |
| Explicit typed propagation | Both arms can return; `return Err(error)` transfers the checked error payload. Nested matches, recursive forwarding and early returns execute in native fixtures. |
| Owned payloads and callbacks | Strings, records, arrays, nested Results and owned reusable/Once callables preserve existing move and invocation rules. No implicit clone is introduced. |
| Borrowing | ReadOnly matching lends payloads; Mutable matching lends exclusive access. Scalar payloads copy. Escapes, overlapping owner access and consuming borrowed payloads are diagnosed. Nested mutations and released arm loans have native witnesses. |
| Cleanup and exceptions | `result-match-cleanup/values.tsr` covers replacement, unused callbacks, early returns, finally, callback throws, Once invocation and loop exits. Its independent TypeScript reference passes. Counting-allocation checks restore the exact live-byte baseline over 32 body invocations at optimization levels zero and three. |
| Editor | Checked payload hover, members, navigation, constructor completion/signatures and diagnostics work in single files and projects. Imported aliases and nested identities remain distinct. Incomplete headers recover lexical assistance without inventing types. TextMate covers match, Result and constructor calls. |
| Formatting | Match arms retain code tokens and stable layout. Successful fixtures exercise idempotent formatting before checking or native execution; the whole-project checkpoint formatter remains required. |
| Benchmark | `result_match` retains seven independent observations per scenario: two readonly inspections, owned transfer, two stateful callback calls, mutable payload updates and callback replacement/failure/finally. Both variants are exercised. Arithmetic oracle, guarded sources, exact hashes, fallible native ABI, runner/profile hooks and registration controls are present. Five native and five Node-after-tsc cases verify. |

## Regression repair found during the milestone

An existing optional-call rejection witness exposed that a statically absent
nongeneric callable could discard its explicit type arguments before validation.
Optional chains now validate applications before short-circuit lowering. Six related
compiler controls pass; the editor has a matching formatted diagnostic witness.
The original failed regression record is preserved; remaining suites resume rather
than replaying the preceding successful suites.

## Deliberate boundaries

This family supports statement match with block arms and optional separating commas.
A void alternative uses a zero-argument constructor and a discard pattern. Match
expressions, guards, nested destructuring patterns and `?` propagation are separate
features. General discriminated unions remain the dedicated D2 family. Borrowed
callables cannot be direct Result payloads. Context-free inline callbacks may require
explicit generic arguments. These limits are documented in the user guide and are
not counted as implemented behavior.

## Evidence publication

Exact focused records and original benchmark records live in `benchmarks/records.sqlite`;
`docs/plans/result-match-plan.md` lists their record paths. The initial timing run overlapped
compiler regressions and is excluded from acceptance performance. Final evidence is recorded in `milestones/186-result-match.md`: compiler 2,754
passes, editor 472 passes, all depth canaries, uncontended timing and five-consumer
refresh.
