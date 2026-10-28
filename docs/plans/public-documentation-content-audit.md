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

## Page checklist

| Page | Status | Evidence / remaining work |
| --- | --- | --- |
| `404.md` | Pending | Verify facts, examples and reading flow. |
| `backend/api-clients.md` | Pending | Verify facts, examples and reading flow. |
| `backend/api-compatibility.md` | Pending | Verify facts, examples and reading flow. |
| `backend/applications.md` | Pending | Verify facts, examples and reading flow. |
| `backend/authorization.md` | Pending | Verify facts, examples and reading flow. |
| `backend/http-transport.md` | Pending | Verify facts, examples and reading flow. |
| `backend/http.md` | Source review complete; rendered review pending | HTTP runtime, builtin catalog and examples/http-context/json.test.tsr; combined route example passes compiler check. Split readers/errors/responses. |
| `backend/static-files.md` | Source review complete; rendered/native example review pending | StaticMount::new/respond/resolve and Router::serve_static in HTTP runtime; clarified 307/400/404, Accept fallback and prerequisites; standalone server example passes compiler check. |
| `index.md` | Pending | Verify facts, examples and reading flow. |
| `language/modules.md` | Pending | Verify facts, examples and reading flow. |
| `library/json-files.md` | Pending | Verify facts, examples and reading flow. |
| `library/json.md` | Pending | Verify facts, examples and reading flow. |
| `library/overview.md` | Pending | Verify facts, examples and reading flow. |
| `library/typed-json.md` | Pending | Verify facts, examples and reading flow. |
| `project/status.md` | Pending | Verify facts, examples and reading flow. |
| `start/editor.md` | Pending | Verify facts, examples and reading flow. |
| `start/installation.md` | Pending | Verify facts, examples and reading flow. |
| `start/project-structure.md` | Pending | Verify facts, examples and reading flow. |
| `start/quickstart.md` | Pending | Verify facts, examples and reading flow. |
| `tools/cli.md` | Source review in progress | Confirmed interactive categories, ASCII name/100-character limit, config defaults and explicit-path semantics. Added headings around dev output/cache. Watcher and full example acceptance remain pending. |
| `tools/install-and-update.md` | Pending | Verify facts, examples and reading flow. |
| `tools/rust-setup.md` | Pending | Verify facts, examples and reading flow. |
| `tools/testing.md` | Source/example review complete; full responsive review pending | testing/discovery.rs, CLI main/project_config and reporter runtime. Native greeting passes; changed expectation fails with both values. Initial narrow preview is readable and highlighted. |
