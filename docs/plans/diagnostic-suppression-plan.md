# Diagnostic suppression directives

Status: focused implementation accepted, prioritized before further S10 work.
Reporting filters and next-node ranges are implemented;
installed consumers are refreshed from `c244202c`. See the
[usage guide](../guides/diagnostic-suppression.md).

The owner approved `// @tsrust ignore - <reason>` applying to the next statement
or declaration, including multiline code. Require a nonempty reason. Suppress
reported diagnostics whose primary location belongs to that node, without
suppressing subsequent siblings or unrelated files. A directive inside a string,
template literal, or block comment must have no effect. Formatting must preserve
the directive and its attachment.

Apply suppression only when presenting `tsrust check` and LSP diagnostics,
including project/workspace scans and unsaved editor buffers. Use the exact source
snapshot associated with each diagnostic. Both errors and future warnings follow
the same policy. Suppression does not repair analysis or create a checked tree.
Strict checking, emit, build, run, test, and dev compilation retain errors and
reject invalid programs. No source rewriting, implicit cast, or unsafe bypass.

Implementation order:

1. Add focused failures proving check/editor suppression and strict build rejection.
2. Centralize comment recognition and next-node attachment in the compiler.
3. Filter at CLI/LSP reporting boundaries, preserving strict project diagnostics.
4. Cover multiline declarations, sibling errors, module imports, unsaved buffers,
   misleading string/comment contents, syntax recovery, required reasons, and
   formatted-source attachment. Document the directive and its limits.
5. Run only focused affected compiler/editor/native checks; preserve the completed
   source-organization regression evidence. Checkpoint and refresh installed
   consumers so the owner can use the directive in VS Code.

No file-wide suppression is implied. Do not add directives to owner files or hide
the current workspace problems automatically; usage remains explicit in source.

## Focused evidence

Six compiler directive tests and two LSP directive tests pass. They cover strict
emit/build rejection, node boundaries, multiline declarations, formatting,
non-directive literals/comments, required reasons, unsaved project sources,
imported errors, Unicode/CRLF, and lexical recovery boundaries. Ten existing CLI
project-diagnostic tests and 21 LSP project/workspace tests pass. Whole-project
formatting changes no files or source fingerprints.

SQLite preserves original compiler/editor failures and focused successful runs
under `tsrust-ignore-*`, including `tsrust-ignore-final-focused-2.log`,
`tsrust-ignore-check-regression.log`, `tsrust-ignore-editor-regression.log`, and
`tsrust-ignore-format.log`. No full regression or timing rerun was needed.
Installed CLI checks confirm suppressed `check`, rejected `build`, and JSON
namespace execution outside the repository. Installed LSP protocol checks confirm
an ignored unsaved diagnostic at version 1 and its restoration after removing the
directive at version 2. The active VS Code server was restarted: its 14 stale
Tsrust errors cleared, leaving six Rust dead-code warnings. No owner source was
annotated automatically. Installation, protocol output, and UI observations are
retained under `tsrust-ignore-consumer-*` in SQLite.
