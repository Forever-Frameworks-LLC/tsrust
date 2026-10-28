# Sparse nested carried byte views

Read AGENTS.md, docs/development-workflow.md and
native-byte-views-implementation.md first. Continue from the carried-environment
checkpoint 439d6df0. Gate 189 and S07 remain open.

The new sparse nested callback witness initially failed with closure-escape.
After the analyzer repair it exposed a generated Rust return-lifetime mismatch.
Concrete closure input shapes now seed environment-relative capture paths;
escaping closure checks distinguish physical borrowed storage from owned carried
views. Native capture lifetime mapping follows nested concrete capture fields.

The witness parses and formats source, asserts caller argument one as its sole
origin, executes complete emitted code at O0/O3 with pointer identity, and rejects
selected-owner escape through independent Rust compilation. Existing private view
controls and all ten normal-stack canaries plus Result remain covered. No timing
or full regression is claimed. Exact check logs reside in SQLite under the sparse
environment verification record
`runtime/results/native-view-sparse-environments-2026-10-10.json`
(74 private views, 107 source/depth checks, 16 native editor checks). Installed consumers remain at f569190a; the warm
CLI is rebuilt for project formatting. Public view APIs remain guarded.

## Continuation prompt

Continue gate 189 with abstract/stored callback families and mixed callback records,
including physical reference capture rejection and nested mutable carried views.
Retain actual source argument identity and typed native lifetime mapping. Finish
byte observer contracts and complete editor/catalog parity before publication.
Use focused TDD, bounded checks, project formatting and byte-exact SQLite evidence.
Refresh consumers before checks that use installed binaries. Full regressions and
matched measurements belong to S07 family completion.
