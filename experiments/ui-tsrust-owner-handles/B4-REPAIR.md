# B4 compiler repair

Historical repair report. Current identity status is in [IDENTITY.md](IDENTITY.md);
the original B4 evidence and acceptance results below remain unchanged.

Baseline: `7510b2542929c094338ce13edbc95692db8ae387`. Branch: `codex/ui-b4-repair`. This bounded repair changes callable analysis, two focused test targets and this experiment. It does not implement identity enforcement or resource observation. B1/B2 remain open; B3 remains adopted.

## Cause and repair

`Analyzer::prepare_specialization` appended a concrete function to `program.functions`, `signatures`, `checked` and `visiting`, but omitted its `assumed_modes` entry. Record callback discovery can call that specialization before its body has been checked. `completed_declared_call_contract` then indexes the missing entry. The original named witness reproduces length 22/index 22; the reduced compiler test reproduces length 20/index 20 even without delivering an event.

The repair admits the new function's parameter coordinates alongside the function, following the existing generic admission in `analysis/generic_calls.rs`. Copy parameters start at Copy and noncopy parameters at Read, exactly as the recursive inference lattice already requires. These are speculative starting coordinates, not final fallback contracts. Existing entries are preserved on subsequent iterations. The existing fixed-point driver adds the corresponding convergence budget, joins inferred modes monotonically and performs definitive ownership checking. Named-capture planning and hidden capture access continue through their existing separate machinery. Neither consumer lookup is weakened or bypassed.

The compiler change is confined to `crates/tsrust-compiler/src/analysis/mod.rs`. New regression targets are `crates/tsrust-compiler/tests/named_handler_specialization.rs` and `crates/tsrust-lsp/tests/named_handler_specialization.rs`. The compiler test formats imported sources, retains the full original named/factory witnesses, and moves the named declaration after its callers. The minimal case places the named handler inside a retained callback and registers it outside too. Existing callback suites verify read/mutate/consume, owned captures, recursion and definitive callable contracts. The editor test checks the valid module and a strict use-after-transfer rejection.

## Validation commands

All commands ran in `/Users/seancotter/.codex/worktrees/ui-b4-repair/tsrust`. TLS explicitly released the shared target after builds/timings finished, with no further build or consumer changes. Cargo used this environment:

```sh
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target
export PATH=/Users/seancotter/.nvm/versions/node/v24.11.1/bin:$PATH
cargo test -p tsrust-compiler --test named_handler_specialization --test recursive_callable_results --test callable_captures --test owned_closure_arguments --offline -- --test-threads=1
cargo test -p tsrust-lsp --test named_handler_specialization --test checker_agreement --test editor_consistency --test completion_matrix --offline -- --test-threads=1
TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust node experiments/ui-tsrust-owner-handles/cli.mjs demo
TSRUST=/Users/seancotter/.cargo/bin/tsrust node tools/format-project.mjs
```

Whole-project formatting ran twice with a read-only Node resolver hook for main's existing TypeScript installation. The hook, exact commands, token checks and results are preserved in SQLite. All 1,338 tracked Tsrust sources retain their tokens; both passes leave all 5,684 tracked files byte-identical. No outside-ownership formatting conflict occurred. Formatted successful witnesses execute; a deliberately wrong expected state fails natively with AssertionError. No compiler suppression or generated Rust patch was added.

Compiler: **38 passed** (2 new, 16 recursive callable results, 6 callable captures, 14 owned closure arguments). UI: **6 passed**, no skipped tests. Editor: **13 passed** (2 checker agreement, 3 completion matrix, 7 editor consistency, 1 new named-handler/ownership regression). Demo: **exit 2**, both self-replacement traces pass; only B1/B2 remain open. All C01–C16 requirements are dispositioned in `b4-matrix.json`.

The prior compiler remains installed. `provenance.json` records its consumer hashes separately from the candidate hash and exact compiler source fingerprint. The candidate was built from the baseline plus the recorded analysis change; those compiler inputs did not change during validation. Full source hashes remain in SQLite. Use explicit `TSRUST`; do not silently replace the harness pin or installed tools.

## Evidence and integration

Closed records live in ignored `evidence.sqlite` with `ui-b4-` record names. Byte-exact restoration and SQLite integrity are verified before loose records are removed. They include original failure, reduced reproducer, passing minimization controls, compiler/editor results, UI subprocess records, demo, formatting checks and candidate/consumer identities. Historical S15 identity remains nested in provenance; earlier canonical records in main SQLite are untouched.

The next separately authorized serial integration must merge reviewed commits, import/verify these isolated records into main SQLite and retain the source/tool provenance. Consumer refresh is a separate action: this task does not refresh it or claim the installed compiler is repaired. The shared target is released after handoff; a subsequent build can replace the candidate, in which case rebuild from the committed source and review provenance before repinning.

No full compiler/LSP milestone, performance timing, HTTP/TLS acceptance rerun, merge or consumer refresh was performed. The fix is covered by affected callable and editor regressions; it makes no new API or performance claim. Full UI lifetime feasibility remains blocked by B1/B2.
