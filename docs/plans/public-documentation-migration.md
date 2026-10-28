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
| docs/guides/async-tasks.md | website/src/content/docs/language/async.md → /language/async/ | Moved; removed obsolete S13/S14 future-tense claims and benchmark protocol from user guidance. Runtime operations are covered by the futures page below. |
| docs/guides/tokio-tasks.md | website/src/content/docs/library/futures.md → /library/futures/ | Moved; runtime operations split into tasks, limits and error tables. Retained executable examples and historical acceptance; updated inbound guide link. |
| docs/guides/module-state.md | website/src/content/docs/language/modules.md#share-module-state | Merged into module guide; removed duplicate, corrected obsolete default/global export restrictions and updated inbound plan link. |
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

## Remaining source inventory — 2026-10-28

The rows below exhaust the remaining `docs/guides/**/*.md` files at checkpoint
715d651c. They are migration decisions to finish, not evidence that their prose is
correct. “Current/mixed” requires a paragraph-level review: migrate useful public
instructions into the existing topic or a focused new page; preserve any historical
portion and update its active inbound links. Do not publish milestone narratives.
A short redirect guide should disappear after its authoritative destination moves.

Operational/research files stay in the repository, but their current claims and
public entry links still require reconciliation. A research label does not make an
inaccurate implementation claim acceptable. Do not delete retained UI consumers or
feature-coverage evidence to reduce the inventory.

| Remaining guide (relative to docs/guides) | Required disposition | Status |
| --- | --- | --- |
| `async-sql.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `collections.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `diagnostic-suppression.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/array-from.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/array-literal-spread.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/arrow-functions.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/assignment-values.md` | Migrated to website language/assignment-values; removed duplicate and updated Markdown links | Source, exact native examples, focused compiler/editor tests and full responsive review accepted |
| `language/borrowed-optional-fallback.md` | Migrated to canonical site language/borrowed-optional-fallback | Accepted: four complete examples, compiler/editor and rejection witnesses, full responsive/keyboard review; duplicate removed and links migrated. |
| `language/call-spread.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/callable-arrays.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/callable-expressions.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/callable-fields.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/callable-locals.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/comma-contexts.md` | Migrated to website language/comma-expressions; removed duplicate and updated Markdown links | Source, exact native examples, focused compiler/editor tests and full responsive review accepted |
| `language/comma-expressions.md` | Migrated to website language/comma-expressions; removed duplicate and updated Markdown links | Source, exact native examples, focused compiler/editor tests and full responsive review accepted |
| `language/declaration-lists.md` | Merged into website language/declarations; removed duplicate and updated Markdown links | Source, examples, focused tests and full responsive review accepted |
| `language/deferred-initialization.md` | Merged into website language/declarations; removed duplicate and updated Markdown links | Source, examples, focused tests and full responsive review accepted |
| `language/destructuring.md` | Split into language/destructuring, pattern-defaults-rest and record-spread-assignment | Accepted: source claims, 16 snippets, affected compiler/editor checks and full responsive/keyboard reviews complete. Removed superseded guide and migrated Markdown links. |
| `language/enums.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/exception-handling.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/global-number-names.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/http-and-hono.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/in-operator.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/interfaces.md` | Migrated to canonical site language/interfaces | Accepted: source audit, exact executable examples, focused regression evidence and full responsive/keyboard review complete. Removed duplicate guide and migrated links. |
| `language/labeled-control.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/logical-operators.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/map-group-by.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/maps.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/native-bytes.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/native-files.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/nested-functions.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/non-null-assertions.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/nullable-callables.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/nullish-assignment.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/nullish-coalescing.md` | Audit and migrate current guidance; preserve any embedded history | In progress: six exact examples format/check/execute; corrected callable and literal-union wording, added behavioral sections. Literal fallback regression passes. Site migration and rendered review remain pending. |
| `language/numeric-literals.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/numeric-string-conversion.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/optional-chaining.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/optional-default-parameters.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/optional-fields.md` | Migrated to canonical site language/optional-fields | Accepted: source audit, exact executable examples, focused regression evidence and full responsive/keyboard review complete. Removed duplicate guide and migrated links. |
| `language/recursive-records.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/rest-parameters.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/result-match.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/scalar-unary-plus.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/scoped-callable-transport.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/sets.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `language/statement-bodies.md` | Migrated to website language/statement-bodies; removed duplicate and updated milestone link | Source, exact native example and full 390px/1440px presentation accepted |
| `language/string-concatenation.md` | Merged into language/strings; old duplicate removed and Markdown links updated. | Source, examples and 390px/1440px rendered review complete |
| `language/string-indexing.md` | Merged into language/strings; old duplicate removed and Markdown links updated. | Source, examples and 390px/1440px rendered review complete |
| `language/string-iteration.md` | Merged into language/strings; old duplicate removed and Markdown links updated. | Source, examples and 390px/1440px rendered review complete |
| `language/string-semantics.md` | Retained original representation/measurement history with explicit superseded-contract label; current usage moved to language/strings. | Core source, examples and 390px/1440px rendered review complete |
| `language/switch.md` | Canonical website language/switch; removed guide stub; preserved retained catalog metadata/route as a link | Content, exact examples, compiler/LSP regressions and responsive presentation accepted |
| `language/type-aliases.md` | Canonical website language/type-aliases; removed guide stub; preserved retained catalog metadata | Content, exact examples, compiler/LSP tests and responsive presentation accepted |
| `language/type-assertions.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `rust-packages.md` | Moved to tools/rust-packages; current shared-cache behavior corrected in the regex example, history retained in package-cli-acceptance. | Source and responsive presentation reviewed; package commands and relocated offline portable example pass locally |
| `schema-diagnostics.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `sqlite.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `tooling/deployment.md` | Current guidance moved to backend/deployment; original vision and core-33 anchor retained in place. | Source and 390px/1440px presentation reviewed; release-command execution remains pending |
| `tooling/editor-commands.md` | Retained implementation/acceptance history with current-guide pointers; run settings moved into start/editor. | Source reconciliation and added editor paragraph 390px/1440px rendering complete |
| `tooling/editor-foundation.md` | Original plan and handoffs retained with explicit historical scope; current capability/installation/monitoring claims no longer implied. | Historical status and current-guide links reconciled |
| `tooling/editor-quick-fixes.md` | User instructions moved to start/editor; implementation and acceptance retained with historical limits labeled. | Source reconciliation and 390px/1440px rendered review complete |
| `tooling/interop-and-targets.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `tooling/missing-linker-diagnostic.md` | User recovery moved into tools/rust-setup; original acceptance evidence retained and labeled historical. Active editor link updated. | Source reconciliation and 390px/1440px rendered review complete |
| `tooling/project-editor.md` | Merged into start/editor; removed stale named-import-only and open-files-only limits, updated active links, removed duplicate. | Source reconciliation and 390px/1440px rendered review complete |
| `tooling/toolchain-and-development.md` | Audit and migrate current guidance; preserve any embedded history | Pending |
| `compiler-source-layout.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `documentation-content.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `language/compiler-and-modules.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `language/data-type-coverage.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `language/diagnostic-improvements.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `language/feature-coverage.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `language/standard-library-coverage.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `language/standard-library.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `releases.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `resource-lifetimes.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `tooling/retained-ui-setup.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `tooling/rust-toolchain-policy.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `ui-applications.md` | Retain developer/evidence/UI purpose; reconcile current claims and links | Pending |
| `language/async-and-runtime.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/databases.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/error-handling.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/identity.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/metadata-and-serialization.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/ownership-and-memory.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/primitives-and-collections.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/server-html.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/type-system.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `language/validation.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `tooling/packages-and-registry.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `tooling/security-and-supply-chain.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |
| `tooling/source-migration.md` | Retain labeled research; reconcile any implemented-scope lead and links | Pending |

### Retained content consumers

`docs/content/` is still consumed by the original documentation UI and authored
content/compiler fixtures. Its metadata-only API pages are not a second handwritten
API catalog: keep the validated builtins source canonical. Before removing a page,
inspect `docs/tsrust.config.json`, the content generator and documentation_content
acceptance. `language/switch.md` now preserves its catalog route and points to the
canonical site source. `language/type-aliases.md` and `language/text-imports.md`
now also preserve metadata while linking to accepted canonical site guides.
Preserve UI-only pages, executable example references and metadata coverage.
Do not equate the current site pages with completion of the repository migration.

### Execution order

1. Finish active entry-point destinations: collections, string semantics, Rust
   packages, deployment and editor operations. Reuse existing page sections where
   they remain digestible.
2. Migrate language reference by coherent families: values/types, functions and
   ownership, expressions/control flow, strings/collections, then database/schema
   operations. Verify complete examples with current consumers; retain accepted
   rejection/runtime evidence and repair only affected failures.
3. Reconcile retained research/developer leads and every active inbound link;
   remove superseded public copies only after their consumers are updated.
4. Finish platform/package acceptance and production-host verification already
   listed in the site plan. Never close those rows using local Markdown checks.
