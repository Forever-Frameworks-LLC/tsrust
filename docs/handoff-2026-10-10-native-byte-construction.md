# Native byte construction checkpoint

Read AGENTS.md, docs/development-workflow.md and docs/resources-bytes-plan.md first.
B10 is accepted at gate 186. B11/S07 gate 187 remains in progress. This checkpoint
adds the first working fromArray/length bindings; it does not accept the entire gate.

## Completed in this checkpoint

Structured native callable signatures resolve through private schema records in the
existing checked arena. Native Result payloads reuse ordinary tagged alternatives.
The bytes backend validates integer bounds before copying, reports the first invalid
offset and uses fallible reservation. Length borrows the owner. Both native operations
have explicit nonthrowing failure contracts. Result failures do not become throws.

Catalog entries include argument help, purpose, errors, examples and ownership.
Hidden native Result payloads receive checked editor field help without explicit type
imports. Function signatures display public native names and structured access modes.
Declaration-only match arms retain the existing lack of payload type inference.

Thirty-five focused tests and two executable catalog examples pass. This includes ten
normal-stack depth canaries, native declarations, source ownership/arity/type rejection,
byte copy independence, checked Result identity and native editor controls. Project
formatting changed one fixture initially and then changed zero files. Exact command
outputs are in SQLite at runtime/results/native-byte-construction-2026-10-10-checks.json;
readback was verified byte for byte and the loose record removed. No full regression,
timing or installed consumer refresh was performed.

## Next increment

Finish gate 187 before the wider gate 188 operations. Cover structured exported aliases
with hidden native elements, native callable storage/forwarding and Result aggregate
storage. Complete the minimal matched byte workload, independent oracle, source contract,
registrations, runtime/profile/report hooks and manifest row appended at the end. Verify
untimed. Do not rebuild already completed native identity/import/capability work.

Resource handles use consuming close by owner decision. Later use and double close are
static errors, even after close returns Err. Drop fallback is nonthrowing. Resource
lifecycle is gate 190; it has not been implemented by this byte checkpoint.

Use the warm target /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0.
Use Node 24.11.1 first on PATH and /opt/homebrew/bin/python3 for store.py. Five installed
consumers still represent accepted B10. Keep this distinction until a verified source
publication. No other agent or protected owner edits exist in this tree.

## Continuation prompt

Continue Tsrust from this handoff and docs/resources-bytes-plan.md. Finish gate 187's
remaining alias/callable/storage witnesses and matched workload support, then proceed
through gates 188–190 with bounded focused checks. Preserve consuming close semantics.
Use whole-project formatting before explicit-path source commits, exact SQLite evidence,
and full regressions/timing only at the completed S07 feature-family milestone.
