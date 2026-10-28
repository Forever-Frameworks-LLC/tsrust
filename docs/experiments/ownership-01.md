# Ownership experiment 01

[Milestone scope](../milestones/01-ownership-proof.md) · [Documentation index](../README.md)

This experiment pairs 11 proposed `.tsr` acceptance inputs with **handwritten** Rust lowerings. It checks whether those specific ownership behaviors can be represented by safe Rust and whether the resulting programs produce the expected output or compiler rejection.

The [Rust compiler](../../crates/tsrust-compiler/README.md) parses, checks, and compiles the `.tsr` inputs. The Cargo witness suite checks only the handwritten Rust files. Its results demonstrate lowering feasibility, not compiler correctness or general inference soundness. The `.tsr` extension remains provisional.

## Run

From the project root:

```sh
cargo test --offline -p tsrust-compiler --test witnesses
```

Requirements: Rustup with the [pinned Rust toolchain](../guides/tooling/rust-toolchain-policy.md) installed. The suite needs no external packages or network access after toolchain setup. Native executables are created in a temporary directory and removed after the run. The witness suite also checks that every fixture has exactly one entry in the typed case inventory.

## Recorded result

On September 30, 2026, the command passed all 11 witnesses with `rustc 1.96.1 (31fca3adb 2026-06-26)`: eight programs compiled and produced exact expected output; three programs were rejected with the expected ownership/access error codes. This establishes only the handwritten lowering evidence described above.

## Case coverage

| Case | Expected behavior |
| --- | --- |
| `read_reuse` | Two reads retain the caller's owned value. |
| `mutate_reuse` | Mutation is visible after an exclusive call borrow ends. |
| `move_assignment` | Assignment transfers ownership to the new binding. |
| `use_after_assignment` | Rust rejects access to the transferred value with `E0382`. |
| `clone_independence` | Mutating an explicit clone leaves the original unchanged. |
| `consume_return` | Returning a parameter transfers its ownership through the call. |
| `use_after_call` | Rust rejects access after a consuming call with `E0382`. |
| `two_read_arguments` | One call may read the same object through two parameters. |
| `overlapping_access` | Rust rejects simultaneous mutable/read access with `E0502`. |
| `transitive_mutation` | A wrapper propagates its callee's mutable access requirement. |
| `owned_string_result` | String transforms return owned output while preserving the input. |

[The typed case inventory](../../crates/tsrust-compiler/tests/support/corpus.rs) records each original claim, expected parameter contracts, exact successful stdout, and rejection codes. It replaces the former JSON manifest without requiring a JSON dependency. The contracts are manually specified expectations checked against inferred `.tsr` contracts by the compiler acceptance suite. The witness suite does not infer them. Rust error codes are backend evidence only; the compiler reports source-language diagnostics.

The `User` record contains a single owned string. `Clone` is derived in the handwritten Rust scaffolding; only the explicit-clone case calls it. String literal construction and string transforms deliberately materialize owned strings. Those operations are distinct from inserting clones to repair an invalid ownership transfer.

## Limits

This experiment does not cover control-flow joins, recursion, field moves, borrowed returns, closures, generic dispatch, multiple modules, shared ownership, concurrency, or async. It does not measure allocations, compile-time performance, or destructor timing. Those claims require separate evidence.

## Compiler acceptance checks

Run `cargo test --offline -p tsrust-compiler --test acceptance` from the project root to check the compiler. It uses the `.tsr` inputs directly, compares inferred contracts, runs generated native programs for the eight valid cases, and checks source-level rejection for the three invalid cases. The handwritten `.rs` files are not inputs to those compiler tests. Run `cargo test --offline` for all suites.
