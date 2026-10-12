# Compiler source organization acceptance

Status: accepted on macOS. This is a structural checkpoint, not S10 family or
performance acceptance. Continue the JSON roadmap after this checkpoint.

Compiler sources now use feature-owned module directories; 14 shared Rust files
remain at the source root. Public imports are preserved through the library
facade. CLI entry, embedded runtime paths, native test includes, and Unicode
source generation follow the new layout. See the
[source-layout guide](../guides/compiler-source-layout.md).

## Verification

- Compiler: 3,286 passed, three existing ignored cases.
- LSP: 554 passed across unit and all 44 integration targets; no unresolved failures.
- Both package doc-test commands pass; neither currently contains doc-tests.
- All 153 final-batch tracked relocations were compared against the prior Git
  checkpoint. Runtime bodies retain exact bytes or only embedded-path edits;
  facade/module declarations are the explicit structural additions.
- Whole-project formatting changes zero files and zero benchmark fingerprints.
  Compiler formatting tests cover token preservation, idempotence, and behavior.
- Native tests include offline package export, relocation, build, and execution.

One full compiler/LSP pass followed the moves. It exposed two existing JSON
catalog harness omissions: JSON family coverage and dependency linking for native
examples. The resumed LSP pass found one stale alias diagnostic assertion.
Repairs add coverage, use automatic CLI dependency linking, and require the
specific unresolved-alias message. Only failed witnesses were rerun; completed
results were preserved and unrun targets resumed. No diagnostic or output assertion
was removed to obtain acceptance.

The example runner now owns disposable output directories. Removed 293 obsolete
catalog files totaling 124,848,671 bytes. No installed consumer was changed by
this checkpoint; the later JSON namespace still awaits the planned refresh.

## Canonical evidence

SQLite retains the exact original run and failure output, resume inventory,
relocation maps, preservation proofs, focused repairs, and final counts:

- `tsrust-final-organization-regressions.log`
- `tsrust-final-organization-run-summary.json`
- `tsrust-final-organization-preservation.json`
- `tsrust-organization-catalog-witness.log`
- `tsrust-organization-catalog-examples.log`
- `tsrust-organization-header-witness.log`
- `tsrust-organization-lsp-resume.log`
- `tsrust-organization-lsp-resume-2.log`
- `tsrust-organization-final-checkpoint-format.log`
- `tsrust-organization-output-cleanup.json`
- `tsrust-organization-acceptance.json`

No timing run was performed. S10 dynamic child access, direct typed runtime
encoding/decoding, borrowed adapters, public writers, allocation baselines, and
family measurements remain outstanding.
