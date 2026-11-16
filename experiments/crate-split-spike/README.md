# Crate-split timing spike

Measures what one-crate-per-module would cost per edit, using Studio's real
generated program before any emitter change.

1. Capture the application source and staged dependencies a `tsrust dev`
   build compiles (wrap `rustup` to copy `program.rs` and the
   `-L dependency=` directory).
2. Pick a source file outside the large callable cycle with
   `tools/crate-split-analysis.py PROGRAM.rs --candidates` (needs an
   `emit --source-locations` build for file mapping).
3. `python3 split.py program.rs '["Item1","Item2"]' OUT` undoes codegen-unit
   chunking, makes items public and writes `rest.rs` (everything else, built
   once as an rlib named `rest`) and `edit.rs` (the file's items plus `main`).
4. Build `rest` once, then time incremental rebuilds of `edit` after
   one-literal edits with identical flags every run. Link a copy of the
   runtime rlib under its hashed crate name (`libtsrust_runtime_<hash>.rlib`)
   so `rest`'s dependency resolves.

Result, 2026-11-16 (Studio, src/services/lsp/settings-tools.tsr, 5 items):
`rest` cold build 34 s once; each edit then rebuilds and links the 99 MB
binary in 0.47 s (rustc 0.02 s, link 0.47 s), versus 13.9 s for today's single
incremental crate. The split text transform is a spike, not product code.
