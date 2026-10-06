# Milestone 78: ordinary template literals

Status: implementation and checkpoint 81 accepted. Focused behavior and complete IntelliSense examples pass; combined regression and selected comparisons are sealed.

## Behavior and native differences

Backticks construct a String from literal chunks and `${expression}` substitutions. Substitutions evaluate once, from left to right. Each value is appended before the next expression runs. Strings preserve their complete UTF-8 content. Interpolation accepts string, int, number and boolean; arrays, records, functions and void require an explicit supported conversion instead of JavaScript object coercion.

Integer and floating-point values use standard Rust Display formatting. Boolean values become `true` or `false`. Floating-point spelling can differ from JavaScript, including negative zero, infinities and exponent formatting. Number interpolation is not a promise of JavaScript Number.toString behavior.

Literal chunks share strict quoted-string escapes, with escaped backticks and `\${` supported. Raw CR and CRLF normalize to LF; raw LF, U+2028 and U+2029 are preserved. Escaped line continuations contribute no content. Every chunk must be valid native Unicode: lone surrogate escapes are rejected even across an interpolation that would produce empty text. Tagged templates remain a separate feature.

Ownership: substitutions borrow their inputs while appending. The completed interpolated result owns an independent standard Rust String. A borrowed input remains reusable, and its borrow ends before the next substitution. Templates without substitutions retain static borrowed storage. Existing surrounding loans still apply; interpolation cannot bypass ownership rules in a larger expression.

## Examples and acceptance

```tsr
function main() {
    const count: int = 3;
    const ready: boolean = true;
    console.log(`count=${count}; ready=${ready}`);
    // count=3; ready=true
    console.log(`Unicode: ${"😀é"}`);
    // Unicode: 😀é
}
```

The example above passes current compiler emission and native execution at O0/O3, with both complete output lines matching. Pool and source-order auxiliary fixtures also pass at both optimization levels: six native builds produce 52 exact output lines in total. Focused fixtures also cover nested templates, multiline normalization, evaluation order, borrow lifetimes, temporary disposal, malformed escapes, unsupported interpolation types and nesting limits. Editor checks cover completion, references, hover, inferred String receivers and unfinished interpolations. This syntax feature adds no built-in method or property; the built-in catalog remains unchanged.

The lexical reference is [ECMAScript template components](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-template-literal-lexical-components). Native formatting and ownership follow the implementation and focused execution evidence rather than JavaScript coercion rules.

## Focused implementation acceptance

All four compiler groups and three editor groups pass across focused runs. The normal-stack depth check accepts 100 nested indexes and rejects 200 with the existing 128-depth diagnostic. Template-only parser predicates and diagnostics use separate frames; limits and worker stack remain unchanged. Initial fixture failures, EOF completion failure and parser stack failures are retained with their repairs.

Two equivalent template workloads pass 274 source-validation cases, 24 independent hand cells, 57 negative source guards and five executed semantic mutants. Native pool/source-order and documentation fixtures pass six O0/O3 builds with 52 exact output lines. The [implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T06-19-03-000Z-template-literals-implementation.json) retains actual sources, commands and outputs. Comparisons remain unmeasured until checkpoint 81; existing measured rows stay unchanged.

## Checkpoint 81 acceptance

The combined checkpoint accepts 700 compiler integration groups and 735 workspace groups with retained passes and no full restart. [Runtime and resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T08-44-00-000Z-checkpoint81-runtime-proof.json) binds 29 selected workloads across eight runtimes and 87 fully disposed allocation samples. [BENCHMARKS.md](../../BENCHMARKS.md) contains actual min/avg/max ops/s. Earlier pending statements describe the implementation-stage queue; this checkpoint closes that queue.
