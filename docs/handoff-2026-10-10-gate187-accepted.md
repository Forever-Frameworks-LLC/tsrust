# Gate 187 accepted: native byte foundation

Read AGENTS.md, docs/development-workflow.md, docs/resources-bytes-plan.md and
[the acceptance record](milestones/187-native-byte-foundation.md) first. B11/S07
remains in progress. Gate 187 is accepted; next is gate 188, not more foundation work.

## Completed scope

Native declaration identity, checked recursive signatures, strict/tolerant type
imports, aliases/privacy, opaque construction/capability guards, public ByteError,
fromArray/length, native Result ABI, stored/forwarded native functions and generic,
record/array/Result payload storage have focused evidence. Full catalog descriptions,
parameter help and executable examples exist. Structured exported aliases keep hidden
native field help and source navigation without capturing same-named user records.
Private editor identities do not appear in completion or displayed help.

Closing the gate found one B10 composition defect: exported Result aliases were
qualified as user generic records. The linker now preserves built-in Result, with a
native failing-then-passing witness for primitive and native payload aliases.

The closing batch has 68 passes (24 compiler, 41 editor, three harness controls).
Earlier retained records include ten normal-stack depth controls and two executable
catalog examples. The native_bytes six-observation workload has five matched native
and Node-after-tsc cases verified untimed. Sources, oracle, guards, registration,
runner/profile/report hooks and manifest hashes are ready. No full regression or timing
was run; both belong to the completed S07 family milestone. SQLite records and exact
readback are listed in the acceptance record; loose records were removed.

Source closes at 6e565765. All five consumers are refreshed atomically from that
commit: target/editor/debug/{tsrust,tsrust-lsp}, target/debug/{tsrust,tsrust-lsp}, and
target/release/tsrust. Compiler debug/release storage witnesses and installed LSP
initialization/shutdown pass. Final native/Node matched verification also passes after
the alias repair. Exact publication hashes/checks and final verification are in SQLite:
runtime/results/gate187-2026-10-10-publication.json and
runtime/results/2026-10-10T14-17-12-756Z-verify.json.

Pruned 257 obsolete test executables (7,968,483,416 bytes) and generated runtime outputs.
Shared libraries and installed consumers remain. The shared target retains warm
libraries, but pruned test executables will need rebuilding when their suites run.

## Next: gate 188

Implement the owned byte operations in the plan: checked at/set, allocation,
append/resize and explicit independent clone/copy. Keep Vec<u8> storage, validate ranges
and fallible reservation, evaluate each argument once and preserve mutation atomicity
on failure. Do not add implicit clones or shared allocation. Existing explicit clone
metadata is not evidence for the complete clone/cleanup matrix.

Complete ownership/aggregate/capture/cleanup witnesses, including exact allocation
restoration and disposal at optimization levels zero and three. Expand the matched
workload with retained observations and an independent counterpart/oracle as each
operation lands. Every public operation needs complete catalog help and verified
examples in the same implementation gate. Unsupported capabilities must diagnose in
source instead of failing generated Rust.

Gate 189 owns scoped zero-copy views and strict UTF-8. Gate 190 owns consuming resource
close and a useful file proof. Close transfers ownership, rejects later use/double close
even on Err, and keeps a nonthrowing drop fallback. Do not publish a fake test handle.
Declaration-only match payload inference remains an existing limit, not a new guarantee.

## Environment and method

One agent in this tree. No protected owner edits remain. Use explicit Git paths.
Use /private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0. Node 24.11.1 goes
first on PATH. Use /opt/homebrew/bin/python3 for store.py. Whole-project formatting
precedes source checkpoints; successful fixture tests format in memory and execute the
formatted source. Keep original evidence, import/read back exact SQLite records, then
remove loose copies. No archives or routine full regressions. Refresh executable
consumers by copying to .refresh-tmp and atomically renaming.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-10-gate187-accepted.md. Gate 187 is accepted;
implement gate 188 owned byte operations with its capability, ownership, mutation and
exact cleanup matrix. Keep compiler/editor parity and matched benchmark support current.
Then proceed through planned gates 189 and 190. Preserve consuming close semantics.
Use bounded checks and whole-project formatting; complete regressions and timing only
at the S07 family milestone. Give the next physical handoff at a concrete checkpoint.
