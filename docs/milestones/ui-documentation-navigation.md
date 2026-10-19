# Landing, documentation navigation and offline search

Source checkpoints: `a5531211` (implementation), `d1870833` (focused repairs).
The accepted app source is shared by the browser and native targets. The public
preview serves the landing page at `http://127.0.0.1:4317/` and documentation at
`http://127.0.0.1:4317/docs`.

## Observable behavior

- A distinct landing page introduces the language, paired UI and editable controls.
  Its counter remains live and retains its state across internal route changes.
- The grouped sidebar contains actual anchors for every checked catalog page.
  The search field stays outside the scrolling index. Pending inventory pages
  state their status; a destination is not evidence of an implemented component.
- `/docs` opens the introduction. Other document paths, selected links, browser
  back/forward, direct entry and an explicit not-found screen work. Heading links
  focus and reveal their destination, including browser-encoded Unicode anchors.
  Returning to a retained route preserves its article scroll offset.
- Search matches page text, mapped checked API declarations/fields and registered
  example source locally. It is a case-insensitive substring scan of this fixed
  catalog, without network search, semantic ranking or fuzzy matching. Empty
  results have a polite status message. Clearing the query restores the index.
- The shared block renderer handles headings, paragraphs, lists, task markers,
  blockquotes, tables, literal HTML and read-only code controls. Lists and table
  parts keep browser/native accessibility roles. Checked examples and API cards
  appear on their associated pages.
- Theme switching retains the mounted component. Page-cache exit disposes mounts;
  a persisted browser page resumes with a fresh application mount.

`std:ui` now supplies `Link`, `location`, `navigate`, `back` and `forward`.
Location is created directly once in the root component and retained through the
existing compiler state lowering. Captures, link callbacks and platform listeners
never retain its owner. A link from a disposed root cannot navigate a later root.
Static links can serialize without an owner; interactive links must be constructed
after their owner. Absolute local paths and heading fragments are supported.
External URLs, queries, authored escapes and traversal remain explicitly rejected.
Browser-generated Unicode escapes decode without accepting escaped ASCII separators.

The visual direction follows the owner's Next.js/Better Auth references: neutral
surfaces, fine borders, grouped navigation, a constrained article and a contents
rail. Generic serif, system and monospace families are honest paired capabilities;
no unavailable named font is advertised.

## Focused acceptance

- Whole-project formatting proves token preservation/idempotence. The formatted
  app passes project checking and native inspection: `Documentation home and links passed`.
- The compiler project fixture executes formatted Link/location/history source.
  Its first failure was the fixture using the standalone import-free helper;
  the repaired project/module witness passes without changing compiler semantics.
- Five targeted native/model cases pass, including owned navigation disposal,
  checked paths, Unicode decoding, actual Link/list/table accessibility roles and
  an existing matching layout case. The affected native semantic case passes again
  with the final native source. Link keyboard activation excludes Space; existing
  button keyboard behavior stays unchanged.
- One scoped LSP UI witness passes diagnostics, JSX prop completion, parameter help and hover for location, href,
  anchor behavior and display typography. Public signatures and parameter docs
  live in the canonical builtin catalog; normal completion uses checked props.
- The real Chrome observer passes actual navigation, article/source content,
  heading focus, table/list/link/search accessibility, text/API/code filtering,
  empty results, sticky search, retained article scroll, counter/theme retention,
  480px layout, deep/Unicode entry, not-found, page-cache teardown/resume and disposal.
  The first observer failure was its repeated top-level JavaScript declaration.
  A subsequent real initial-focus failure was fixed in the shared browser mount:
  headings are focused after the root is attached, with scroll preservation.
- The native acceptance entry opens the same documentation renderer at its Unicode
  heading route. A bounded five-second launch remains alive without output/errors
  and is reaped. This is a launch smoke check, not an automated OS gesture or
  native pixel comparison. Native rendering/scrolling/editing evidence is reused.
- Five CLI/editor consumers are refreshed. Both editor settings files are restored
  byte-for-byte with their timestamps. The actual running editor maps the installed
  executable inode, rather than the replaced image.
- The live preview's root, `/docs`, a direct nested URL, missing-asset errors and
  Wasm MIME type pass HTTP checks. Only one intentional preview server remains.

No full compiler/LSP matrix, performance measurement or database suite was rerun.
S18 PostgreSQL/MySQL/SQLite and unchanged lifetime/cancellation evidence remain
accepted. Build durations are diagnostics, not benchmark measurements.

## Required continuation

This is a fixed, retained documentation catalog, not the completed generic route
framework. Dynamic keyed lists, parameterized routes, lazy outlets and general
application route lifecycle policy remain required framework work.

Rich inline emphasis/links and selection are unfinished: paragraph links are
currently separate accessible links; image nodes are labeled references. Code
controls normalize line endings. Syntax-colored code, exact-byte copy feedback,
asset rendering and advanced tables remain pending. Checked API cards describe
current partial controls; they do not establish complete Base UI/Nova parity.

Continue rich inline text/code rendering and the remaining pinned component,
overlay/focus, block and theme-editor families. Mobile navigation below the
verified 480px viewport, curated benchmark charts, semantic search and integrated
ordinary CLI web build/dev also remain in the active plan. Preserve the accepted
preview and evidence while implementing complete paired sections.

## Durable evidence

SQLite `benchmarks/records.sqlite` stores exact closed `tsrust-docs-routing-*`
records, browser assets/captures, content outputs, consumer identities and launch
receipts. The closure receipt records independent zlib/SHA-256/length restoration
and integrity checks before verified loose copies are removed. Active preview
assets, generated/editor-consumed content, installed consumers and warm runtime
workspaces remain available. No source snapshots or archive directories are added.
