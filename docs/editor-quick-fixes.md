# Compiler-backed editor quick fixes

The editor offers **Insert missing ;**, **Insert missing )**, **Insert missing ]** and **Insert missing }** for parser diagnostics with an exact insertion location. Place the cursor on the diagnostic and use VS Code's lightbulb or **Cmd+.**. A fix preserves intervening whitespace/comments and edits the unsaved document.

## Plan and implementation

1. The compiler's public Diagnostic carries SuggestedFix entries: title, half-open UTF-8 source span and replacement text. Parser expectations supply punctuation insertion candidates at the end of the last accepted token, including a missing block brace at EOF. Existing CLI diagnostic JSON remains unchanged.
2. The server advertises quickfix code actions and handles textDocument/codeAction against the current open document snapshot. It computes current diagnostics itself, respects the requested range/kind, converts edits to UTF-16 and associates each action with its diagnostic.
3. Before offering an edit, apply it to a source copy and require the complete document to pass strict compiler checking. Return a versioned WorkspaceEdit using documentChanges. Stale client diagnostics cannot manufacture a fix after repair; document versions prevent edits being applied to a later version.
4. The existing VS Code language client supplies standard lightbulb/Cmd+. discovery and edit application. Verify the actual packaged client on unsaved Unicode source, preserve comments and confirm diagnostics clear after applying the action.

This is deliberately conservative: fixes are withheld when unrelated errors remain, when an edit does not pass checking, or when the request does not intersect the current diagnostic. There is no fix-all, speculative type conversion or method-name replacement. Clone fixes are now available for transferred strings, arrays and records: the earlier transfer is replaced with clone(value) only after the edited document passes checking. Their title states that copying may allocate, and they are not marked preferred. Callable consumption and partial-field repairs remain unsupported. Checking valid syntax cannot prove user intent; punctuation changes are always explicitly selected by the user.

## Future compiler handoffs

Method spelling fixes need the compiler's receiver-specific supported-member list and an unambiguous candidate before supplying a replacement. Argument/type fixes need a checked conversion contract and clear behavioral consequences. Ownership fixes require proof of valid access plus explicit allocation/mutation consequences in the action title or explanation. Use the same SuggestedFix boundary and current-version validation; do not infer semantic repairs by matching diagnostic prose.

This editor feature has no generated runtime operation to benchmark. Focused compiler/parser and editor/protocol/E2E checks apply; the scheduled full checkpoint remains gate 86.

## Acceptance evidence

Two quick-fix adapter groups and three framed-protocol tests pass, including all four insertions, Unicode/CRLF/comments, versioned edits, stale context, range/kind filtering and withholding invalid or incomplete repairs. The packaged extension E2E applies a semicolon quick fix to unsaved Unicode source, preserves its trailing comment and observes the diagnostic clearing. Extension TypeScript/grammar checks and packaging pass. Twenty-one focused compiler tests across operators, assignment_values and quoted_strings pass; LSP all-target Clippy passes.

A broader editor run currently has three failures in existing nullable-union catalog/inference/example fixtures: ArrayConstructor.isArray examples, Array.toString/String examples and nullable_union_help_and_observations_preserve_whole_value_types. They contain union annotations that the current compiler parser rejects with Expected "=". The new quick-fix fixtures pass independently; union compiler/editor progression remains separate work and these failures are not represented as passed acceptance.

Successive ownership transfers can now produce one combined action, such as **Clone before 2 transfers (copies data; may allocate)**. Follow-up clone candidates come from the compiler, are mapped back to the original snapshot, and are grouped only when the complete edited file passes checking. Discovery is bounded to eight transfers; unrelated errors and unsupported moves still suppress the action. The two-transfer user example is a focused regression fixture.

The exact successive-transfer fixture passes alongside all four code-action groups. Installed-VSIX E2E also requests and applies a combined clone fix and verifies ownership diagnostics clear; debug/release server builds and workspace formatting pass. Existing concurrent union Clippy warnings remain the only suppressed warning classes in the focused check.
