# Public documentation content audit

Audit every public page against current compiler, CLI, runtime and example sources.
A successful site build or link check does not prove technical accuracy. Migrated
content is not accepted merely because its original file existed.

## Acceptance per page

- Verify commands, signatures, defaults, errors, limits and platform claims against
  implementation. Use focused executable witnesses for uncertain behavior.
- Separate currently supported behavior, limitations and proposed work. Remove
  internal gate narratives from user guidance.
- Start with the reader's task. Supply a usable example and expected result where
  appropriate. Explain prerequisites before the command that needs them.
- Use short sections, tables for comparable options and explicit steps for ordered
  tasks. Keep detailed error/ownership contracts near the relevant operation.
- Check code highlighting, narrow-screen layout, navigation and internal links.
- Record source evidence and pending execution checks. Do not call an unchecked
  page audited. Preserve existing completed native evidence without rerunning the
  entire compiler suite for prose edits.

## Completion standard

Every public page requires both a complete content audit and a presentation audit.
A partial review, migration, successful build or checked example cannot close its row.
For each page, review every paragraph, table row, code block, callout and link:

- Trace factual claims to current implementation or applicable acceptance evidence.
  Treat historical incidents as historical; check for subsequent repairs before
  describing them as current defects. State remaining uncertainty precisely.
- Execute complete examples and commands where their output or side effects matter.
  Label intentional fragments and provide their prerequisites. Verify warnings,
  defaults, version requirements, platform claims and recovery instructions.
- Review the page as a reader: clear purpose, useful ordering, manageable sections,
  consistent terminology, meaningful links and no unsupported blanket warnings.
- Inspect the rendered page at desktop and narrow widths, including code/table
  overflow, highlighting, navigation and keyboard access. Record any unavailable
  check as pending. Do not substitute source review for rendered inspection.

Final acceptance requires every row to have complete content and presentation
coverage, with no unresolved inaccurate claim. Evidence must identify what was
checked; "100%" is not a substitute for that evidence.

## Page checklist

| Page | Status | Evidence / remaining work |
| --- | --- | --- |
| `404.md` | Content review complete; rendered review pending | Recovery copy and issue destination are appropriate; search/sidebar interaction and actual unknown-route behavior still pending. |
| `backend/api-clients.md` | Source review in progress | Corrected mandatory --out for URL/saved-description generation against api_cli.rs. Separated source configuration and generated output; full wire-contract and rendered audit pending. |
| `backend/api-compatibility.md` | Pending | Verify facts, examples and reading flow. |
| `backend/applications.md` | Source review in progress | Reorganized into create/run, access policy, checks, client generation and OpenAPI scope. Checked application_scaffold.rs, starter templates and api_cli.rs. Live starter description, TypeScript/Rust SDK generation and saved-description regeneration pass; rendered review pending. |
| `backend/authorization.md` | Pending | Verify facts, examples and reading flow. |
| `backend/http-transport.md` | Pending | Verify facts, examples and reading flow. |
| `backend/http.md` | Source review complete; rendered review pending | HTTP runtime, builtin catalog and examples/http-context/json.test.tsr; combined route example passes compiler check. Split readers/errors/responses. |
| `backend/static-files.md` | Source review complete; rendered/native example review pending | StaticMount::new/respond/resolve and Router::serve_static in HTTP runtime; clarified 307/400/404, Accept fallback and prerequisites; standalone server example passes compiler check. |
| `index.md` | Source/example review complete; rendered review pending | Hello example verified through quickstart native execution; scope matches backend scaffold and independent SDK walkthrough. Removed migration-planning prose. |
| `language/modules.md` | Pending | Verify facts, examples and reading flow. |
| `library/json-files.md` | Source review in progress | Replaced incomplete reader snippet with complete open/read/EOF/close example; executed with Ada and Grace output. Split writer cleanup. Writer example and rendered review pending. |
| `library/json.md` | Source review in progress | Corrected integer overflow and raw serialization claims against runtime/parser.rs and json_scalar_backend.rs; separated errors, resource limits and parsing rules. Remaining examples and rendered review pending. |
| `library/overview.md` | Source/example review complete; presentation review in progress | All 23 module names exist in catalog and implementation registry; basename example executes as documented. Narrow 587px layout/highlighting and table wrapping inspected; desktop/keyboard review pending. |
| `library/typed-json.md` | Source review in progress | Corrected overflow and JsonRaw support against json_typed.rs/json_raw.rs and builtin catalog. Added shape table and raw-field example; example executed with exact preserved output. Remaining examples and rendered review pending. |
| `project/status.md` | Source review complete; rendered review pending | Scope matches backend-only plan; four release targets verified against release.yml. Clarified pinned source-commit adoption when releases are absent. |
| `start/editor.md` | Source review in progress | VS Code manifest minimum, executables.ts discovery order, extension.ts configuration/restart behavior checked. Separated stale-path recovery from onboarding. Replaced stale blanket memory warning with landed retention fixes and explicit long-session macOS uncertainty; verified restart window against installed language-client code. Full editor/platform execution and rendered review remain pending. |
| `start/installation.md` | Source review in progress | Verified archive naming, current exact Rust pin and executable paths; made extraction conditional on checksum success and separated installation/update tasks. Fresh-platform instructions remain pending. |
| `start/project-structure.md` | Source review complete; rendered review pending | File list, build entry/output and explicit API configuration match application_scaffold.rs and starter templates; independently created starter passes check/test/format. |
| `start/quickstart.md` | Source/example review in progress | Exact hello snippet checks and executes with TypeScript + Rust output. Fresh --app project check/test/format pass. Live HTTP description and both SDK commands pass; rendered review pending. |
| `tools/cli.md` | Source review in progress | Confirmed interactive categories, ASCII name/100-character limit, config defaults and explicit-path semantics. Added headings around dev output/cache. Watcher and full example acceptance remain pending. |
| `tools/install-and-update.md` | Pending | Verify facts, examples and reading flow. |
| `tools/rust-setup.md` | Pending | Verify facts, examples and reading flow. |
| `tools/testing.md` | Source/example review complete; full responsive review pending | testing/discovery.rs, CLI main/project_config and reporter runtime. Native greeting passes; changed expectation fails with both values. Initial narrow preview is readable and highlighted. |
