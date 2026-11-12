# Studio editor fixtures

Other-language examples and their package/module manifests stay in this directory;
they are fixtures for file icons, highlighting and language-service interactions.

`editor-language-mat.go` deliberately assigns a string to an integer so that Go
diagnostics appear in Problems. Its `go.mod` belongs here, not at Studio's root.
The fixture is not part of Studio's application or its Tsrust test suites.
