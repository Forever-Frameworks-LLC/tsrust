# S04 increment: project input snapshots

Status: S04 is accepted at gate 185 on 2026-10-10. See [the gate record](185-project-editor.md) and [the acceptance map](../b9-project-editor-acceptance-map.md). The sections below are the history of the increments; each "remaining" list describes the state at that increment.

## Implemented contract

crates/tsrust-lsp/src/workspace.rs provides Workspace and WorkspaceSnapshot independently of transport and analysis scheduling. Open buffers retain version, generation and Arc-backed text. A snapshot shares immutable open text and lazily caches disk fallback inputs and path resolution. Source identities are canonical absolute paths for existing files and normalized absolute paths for new unsaved files. Input identity normalization is a host responsibility; URI decoding/integration remains pending.

SourceProvider reads open buffers before disk. The compiler project loader/checker remains the only authority for imports, exports, types and ownership. sources() returns exact loaded texts for subsequent UTF-16 conversion. No native backend is involved.

Workspace.current compares only observed source revisions, so unrelated edits do not invalidate a checked project. Changes reject stale document versions. Closing/reopening advances generation even when versions are reused. Attempted missing imports are observed too: opening a missing module invalidates the failed result. register accepts dependencies only for a current snapshot; affected reports registered entries depending on a changed source. remove_entry releases tracked entry dependencies.

invalidate_file is the boundary for disk writes/removal/rename notifications; it invalidates prior revisions while preserving open-buffer authority. No watcher is registered yet. Snapshots pin first loaded disk text and first path-resolution result; new snapshots see new disk state. This does not promise atomic filesystem snapshots across multiple initial reads or detect external disk changes without a host invalidation signal.

## Evidence

Six focused tests in crates/tsrust-lsp/tests/workspace.rs verify entirely unsaved project checking, versioned edits, close/reopen staleness, unrelated edit isolation, missing imports becoming available, file invalidation, dependency release, disk caching and open-buffer precedence. Four existing code-action tests still pass. Logs: target/editor/workspace-snapshot-verified.log and workspace-snapshot-final.log. Cargo uses target/editor with offline/locked dependencies. Clippy library check completes successfully (workspace-snapshot-clippy.log); Rustfmt applied to new source/test files. Resolution caching was added after the first passing test run and focused snapshot tests rerun.

No runtime performance claim or full-regression restart. This foundation adds no built-in methods/properties and does not change installed VSIX behavior.

## Next integration

Decode file URIs to canonical identities; capture project jobs with exact workspace snapshots; analyze off the protocol loop; register dependency observations and suppress results invalidated by any dependency. Map compiler project errors/related locations using loaded texts, clear old dependent diagnostics and retain per-file publication versions. Integrate disk watcher changes/removal/rename explicitly. Do not treat a dependency with no main as an independent executable entry. Project navigation/hover and whole-project versioned fixes follow those diagnostics contracts.

## Diagnostic conversion and scheduler hook increment

project_diagnostics.rs now converts source-indexed compiler project errors with loaded WorkspaceSnapshot texts into per-file LSP diagnostics. Primary and cross-file related ranges use each source's own UTF-16 LineIndex; no disk reread occurs. Every loaded file receives an empty bucket on success so a future publication layer can clear repaired errors. Invalid or missing spans fail conversion rather than inventing positions.

Local file URI handling covers percent escapes, spaces, Unicode, percent/query/fragment characters in filenames, localhost authorities and normalized new-file identities. Non-file/remote authorities, invalid escapes, null bytes and literal URI queries/fragments are rejected. Tested on the current native host; cross-platform/UNC support is not claimed.

Documents::reschedule queues dependency-driven work without changing source/version/generation, retains fair pending order and bounds repeated rechecks to one pending document. It invalidates the assistance index for future dependent metadata updates. Result freshness still requires both document and Workspace snapshot checks; reschedule alone does not suppress dependency-stale publication.

Focused verification: 4 diagnostic/queue tests, 6 snapshot tests and 3 library unit tests pass in target/editor/project-diagnostics-verified.log. Coverage includes real unsaved dependency compiler errors after emoji, cross-file related locations, invalid source ranges, URI roundtrips, empty repaired buckets and repeated dependency requeue fairness/version preservation. Clippy library check completed cleanly before the queue hook addition (project-diagnostics-clippy.log); no broader check is implied.

Protocol integration remains pending. These APIs are not called by the active stdio worker yet. Next: attach immutable workspace input to queued jobs, mirror accepted open/change/close events, requeue registered dependents, register completed input observations, suppress stale results and publish/clear per-file diagnostics with document versions. Watch notifications and imported-symbol assistance still follow.

## Live worker and publication integration

The stdio worker now receives a Job containing the document snapshot and optional immutable WorkspaceSnapshot. Real top-level module declarations are detected with the compiler lexer. Local file modules use compiler project analysis off the protocol loop; ordinary single-file and untitled buffers retain the existing checker path. Export/import library buffers without a top-level main are checked semantically through importing entries, rather than reporting a spurious standalone entry error.

Accepted open/change/close events update the project workspace. Registered dependent entries are rescheduled without version changes. A dependency change during the first analysis invalidates and reschedules the entry even before its graph has been registered. Completion requires both document currentness and workspace observed-source currentness. Shutdown/obsolete-check responsiveness remains intact.

project_session.rs owns dependency lifecycle and per-entry diagnostic contributions. Publications merge contributions across entries and deduplicate equal diagnostics, so one successful entry cannot erase another's shared-file errors. Replacing/closing a project or switching to a single-file source releases its prior contributions. Loaded source identities map back to original open URIs and publication versions; closed/disk-only files have no invented version.

VS Code now synchronizes a disposed-with-extension **/*.tsr watcher. workspace/didChangeWatchedFiles invalidates observed identities and reschedules dependents for creation/write/removal; rename is represented by removal/creation. Open buffers remain authoritative. Canonical symlink alias identity changes and URI aliasing need further boundary coverage before claiming complete rename semantics.

Focused evidence: 3 binary protocol/scheduler tests, 4 existing code-action tests, 4 diagnostic/queue tests, 2 project-session tests and 6 workspace tests pass in target/editor/project-live-verified.log. The new real in-memory protocol exercise opens unsaved modules, changes dependency semantics, sees versioned dependency errors, repairs them, closes/reopens a dependency at the same version and sees importing entry errors clear; clean shutdown is verified. Session tests verify shared-file contribution ownership and stale/watch invalidation. npm test passes TypeScript compilation and the real TextMate test. Library/binary Clippy completes cleanly after simplifying a newly flagged boolean expression (project-live-clippy-final.log; original warning retained in project-live-clippy.log). Earlier syntax error in the extracted module-detection helper was fixed before any tests/build passed.

S04 remains in progress. Imported hover/completion/signature/navigation, project versioned quick fixes and installed extension end-to-end validation remain pending. No VSIX or global CLI installation happened in this increment; current source/development builds provide the worker integration. No performance measurement or full-regression restart.

## Imported declared-source assistance increment

project_intelligence.rs builds per-module SourceIndex assistance from a successfully checked graph. Import/export visibility and target identities come from the compiler loader. Imported function/type declarations supply alias-aware hover, completion, signature help and cross-file definitions. Primitive record fields follow imported type aliases; local lexical shadowing keeps local definitions. Export-adjacent user documentation is preserved. Imported main never receives automatic-entry help.

The worker builds this assistance alongside successful project checking, then ProjectSession caches it with the exact observed workspace snapshot. Request handling reuses the cache and immediately rejects it after dependency revisions change; no project compilation is added to hover/completion requests. Dependency declaration indices are built once per module and reused across imports; initializers refresh once after all imports attach.

Imported function return inference maps visible imported nominal types to their local aliases. Unmapped nominal/complex return types remain unknown rather than binding to an unrelated same-named local type. Primitive field assistance is implemented; nested private record graphs, richer canonical type/projection metadata and cross-file references remain pending. This is declared-source assistance gated by a checked module graph, not a general compiler semantic symbol API. Failed/incomplete project checks fall back to existing single-file tolerant assistance rather than reuse stale project types.

Evidence: four focused project_intelligence tests pass (project-imported-boundaries.log); three protocol/scheduler tests plus four project intelligence tests pass after batching initializer refresh (project-imported-cached-final.log). Existing 34 IntelliSense tests passed against the changed import/type mapping (project-imported-final.log). Protocol exercise now requests imported hover after a checked project is published. Tests cover alias docs/signature/completion/definitions, imported primitive record fields, local shadowing, imported main, cache expiration and same-name nominal return isolation. Clippy completes cleanly after repairing a newly flagged nested conditional (project-imported-clippy-final.log; original warning retained).

S04 remaining: complete nested/project semantic metadata and cross-file reference support, conservative incomplete-source project assistance, whole-project versioned quick fixes, canonical alias/rename boundaries and installed extension end-to-end validation. No VSIX/CLI reinstall or performance measurement occurred.

## Cross-file references increment

ProjectIndex now builds a reference table once from the checked graph's source indices. Declaration URI/range identities connect named imports, aliases and uses across loaded modules. Existing lexical resolution preserves local shadowing. Find References works from either an imported alias or its original declaration; includeDeclaration=false removes the actual declaration while retaining import binding occurrences. Both the exported name and local alias in an aliased import are reference locations. ProjectSession routes reference requests through its current-snapshot cache, with existing stale-dependency rejection.

Focused evidence: 3 binary protocol/scheduler tests, 34 existing IntelliSense tests, 5 project intelligence tests and 2 project session tests pass in target/editor/project-references-focused.log. The new fixture exercises two importing modules, original and renamed bindings, definition-origin requests, declaration filtering and a shadowed block-local value. No new built-in API was added. This remains declared-source assistance over the current entry's loaded graph: unrelated project roots and nested private record metadata are not included. No runtime benchmark comparison is meaningful for this editor-only behavior; editor indexing cost measurements remain pending at the checkpoint.

S04 remains open for nested/project semantic metadata, incomplete-source assistance, versioned project fixes, canonical alias/rename boundary coverage and installed extension validation. No extension installation or full regression restart occurred.

Clippy passes with warnings denied for the LSP library, binary and project intelligence tests (target/editor/project-references-clippy.log).

## Module syntax highlighting

The VS Code TextMate grammar highlights import/export declarations, contextual from/as keywords in named imports, and both single- and double-quoted module paths. Import state spans multiple lines and ends before the terminating semicolon; from/as outside imports remain ordinary identifiers. Strings and comments retain their own scopes. Supported syntax remains named imports with optional local aliases and direct export function/export type declarations; default exports, export lists and re-exports are not implemented by this change.

Focused verification: npm test passes extension compilation and real TextMate tokenization, including multiline aliased imports, export declarations, quoted paths, comments/strings and import-state termination. No compiler behavior or built-in API changed. The installed extension was not replaced.

## Versioned project quick fixes increment

project_actions.rs validates compiler-suggested edits against pinned loaded sources using an overlay SourceProvider. Validation checks the entire importing entry graph rather than a standalone file. Successive transfer diagnostics can accumulate up to eight non-overlapping clone edits, with follow-up coordinates mapped back to each original file. A candidate is offered only when the edited graph checks successfully. Invalid spans, unpinned files and unrelated remaining errors suppress the candidate.

The analysis worker computes actions alongside diagnostics. ProjectSession caches them with the observed snapshot, merges duplicate actions across entries, maps canonical targets back to open URIs and rejects stale dependency inputs. Requests filter by the diagnostic's source/range and requested quickfix kind without compiling on the protocol loop. Closing or leaving project mode releases cached actions. Edits carry the target open document's version. Closed-file edits are deliberately withheld pending a disk-content application guard; validation covers the loaded entry graph, not unrelated roots.

Evidence: 3 live protocol/scheduler tests, 4 existing single-file action tests, 2 project-action tests and 2 project-session tests pass in target/editor/project-actions-focused.log. The live protocol fixture requests a dependency semicolon repair through the real worker and checks its URI/version/replacement. Project tests cover whole-graph rejection, UTF-16 source inputs, repeated clone transfers, per-file versions, diagnostic ownership and action-kind filtering. The additional watched-dependency invalidation assertion passes in project-actions-stale.log. Clippy passes with warnings denied for the library, binary and project-action tests (project-actions-clippy.log).

S04 remains open: nested/project semantic metadata, incomplete-source assistance, disk-only edit guards, canonical alias/rename boundaries and installed extension validation. These checks are focused acceptance evidence; performance/checkpoint coverage remains pending. No new built-in API, global CLI refresh, extension installation or full regression restart occurred.

## Installed highlighting and import formatting repair

The user's screenshots exposed two separate gaps: the installed VSIX still contained the older grammar, and the formatter treated named import braces as statement blocks, inserting a blank line before from. formatting.rs now distinguishes import layout: inline bindings remain inline, multiline bindings retain their line structure without artificial blank lines, and the closing brace joins from/path. Comments and token spelling remain intact. Blank separation before exported declarations is handled consistently. The diamond left.tsr fixture shown in the screenshot was repaired with the updated formatter.

Six formatter tests pass, including exact screenshot-layout regressions and idempotence (module-formatting.log). Compiler library/formatter Clippy passes with warnings denied (module-formatting-clippy.log). Both editor compiler and LSP development binaries were rebuilt (module-editor-build.log). The extension package built successfully (module-highlight-package.log), code --install-extension completed successfully, and the installed grammar was inspected to confirm the new module scopes. A VS Code window reload is still needed to activate its updated grammar/server. This confirms installation, not a rendered-theme or full Extension Host end-to-end check.

## Installed module Extension Host acceptance

editors/vscode/src/test/modules.ts isolates module checks inside the existing temporary-profile suite. It uses the packaged VSIX and the rebuilt editor LSP/compiler: imported alias documentation hover, cross-file definitions/references, unsaved dependency syntax errors, requesting/applying a whole-project semicolon fix, diagnostic repair, import formatting, CLI checking and native module execution all pass. Existing editor lifecycle, ownership, clone fixes, formatting/save, single-file navigation and command checks also pass (target/editor/module-installed-e2e-final.log, process exit 0). The test does not assert rendered theme colors; real TextMate scope coverage and installed grammar inspection remain the highlighting evidence.

The first successful host launch exposed canonical /private/var versus original /var URI navigation results (module-installed-e2e.log). ProjectSession now maps definition/reference locations back to open document URIs. A focused localhost URI-alias test covers this mapping without platform-specific paths. Three binary protocol tests, six project intelligence tests and two project session tests pass (module-uri-focused.log); library/binary Clippy passes with warnings denied (module-uri-clippy.log). Earlier launcher attempts used an obsolete Electron executable name and encountered sandbox SIGABRT; the installed Code executable and approved temporary-profile launch resolved those launch issues.

S04 still needs richer nested type metadata, incomplete-project assistance, guarded closed-file edits and comprehensive alias/rename lifecycle coverage. Installed module execution/fixes/navigation/formatting are now verified; no broad performance/checkpoint completion is implied.

## Nested declared record fields

Imported records now traverse their defining module's declared record fields with an iterative worklist and visited identities. Primitive, record, array and nullable component types map to visible aliases or internal module-qualified identities. Private records supply field metadata without adding private names to lexical completion. Module-qualified owners prevent a same-named local record from capturing nested fields. Existing foreign definition locations carry through nested fields into navigation and project references. Repeated imported field metadata is deduplicated by owner/name.

Focused evidence: seven project intelligence tests and 34 existing IntelliSense tests pass (project-nested-focused.log). The new checked-project fixture verifies nested private Address fields and indexed Address arrays, city completion/definition/reference locations, and isolation from a local Address with a wrong field. This is declared-source record metadata: records themselves imported by the dependency from another module, complex callback type graphs and incomplete-project analysis still need additional work. No new built-in API or runtime behavior changed; full checkpoint measurements remain pending.

Library, binary and project intelligence Clippy passes with warnings denied (project-nested-clippy.log). The configured development LSP binary was rebuilt (project-nested-build.log); restarting the language server loads this increment.

## Nested records across dependency chains

ProjectIndex now builds enriched declaration indices in dependency-first graph order. Importing modules reuse each dependency's already resolved field metadata, including internal private record owners. Copying foreign field metadata preserves its original declaration location instead of substituting an intermediate import binding's span. Visible aliases and internal module identities remain separate from unrelated local types.

Focused evidence: eight project intelligence tests, 34 existing IntelliSense tests and three binary protocol tests pass (project-transitive-focused.log). The new three-module checked fixture imports Address as Location into an exported User record, then imports User as Person into the entry. Both city and nested private Geo.code supply completion, original leaf-file definitions and project references despite a same-named local Address. Callback/complex type graphs and incomplete-project assistance remain pending; no new compiler syntax, built-in API or runtime behavior was added. Earlier compiler type-inference failure for the declaration map was fixed before the focused tests passed.

Library/binary/project intelligence Clippy passes with warnings denied (project-transitive-clippy.log); the development server is rebuilt (project-transitive-build.log). Checkpoint measurements and installed-host verification of this particular metadata increment remain pending.

## Current-source assistance during incomplete bodies

The compiler exposes load_project_headers for editor metadata only. It shares module path/alias/export visibility/cycle/depth handling with the strict loader and checks lexical top-level declaration/import-name conflicts. It retains exact sources and spans while allowing declaration bodies to be incomplete. It does not prove syntax/type validity. Compilation and CLI behavior continue to use strict load_project and check_graph.

When normal project checking fails, the worker can build declared-source assistance from current header-valid inputs. Diagnostics still report the strict compiler failure, and quick fixes still require full graph validation. No previous successful index is reused across dependency changes. Malformed module headers, removed exports, missing imports and lexer failures suppress this fallback; unterminated strings/comments and complex signature recovery remain limitations. Dependency bodies may be incomplete too, so assistance describes declarations rather than asserting their bodies compile.

Evidence: 20 compiler project tests pass (project-tolerant-compiler.log), including strict-versus-header loading and private/alias/duplicate-binding boundaries. Nine project intelligence and 34 existing IntelliSense tests pass (project-tolerant-editor.log), including imported field completion and documented function hover/definition with an unfinished entry body and removal of an export in the current dependency. Three real protocol/scheduler tests pass (project-tolerant-protocol.log); the live project test now requests imported completion after an incomplete edit while diagnostics remain active. Compiler/LSP library and LSP binary Clippy passes with warnings denied (project-tolerant-clippy.log). The new fixture's initial in-memory path mismatch was corrected before the final compiler pass.

S04 remains open for complex signature recovery, guarded disk-only edits and remaining file identity/rename boundaries. No new built-in API, full checkpoint run or performance measurement occurred.

## Disk rename and symlink lifecycle

Workspace resolution now observes normalized import paths alongside canonical targets. Watch handling invalidates both identities and deduplicates dependent entries. This repairs removal/retarget of a symlink: once the link disappears, the event can no longer canonicalize to its old target, but its observed normalized path still invalidates old project snapshots and cached assistance. source_path_id exposes the existing local URI normalization independently of filesystem canonicalization; source_id retains canonical behavior.

Focused evidence: 25 tests pass across the binary protocol, project session, diagnostics, workspace and intelligence suites (project-lifecycle-focused.log). A new Unix real-filesystem fixture removes a symlink, checks stale navigation rejection and a missing-import failure, recreates it pointing to another module and verifies navigation to the replacement, then renames an ordinary dependency and updates the importing buffer to restore successful checking. Temporary files are cleaned up. Existing unsaved precedence, alias URI conversion, close/reopen and version freshness tests remain passing.

This establishes watched dependency lifecycle behavior. Multiple simultaneously open URI aliases for one canonical buffer, open-buffer symlink retarget semantics and guarded closed-file edits remain unresolved; no complete file-identity acceptance is claimed. No built-in API or compiler execution behavior changed.

Library/binary/project-session Clippy passes with warnings denied (project-lifecycle-clippy.log); the development LSP is rebuilt (project-lifecycle-build.log). No full regression or checkpoint measurement was restarted.

## Closed dependency freshness before watcher delivery

WorkspaceSnapshot records metadata stamps for closed loaded sources and closed resolution paths, including missing paths. Workspace.current checks those stamps alongside observed editor revisions. Delayed watch delivery therefore cannot preserve cached assistance after ordinary disk writes/removal/creation/replacement. Stamps use length and modification time plus device/inode/change time on Unix. Reads compare pre/post stamps and reject changes during reading. Snapshot text remains immutable; freshness rejection does not silently replace its contents. Open buffers remain authoritative and do not depend on disk stamps for their content.

Fourteen focused tests pass across workspace, session, project actions and real protocol suites (project-disk-freshness.log). The real disk fixture now checks invalidation before watch notification, missing-file creation, atomic same-length replacement and continued open-buffer authority after disk removal. Library/binary Clippy passes with warnings denied (project-disk-clippy.log); the server is rebuilt (project-disk-build.log).

This uses metadata checks without rereading contents on requests. It is not a content-hash or atomic filesystem guarantee, and adds metadata work proportional to observed closed paths; indexing/request measurements remain pending. Closed-file edits remain withheld because request-time freshness alone cannot protect application after a later disk change. Multiple open aliases and open-buffer retarget semantics remain separate S04 work.

## Independent open-alias versions and close restoration

ProjectSession retains each URI buffer's canonical identity, snapshot and accepted-event sequence. Changes compare versions against that URI's previous snapshot, rather than another alias's unrelated version. The most recently accepted open/change supplies the shared canonical workspace input. Closing one alias restores the most recently updated remaining alias, keeps its unsaved contents/version and reschedules dependents; only closing the last alias falls back to disk. This also pins change/close identity to the original open event instead of re-canonicalizing a potentially changed path.

Eighteen focused tests pass across project session, actions, intelligence and binary protocol suites (project-alias-focused.log). The new two-URI fixture exercises unequal version numbers, a lower-version valid update, same-version rejection, closing the active alias, restored unsaved source and final close. Library/binary/project-session Clippy passes with warnings denied (project-alias-clippy.log); the development server is rebuilt (project-alias-build.log).

This repairs alias version and close ownership. Divergent alias views still share one canonical analysis authority; fully independent assistance/diagnostic routing for those views and open-symlink retarget entry dispatch need further acceptance. No complete multi-alias gate closure is claimed. Guarded closed-file edits and metadata/request performance measurements remain pending.
