# Runtime comparisons

[Benchmarks](../BENCHMARKS.md) · [Runner sources](../benchmarks/runtime/README.md) · [Native string contract](./string-semantics.md)

Compare equivalent observable operations using the pinned Rust, Node, TypeScript, tsx, ts-node, Bun, and Deno versions recorded in each raw run. Native generated programs use optimization level 3 with overflow checks; startup and compilation are reported separately. Execution tables show min/average/max operations per second, where average is the arithmetic mean of sample rates.

Each workload defines its operation, input/count/seed, result validation, and timed boundary. Independent oracles check every correctness case. Returned-result hashing and native disposal occur outside warmed execution; cleanup of function-local values and natural engine GC remain inside. There is no forced GC in normal execution timing. Memory probes report normalized bytes separately from allocation counts.

## Native UTF-8 epoch

The user selected native Rust semantics rather than general JavaScript string parity. Length counts UTF-8 bytes, slices and substring searches use checked byte coordinates, charAt iterates Unicode scalars, and Rust trim/case conversion apply. JavaScript counterparts must explicitly perform the same operation, with their UTF-16/UTF-8 conversion costs disclosed. Invalid byte boundaries belong in correctness tests; valid slice and search benchmarks must use independently checked boundaries. Exact result content, order, and scalar validity are checked outside timing.

ASCII comparisons continue to use equivalent content. Unicode comparisons are refreshed under a new semantic epoch. Old UTF-16 timings cannot be called measurements of the new operations, mixed into a fabricated combined run, or used to claim a speedup caused by doing less work. Raw histories remain immutable and retain their original compiler, source, driver, oracle, and runtime identities.

Native standard data structures have different allocation and ownership behavior from JavaScript engines. This is a representation difference, not an excuse to skip a matched operation. Record cloned mutable state, input conversion, returned data, retained memory, and full disposal. Different Unicode table versions and whitespace rules remain explicit limitations.

## Limits

The implemented subset uses bounded integers and a scoped ownership model. Results do not establish general language performance, coercion parity, or equal allocation strategies. Short batches and ordinary machine variation limit interpretation of small differences. Only changed native instructions support attributing a timing change to an implementation change; identical-code variation remains a control.

Earlier methodology and experiment descriptions are retained in the [historical archive](../archive/utf16-strings-2026-10-01/README.md). Current run details and reproducing commands live alongside the benchmark runner.
