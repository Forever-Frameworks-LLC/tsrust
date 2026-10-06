# Milestone 63: primitive compound assignment

Status: complete. Four focused groups, 609 distinct regression groups with retained passes and targeted continuations, required Rust checks and fresh O0/O3 release checks pass. Nineteen matched comparisons across eight runtimes and fifty-seven fully disposed resource samples close acceptance.

## Scope

Recognize all twelve arithmetic, exponent, bitwise and shift compound tokens. Apply the existing primitive operator contracts: checked int `+= -= *= /= %=`, all twelve Number compounds, and String/String `+=`. Int exponentiation, bitwise and shift operators remain pending underlying capabilities; compound syntax does not invent their semantics. Logical assignments, increment/decrement and assignment-valued expressions remain separate tracked operations.

Select a supported named target once, read its old value before RHS evaluation, compute the result, then commit. Every selected index must evaluate once. Compound reads perform bounds checks before RHS effects. Numeric targets use copied old values; strings borrow the old value and RHS and construct an independent owned result with the existing standard Rust addition helper. Self-append must work. Preserve let rebinding, const/parameter restrictions, capture loans, nested parent loans, partial transfers and fixed-array eligibility. A compound cannot restore an unavailable old value.

This ordering follows [ECMAScript assignment evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-assignment-operators-runtime-semantics-evaluation). Native ownership, dense bounds and strict primitive type rules remain explicit differences.

## Acceptance

All twelve compound tokens are recognized under eighteen supported typed forms: five checked int arithmetic operations, twelve Number operations, and String/String +=. Target selection happens once; the old value is read before RHS evaluation, then the result is committed. Numeric old values are copied. Strings borrow the old value and RHS and construct a fresh independent owned result, including self-append. Initial bounds checks precede RHS effects. Existing const/parameter restrictions, capture and parent loans, partial transfers, restoration rules and fixed-array eligibility remain enforced. Int exponentiation, bitwise and shift compounds are pending underlying capabilities; logical assignment, increment/decrement and assignment-valued expressions remain separately pending.

Four focused groups cover named, field and indexed targets, nested storage, source order, checked traps, exact Number values, ownership diagnostics and normal/unwind String disposal. One full regression invocation encountered parser stack failures. Two narrow parser repairs unify assignment parsing and extract block dispatch, preserving grammar and primitive typing. Previously passing groups were retained. Five accepted targeted continuation commands finish all 609 unique groups; two failed diagnostic commands and the original failure remain in the proof. Required format, Clippy, debug/release builds and fresh O0/O3 checks pass. No full regression restart was performed.

Nineteen rows directly compare actual compound operations: five int, twelve Number, and separate ASCII/Unicode String +=. Mutable targets reset from runtime-selected operands each iteration, and every complete int[], number[] or string[] result is retained. Operand setup, dynamic selection, computation and retention are timed; complete observation runs outside timing. Int timing values are bounded and nonnegative. Division starts from exact multiples of nonzero divisors, so natural JavaScript /= agrees with native checked truncation. Separate signed, non-divisible correctness cells explicitly apply Math.trunc after JavaScript /= outside timing. Number arithmetic uses exactly represented quarters; power uses exactly represented positive integer bases and exponents from 1 through 8 and makes no approximate-domain claim. Bitwise and shift conversions use the existing primitive contracts. Number arrays keep the existing ieee-f64-bits-v1 protocol; int/string arrays keep their existing shape protocol.

String pool cloning/repetition, mutable target initialization and whole-result retention remain timed. Native += creates an independent owned string; JavaScript may use immutable ropes or shared storage. The existing TypeScript clone identity and native clone costs remain explicit. No custom storage, coordinate bridge, allocation equivalence or isolated-call latency is claimed.

Source acceptance records nineteen actual TypeScript programs, 1,654 complete cases, fifty-eight mutation/domain/typing negatives, 134 independent hand cells and two Node groups. Numeric edges include NaN, infinities, signed zero, and shift counts 0, 31, 32 and above 32. The complete order fixture checks one selected index, the old-value snapshot before RHS mutation, String self-append and signed int division/remainder. O0/O3 native preflight completes 78 builds, 190 complete output cases, thirty-eight disposal checks and two order executions.

An initial measurement invocation omitted the required execution-only mode. It was stopped after the first logged workload/process; additional partial measurements are unknown. It published no durable timing record or report. Its actual log, terminal and explanation remain preserved, and partial timings are excluded. The corrected wrapper requires execution-only mode. Only the final accepted record supplies rates and resource summaries.

The accepted timing record contains 456 process cells and 14,136 measured batches, publishing 152 min/average/max rate cells. Fifty-seven resource samples retain raw counters and three-sample min/average/max allocations, reallocations and requested, retained and peak bytes per complete operation. Input adaptation remains separate. Peak is the retained-batch peak divided by 2,048 operations, not an isolated-call peak. Output and final disposal stay absolute zero checks. Earlier 362 measurement tables, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed. The grouped report now contains 381 separately dated rows.

Native leads Node after tsc on 17/19 rows and the fastest scripting runtime on 16/19. Ratios below use measured average rates. Resource columns show three-sample averages per complete operation; the runtime proof retains all min/average/max statistics and raw samples.

| Workload | Native / Node after tsc | Native / fastest scripting | Allocations / operation | Requested bytes / operation | Retained-batch peak bytes / operation |
| --- | ---: | ---: | ---: | ---: | ---: |
| `compound_int_add` | 2.536× | 2.517× | 0.0005 | 15.8750 | 8.0000 |
| `compound_int_subtract` | 3.370× | 3.269× | 0.0005 | 15.8750 | 8.0000 |
| `compound_int_multiply` | 2.870× | 2.870× | 0.0005 | 15.8750 | 8.0000 |
| `compound_int_divide` | 3.451× | 3.314× | 0.0005 | 15.8750 | 8.0000 |
| `compound_int_remainder` | 2.959× | 2.922× | 0.0005 | 15.8750 | 8.0000 |
| `compound_number_add` | 2.981× | 2.917× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_subtract` | 2.941× | 2.941× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_multiply` | 2.873× | 2.873× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_divide` | 2.946× | 2.946× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_remainder` | 2.075× | 2.025× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_power` | 1.656× | 0.904× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_and` | 1.267× | 1.202× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_or` | 1.212× | 1.160× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_xor` | 1.166× | 1.166× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_shift_left` | 1.210× | 1.210× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_shift_right` | 1.225× | 1.142× | 0.0005 | 15.9844 | 8.0000 |
| `compound_number_shift_right_unsigned` | 1.520× | 1.497× | 0.0005 | 15.9844 | 8.0000 |
| `compound_string_add_ascii` | 0.232× | 0.221× | 1.5029 | 72.1519 | 41.3760 |
| `compound_string_add_unicode` | 0.248× | 0.240× | 1.5029 | 76.6567 | 44.6318 |

Compiler evidence: [609 distinct regression groups and fresh release checks](../../benchmarks/runtime/diagnostics/2026-10-02T16-53-53-000Z-compound-assignment-compiler-proof.json). Source evidence: [nineteen TypeScript programs and independent complete outputs](../../benchmarks/runtime/diagnostics/2026-10-02T16-38-33-000Z-compound-source-validation.json). Runtime evidence: [actual matched rates and normalized resources](../../benchmarks/runtime/diagnostics/2026-10-02T17-06-35-000Z-compound-assignment-runtime-proof.json).
