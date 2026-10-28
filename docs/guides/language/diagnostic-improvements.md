# Diagnostic improvement plan

Explain the failed language rule, identify the relevant source location, and give a concrete repair. Preserve diagnostic codes and ownership provenance. Offer automatic edits only when the language server verifies the repaired source.

## Gate D1: arrow syntax and inference

- Implemented: a typed arrow missing its opening parenthesis points at the colon, names the missing delimiter, relates the parameter location, and offers an insertion fix.
- Implemented: a parameter without a contextual type explains that inference failed and suggests an annotation or a typed callback context.
- Compiler acceptance: contextual_arrows exercises the original malformed example, applies its edit, and verifies native uppercase output; existing shorthand, context, capture and formatting coverage stays in place.
- Editor acceptance: code_actions verifies the descriptive diagnostic and the versioned insertion action.
- Runtime performance measurements do not apply: this changes rejected-source diagnostics and documentation.

## Gate D2: parser expectations

Inventory delimiter, parameter, argument, import/export and block parsing failures. Replace ambiguous expected-token messages with the construct being parsed and the missing or unexpected token. Distinguish missing delimiters from unsupported syntax. Test malformed examples beside valid near-neighbors, including ternaries and nested arrows; ensure no valid expression is rejected by recovery lookahead. Preserve source-depth limits.

## Gate D3: type and inference failures

Prioritize callback parameter/result inference, incompatible callback returns, argument count and argument type errors. Explain expected and actual types, the argument position/name, and the declaration supplying the expectation. Give explicit annotation guidance only where context truly cannot determine a type. Verify representative compiler diagnostics and LSP ranges.

## Gate D4: names, modules and ownership

Add declaration-related locations for unresolved imports/exports, unavailable names and invalid transfers. Keep ownership transfer and borrow provenance visible. Offer spelling/import/clone repairs only when their semantics are supported and the candidate edit passes validation. Test false-positive candidates and multi-module locations.

## Gate D5: acceptance

Run affected diagnostic, ownership, project and code-action suites; verify terminal rendering and editor presentation with representative examples. Record focused evidence separately from the compiler's scheduled five-gate regression/performance checkpoint. Do not restart an existing checkpoint or claim unmeasured performance.

## Evidence

D1 compiler evidence: target/editor/arrow-diagnostic-focused.log (four tests passed, including native repaired execution). Editor and binary-build evidence: target/editor/arrow-diagnostic-editor.log and target/editor/arrow-diagnostic-build.log. Gates D2–D5 remain pending; D1 is a focused improvement, not complete diagnostic coverage.
