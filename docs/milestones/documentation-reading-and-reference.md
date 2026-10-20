# Documentation reading and reference acceptance

Implementation: `0174566d` through `8803a208`; final browser witness is included in
this checkpoint. Compiler/LSP consumers are built from `3239330f`.

## Observable changes

Reading positions belong to each full route, including fragments. Article, landing
and code offsets survive navigation and theme changes; shared sidebar offsets stay
independent. Links expose exact-page and ancestor selection separately. Semantic
CSS colors follow live theme scopes, including prose links and syntax tokens.
Ordinary previews grow with their content; bounded source panels still scroll.
Small screens give articles their full width rather than reserving a side column.

The landing has actual project commands, a Rust result and an editable workspace
composition with exact source. It does not publish invented performance numbers or
an unverified installer. Reference introductions explain the module rather than
repeating a generic publication notice.

The content inventory has 160 destinations, 13 checked examples and 941 API
declarations. All 101 public builtin receivers map to references derived from the
existing editor catalog. Two reviewed language guides moved under `content/language`
with old links preserved. The pinned 64 components, three recipes and 30 blocks
remain an implementation inventory; their pending entries are not fake APIs.

## Content representation and editor

Generated content is readable JSON text, imported as immutable UTF-8 assets.
A shared checked decoder constructs each DTO; page-specific contexts construct only
the selected page. API construction uses bounded function groups. The explicit
full snapshot remains for tools and builds its arrays in loops.

The original generated loadPage function reserved 1,026,464 Wasm stack bytes.
Separating page construction reduces the maximum observed frame to 261,552 bytes
without increasing the stack. Generated executable modules fall to 372, alongside
1,117 text payloads. Raw text is embedded, not fetched separately at runtime.

Text imports preserve exact UTF-8, BOM, CRLF and NUL bytes, reject invalid UTF-8,
expose a readonly default string and track asset edits in native caches. The LSP
indexes text as data rather than executable source. Completion, hover, definition
and unsaved edits agree with the compiler.

The native bundled runtime needed a relative module path for selected-link rules.
Its initial failed build is retained in SQLite; the repaired real app builds and
opens the largest schema reference quietly for five seconds, then is reaped.

## Focused verification

- Six compiler integration checks: text boundaries/ownership/cache invalidation,
  complete receiver mapping, readable loader publication/tamper/reuse/cleanup,
  structured builtin Markdown and exact highlighted language aliases.
- One compiler unit check: API prose permits literal bracket labels while authored
  Markdown and unsafe URLs retain their strict rejection rules.
- Three native-feature model checks: exact/ancestor selection, live scoped colors
  and route-owned reading identities. Only the affected selected-theme case repeats
  after the bundled module-path repair.
- Two installed-LSP checks: raw asset members/hover/definition/unsaved edits, and
  selected-navigation utility completion/help.
- Actual dev browser: independent search oracle over prose/API/code, long article
  offsets 1,800/600, landing/sidebar/code offsets, history/fragments, retained form
  state, selection, both palettes, 390/768/1440 layouts, largest schema page and
  disposal. Syntax contrast minima are 6.45:1 in light and 7.88:1 in dark.
- Real native shared-app build and bounded five-second schema-route startup.
  This is not an OS gesture or native pixel claim.

The expanded-content diagnostic at `f3a008ad`, before the final copy/layout polish,
records readiness at 262.7 ms,
a 149.7 ms maximum event-loop gap, no eager articles and streaming Wasm
instantiation. It transfers 3,799,772 bytes for Wasm. These are observations under
build activity, not controlled performance benchmarks or a production-loading
claim. Reuse the earlier throttled diagnostic; do not merge its samples with these.

Installed CLI/default LSP/editor override are atomically refreshed. The active
editor maps the current inode. Its already open landing (version 7) and generated
catalog (version 18) publish empty diagnostics. Temporary trace and server settings
are restored byte-for-byte with their original modification time.

## Evidence and remaining work

Canonical originals are in `benchmarks/records.sqlite` under
`tsrust-docs-experience-*`, including failed attempts, compiled-frame reports,
formatting, consumer receipts, browser captures and diagnostic profiles. Each loose
record is independently decompressed and checked for exact bytes, size and SHA-256
before removal; database integrity is checked. Obsolete test/build and native cache
outputs are pruned: the shared target falls from about 29 GiB to 8.9 GiB and the
native cache from 7.0 GiB to 1.3 GiB. All five current consumer hashes are preserved;
current web/native dependencies stay warm. No rebuild is run merely to refill
removed caches. No SQL driver suite, full regression
matrix or performance workload rerun was triggered.

The actual `tsrust dev --target web --port 4317` preview remains running. The whole
repository editor scan still takes minutes after a cold restart; raw-data indexing
is repaired, but shared project analysis and background scan scheduling remain
required work. The HTML design reference still needs alignment with the revised
shared landing. HTML-first loading and executable splitting remain pending.

Continue the [section checklist](../plans/documentation-experience-audit.md) and
[complete library plan](../plans/documentation-and-component-library.md): generic
reactive custom props and base composition, focus/disclosure/overlays, remaining
components, chart geometry/data/accessibility, theme editing and composed blocks.
Do not call the mapped references or these reading repairs full library acceptance.
