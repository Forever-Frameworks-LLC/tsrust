# Milestone 73: integer formatting

Status: complete. 649 compiler groups, 665 workspace groups, four matched runtime comparisons and twelve fully disposed native profiles pass.

## Scope

Implement `int.toString(radix?: int): string` over the entire native signed-i64 domain. Omitted radix means decimal; accepted explicit radices are 2 through 36. Digits are lowercase and negatives carry a leading minus, including binary and hexadecimal. Zero is `0`. Invalid radices fail before producing an output; arguments have strict integer types and at most one radix is permitted.

Use Rust's standard decimal formatter and ordinary owned UTF-8 storage. Other radices require a small bounded digit conversion because Rust has no standard arbitrary-radix integer formatter. Convert the unsigned magnitude rather than negating a signed minimum. Preserve once-only, left-to-right evaluation of the receiver and radix, reusable primitive receivers, existing inferred contracts, captures, expression contexts and temporary lifetimes.

The syntax follows [Number.prototype.toString](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.prototype.tostring), with native signed-i64 values and strict integer arguments. This does not implement floating Number formatting, boxed objects or JavaScript radix coercion. JavaScript Number radix formatting may be implementation-approximated; exact-output comparisons must establish a shared domain separately from the full native implementation.

## Acceptance

Focused semantic checks cover all 35 radices, both i64 extremes, zero, signs, lowercase digits, decimal omission, rejection, receiver/radix order, result storage and affected expression contexts. Editor completion, hover, optional signature help and inferred result chains are included before the sole full regression.

Four matched workloads cover omitted decimal, binary, hexadecimal and base36 formatting with dynamic signed safe integers. Each operation retains a complete string result. Input setup, selection, formatting and output construction are timed; observation stays outside the timer. Original run records and three native allocation profiles per row supply min/average/max rates and normalized resource counters. Prior 468 workload results and nine memory tables remain unchanged. Completed evidence is compacted into the existing SQLite archive after its consumers finish.

## Executed acceptance

Four focused groups pass at O0/O3, including all 35 radices, signed-i64 extremes, strict types/arity/range failures, once-only receiver/radix evaluation, inferred contracts, captures, fixed storage, temporary lifetimes and normal/unwind disposal. Three initial groups remain accepted after renaming a duplicate loop binding in the remaining fixture; the exact fourth group then passes. The sole full regression, doctests, formatting, all-target Clippy and debug/release builds pass. Fresh debug/release emitters match the accepted candidate byte-for-byte. Editor discovery and optional radix signature help pass.

Source acceptance contains four TypeScript programs, 280 complete cases (100 matrix, 176 deterministic random and four complete timed domains), 245 independent literal hands across all radices, 76 negatives including four executed wrong-radix mutants, and six receiver/radix-order outputs. Native O0/O3 matches all 245 hands and six order outputs at both levels. Comparisons use signed safe-integer values through both JavaScript safe limits; full i64 extremes have separate native compiler evidence.

The execution-only record contains 96 process cells and 2,976 timed batches across eight runtimes. Native leads compiled Node on 4/4 rows and the fastest scripting runtime on 3/4. Rates describe the complete workload, including pool setup, selection, formatting and result retention. Twelve profiles retain raw counters and normalized min/average/max allocations, reallocations and requested/retained bytes per complete formatting operation over three 2,048-operation samples. Input adaptation is separate; peak means retained-batch peak divided by operations. Every output and final disposal returns live bytes to zero. JavaScript allocation parity is not claimed. Earlier 468 runtime sections, nine memory tables and auxiliary tables remain exact and were not retimed. The 472 separately dated report rows do not claim a combined run.

The [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-03T02-24-30-000Z-int-to-string-compiler-proof.json), [source proof](../../benchmarks/runtime/diagnostics/2026-10-03T02-09-32-000Z-int-to-string-source-validation.json) and [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-03T02-29-01-000Z-int-to-string-runtime-proof.json) retain actual commands, inputs and outputs.
