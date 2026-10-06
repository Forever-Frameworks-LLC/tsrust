# Foundation audit

[Native string contract](./string-semantics.md) · [Primitive follow-up](./primitive-followup.md) · [Benchmarks](../BENCHMARKS.md)

## Current direction

TSR offers TypeScript-style syntax with Rust-native data structures, ownership, behavior, and operation costs. JavaScript compatibility is optional and explicitly scoped. The user selected native UTF-8 strings rather than continuing the surrogate compatibility and custom storage experiments.

Strings use standard `Cow<'static, str>` with borrowed static literals and owned Rust `String` results. Native length counts UTF-8 bytes; slices check byte boundaries; character access iterates Unicode scalars. Rust trim and lowercase rules apply. Old encoding variants, interning/cache graphs, shared substring owners, and special destructor work are removed from active crates.

## Arrays and records

Stable scalar literals use Rust `[i64; N]` and `[bool; N]`; standard indexing, Clone, and slice search preserve source order and bounds diagnostics. All-use proof follows named calls and rejects resizing, rebind, transfer, return, capture, and unknown operations. Existing stack and specialization limits force conservative fallback. Read-only record arrays reuse fixed array storage.

Growable arrays retain contiguous standard vector storage and bounded initialized small storage. Mutable collection clones remain independent. Recursive records use bounded layout analysis; generated cloning and disposal use explicit work stacks to avoid recursive destructor overflow. User-written recursive functions retain native call-stack limits.

## Verification boundary

Changes must preserve ownership joins, loop availability, argument/capture contracts, field restoration, checked arithmetic, source diagnostics, evaluation order, and disposal. Standard storage changes need representative edge tests and the full compiler suite; benchmark scores alone do not prove correctness.

Historical audit results, including the earlier 474-test fixed-array snapshot, remain in the [UTF-16 archive](../archive/utf16-strings-2026-10-01/README.md) and immutable benchmark records. Those measurements describe the old string contract. Native UTF-8 evidence is recorded separately after migration.
