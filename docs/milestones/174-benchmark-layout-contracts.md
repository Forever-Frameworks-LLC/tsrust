# Gate 174: benchmark layout contracts

Date: 2026-10-07. This gate repairs the benchmark harness after automatic project
formatting. It adds no language feature and performs no timing or allocation run.

## Behavior

Source guards compare code tokens instead of exact line layouts. Literal contents,
operators, argument order, required observations and occurrence counts remain
checked. Mutation tests locate the same token sequences, so a missing mutation
fails explicitly instead of executing unchanged source. Native fixture drivers now
replace the intended `main` body even after formatting.

Manifest prefix contracts pin all workload metadata and row order while excluding
only `sourceSha256` and `counterpartSha256`. Registration tests check those exact
source fingerprints separately. Every migrated prefix was compared with commit
0110953: all 23 prefixes retain identical non-fingerprint metadata. New controls
reject changes to iterations, samples, source paths, result types, denominators and
row order.

Historical numerical policies and MPFR references keep their original bytes and
identities. The formatting identity registry lists 60 specific transitions proven
against source bytes from commit 0110953. Both original and formatted source digests
and the common token digest are recorded. Matching accepts only the exact registered
pair or identical hashes; adding whitespace or changing code does not create an
unregistered identity. Concat and declared-function ABI guards use the same proof.
The runtime and allocation runners include this registry and its helper in their
recorded oracle inputs.

## Evidence

The original harness run reproduced 368 passes and 51 failures. The completed
whole harness pass retained 420 passing tests; one remaining stale mutation was
repaired and its two-test suite passed. Together these checks cover all 421 tests,
including two new metadata and source-identity contracts. Completed checks are
retained rather than rerun solely for presentation.

Five representative workloads pass five untimed cases each on native and
Node-after-tsc: `math_sin`, `array_concat_int_zero`, `int_assignment_value`,
`compound_value_int_add` and `optional_chain_int_field`. Genuine numerical reference
tests also verify 880 actual Node point calls and 110 scalar cases. These checks are
correctness evidence, not performance measurements.

The whole compiler run exposed two additional aborted suites: deep branch parsing
and nullable nested calls overflowed normal test stacks. Both reproduced alone.
LLDB identified parser recursion, and disassembly measured an 8,752-byte conditional
frame and a 2,272-byte call-step frame. Conditional construction and else-chain
handling now own separate temporaries. Ordinary calls prepare the existing boxed
operand before argument recursion and construct the result afterward. No extra
operand clone, larger worker stack or reduced source-depth limit is introduced.
The original depth witnesses now pass; affected suites, depth canaries and the whole
editor crate are checked before closure.

The two existing callable replacement witnesses remain outside this infrastructure
gate. Completed whole-crate checks are preserved; only affected groups are rerun.
Project formatting changes no source file or fingerprint. Consumer refreshes use
atomic replacement after the compiler repair is verified.

## Closure checks

The whole compiler command completed with 2,178 passes and the two known callable
replacement failures, plus aborted `control_flow` and `nullable_callables` targets.
The aborted suites were not included in that numerical pass count. Their resumed
suites now pass 11 and 24 tests. The new normal-stack parser unit also passes,
including formatter idempotence and analysis of formatted nested calls. This gives
2,214 distinct passing compiler tests and only the two retained replacement
witnesses. All eight depth canaries pass in the same affected-suite continuation.
The whole editor crate passes 339 tests.

One previously hidden expectation rejected ordinary lexical shadowing. Gate 170
already accepts it. The replacement control checks inner `2` and outer `1` through
native execution after formatting; Node produces the same output.

Disassembly after the repair measures 5,120 bytes for `if_statement` and 832 bytes
for `call_step`, compared with 8,752 and 2,272 bytes before. The normal worker stack
and 128 source-depth diagnostics remain unchanged. These are frame measurements,
not runtime performance measurements.

The final complete harness command passes 421 of 421. Nine affected native fixture
tests pass after the corrected five-consumer refresh. The five workload pairs are
reverified on those consumers. Exact logs and proof are stored under
`runtime/diagnostics/benchmark-layout-contracts/` in SQLite. The current-consumer raw
record is `runtime/results/2026-10-08T00-28-38-163Z-verify.json`. Each closed record
was read back byte for byte before removing its loose copy. No timing was run.
