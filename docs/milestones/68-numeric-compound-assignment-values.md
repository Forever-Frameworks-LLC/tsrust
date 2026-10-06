# Milestone 68: numeric compound assignment values

Status: complete. All twelve int and twelve Number compound assignment values, 629 distinct compiler groups, required checks, twenty-four matched comparisons and seventy-two fully disposed profiles are accepted.

## Scope

Implement value-producing `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `&=`, `|=`, `^=`, `<<=`, `>>=` and `>>>=` for both int and Number. Support every existing expression context, low precedence and right-associated chains, including mixtures with ordinary and logical assignment and updates. Return the exact successfully committed scalar. Infer RHS type from its selected target; preserve strict matching, capture, parameter, iteration, partial-field and ownership rules.

Select targets and indices once, check bounds and snapshot the old scalar before the RHS. Evaluate RHS once. Apply the existing checked i64 arithmetic, shifts and power, or existing Number IEEE/32-bit operator contract, then commit only after successful computation. Reacquire the final destination using saved indices after permitted RHS growth, reversal or removal. Preserve incoming, ancestor and index-evaluation loans; reject transferred or invalidated destinations. RHS effects remain unconditional. Use ordinary Rust scalars and standard checked methods, with no new containers, implicit clones or unsafe code.

Standalone numeric compound statements and for clauses share this value contract and discard the result. Preserve existing standalone string `+=` behavior, allocation and disposal. Owned string compound values, owned ordinary assignment values, coercion, destructuring and nullish operations remain separate gates.

## Acceptance

Four focused O0/O3 groups pass all twenty-four typed forms, expression contexts, right-associated chains with ordinary/logical assignment and updates, exact returned/committed Number bits, full-width i64 shifts and power, once-only selection and old snapshots, indexed-record growth and saved-index destination reacquisition. Strict diagnostics preserve incoming/ancestor/index-evaluation loans, captures, const/parameter/iteration rules and partial fields. Checked failures and RHS panics never commit the outer result; earlier RHS effects remain. Normal-stack depth guards, zero scalar allocations and normal/unwind disposal pass. Standalone String += and its existing ownership/allocation contract remain unchanged.

The first focused command passes three groups. Its fourth group reaches a handwritten Rust wrapper error: the generated mutable `Closure4` uses `.call`, not direct Rust function invocation. Only the two wrapper calls were corrected; compiler code and the three passed groups remained unchanged. The failed source/log/terminal and repaired source are retained, and one exact retry passes the fourth group. The sole full invocation passes 629 distinct compiler groups and 644 workspace groups, with no failed regression command or full restart. Doctests, format, all-target Clippy, debug/release builds and fresh identical CLI emissions pass. The saved Gate67 executable rejects numeric compound values.

Source validation passes twenty-four TypeScript workloads, 1,680 complete cases, 304 mutation/domain negatives, 218 independent hand cells and two focused Node groups. Two auxiliary TypeScript programs verify hand results and source order. Native O0/O3 preflight passes ninety-eight builds, 240 complete outputs, forty-eight untimed disposal checks and two order executions.

Each timed operation retains its actual compound expression result and final destination. Int uses int[] and the existing scalar shape observer; Number uses number[] and the existing IEEE observer, with canonical NaN comparison and no cross-runtime NaN-payload identity claim. Dynamic initialization/operand selection, the actual typed operator, paired output construction and retention are timed; independent complete observation is outside timing. Matched int domains use exact common integers, exact division, nonoverflowing arithmetic, shift counts below 32 with matching signed results and exactly representable powers. Full-width i64 and checked-failure semantics remain separate native evidence. Number comparisons use the existing IEEE/32-bit/power contract and actual JavaScript operators.

The execution-only run contains 576 process cells and 17,856 timed batches, publishing 192 min/average/max rate cells across eight runtimes. Native leads compiled Node on 24/24 rows and the fastest scripting runtime on 24/24. Average-rate ratios span 1.391–3.849× versus Node and 1.019–2.647× versus the fastest scripting runtime. Rates below round to whole operations per second:

| Capability | Native min / avg / max ops/s | Compiled Node min / avg / max ops/s | Native / fastest scripting avg |
| --- | ---: | ---: | ---: |
| `int.addAssignExpression` | 254,663,019 / 407,702,541 / 517,433,047 | 17,535,448 / 212,316,623 / 296,082,117 | 1.905× |
| `int.subtractAssignExpression` | 289,143,018 / 423,338,809 / 446,868,863 | 17,674,218 / 216,152,793 / 289,143,018 | 1.953× |
| `int.multiplyAssignExpression` | 338,960,609 / 432,817,743 / 486,692,015 | 18,846,568 / 199,964,894 / 271,582,018 | 2.012× |
| `int.divideAssignExpression` | 412,986,489 / 515,156,726 / 540,084,388 | 16,094,303 / 184,359,668 / 243,317,096 | 2.645× |
| `int.remainderAssignExpression` | 346,121,345 / 455,780,040 / 491,598,656 | 19,535,647 / 161,431,007 / 217,479,027 | 2.647× |
| `int.exponentAssignExpression` | 254,694,690 / 291,530,179 / 315,076,923 | 26,270,219 / 75,750,842 / 89,694,740 | 2.140× |
| `int.bitAndAssignExpression` | 372,363,636 / 464,814,845 / 481,882,353 | 18,926,512 / 188,781,092 / 262,833,676 | 2.258× |
| `int.bitOrAssignExpression` | 305,307,096 / 460,011,121 / 481,882,353 | 13,888,606 / 200,094,135 / 257,318,759 | 2.204× |
| `int.bitXorAssignExpression` | 296,082,117 / 464,481,746 / 486,692,015 | 15,855,566 / 192,679,120 / 256,000,000 | 2.296× |
| `int.shiftLeftAssignExpression` | 383,952,006 / 474,283,076 / 491,598,656 | 31,030,303 / 215,539,053 / 261,457,934 | 2.200× |
| `int.shiftRightAssignExpression` | 101,135,802 / 471,132,758 / 491,598,656 | 15,870,802 / 179,285,677 / 264,258,065 | 2.378× |
| `int.shiftRightUnsignedAssignExpression` | 406,188,021 / 473,539,796 / 491,480,682 | 17,667,814 / 183,723,239 / 257,318,759 | 2.346× |
| `number.addAssignExpression` | 207,392,405 / 392,008,115 / 592,249,855 | 45,093,247 / 163,999,873 / 195,830,943 | 2.331× |
| `number.subtractAssignExpression` | 224,438,356 / 397,671,060 / 614,461,446 | 43,612,513 / 158,673,879 / 201,436,018 | 2.382× |
| `number.multiplyAssignExpression` | 234,057,143 / 394,286,474 / 614,461,446 | 46,194,794 / 165,565,987 / 198,989,506 | 2.192× |
| `number.divideAssignExpression` | 215,578,947 / 390,161,544 / 606,814,815 | 67,237,926 / 164,668,412 / 201,436,018 | 2.222× |
| `number.remainderAssignExpression` | 136,152,107 / 273,375,469 / 292,571,429 | 21,086,229 / 123,671,745 / 148,491,879 | 1.921× |
| `number.exponentAssignExpression` | 121,962,839 / 153,326,353 / 158,563,023 | 23,871,968 / 84,050,133 / 95,074,509 | 1.019× |
| `number.bitAndAssignExpression` | 210,051,282 / 282,112,623 / 311,104,360 | 34,613,889 / 197,643,421 / 250,765,275 | 1.416× |
| `number.bitOrAssignExpression` | 213,711,781 / 285,815,152 / 311,104,360 | 48,810,715 / 202,042,339 / 270,077,806 | 1.410× |
| `number.bitXorAssignExpression` | 167,745,106 / 280,585,598 / 305,307,096 | 31,917,215 / 201,658,245 / 252,061,538 | 1.369× |
| `number.shiftLeftAssignExpression` | 210,960,033 / 285,613,587 / 307,230,723 | 42,153,795 / 196,803,002 / 253,371,273 | 1.354× |
| `number.shiftRightAssignExpression` | 199,804,878 / 285,833,216 / 307,184,641 | 8,213,882 / 192,590,274 / 260,063,492 | 1.366× |
| `number.shiftRightUnsignedAssignExpression` | 250,795,983 / 287,434,996 / 307,184,641 | 38,370,023 / 149,471,613 / 191,258,872 | 1.743× |

All seventy-two profiles dispose output and final owners to absolute zero. Across the twelve int rows, average retained bytes per operation span 16–16, cumulative requested bytes span 31.875–31.875, and retained-batch peak bytes divided by operations span 16–16. A 2,048-operation batch averages 1–1 output allocations and 7–7 reallocations. Across the twelve number rows, average retained bytes per operation span 16–16, cumulative requested bytes span 31.9844–31.9844, and retained-batch peak bytes divided by operations span 16–16. A 2,048-operation batch averages 1–1 output allocations and 10–10 reallocations. These are retained output-storage costs; focused scalar compound assignment allocates no heap storage. Profiles preserve raw counters and per-operation min/average/max across three samples. Input adaptation remains separate. Peak denotes retained-batch peak divided by operations, not isolated-call peak.

The report contains 434 separately dated rows. Earlier 410 measured sections, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed. No combined 434-row run is claimed. Actual owner commands, source identities, failed wrapper evidence, timing and profile histories are bound by the proofs.

[Compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T21-29-16-000Z-numeric-compound-assignment-values-compiler-proof.json) · [Source proof](../../benchmarks/runtime/diagnostics/2026-10-02T21-15-11-000Z-numeric-compound-assignment-value-source-validation.json) · [Runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T21-37-19-000Z-numeric-compound-assignment-value-runtime-proof.json)

## Remaining separate work

Owned ordinary and compound assignment values, coercion, destructuring and nullish operations remain pending. Existing standalone owned assignments and String += remain supported.
