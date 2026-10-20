# Paused for memory pressure — 2026-10-20

The user explicitly requested a break, process shutdown, temporary-build cleanup
and a prompt for another agent. Do not resume implementation in this chat. The
next agent must investigate memory pressure before restarting the normal editor
server, dev preview or broad checks. Earlier “continue” instructions do not revoke
this pause.

## State and cleanup

Source checkpoints are `c49c4bc4` (portable Table implementation) and `1085b9c8`
(SDK helper bundling and inherited property documentation repairs). Table is not
accepted. The last repair build was interrupted by the user's request; it is not
a successful build or test result. No candidate Table CLI/LSP was installed.

Stopped the dev server, configured editor LSP, Cargo and its rustc child. A second
process inspection found no remaining tsrust, tsrust-lsp, Cargo or rustc process.
The editor extension uses DoNotRestart on server closure. Workspace settings were
not modified: SHA-256
`7cc52a46502c43143e51d9ee30149a2f18acddd2df201d34ae22c6cb5bb1f3d6`,
mtime `1792362521472639942` ns. Browser automation was reset; its preview-tab close
could not be confirmed because the webview was unavailable. Port 4317 is stopped.

Removed 30,725,697,536 allocated bytes of generated outputs: the shared
`/private/tmp/tsrust-gate100-editor-target`, `docs/target`, `target/editor`, the
standalone counter's `target`, the old temporary site and abandoned Tsrust test
workspaces. Source, installed tools, Rust toolchains/registry, SQLite evidence and
`/private/tmp/tsrust-wasm-tools` are preserved. The configured editor LSP path now
points to the removed temporary copy; rebuild it only after the memory issue is
under control. Installed `~/.cargo/bin/tsrust` and `tsrust-lsp` remain the earlier
accepted latency/Kbd consumers, not the Table candidates. Do not mistake them for
current source builds.

The six unrelated untracked owner plans remain untouched:
`ui-b4-compiler-repair-first-task.md`, `ui-identity-enforcement-first-task.md`,
`ui-lifetime-contracts-resource-observation-first-task.md`,
`ui-resource-release-first-task.md`, `ui-state-lifecycle-first-task.md` and
`ui-tsrust-owner-handles-first-task.md` under `docs/plans`.

## First priority: memory

The user reports extreme long-lived LSP memory and swap usage. This is unresolved.
The earlier fresh standalone scan observation (482 MB peak RSS) does not establish
long-lived editor memory safety. The shutdown snapshot's RSS likewise does not
measure swapped/compressed private memory. Do not dismiss the report using RSS.

Inspect retention, invalidation, workspace/project caches, repeated edits,
interrupted scans and foreground/background analysis lifetimes before a bounded
reproduction. Track physical footprint and swapped/compressed memory as well as
RSS, process count and pending work. Use one guarded process with a time/memory
stop condition. Do not immediately run another full workspace scan or restart all
consumers. Avoid arbitrary cache-size caps as a substitute for finding retention.

## Accepted evidence to reuse

`docs/milestones/compiler-dev-and-workspace-latency.md` records the earlier repair:
warm docs rebuild 14,870 ms rather than roughly 105 seconds; the runtime-support
scan change preserved emitted Rust bytes. Cold/warm standalone LSP scans were
133,894.5/1,068.0 ms and foreground hover 238.8 ms with zero counters. These are
observations, not a guarantee about the reported memory regression.

Kbd 2 and the Markdown frame repair pass through `44c9fa3b`; storage closure is
`ff9dbd70`. See `docs/milestones/ui-keycaps-and-markdown-frames.md`. The last live
preview was Kbd, not Table. Reuse responsive navigation, themes, ownership,
window/input and prior service evidence. S18's three SQL drivers already passed;
rerun their explicit runner only for relevant SQL/task-lifecycle changes.

SQLite `benchmarks/records.sqlite` is canonical. Sixteen Table records totaling
28,015 original bytes were independently restored, hashed, verified and removed
from loose storage before the pause. They include pinned upstream source,
passing model checks and original failures. Twelve additional pause/build/cleanup records totaling 21,612 original bytes
were independently verified and stored there, including
`tsrust-pause-evidence-closure.json`. Final inspection confirmed zero Tsrust/Rust
build processes and no listener on port 4317. Do not recreate archives or source/binary snapshots.

## Table section in progress

Read `docs/plans/ui-table-section.md` and `docs/milestones/ui-table-layout.md`.
The eight actual exports, tags and slots are implemented in the editable offline
Table 1.0.0 bundle, requiring compiler API 11. Source is pinned to shadcn Base UI
+ Nova commit `daaf156ee00ed91cc7e6bdd937c1b4973152d141`, Table blob
`160cf7aa14b8467a777eb2405053d2d968f641b9`.

Native shared columns, pixel preferences, spans, captions, footer/header roles,
selected-row metadata and bounded Nova selectors have model coverage. Initial
20 checks pass; focused repair checks pass for constrained widths, hidden rows,
nested-table rejection and malformed rows/groups. Preserve those results.
Nested portable tables reject explicitly; rowSpan, percentage cell widths,
complete CSS table border models, unrestricted DOM events/refs and motion remain
unsupported. The reference page states these limits.

The generated-native gate found an owned scope string read after transfer; source
now reads config.scope. It then found an omitted module in the flattened SDK;
`stdlib/ui/mod.rs` now embeds both table helpers. The real native-renderer package
witness is added but unrun. LSP correctly reported colSpan's type but lost its
inherited documentation. Source now enriches checked synthetic fields during
imports/inheritance, preserving explicit local documentation and checked types.
That repair and its multi-import/generic/override regression are unverified.
Do not weaken the hover assertion to accept missing documentation.

After memory is safe, rebuild once with one Cargo target, CARGO_INCREMENTAL=0 and
one build job. The old shared target was deliberately removed. Compile the two
ui_content integration targets with --no-run, then execute exact witnesses:

Compiler:
- current_table_exports_keep_spans_scope_and_owned_children
- table_native_renderer_bundle_inlines_its_layout_helpers
- table_documentation_keeps_formatted_paired_examples_executable

LSP:
- table_props_keep_hover_completion_and_reject_unimplemented_row_spans
- inherited_interface_docs_arrive_after_imports_without_replacing_own_docs

Use --exact. The substring table_ also matches editable_ and accidentally selects
an unrelated old fixture. Do not rerun passing matrices because of checkpoint
numbers. Repair failures and rerun affected checks only.

The docs examples are `docs/src/examples/table-layouts.tsrx`, reference
`docs/content/ui/components/table.md`, and native acceptance
`docs/src/acceptance/table-documentation.tsr`. Register the real Table installation
in the docs lock through the offline installer; the current new source file was
written from the bundle and has no Table lock entry yet. Inspect placement rules
and preserve existing source/imports. Regenerate docs content, refresh verified
consumers/editor, and restart the real tsrust dev preview only when appropriate.
Verify actual web selection/density, retained identities, column/span geometry,
caption placement, narrow overflow, both palettes and source preview/copy. Reuse
native OS-window evidence; the new packaging witness builds without opening a
second window.

## Continuation

Use the requested caveman and ponytail skills, plus the existing design skills
when editing visuals. Read AGENTS.md and docs/development-workflow.md. Format the
entire project before source checkpoints and preserve owner files. Keep one writer
and build owner. Store verified closed records in SQLite, then remove loose copies.

After memory and Table acceptance, continue the actual Base UI + Nova component
library and paired web/native docs, including missing composition/focus foundations,
charts, themes and blocks. Match actual APIs rather than inventing similar props.
Keep application logic in project files and ownership machinery private. Batch
focused checks after coherent sections; do not start another full regression or
benchmark pass for each component. Follow the latest handoff over historical
“next” statements. The user will resume through a new chat when ready.
