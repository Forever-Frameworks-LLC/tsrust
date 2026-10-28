# Public documentation migration map

This map tracks canonical source moves. Repository links temporarily target the
new source files while production hosting is being prepared. Once deployment is
verified, active entry points will link to the public routes. Historical evidence
and retained UI content are not automatically published.

| Previous source | Canonical source / route | Disposition |
| --- | --- | --- |
| docs/guides/toolchain-installation.md | website/src/content/docs/tools/install-and-update.md → /tools/install-and-update/ | Moved; old copy removed; inbound Markdown links updated. |
| docs/guides/tooling/machine-setup.md | website/src/content/docs/tools/rust-setup.md → /tools/rust-setup/ | Moved; old copy removed; inbound Markdown links updated. |
| docs/guides/application-projects.md | website/src/content/docs/backend/applications.md → /backend/applications/ | Moved; old copy removed; inbound Markdown links updated. |
| docs/guides/generated-api-clients.md | website/src/content/docs/backend/api-clients.md → /backend/api-clients/, /backend/authorization/, /backend/api-compatibility/ | Split by client generation, access policy and compatibility; old copy removed; inbound Markdown links updated. |
| docs/guides/project-cli.md | website/src/content/docs/tools/cli.md and tools/testing.md → /tools/cli/, /tools/testing/ | CLI/reporting split; old copy removed; original UI instructions moved to retained UI setup. |
| docs/guides/modules.md | website/src/content/docs/language/modules.md → /language/modules/ | Moved; old copy removed; inbound Markdown links updated. |
| New site overview | website/src/content/docs/index.md → / | Reviewed initial page. |
| New quickstart | website/src/content/docs/start/quickstart.md → /start/quickstart/ | Reviewed initial journey; installation and editor detail still pending. |
| New public status | website/src/content/docs/project/status.md → /project/status/ | Current product scope; internal milestone details stay in repository. |
| docs/content/getting-started/installation.md | website/src/content/docs/start/installation.md → /start/installation/ | Moved; old copy removed; retained UI consumers updated. |
| docs/content/getting-started/project-structure.md | website/src/content/docs/start/project-structure.md → /start/project-structure/ | Moved; old copy removed; retained UI consumers updated. |
| editors/vscode/README.md | website/src/content/docs/start/editor.md → /start/editor/ plus short extension README | User guidance moved; package build/test instructions retained in the extension README. |
| docs/guides/language/standard-library.md | New website/src/content/docs/library/overview.md → /library/overview/ | Old file explicitly historical; active entry points use current module map and compiler-owned IntelliSense guidance. |
| docs/guides/http.md | website/src/content/docs/backend/http.md, static-files.md and http-transport.md | Split routing/body ownership, public assets and transport; corrected stale TLS/producer/configuration claims; old copy removed. |
| docs/guides/json.md | website/src/content/docs/library/json.md, typed-json.md and json-files.md | Split dynamic values, typed codecs and file formats; removed stale pending-codec claim and internal milestone prose; old copy removed. |
| CONTRIBUTING.md | Repository contribution entry point | Retained operational; setup dependency graphs and stag/main CI behavior reconciled with workflows. Public onboarding remains in the site. |
| docs/guides/tooling/rust-toolchain-policy.md | Developer pin/upgrade policy | Retained operational; links user setup and managed-install instructions to canonical site sources. Removed stale dependency-free and unimplemented CLI/LSP distribution claims. |
| crates/tsrust-compiler/README.md | Compiler crate entry point | Replaced obsolete LSP/dependency/recovery claims and chronological feature dump with source-checked API/layout guidance and canonical guide/milestone links; historical source remains in Git. |
| docs/guides/language/compiler-and-modules.md | Retained architecture research | Current implementation lead corrected; original planning explicitly separated from implemented editor behavior. |
| Remaining docs/guides/ | Language/backend/library/tools destinations in the site plan | Review and map individually before moving. |

The moved guides have no authored heading IDs or linked internal anchors to
rename. External links and behavior contracts are retained. Their existing
commands have focused compiler/installer/cache evidence; migration itself requires
site rendering and repository-link checks, not another compiler matrix.
