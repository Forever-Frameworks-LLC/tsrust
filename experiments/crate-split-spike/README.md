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

Async follow-up: async source functions are emitted as closure structs with
`async fn call`. Cross-file calls returning such closures (type inferred):
Studio 16 calls / 12 closures, Worklist desktop 7 / 6, Worklist API 0. Only
those against the crate dependency direction force modules to share a crate.

## Spike 3: every module in its own crate

`split_modules.py PROGRAM.rs PLAN.json OUT` splits a whole program, and
`build_split.py OUT RUNTIME_ARTIFACTS BINARY [--only CRATES]` builds it with
rustc. PROGRAM.rs is the program a `tsrust build --release` compiles (runtime
modules come from `tsrust_runtime`, plain numbering); PLAN.json is
`tsrust emit --crate-plan` from the same compiler.

- `tsrust_types`: every item used from more than one crate (records, unions,
  families, shared closures, runtime glue) plus compiler-made functions.
- `tsrust_m<K>`: source module K's generated functions, closures and types
  only it uses, and the bodies of its synchronous closures whose structs had
  to move to `tsrust_types`. A per-closure trait keeps the body text unchanged;
  an `export_name` function calls it, and the closure's `call` in
  `tsrust_types` calls that function through a link-time declaration.
- `program`: `main` and `let _ = [declared, defined];` for every link-time
  declaration. Module crates depend only on `tsrust_types`, so they build in
  parallel. Function calls between module crates are link-time declarations.

The staged runtime rlib has a hashed crate name (`tsrust_runtime_<hash>`);
`build_split.py` links it under that name for dependents.

Results, 2026-11-16 (dev flags, opt-level 0, incremental per crate):

| Program | Crates | Cold build | Literal edit in one module | Shared types change |
|---|---|---|---|---|
| Studio | 103 (types 5.9 MB, std:ui 4.9 MB) | 15.8 s | 1.1-1.2 s (single crate: ~14 s) | 9.7 s |
| Worklist desktop | 9 | 8.0 s | - | - |
| Worklist API | 4 | 1.2 s | - | - |

All three binaries start and behave like their single-crate builds (API: same
missing-database error; Studio and Worklist desktop open their windows).
Studio needs 1,319 link-time function declarations and 857 closure-body detours.
