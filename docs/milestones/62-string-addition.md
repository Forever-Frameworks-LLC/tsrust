# Milestone 62: string addition

Status: complete. Four focused O0/O3 groups, one 605-group regression, required Rust checks and fresh release checks pass. Four matched comparisons across eight runtimes and twelve normalized resource profiles complete acceptance.

## Scope

Support `string + string` using standard Rust UTF-8 string construction and the existing Cow representation. Require matching string operands; implicit JavaScript coercion remains a separate operation. Numeric addition retains its current behavior. Compound assignments, including `+=`, belong to a subsequent gate covering numeric and string assignments together.

Evaluate operands once, left to right. Read both operands without consuming reusable source storage. Produce an independent owned result. Use standard capacity allocation and `push_str`; do not introduce custom encodings or string storage. Preserve existing ownership rules for temporary expressions, parameters, fields, array elements and captures. Reject conflicting mutations while an operand is borrowed.

## Acceptance

Focused checks cover empty strings, ASCII, astral and combining Unicode, embedded NUL, exact unmodified whitespace, contextual string results, chains, numeric addition preservation and strict mixed-type rejection. Operands evaluate once, left to right; parameters, captures, fields, array elements and temporaries preserve existing loans and source reuse. Independent owned results survive source disposal. Nonempty binary results allocate once with exact standard capacity; empty results allocate no storage. Normal and unwind disposal pass.

Three passing groups were retained while two fixture assumptions and a fused trim/lowercase borrow emission were corrected. Only affected groups were rerun. Fusion now honors the ordinary borrowed-expression path, including either addition operand, comparisons and read arguments. Regression acceptance covers 605 distinct groups. One full invocation exposed exponential nested-addition type inference and was interrupted. Each operand hint now evaluates once. The bounded-depth check and only unstarted targets completed after repair; previously completed targets were retained.

Four new rows cover binary and left-associated chained addition over ASCII and Unicode input. TypeScript uses actual string +. Runtime input cloning/repetition, operand selection, addition and retention of every complete String[] result are timed. Full content and shape observation runs outside timing. A native chain constructs and disposes its real intermediate owned string; it is not flattened. Native results copy into independent standard storage. JavaScript may use immutable ropes or shared storage, so allocation equivalence and isolated-call latency are not claimed.

Source validation retains four actual TypeScript programs, 992 complete observations, sixteen mutation/domain negatives, sixteen independent hand cells, two Node harness groups and complete source-order output. Native O0/O3 preflight accepts complete content, order and disposal. Timing records 96 process cells and 2,976 measured batches, publishing thirty-two min/average/max rate cells. Twelve resource samples retain raw counters and report three-sample min/average/max allocations, reallocations and requested, retained and peak bytes per complete operation. Input adaptation stays separate. Peak is the retained-batch peak divided by 2,048 operations, not an isolated-call peak. Output and final disposal remain absolute zero checks. Earlier 358 runtime tables, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed.

Native leads Node after tsc on 0/4 rows and the fastest scripting runtime on 0/4. Ratios below use measured average rates. Resource columns show three-sample averages per complete operation; the runtime proof retains all min/average/max statistics and raw samples.

| Workload | Native / Node after tsc | Native / fastest scripting | Allocations / operation | Requested bytes / operation | Retained-batch peak bytes / operation |
| --- | ---: | ---: | ---: | ---: | ---: |
| `string_add_direct_ascii` | 0.360× | 0.356× | 1.0029 | 65.2769 | 41.3706 |
| `string_add_direct_unicode` | 0.379× | 0.378× | 1.0029 | 68.5317 | 44.6255 |
| `string_add_chained_ascii` | 0.228× | 0.228× | 2.0029 | 91.1519 | 50.0010 |
| `string_add_chained_unicode` | 0.242× | 0.227× | 2.0029 | 99.2817 | 54.8818 |

Compiler evidence: [605-group regression and fresh release checks](../../benchmarks/runtime/diagnostics/2026-10-02T16-04-00-000Z-string-addition-compiler-proof.json). Source evidence: [actual TypeScript and independent complete outputs](../../benchmarks/runtime/diagnostics/2026-10-02T15-52-00-000Z-string-add-source-validation.json). Runtime evidence: [matched rates and normalized resources](../../benchmarks/runtime/diagnostics/2026-10-02T16-10-53-000Z-string-addition-runtime-proof.json).
