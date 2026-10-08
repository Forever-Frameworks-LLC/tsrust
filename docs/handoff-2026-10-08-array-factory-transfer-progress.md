# Handoff: bounded array provenance and independent factory extraction

This is implementation progress after `54bd1b0`, not family acceptance. Read AGENTS.md, docs/development-workflow.md and docs/callable-replacement-plan.md before continuing. Fifteen protected owner fixtures remain unstaged and must not be committed. Use explicit Git paths.

## Implementation

Generic source flow tracks array elements as an ordered prefix and optional repeated tail. Equal-length exact joins keep every position; changing loop lengths summarize a bounded common prefix and repeated tail. Push, unshift, pop and shift preserve callable alternatives. forEach still requires an exact sequence. A summarized element is not a concrete runtime count.

Successful non-null assertions preserve callable contracts. Known nullish assertions have exceptional completion only. The shape controls check concrete binary words through length four and 1,024 loop-growth joins.

Array removals assign callable environments independent analysis identities using the original construction root, extraction source site, caller context and current finite capture facts. Transfer chains do not recursively grow identities. The non-inlined helper is generic_call_array_transfer.rs. Native captures are transferred under existing source ownership checks, never cloned. Unknown or consumed capture state remains rejected; repeated extraction into an existing abstract identity uses merged state.

## Verified evidence

All 88 adjacent generic-call unit controls pass. All 47 capture-effect integration witnesses passed before adding two more successful focused native controls: reinsertion after mutation, and repeated extraction from one loop site. The original retained-loop failure now executes `1 2 9`. Editor diagnostics and both independent one-argument and changed two-argument signatures pass. All eight rebuilt default-stack depth canaries pass after helper extraction.

Original failures and repaired checks are retained under `tsrust-array-` identities in benchmarks/records.sqlite. Exact readback precedes removal of closed loose copies. The nested-functions guide includes the checked straight-line example. No full regression, timing or consumer refresh is claimed.

## Benchmark support

The generic_callable_replacement workload now has sixteen scalar observations per iteration, including independently retained loop-created factories. Tsrust and TypeScript sources, independent oracle, token contracts and mutation guards are updated. The existing manifest row remains at the end. Whole-project formatting changed only this workload and refreshed its source fingerprint. Formatted registration checks pass. All five untimed native/Node-after-tsc cases pass on the current warm release compiler. The exact record is runtime/results/2026-10-08T07-10-47-227Z-verify.json in SQLite, with zero timing samples.

## Remaining work

Audit updates to array-held cohorts, repeated extraction aliasing and nested owned capture graphs. Rooted reachability and allocation multiplicity need proof; passing the simple generic declaration witnesses is insufficient. Extend controls if those paths reveal defects. Transfer-key controls also pass their additional construction-root and caller-context assertions. Benchmark support is verified untimed. Record acceptance separately from pending family measurements and complete the state audit before family acceptance. This handoff accompanies an implementation source checkpoint.

Keep main ownership exclusive. The stdlib worker may implement path.relative in its isolated tree and must obtain main ownership before integration. Existing main editor/CLI consumers remain built from accepted source `31e367f`.

## Environment

Use `/private/tmp/tsrust-gate100-editor-target` with CARGO_INCREMENTAL=0 and offline/locked Cargo. Node 24.11.1 must be first on PATH. Use `/opt/homebrew/bin/python3` for SQLite storage tools. Preserve exact evidence identities and historical analytics. Do not create archives, stash owner edits, run routine full matrices or time unaccepted work.
