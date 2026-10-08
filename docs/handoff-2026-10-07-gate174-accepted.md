# Handoff: gate 174 benchmark layout contracts accepted

Date: 2026-10-07. Read `AGENTS.md`, `docs/development-workflow.md` and
`docs/milestones/174-benchmark-layout-contracts.md` first.

## State

The benchmark repairs are checkpointed in `ef255d6`. The normal-stack parser
repairs and coverage records are checkpointed in `191cce9`. The closing commit
adds this handoff and the final evidence references. Fifteen owner fixture edits
remain uncommitted and unstaged. Preserve them exactly and use explicit Git paths.
No worktree, archive directory or source snapshot was created.

Five consumers are refreshed atomically from the verified gate 174 compiler source:
`target/debug/tsrust`, `target/debug/tsrust-lsp`, `target/editor/debug/tsrust`,
`target/editor/debug/tsrust-lsp` and `target/release/tsrust`. Their bytes were compared
with the shared target outputs. An initial refresh used the wrong working directory;
the five misplaced copies were removed, the real destinations were corrected, and
nine affected native tests and matched verification ran after the corrected refresh.

## Accepted behavior and evidence

- Benchmark harness: 421 of 421, compared with 368 of 419 at entry. The two added
  tests pin metadata and formatting identities. Required code snippets and mutations
  ignore layout while retaining literals, operators, order and occurrence counts.
- Twenty-three manifest prefixes retain exactly the metadata from commit 0110953
  after excluding only independently checked source fingerprints.
- Sixty original-to-formatted source transitions are proven token-identical against
  commit 0110953. The numerical policies and MPFR reference files are unchanged.
  Arbitrary changed bytes do not match the registered identity transitions.
- Five representative workloads pass five untimed native/Node-after-tsc cases on
  refreshed consumers: `math_sin`, `array_concat_int_zero`, `int_assignment_value`,
  `compound_value_int_add` and `optional_chain_int_field`. No timing or allocations.
- Whole compiler coverage: 2,214 distinct passing tests and the two known callable
  replacement witnesses. The initial whole command had 2,178 passes, two failures
  and two aborted targets. Aborted targets were absent from that numerical pass
  count. The recovered suites add 11 control-flow and 24 nullable-callable passes;
  the new parser unit adds one. All eight depth canaries pass.
- Whole editor crate: 339 of 339.
- Project formatting changes zero files and zero fingerprints.

The whole compiler check exposed two normal-stack parser overflows. LLDB and actual
compiled frames identified conditional and call parser temporaries. Separate
construction helpers reduce `if_statement` from 8,752 to 5,120 bytes and `call_step`
from 2,272 to 832 bytes. The worker stack and 128 source-depth guards are unchanged.
A stale rejection of ordinary lexical shadowing was replaced after verifying inner
`2` and outer `1` in native execution and Node.

Closed logs, command scripts, metadata proof and both verification records are in
`benchmarks/records.sqlite`, read back byte for byte before loose copies were
removed. Diagnostic paths begin `runtime/diagnostics/benchmark-layout-contracts/`.
The current-consumer raw verification record is
`runtime/results/2026-10-08T00-28-38-163Z-verify.json` (UTC filename).
`target/editor/gate174.log` is the concise local index.

The shared Cargo target is about 7.1 GiB. Cache inspection protected all executable
paths referenced by the compiler/editor checks and newest variants. The duplicate
names are also used by separate crate test targets; no unreferenced obsolete
executable variant was eligible for deletion. The five consumers remain intact.

## Next gate

Recommend callable replacement before the named-declaration family milestone.
Keep both existing red witnesses and fix their causes:

1. `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`:
   a generic callable local is replaced by a different origin and needs a common
   native carrier without weakening capture or ownership rules.
2. `generic_call_validation::branch_flow::invocation_worklist::mutable_environment_sequence_tests::owned_factory_updates_are_point_sensitive_and_instances_stay_separate`:
   retained mutable factory environments need point-sensitive state and independent
   factory instances.

Start with their focused reds and inspect the current callable storage and invocation
modules. Use the existing three public invocation wrappers and ownership model;
no implicit clone or shared JavaScript capture cells. Include native evaluation
order, ownership, disposal, failure and editor witnesses, followed by matched
workload support and acceptance records. Run the whole compiler crate at gate
acceptance, preserving completed checks and rerunning affected failures.

Other open work remains unchanged: scoped callable aggregate composition, fallible
callback fields behind record parameters, unreachable catch body checking, recursive
result inference and the named-declaration family milestone with actual measurements.
Gates 172 and 173 are accepted; do not reopen their completed components.

## Environment

Put `/Users/seancotter/.nvm/versions/node/v24.11.1/bin` first on `PATH`. Homebrew
Node 26 is incompatible with the runner. Use `/opt/homebrew/bin/python3` for
`benchmarks/storage/store.py`.

Set `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target` and
`CARGO_INCREMENTAL=0`; use Cargo `--offline --locked`. Append manifest rows at the
end of the workload list. Use absolute paths for consumer refresh destinations.
