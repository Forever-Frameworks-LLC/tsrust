# Scoped worker ergonomics

This slice adds `spawnScoped` and `ScopedThread<T, E>` to `std:threads`, plus
`unwrap` in `std:result`. The original `spawn`, `join` and `detach` contracts remain
available. No general defer syntax, scheduler, pool, cancellation or borrowed
worker captures are added.

`spawnScoped` starts one OS thread immediately and returns
`Result<ScopedThread<T, E>, SpawnError>`. The returned owner registers fallback
joining. Leaving its owning scope waits for the worker if the handle was not
already joined, detached or moved. This includes normal exits, return, throw,
break and continue. A move transfers responsibility to the destination owner.
Fallback joining discards the completion value or failure, never replaces a
pending caller error, and never implicitly detaches. A worker that never finishes
can block scope exit indefinitely. Call `join` to observe worker failures.

`unwrap(result)` consumes a Result, returns its Ok payload, or throws its original
Err payload through existing typed error transport and try/catch/finally.
`Result<void, E>` returns void. Payloads move without cloning or string conversion.
Direct calls and import aliases are supported; polymorphic operation values remain
unsupported. Throwing a thread owner through unwrap is rejected. Explicit match
remains available for local recovery.

Scoped handles keep the existing transfer restrictions: concrete owned callbacks,
owned transferable inputs and completion values, no caller or program-state loans,
no cloning, no containers and no retained callback captures. Raw Thread owners
still require explicit settlement and still prohibit fallible work while live.
Scoped owners permit fallible caller work because their native owner performs
fallback joining on every exit. Inconsistent branch consumption remains rejected.

The readable example is `examples/scoped-workers/main.tsr` (output 25205).
The original sequential spawn/join benchmark remains unchanged. The additional
paired `scoped-join` workload retains the eight-worker checksum oracle (140) and
original measurement protocol. `--verify --scoped` records untimed correctness,
source fingerprints and tool identities; it does not establish multicore speedup.

Acceptance witnesses cover generated native cleanup on normal, return, error and
loop exits; preservation of caller errors during cleanup; explicit join/detach;
void completion; owned/union error propagation; single-use and transfer rejection;
and editor hover, signatures, completion and executable catalog examples.
The generated overlap witness requires two workers to observe each other's file
markers before either may finish, with a bounded deadline. Native runtime tests
also synchronize two workers with a barrier and verify disposal after fallback
joining. These prove overlap without relying on a timing-ratio threshold.

Acceptance results and consumer refresh evidence are recorded after validation in
SQLite under the `tsrust-thread-ergonomics` prefix. Full-family regression and timed
multicore performance comparisons remain separate milestone work.

## Focused acceptance

- Nine thread integration tests, three native runtime tests and 46 error-effect
  tests pass. The generated rendezvous proves overlapping workers; cleanup checks
  cover every scoped exit and preserve a pending caller error.
- 128 affected Result, exception, formatting and callable-ownership regressions
  pass. The formatter corpus preserves tokens and idempotence for 240 fixtures
  and 84 standalone contracts. The whole-project formatter runs before checkpoint.
- 31 editor checks pass across thread support, completion matrix, checker
  agreement, editor consistency and schema editing. The new propagation row passes
  immediate, paced, settled and unrelated-error editing modes. Catalog examples
  execute at Rust optimization levels 0 and 3.
- Both original sequential and new paired scoped workloads match the independent
  native/Node checksum 140. These are untimed checks, not performance measurements.
- The readable scoped example executes with output 25205. Compiler workspace
  all-target checks are warning-free. Original failed attempts and passing repairs
  remain in SQLite; only affected checks are repeated after repairs.
