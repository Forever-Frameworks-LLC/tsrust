# Generic function frontend scaling

Run `python3 benchmarks/compiler/generic-functions/check.py` from the repository root after building the current CLI in target/editor. The validator checks eight deterministic source pairs, emits Rust, strictly checks TypeScript and prints source hashes. It does not collect timing samples.

Counts 1, 16, 64 and 128 vary distinct nominal identity<T> instances. Inferred mode uses implicit applications; mixed mode alternates explicit/inferred calls. Both modes repeat Argument0 through inferred and grouped explicit paths to verify cache reuse at the boundary. Record layouts are deliberately identical; Tsrust nominal identity keeps the instances distinct while TypeScript checks structural types.

Measurement remains pending checkpoint 96. Record check and emit separately; include process startup/source reads, exclude rustc from frontend emit, and do not present different compiler validation coverage as equivalent operations. Preserve exact source, generator and toolchain identities plus individual samples in canonical SQLite before accepting measurements. Runtime performance uses the separate generic_function_record_pipeline workload.
