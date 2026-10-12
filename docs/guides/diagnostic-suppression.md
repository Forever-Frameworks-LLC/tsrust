# Suppressing reported diagnostics

Place a directive before a statement or declaration in a `.tsr` file:

```typescript
function main() {
  // @tsrust ignore - intentional negative fixture for ownership checks
  const value: int = "wrong";
}
```

`tsrust check` and the language server hide diagnostics whose primary location
falls within that next statement or declaration. Multiline code and nested bodies
belong to the same node. Other statements and files remain checked. A directive
before an import does not hide an error located in the imported file; place the
directive beside the offending declaration in that file instead.

A nonempty reason after `-` is required. Incomplete directives are ordinary
comments and suppress nothing. Blank lines and other comments may separate the
directive from its node. Text inside string literals, template text, or block
comments is not a directive. There is no implicit file-wide suppression. If
malformed syntax prevents reliable node attachment, diagnostics remain visible.

Suppression changes reporting only. `emit`, `build`, `run`, `test`, and `dev`
compilation still report errors and reject invalid programs. Suppression never
creates a checked tree, changes types or ownership, or enables code generation.
Consequently a successful `check` with ignored errors does not promise a successful
build. Removing the comment restores reported diagnostics.

Editor suppression uses the current unsaved document snapshot and applies to
workspace scans as well as open documents. Formatting preserves the directive.
