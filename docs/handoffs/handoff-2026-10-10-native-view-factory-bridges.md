# Guarded view factories and complete forwarding bodies

Continue from `d0d5c9ad`. Read AGENTS.md, docs/development-workflow.md and
docs/milestones/native-byte-views-implementation.md first. Gate 189 and S07 remain open.

VIEW and VIEW_MUT now have native metadata and registry entries. Both borrow Bytes
argument zero and return Result success payloads with readonly/exclusive access.
Ordinary prepare still rejects these bindings with native-return-borrow. Public
imports, completion and catalog entries are not accepted by this checkpoint.

Byte bridges include the existing real slice runtime and explicit source lifetime.
Borrowed Results use generic variant payloads, not a lifetime on Union itself.
That ABI mismatch was caught by native compilation and fixed. The private test-only
prepare helper bypasses only publication and projects the actual native signatures.

The new isolated views_emit_tests.rs harness parses and formats forwarding bodies,
projects native annotations and links their factory calls. Analysis infers Storage
provenance rooted in argument zero; no return summaries or generated function bodies
are injected. Normal emission supplies the full forwarding bodies and bridges.
O0/O3 execution checks pointer identity, empty ranges, range errors and mutable
write-through. Independent Rust compilation rejects owner escape.

Focused evidence: 63 private view tests, two byte runtime/cleanup tests, eight
returned-borrow contract tests and 16 native editor checks. Final factory tests add explicit inferred-provenance
assertions and a clean negative compilation input. No parser/analyzer dispatcher
changed; depth canaries were not repeated. No full regression or timing.
SQLite record: `runtime/results/native-view-factory-bridges-2026-10-10.json`.
CLI build was warning-free and project formatting changed zero files/fingerprints.
The registry-wide test now distinguishes owned contracts from guarded borrowed
contracts rather than assuming the registry contains no borrowed entries.
Installed consumers remain f569190a.

## Continuation prompt

Continue gate 189 with complete capturing factory bodies, including owned/mutable
receiver storage, shared external storage and escaped/rebound environments. Finish
mixed callback records and specialized storage families, then readable/mutable byte
observer contracts and editor/catalog parity. Keep public preparation guarded until
the implementation-plan matrix agrees. Preserve real slices and explicit ownership.
Use focused TDD, bounded native/compiler/editor checks, formatting, SQLite exact
evidence and explicit-path commits. Refresh consumers before executable checks.
Full regressions and timing belong to S07 completion.
