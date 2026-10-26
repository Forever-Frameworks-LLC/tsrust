# GitHub documentation and distribution preparation

Date: 2026-10-26. Source base: `72b79808`.

## Scope

Replaced the historical root README with a concise product introduction, real
syntax, source/release setup, current boundaries and useful entry points. Added
an examples index and refreshed the docs index, installation and VS Code guides.
Added contribution and security guidance, issue forms and a PR template.
The owner chose MIT with Forever Frameworks LLC as copyright holder. Crate and
extension metadata agree; third-party notices retain their separate licenses.

The CI workflow covers Linux x64 and macOS ARM selected compiler/CLI, LSP agreement,
native renderer model, Clippy, formatting and VS Code checks. Full regressions
are an explicit dispatch option. The release workflow runs the same CI, then
builds optimized CLI/LSP archives on Linux x64 and macOS ARM/Intel. Packaged CLI
smoke checks precede publication. Assets include archives, VSIX and SHA256SUMS.
Only publication has write permission. Actions are pinned to verified upstream
commits, with monthly Dependabot updates. No remote, repository settings, tag or
release was created.

Packaging reuses the existing toolchain script. It adds MIT/upstream notices,
vendored dependency license files and source/version identities. Schema 2 uses
relative package paths as manifest file keys; there are no existing consumers
of the old keys. The four-argument development packaging call remains supported,
marked as incomplete for redistribution. Official release packaging supplies the
fifth vendored-source directory. Release preflight rejects dirty source and
mismatched or malformed tags. Two pre-existing Node oracle invocations now use
PATH rather than the owner's absolute machine path.

## Local acceptance

- Five Node distribution regressions pass: byte/hash/notice fidelity, preserved
  destinations, cleanup on failure, development labeling, strict tag/cleanliness
  checks and public documentation links.
- actionlint 1.7.12 passes both workflows; its official binary download was checked
  against the upstream SHA-256 digest. Removed the temporary tool after validation.
- Extension TypeScript/build, three executable discovery tests and the real
  TextMate grammar test pass. VSIX packaging succeeds with 12 files. Independently
  checked that its MIT and third-party notice bytes match the source files.
- README example formats idempotently and checks with the installed accepted
  compiler. Native example execution was not repeated.
- Whole-project formatting reports zero changes and zero refreshed fingerprints;
  formatting check and `git diff --check` pass.

Initial local distribution checks caught a missing plural `licenses/` directory
match and a stale documentation link. Both were repaired; affected checks pass.
Canonical evidence: SQLite record `tsrust-github-preparation-20261026.json` and
`tsrust-github-distribution-tests-20261026.log`. Closed loose copies are removed
only after exact-byte restoration and integrity verification.

## Explicitly pending

No hosted workflow has run: the checkout has no remote. Cold Rust compilation,
Clippy, cross-platform execution and packaged native smoke remain unverified
until Actions executes. Local Rust builds, broad regressions, GPU/browser sessions
and LSP scans were deliberately not restarted under the memory-pressure pause.
The two portable Node oracle changes were inspected, not rerun through Rust tests.
CI may expose existing failures; it must fail rather than mask or skip them.
Default CI is not the entire compiler/LSP matrix, SQL service acceptance, Wasm
browser acceptance, Extension Host integration, timing or real OS accessibility.

LSP memory pressure remains unresolved. Table implementation/repair acceptance
is still pending. Release notes disclose both. This section does not accept them
or resume the paused component-library roadmap. Source executables remain the
previous accepted consumers; no candidate CLI/LSP was installed.

Before publishing, follow [repository setup and release steps](../guides/releases.md),
resolve or disclose intended release limitations, and inspect the first hosted
run. Unsigned executables, no notarization/attestations, no Windows/musl/Linux ARM
archives, no installer and no Marketplace publishing are deliberate current limits.
