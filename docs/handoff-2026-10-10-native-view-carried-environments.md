# Escaped carried-view environment transport

Continue from TDD checkpoint `43aaaccc`, which retained the callback-escape red
witness. Read AGENTS.md, docs/development-workflow.md and
docs/native-byte-views-implementation.md. Gate 189 and S07 remain open.

native_view_environments.rs records native capture-field loans in the current
construction context. Value analysis wraps carried loans in capture fields and
transports returned environments through ordinary function return summaries.
Source::Capture now identifies a capture's inner field, resolved from the actual
callee binding. It no longer names a factory's outer slot in a different caller.

The callback local-storage guard follows owned capture payload types when native
borrowed identities are present, retaining physical reference and unknown callback
rejections. Native return provenance rejects an escaping closure whose view comes
from local receiver storage. native_view_environment_emit.rs renders named lifetime
arguments from typed capture paths for returned concrete environments.

The carried-view witness uses a two-parameter saving factory and a caller whose
selected source occupies a different slot. Analysis retains caller argument one.
O0/O3 native execution allows the unrelated owner to end first and preserves the
selected pointer; independent native compilation rejects selected-owner escape.
Nested owned readonly/exclusive factory controls and move/rebinding controls pass.

Focused checks: 73 native-view tests, 107 source tests including all ten normal-stack
depth canaries and Result, nine registered signatures and 16 native editor tests.
Final test-only strengthening changed argument order and added owner-escape control;
the focused carried-view native test was rerun. No full regression or timing.
SQLite evidence: `runtime/results/native-view-carried-environments-2026-10-10.json`.
CLI build was warning-free; project formatting changed zero files/fingerprints. Public APIs
remain guarded; installed consumers remain f569190a.

## Continuation prompt

Continue gate 189 with nested carried environments and sparse capture-context
projection, then abstract/stored families and mixed callback records. In particular,
capture_context clones binding loans but does not remap their roots; native parameter
seeding currently traverses records/arrays/unions, not captured concrete closure
fields. Add a sparse nested carried-view witness before extending that model.
Keep actual source argument identity and typed named native lifetimes, with local
owner rejection. Finish observer contracts and editor/catalog parity before lifting
publication guards. Use focused TDD, bounded checks, formatting, SQLite exact
evidence and explicit-path commits. Refresh consumers before executable checks.
Full regressions and timing belong to S07 completion.
