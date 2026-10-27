# Primitive performance history

[Roadmap](../roadmap.md) · [Latest benchmarks](../../BENCHMARKS.md) · [Foundation audit](../plans/foundation-audit.md)

The original two optimization passes are preserved in the [historical archive](../../archive/utf16-strings-2026-10-01/docs/measurements/primitive-performance.md). They describe earlier string representations and measurements, not the current runtime.

The active implementation uses standard Rust UTF-8 strings and methods, documented in the [string contract](../guides/language/string-semantics.md). Obsolete literal normalization dispatch, encoding variants, interning domains, and shared string owners are removed from the compiler crates. Array storage proofs and independent mutable cloning remain active.

[Milestone 33](../milestones/33-native-utf8-strings.md) completes compiler, timing, allocation, and disposal acceptance for the native contract. Current min/avg/max measurements live in top-level [BENCHMARKS.md](../../BENCHMARKS.md). Earlier raw records remain immutable and are excluded from current epoch tables.

Follow the [primitive follow-up](../plans/primitive-followup.md) for measured costs and the native Rust direction. A performance difference from a JavaScript engine does not impose a compatibility runtime or block every new language feature.
