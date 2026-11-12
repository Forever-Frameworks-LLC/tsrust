# Studio editor fixtures

Other-language examples and their package/module manifests stay in this directory;
they are fixtures for file icons, highlighting and language-service interactions.

`editor-language-mat.go` deliberately assigns a string to an integer so that Go
diagnostics appear in Problems. Its `go.mod` belongs here, not at Studio's root.
The fixture is not part of Studio's application or its Tsrust test suites.

`editor-language-mat.tsr` is the corresponding Tsrust fixture for function
completion, scalar hover/signatures and numeric match-arm snippets. Make temporary
editor changes without saving, then restore its original text after manual checks.

The filename-search case in `explorer.test.tsr` exercises the real registered read,
present and open adapters, including Unicode paths, ignore rules, explicit grants,
missing results and retention of an existing draft. Native keyboard and typography
acceptance is recorded in the continuation handoff.
