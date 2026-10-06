# Milestone 69: conditional expressions

Status: complete. Strict-boolean conditional expressions, 633 compiler groups, 648 workspace groups, ten matched comparisons and thirty fully disposed profiles are accepted.

Source, compiler and native/runtime validation are accepted. Complete original outputs and actual commands are retained by the [source proof](../../benchmarks/runtime/diagnostics/2026-10-02T22-20-19-000Z-conditional-expression-source-validation.json), [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T22-58-28-000Z-conditional-expressions-compiler-proof.json) and [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T23-12-32-000Z-conditional-expression-runtime-proof.json).

## Scope

Implement `predicate ? consequent : alternate` across currently supported expression contexts and compatible implemented value types: int, Number, boolean, String, arrays, records and callable values under the existing concrete callable-origin rules. Require a boolean predicate and the same exact branch type. Forward contextual types into literals, including named records and empty arrays. Preserve the existing native integer, UTF-8 and ownership contracts.

Match the [ECMAScript conditional grammar](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-conditional-operator): the predicate binds more tightly than assignment, and both branches accept assignment expressions. Nested conditionals associate to the right. Evaluate the predicate once, then evaluate only the selected branch. Emit ordinary Rust `if` expressions without implicit cloning, shared ownership or a new runtime container.

Analyze both branches from the state after the predicate. Join availability conservatively while retaining inferred access, mutation and capture contracts. An owned value transferred on either reaching branch is unavailable after the join unless both branches restore it. Borrowed results retain every possible source loan through the consuming expression. Preserve partial-field, loop, callback and indexed-storage restrictions. A conditional result is not an assignment target.

Record field and array index access through a conditional owner follows the existing temporary-selection contract: selection consumes the chosen owner, and the availability join protects both possible named sources afterward. Read-only string and array `.length` remains a borrowed observation. Direct borrowed conditional arguments retain both possible source loans. Fixed-storage inference must reconcile branch representations or conservatively use the existing growable representation for affected values; unrelated fixed-storage proofs remain valid.

Truthiness coercion, heterogeneous union results, optional/nullish types, new callable representations and owned assignment-expression results are separate work. These exclusions do not restrict conditional selection of compatible existing owned values.

## Acceptance

Four focused O0/O3 groups cover precedence and expression contexts, lazy predicate/branch effects, exact Number bits, contextual arrays/records, conservative transfers/restoration, partial fields and all possible borrowed-result roots. Same-origin reusable and single-use callables retain their access/escape contracts, including forwarded and recursive returned origins. Conditional field/index selection follows the existing owning-temporary path; borrowed observations keep selected temporaries alive through their consumer and drop them afterward. Fixed storage remains valid for unrelated values. Normal-stack diagnostics, zero-allocation scalar selection and normal/unwind owner disposal pass.

Six actual focused commands retain three passed groups while repairing compiler/exhaustiveness and temporary/callable issues plus authored fixture mistakes, then pass the remaining owned group exactly. Failed commands, source snapshots and repair records remain preserved. The sole full regression passes all 633 compiler groups and fifteen additional workspace groups without a restart or regression resumption. Doctests, formatting, all-target Clippy and debug/release builds pass. One final Clippy command failed on a collapsible if; only the standard let-chain form was applied, retaining the 648 regression passes, and the successful retry is recorded separately. Fresh debug/release emissions are byte-identical; the saved Gate68 release rejects the syntax. An untimed single-use callable supplement passes twenty-four exact output lines at both O0/O3 and five consumption/origin/escape negatives; it does not add regression groups.

Source validation passes ten TypeScript workload programs, 1,330 complete-output cases, 45 independent hand checks, 151 mutation/domain negatives, ten eager-both-branches rejection checks, two Node groups and twenty-two auxiliary order lines. Native O0/O3 preflight passes forty-two builds, one hundred complete outputs, twenty disposal checks and two order executions.

The ten comparisons cover int, Number, boolean, ASCII/Unicode String, int/Number/boolean/String arrays and a named record. Each actual conditional evaluates a strict boolean predicate once and calls only the selected branch. Timed work includes setup, predicate/branch call-counter instrumentation, selected computation or fresh owned construction, Boolean projection where required, and complete output/counter-marker retention. Independent observation is outside timing. Owned comparisons measure fresh selected construction and retention, not existing-owner move cost, pure branch latency, JavaScript ownership parity or equal allocation/storage behavior. Existing-owner transfers remain untimed semantic evidence.

The execution-only run contains 240 process cells, 7,440 timed batches and eighty min/average/max rate cells across eight runtimes. Native leads compiled Node on 7/10 rows and the fastest scripting runtime on 7/10. Average-rate ratios span 0.356–3.494× versus Node and 0.330–3.454× versus the fastest scripting runtime. Fresh ASCII/Unicode String construction and String-array construction lag both comparisons; this is not a universal native-speed claim. Full per-row rates remain in [BENCHMARKS.md](../../BENCHMARKS.md), rather than a duplicate table here.

All thirty profiles dispose output and final owners to absolute zero. Profiles retain raw counters and normalized min/average/max allocations, reallocations and requested/retained bytes per complete operation across three 2,048-operation samples. Input adaptation remains separate. Peak is retained-batch peak divided by operations, not isolated-call peak. Variable payloads and counter markers prevent a uniform retained-byte or allocation-per-selection claim.

The report contains 444 separately dated rows. Earlier 434 measured sections, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed. No combined 444-row run is claimed. The closed certificate binds the proof, release, source, manifest and report identities without altering historical acceptance proofs.

## Remaining separate work

Truthiness coercion, heterogeneous union results, optional/nullish types, new callable representations and owned assignment-expression results remain pending. Conditional selection of compatible existing owned values is accepted under the documented ownership rules.
