# Gate 109: Math.random

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the math-functions feature-family milestone.

## Behavior

`Math.random()` returns a pseudo-random `number` that is at least 0 and less than 1.

- No arguments are accepted (`arity`). The result is a `number`, never an `int`.
- Every multiple of `2^-53` below one can occur, and values are about equally likely.
- Each run of a program produces a different sequence. The sequence cannot be seeded, and each thread has its own.
- The result is not suitable for passwords, tokens or other values that must be unpredictable.

The generator is not the one Node uses, so individual values differ from Node by design. Programs can rely only on the range and on uniformity.

## Implementation

`CallTarget::MathRandom` joins the exact `Math` group and lowers to the runtime function `tsr_math_random` in `runtime_math.rs`. The function keeps one xorshift128+ state per thread, seeded on first use from the standard library's per-process hash randomization, and returns the top 53 bits scaled by `2^-53`. Nothing is allocated and no external crate is needed.

## Evidence

- `crates/tsrust-compiler/tests/math_random.rs`: two native tests written before the implementation. The first draws 20,000 values and checks the range, the mean, the half split, ten equal buckets, absence of consecutive repeats, and that two calls in one expression are two draws. It prints only properties that hold for every valid sequence, and passed on three consecutive runs. Two runs of a probe program printed different values.
- `crates/tsrust-lsp/tests/intelligence.rs`: `math_random_has_hover_help_completion_and_number_result`; the catalog example executes natively. All 159 editor tests passed.
- Stale expectations updated: `math_approx.rs` and `math_unary.rs` asserted that `Math.random()` is unsupported; both now use a member that is still unsupported.
- Focused compiler suites passed: `math_random`, `math_exact`, `math_approx`, `math_unary`, `math_f16round`, `fixed_storage`, `generic_default_builtins`, `templates`, `call_spread`.
- Benchmark support: `math_random_range` with a source contract and registration test. Because draws differ per run and per engine, the retained result holds range and difference properties only, and the oracle states those properties; the two generators are different algorithms and are not compared value for value. Untimed native and compiled-Node verification passed; all 300 harness tests passed. No timing was run.
