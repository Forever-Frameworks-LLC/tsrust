# Source documentation in IntelliSense

Source comments preserve authored line breaks and paragraphs in hover, completion
and signature help. Markdown code fences retain their contents and indentation.
JSDoc-style `@param` (also `@arg`/`@argument`), `@returns`/`@return`, `@throws`,
`@example` and `@deprecated` tags render as separate sections. Unknown tags remain
visible. Multiline parameter descriptions appear on the corresponding parameter;
legacy `{type}` and optional `[name=default]` spellings are accepted for matching.

Parameter and return types come from the compiler-backed signature. Comments are
optional descriptions, never a second type system. A plain comment is enough for
a summary; use tags only when individual parameter or return descriptions help.
Types written inside comment tags do not override checked types. No generated
prose claims to infer the purpose or behavior of a function from its implementation.

Example:

```tsrust
/**
 * Builds a greeting.
 * Keeps the supplied name unchanged.
 * @param name - Name shown in the greeting.
 * @returns Greeting text.
 */
function greeting(name: string): string {
  return "Hello " + name;
}
```

Decorator-aware IntelliSense is recorded in the compiler completion plan as a
future requirement, conditional on actual decorator language semantics. It must
use checked metadata rather than comment guesses.

Focused evidence: `tsrust-editor-docs-*` in SQLite. Checks cover local and imported
symbols, aliases, parameter help, completion, Markdown fences, CRLF, optional
parameter spellings and exact parameter-name matching. This editor change is
independent of the unfinished JSON acceptance milestone.
