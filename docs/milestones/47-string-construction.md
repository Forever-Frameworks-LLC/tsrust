# Milestone 47: string construction and splitting

[Roadmap](../roadmap.md) · [String semantics](../guides/language/string-semantics.md)

## Scope

Add `repeat(int)`, `replace(string, string)`, `replaceAll(string, string)`, and zero/one/two-argument `split` with a string separator and optional int limit. Borrow operands once in source order and return independent owned output through existing Rust string and string-array storage.

Use standard Rust repetition, matching, splitting, and string construction. Negative repetition counts fail explicitly; zero returns empty. Check capacity overflow before construction; allocation failure follows the existing Rust allocator contract.

Replacement supports first or all nonoverlapping string matches. Preserve replacement-template substitutions for a literal dollar, matched text, original prefix, and original suffix. Capture references remain literal when the search pattern is a string. Use a small safe helper only where standard Rust replacement lacks these substitutions. Regex patterns and callback replacements require separate capabilities.

Split returns one independent string when the separator is omitted. Nonempty separators preserve empty pieces; a limit stops output without adding a remainder element. Convert an explicit int limit modulo 2^32, with zero producing no elements and the omitted limit using 2^32 minus one. This follows the integer domain of [ECMAScript split](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.split); it does not add implicit coercion or undefined values.

Keep the native UTF-8 contract: empty-separator split yields Unicode scalars without edge-empty pieces, and empty-pattern replacement uses scalar boundaries including both ends. An empty receiver split by an empty separator yields no elements. Astral text therefore differs from JavaScript's UTF-16 code-unit behavior. Do not introduce surrogate storage or normalization.

## Acceptance

Focused native programs run at optimization levels 0 and 3. Cover empty and Unicode inputs, NUL, matches and misses, repeated and overlapping patterns, replacement tokens and their original-source context, repetition limits, split trailing empties and integer limits, operand effects and source loans, result independence, and disposal. Run one settled full regression after integration.

Matched benchmarks use actual TypeScript methods and independently checked complete output. Include construction and output retention costs. Keep shared-domain timing separate from native Unicode semantic checks. Preserve historical measurements and record allocation/disposal evidence.

## Dependencies and status

The exact reviewed compiler candidate is integrated after milestone 46 acceptance. Changed compiler timestamps are refreshed to prevent stale Cargo artifacts. All four focused groups pass at optimization levels 0 and 3 without repairs. Affected depth, generic callback, and existing string operand-order and diagnostic checks pass. The single settled full regression passes all 539 groups across 57 integration binaries and the doctest stage, without failures or restarts. Formatting, Clippy with warnings denied, and the fresh release build pass. Actual debug/release emissions agree, the preceding compiler rejects the new syntax, and the new release executes all four methods correctly. The [compiler acceptance proof](../../benchmarks/runtime/diagnostics/2026-10-02T05-00-07-000Z-string-construction-compiler-acceptance.json) preserves source identities, logs and release evidence.

Source review covers standard Rust fast paths, original-source replacement substitutions, wrapped split limits, scalar boundaries, operand loans, and owning outputs. Split support is gated separately so unrelated scalar string programs do not acquire array runtime support. No new result type or storage representation is required; regex, replacer callbacks, general coercion, and undefined remain separate work.

Twelve matched comparison pairs complete runtime acceptance. The original 109 light Node groups cover 1,644 complete content/shape/effect cases, 124 domain guards, 21 edge/transport checks, and 24 prepared TypeScript driver cases. Native preflight validates twelve actual wrappers and 48 cases. The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T05-09-41-000Z-native-string-construction-runtime-proof.json) seals 480 engine correctness checks, 8,928 measured batches, and 36 profiles with zero live bytes after output and input disposal. BENCHMARKS.md now contains 156 separately dated measured rows and nine retained memory probes; prior rows and auxiliary baselines were not retimed.

Native throughput leads eight of twelve comparisons against both Node after tsc and the fastest scripting runtime. Repeat and normal split trail; results describe bounded whole iterations including construction, retention and effect markers, not isolated method latency. Empty-pattern and empty-separator comparisons use the shared BMP domain; native Unicode scalar behavior remains separate semantic coverage. Native owned cloning versus JavaScript immutable sharing costs are disclosed.

Two preparation failures produced no timing records: new Probe type fields needed existing semicolon syntax and canonical workload declarations, and TypeScript's declarations needed an explicit zero-argument split overload. Both corrections alter declarations only and preserve authored/prepared JavaScript and executable bodies. Passed native preflight and compiler regression were retained without repeats. Original sources, failures and repairs remain in the immutable proof.
