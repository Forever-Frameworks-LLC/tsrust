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

## Spike 2: link-time declarations inside the cycle

`inbound.py PROGRAM_WITH_LOCATIONS.rs OUT.json` lists, per source file, which
of its generated items other files reference and whether any are async or
non-function items. `split_extern.py PROGRAM.rs MEMBERS.json OUT INBOUND.json`
splits like `split.py`, but the library crate declares the module's inbound
functions in `unsafe extern "Rust" { pub safe fn …; }`, the module crate
defines them with `#[unsafe(no_mangle)]`, and the module crate contains
`let _ = [rest::f, f];` for each, so a signature mismatch is a compile error.
(Spike-only fixups: `pub(super)`/`pub(crate)` widened to `pub` in the library,
and `use tsrust_runtime::*;` added to the module crate.)

Result, 2026-11-16 (Studio, src/services/ai/context.tsr: 594 KB, 46 items,
inside the 37% cycle, reached through 4 plain functions):

- Both crates compile; each declared symbol resolves to one definition; the
  split Studio binary starts and shows its window.
- One-literal edits in the module crate rebuild and link in 0.60-0.70 s
  (type check 0.004 s, LLVM 0.07 s, link 0.36 s), versus 13.9 s for today's
  single incremental crate.
- A definition whose signature differs from its declaration fails to compile
  at the `[rest::f, f]` check.
- Across Studio, 1,883 cross-file references: 1,539 plain functions, 344
  closure types, 0 async functions. 43 of 92 files form one file-level cycle.
  Closure types need their data struct in a shared crate and their body as a
  link-time-declared function in the module crate (one direct call, same
  machine code).
