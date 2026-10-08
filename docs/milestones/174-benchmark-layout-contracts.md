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

Whole compiler acceptance and project formatting are recorded at closure. The two
existing callable replacement witnesses remain outside this infrastructure gate.
No compiler or editor source is changed. Installed consumers therefore retain the
accepted gate 173 compiler source.
