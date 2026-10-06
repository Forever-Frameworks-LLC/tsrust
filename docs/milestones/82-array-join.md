# Milestone 82: Array.join

Status: complete. Focused implementation and documentation, the retained checkpoint 86 regression, and matched runtime/resource comparisons are accepted.

## Contract

`join(separator?: string): string` returns one string without changing the receiver. The default separator is a comma; an empty separator inserts no text. Empty arrays return empty text. The receiver evaluates once before the separator, and the separator still evaluates for an empty receiver. More than one argument or a non-string separator, including explicit undefined, is rejected.

Dense homogeneous arrays cover supported integer, Number, boolean, String, null, undefined, record and nested-array elements. Nested arrays use commas internally, independently of the outer separator. Null and undefined positions contribute empty text; records contribute `[object Object]`. Number negative zero becomes `0`, and NaN/infinities retain their names. Native integer values retain signed 64-bit decimal text. Finite Number formatting follows the compiler's documented native conversion contract; exact JavaScript formatting parity outside the matched comparison domain is not assumed.

Ownership: borrow the receiver and separator while constructing an independent standard Rust String. Traverse existing borrowed iterators, preserving fixed arrays, Vec storage and inline array storage. Do not construct a diagnostic value tree, clone receiver elements or materialize a temporary string array.

## Acceptance

Focused compiler fixtures cover content, nested and empty arrays, Unicode/NUL separators, numeric boundaries, source order, loans, temporary lifetimes, captures, storage and disposal. Strict diagnostic fixtures cover argument counts, argument types and unsupported receiver types. Editor fixtures must keep hover, completion, return inference, signature and parameter help consistent and execute all new catalog examples with the current compiler and native execution.

Register matched complete-result JavaScript `.join` workloads with independent expected results and source guards. Timing includes receiver setup, selection, join and retained output, rather than claiming isolated method latency. Preserve previous source entries and measured rows. Record actual command and example evidence before marking this gate complete.

## Focused evidence

[Implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T09-12-23-000Z-array-join-implementation.json) accepts four compiler groups and one affected editor group. Eleven catalog examples compile and match thirteen output lines at native O0/O3. Indexed-result inference and shared catalog type declarations are covered by the same focused fixture. Six comparison sources pass 111 independent cases, twelve literal hands, 101 negative guards and six executed semantic mutants. Current compiler checks accept all six pairs; two source-order/content witnesses match complete native output at O0/O3.

Allocator fixtures verify bounded output-buffer growth rather than per-element diagnostic allocations, including 256-element receivers, normal disposal and unwind disposal. The initial compiler fixture assertion and editor inference failure remain preserved; only their affected groups rerun. Earlier 514 report rows remain unchanged. No full regression or performance run occurs in gate 82.

## Checkpoint 86 measurements

[Completed checkpoint evidence](../../benchmarks/runtime/diagnostics/2026-10-03T14-02-43-000Z-checkpoint86-completed-proof.json) accepts 806 named checks on the tested revision, retaining successful targets and repairing failures without a full restart. [Runtime/resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json) accepts 6 comparisons for this milestone across eight runtimes. Native leads compiled Node on 3/6 and the fastest scripting runtime on 2/6. Results measure complete setup, selection, operation and retained output; they do not establish isolated method latency or allocation parity.

Across gates 82–86, forty new rows and one existing-row refresh produce 30,504 timed batches and 123 native profiles, all with zero final live bytes. The report now contains 554 individually dated measured rows. The other 513 report sections, nine primitive memory tables and auxiliary baselines remain byte-exact. Concurrent project/editor revisions outside the tested checkpoint are explicitly excluded from its regression claim.
