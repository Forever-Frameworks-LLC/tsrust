# B9 project editor (S04): acceptance map

Date: 2026-10-10. Ledger row: B9 in `docs/plans/compiler-completion-plan.md`; gate S04 in
`docs/plans/standard-library-roadmap.md`. User guide: `docs/guides/tooling/project-editor.md`.
History of the increments: `docs/milestones/modules-03-workspace-snapshots.md`.

Tests are in `crates/tsrust-lsp/tests/` unless a path is given. "Protocol" means the
in-memory stdio test in `crates/tsrust-lsp/src/main.rs`.

## Supported

| Requirement | Evidence |
| --- | --- |
| Unsaved imported buffers are checked; open text has priority over disk | `workspace.rs`, protocol |
| Dependent invalidation: an edit, close or reopen of a dependency rechecks its importers; unrelated edits do not | `workspace.rs`, `project_session.rs`, protocol |
| Stale results are not published or answered | `workspace.rs` (`stale_results_are_rejected_...`), `project_session.rs` (`stale_inputs_cannot_replace_publications_...`), `project_actions.rs` |
| Cross-file diagnostics with related locations, per-file versions, shared-file ownership across entries | `project_diagnostics.rs`, `project_session.rs` |
| Cross-file hover, completion, signature help, definitions, references; aliases; nested and private records; generic and callable contracts; native modules | `project_intelligence.rs` (59 tests) |
| Versioned quick fixes for open files, validated on the whole project | `project_actions.rs`, protocol |
| Quick fixes for closed files, offered only while the disk text equals the checked text | `project_actions.rs` (`closed_dependency_fix_...`, `session_offers_closed_dependency_fixes_...`) |
| Help with unfinished bodies, strings, templates, comments, signatures and types | `project_incomplete.rs`, `crates/tsrust-compiler/tests/project.rs` (`editor_header_loading_...`) |
| Help while an import is typed or names a missing export; other names keep their help | `project_incomplete.rs`, protocol |
| Completion of export names between import braces, local and native modules | `project_incomplete.rs` (`import_braces_complete_...`), protocol |
| A module buffer with unfinished text stays in project mode | `project_incomplete.rs` (`unfinished_text_keeps_a_module_buffer_in_project_mode`) |
| Disk creation, write, removal, rename; symlink removal and retarget; change before the watch event arrives | `project_session.rs` (`disk_dependency_rename_...`), `workspace.rs` |
| One file open under two URIs: independent versions, close restores the other buffer | `project_session.rs` (`open_aliases_keep_independent_versions_...`) |
| Navigation results use the URI the editor opened | `project_intelligence.rs` (`session_navigation_preserves_open_uri_aliases`) |
| Single-file behavior is unchanged | the other 37 editor suites |

## Defined limits

These are decided behaviors, not open work.

| Case | Behavior |
| --- | --- |
| Two open URIs of one file with different unsaved text | The most recent edit is the checked text for both |
| A closed-file edit and a disk change between offer and application | Not detected; the protocol has no precondition for disk text |
| Unfinished template or block comment | The rest of that line is ignored for import help |
| Malformed import path, missing module, duplicate declaration | No help for incomplete code; diagnostics report the error |
| Files that no open file imports | Not checked |

## Not part of S04

Symbol rename (`textDocument/rename`), default exports, export lists and re-exports are
not implemented and are not in the S04 contract. "Rename" in S04 means a file that is
renamed on disk.

## Cost

Debug build, Apple silicon, medians. A project of chained modules, each with one
function and one record type. `crates/tsrust-lsp/tests/project_scaling.rs` (opt-in).

| Modules | Check | Index (checked) | Index (incomplete) | Hover | Completion | References | Freshness |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 10 | 4.1 ms | 4.8 ms | 4.5 ms | 0.02 ms | 1.9 ms | 0.04 ms | 0.02 ms |
| 50 | 11.0 ms | 19.4 ms | 21.7 ms | 0.02 ms | 1.9 ms | 0.04 ms | 0.09 ms |
| 200 | 45.1 ms | 80.0 ms | 87.6 ms | 0.02 ms | 1.9 ms | 0.04 ms | 0.37 ms |

Analysis grows linearly with the module count and runs off the protocol loop. Request
cost does not depend on the module count. The freshness check reads the metadata of
each closed file. This is one observation, not a benchmark series.
