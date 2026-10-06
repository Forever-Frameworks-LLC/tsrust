# Milestone 79: split storage and documentation

Status: implementation and checkpoint 81 accepted. Focused behavior and complete IntelliSense examples pass; combined regression and selected comparisons are sealed.

## Existing behavior preserved

`split(separator?: string, limit?: int): string[]` borrows its receiver and arguments once in source order and returns an independent array. A missing separator produces one whole-text element. A nonempty separator removes nonoverlapping exact matches and preserves empty leading, interior and trailing pieces. A positive effective limit keeps the first pieces and discards the remainder instead of joining it into the last element.

Explicit limits accept signed native int values and convert modulo 2^32. The default is 4294967295. Thus -1 permits up to 4294967295 elements, 4294967296 yields none, and 4294967297 permits one. A zero effective limit returns an empty array after argument evaluation. There is no implicit numeric coercion, undefined argument or regex separator.

An empty separator returns Unicode scalars without artificial edge-empty pieces. An empty receiver and empty separator return an empty array. Astral scalars remain whole; combining sequences can contain multiple elements. This differs from JavaScript UTF-16 code-unit splitting. Exact matching performs no Unicode normalization.

## Storage improvement

Use standard Rust splitting and scalar boundaries. Static borrowed receivers can produce borrowed static text pieces. Owned receivers produce independently owned pieces, so the source can be dropped or mutated after the operation. The returned array retains its existing ordinary growable storage and methods. No shared owner graph, custom encoding or lifetime escape is introduced.

## Documentation and acceptance

Correct split's hover description and parameter help to state the accepted signed limit domain, wrapping, defaults, result and receiver behavior, examples and native Unicode differences. Put ownership after behavior. Verify executable examples with the current compiler and O0/O3 native output. Preserve signature, completion and existing editor fixes.

Focused checks cover empty/missing/overlapping separators, trailing pieces, wrapped limits, UTF-8 widths, evaluation order and loans, captures and result independence, and normal/unwind disposal. Reuse the original construction fixtures where practical. No new built-in method or property is added, but the changed entry must meet the complete documentation requirement before acceptance.

Existing split workloads retain their exact sources and records. Their affected performance rows, plus any new independently matched storage/scalar comparisons, are queued for checkpoint 81; historical numbers remain dated evidence.

## Focused implementation evidence

Eight compiler groups pass across the original and affected repair runs, including all four original construction groups. Seven editor groups pass, including split hover, completion, optional parameter help, inferred String[] initializers and five executable examples (18 exact output lines at O0/O3). Static three-piece splitting uses one array allocation without allocating piece text; owned pieces remain independent and all observed normal/unwind storage is disposed.

Two comparison sources pass 274 cases, 27 independent hand cells, 57 negative source guards and five executed semantic mutants. Native source-order witnesses match 24 output lines across O0/O3. Original fixture, editor-inference and TypeScript declaration failures remain preserved with their focused repairs in the [implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T06-37-34-000Z-split-storage-implementation.json). No full regression or timing run starts here. New measurements and the three affected older split rows are queued for checkpoint 81.

## Checkpoint 81 acceptance

The combined checkpoint accepts 700 compiler integration groups and 735 workspace groups with retained passes and no full restart. [Runtime and resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T08-44-00-000Z-checkpoint81-runtime-proof.json) binds 29 selected workloads across eight runtimes and 87 fully disposed allocation samples. [BENCHMARKS.md](../../BENCHMARKS.md) contains actual min/avg/max ops/s. Earlier pending statements describe the implementation-stage queue; this checkpoint closes that queue.
