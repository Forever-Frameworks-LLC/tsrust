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

`fixtures/model-ui-host.tsr` exercises the real private-stdio chat transport and
native rendering without loading a model. Its startup notice and every response
identify it as synthetic. Build it once from this package with
`tsrust build tests/fixtures/model-ui-host.tsr -o ../../target/studio-model-ui-host`,
then start the dev preview with `TSRUST_STUDIO_MODEL_HOST` set to that executable's
absolute path. Check streamed Unicode/Markdown, response growth and the model
automatic startup, conversation switching, draft recovery and context controls. This fixture does not qualify inference, model cancellation,
coding quality or memory fit. Stop the fixture and restore the real model-host
override before leaving the normal development preview running.
