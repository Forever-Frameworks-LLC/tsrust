# Milestone 65: native integer operators

Status: complete. All sixteen forms, 617 distinct regression groups with retained resumption, required checks, sixteen matched rows and 48 fully disposed profiles are accepted.

## Scope

Add sixteen int forms: unary `+` and `~`; binary `&`, `|`, `^`, `<<`, `>>`,
`>>>` and `**`; and the seven matching compound assignments. Use standard Rust
i64 operations without JavaScript's implicit 32-bit conversion. Unary plus is
identity. Bitwise operations preserve the signed 64-bit representation.

Shift counts must be in 0..63; invalid counts fail with the operator's source
range. Standard checked shifts validate the count while allowing discarded bits.
Arithmetic right shift preserves the sign. Logical right shift uses u64 bits and
casts the result back to i64, so `-1 >>> 0` remains -1 and `-1 >>> 1` is i64::MAX.
Exponentiation requires an exponent representable as u32 and uses checked i64
power. Negative/out-of-range exponents and result overflow fail; `0 ** 0` is 1.

An explicit int context or a concrete typed int operand selects integer behavior.
Existing literal-only Number defaults for bitwise/shift/power expressions remain
unchanged. Mixed typed int/Number operands require explicit conversion. Keep
Number's 32-bit conversions, masked shift counts and IEEE power behavior.

Evaluate operands once, left to right. Compound targets select storage and read
the old value before the RHS; compute successfully before committing. Existing
ownership, capture, parameter, partial-transfer, bounds and loan rules apply.
Preserve precedence, right-associative power, unary-base grammar and depth limits.

## Acceptance

Four focused O0/O3 groups cover all sixteen forms, typed and inferred contexts, strict mixed-type rejection, precedence, source order, wide bits, checked failures, zero scalar allocations and owned disposal on success/unwind. One full regression invocation stopped at an obsolete unsupported `+1` fixture cell. Only that cell was removed; compiler code was unchanged. The 439 passing groups were retained, the failed group alone passed, and the 23 unstarted targets completed in one continuation followed by doctests. All 617 distinct groups, format, all-target Clippy, debug/release builds and fresh identical CLI emissions pass. The saved Gate64 executable rejects the new typed int operations. No full restart or parser repair was needed in this gate.

Sixteen TypeScript workload programs pass 1,120 complete cases, 40 negatives, 128 independent hand cells and two focused Node groups; two auxiliary TypeScript programs cover signed scalar results and source order. The existing int[] protocol observes every returned scalar without offsets or a new encoding. Complement timing selects small negative operands whose actual results are nonnegative. JavaScript uses its real operators on the exact common domain: bounded integer operands, shifts below 32 and exactly representable integer powers. Dynamic setup, selection, the actual operation and complete output construction/retention are timed; independent observation is outside timing. This establishes matched common-domain rates, not general equivalence between JavaScript's 32-bit conversions and native i64 behavior.

Native O0/O3 preflight passes 66 builds, 160 complete outputs, 32 untimed disposal cases and two order executions. A separate native-only supplement has 22 builds, eighteen exact wide scalar lines at each optimization level, eighteen checked traps, eight no-commit cases and eight additional inline-to-heap no-commit cases. Implicit Rust overflow checks are disabled. These witnesses cover the full i64 width, invalid counts/exponents, power overflow, RHS effects and successful computation before commit; they produce no throughput record.

The accepted execution-only run contains 384 process cells and 11,904 timed batches, publishing 128 min/average/max rate cells. Native leads Node after tsc and the fastest scripting runtime on all sixteen rows. Average-rate ratios span 2.176–3.911× versus Node and 1.874–2.990× versus the fastest scripting runtime. All forty-eight native profiles dispose output and final owners to zero. Each complete operation retains one i64 result: 8 retained bytes per operation. A 2,048-operation batch uses one output allocation and six reallocations; cumulative requested bytes are 15.875 per operation. These are output-storage costs; focused scalar operations allocate no heap storage. The proof retains raw counters and per-operation min/average/max summaries across three samples. Input adaptation is separate; peak is the retained-batch peak divided by 2,048 operations, not an isolated-call peak.

The grouped report now contains 405 individually dated rows. Earlier 389 measurement sections, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed. Source, native, timing/profile and runtime sealing completed without failed measurement attempts. No combined 405-row run is claimed.

[Compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T18-48-35-000Z-native-int-compiler-proof.json) · [Source proof](../../benchmarks/runtime/diagnostics/2026-10-02T18-47-23-000Z-native-int-source-validation.json) · [Runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T19-00-25-000Z-native-int-runtime-proof.json)

## Remaining separate work

Logical assignment, assignment-valued expressions, coercions and other pending
inventory operations remain separate gates.
