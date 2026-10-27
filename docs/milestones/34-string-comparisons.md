# Milestone 34: native string comparisons

[Roadmap](../roadmap.md) · [String semantics](../guides/language/string-semantics.md)

## Outcome and scope

Support `===`, `!==`, `<`, `<=`, `>`, and `>=` between two strings using standard Rust content comparison. Comparison borrows operands, evaluates each exactly once from left to right, and produces a boolean. Comparing strings does not transfer or clone them.

Ordering follows Rust UTF-8 lexical ordering. JavaScript orders UTF-16 code units, so ordering astral scalars against some other Unicode scalars can differ. Equality compares exact contents without Unicode normalization.

Keep existing integer and boolean operator rules. Mixed operand types remain errors. This milestone does not add coercion, string concatenation, locale collation, or comparisons of records, arrays, and callables.

## Acceptance evidence required

- Native execution at both development and release optimization levels covers equal and unequal strings, empty strings, prefixes, NUL, combining sequences, and astral scalars.
- Function calls and temporary operands prove single evaluation and source order.
- Named parameters and closure captures infer read access; source values remain reusable.
- Fields, indexed arrays, and borrowed iteration work without hidden copies.
- A left operand's borrowed source remains protected during right operand evaluation. Mutation and consumption conflicts retain source diagnostics, including enclosing argument loans.
- Existing scalar operators, storage proofs, and compiler regression checks remain valid.
- Equivalent Node/TypeScript workloads validate equality directly and use explicit UTF-8 ordering where JavaScript ordering differs. Preserve true run records and update top-level `BENCHMARKS.md` with measured min/avg/max throughput.

## Current status

The reviewed implementation is integrated after milestone 33's measurements completed. All 481 compiler integration groups pass, including six string comparison groups with native execution at optimization levels 0 and 3. Formatting, Clippy with warnings denied, release build, and documentation checks pass. A NUL fixture uses the supported `\u0000` escape rather than unsupported `\0` shorthand.

The [compiler acceptance proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-35-58-133Z-native-string-operators-compiler-acceptance.json) preserves all commands, complete logs, source hashes, and the release binary identity.

This gate is complete. Benchmark source preparation passes 41 lightweight groups, including 168 independent Node comparison cases and invalid-Unicode guards. Final source setup explicitly clones the borrowed input into owned arrays. All 80 cross-runtime correctness checks pass, followed by 1,488 timed batches across eight runtimes and six native allocation samples. Every native sample releases owned storage. The original 50 workload pairs and their measurements remain unchanged.

The [timing record](../../benchmarks/runtime/results/2026-10-01T23-37-48-752Z.json) measures equality at 348,378,317 workload iterations/s versus Node after tsc at 215,967,899. ASCII ordering measures 71,475,068 versus 64,726,587. Each equality iteration contains two comparisons; each ordering iteration contains four. The [allocation record](../../benchmarks/runtime/diagnostics/2026-10-01T23-38-12-064Z-native-allocations.json) and [final runtime proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-38-46-000Z-native-string-operators-runtime-proof.json) preserve source identities, setup costs, disposal checks, and unchanged historical records. Top-level `BENCHMARKS.md` layers the two new rows alongside the separately dated prior 50; it does not invent a combined run.

The [Node draft validation record](../../benchmarks/runtime/diagnostics/2026-10-01T23-15-44-gate34-node-draft-validation.json) preserves the command, observed result, and exact source snapshots. It contains no timing results.
