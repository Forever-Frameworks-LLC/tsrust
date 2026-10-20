# Documentation experience audit and delivery checklist

This is required implementation work, not a completion claim. Keep component status
in `ui-component-coverage.json`; this file records presentation and coverage work.

## Design review

Reviewed the current Next.js docs, Better Auth introduction, shadcn components and
chart galleries on 2026-10-19. The useful patterns are section-level navigation,
persistent selected pages, a readable article column, a compact contents rail,
working examples beside their exact source, and separate chart/block galleries.

- [Next.js](https://nextjs.org/docs): tutorials, guides and API reference have
  different jobs. Adopt that separation instead of a single undifferentiated list.
- [Better Auth](https://better-auth.com/docs/introduction): concise introduction,
  grouped concepts and a stable reading shell. Do not copy its auth responsibilities.
- [shadcn components](https://ui.shadcn.com/docs/components): dedicated references
  and installable local source. Keep our pinned Base UI/Nova inventory authoritative.
- [shadcn charts](https://ui.shadcn.com/charts/area): chart families, interactive
  variations and exact source previews deserve their own gallery, not one Chart page.

Iteration 1: fix the reading shell and expand source-backed coverage. Iteration 2:
replace the generic counter pitch with a compact product example and stronger code
story. Iteration 3: use actual completed components and chart primitives to build
the galleries. Do not use screenshots, fake metrics or inert controls as acceptance.

## Reported defects and stronger acceptance

- [ ] Restore long article, landing, sidebar and code offsets across link clicks,
  browser back/forward, fragments, section changes, search and theme changes.
  Test actual overflow and wheel input; a 90-pixel offset on one page is insufficient.
- [ ] Show the selected section on every descendant URL and the exact selected
  page independently. Preserve the selected group when a search is cleared.
- [ ] Use project primary colors for selected navigation, prose links and calls
  to action. Preserve foreground contrast and independent nested theme scopes.
- [ ] Verify distinct syntax classes in both modes, after switching modes and
  opening a previously hidden source panel. Plain command fences must be identified
  as plain commands, not presented as broken TypeScript highlighting.
- [ ] Let ordinary component previews size to their content. Restrict scrolling
  to examples explicitly demonstrating a scroll container or bounded code panel.
- [ ] Replace the landing's oversized diagnostic counter with a concise useful
  interaction. Keep source, native implementation and browser behavior identical.

## Content inventory: what each section must present

### Landing

- [ ] Clear compile-to-Rust proposition, actual build command and explicit release link.
- [ ] A small readable Router example with middleware order and application policy.
- [ ] A real Rust import, supported targets and compiler diagnostics for unavailable APIs.
- [ ] An attractive live component composition, light/dark variants and exact code.
- [ ] Offline local installation, editable source and theme configuration.
- [ ] Honest measured performance links; no invented timings, sizes or claimed parity.

### Getting started and CLI

- [ ] Installation, toolchain prerequisites, releases and offline constraints.
- [ ] New console/service/UI project walkthroughs, entrypoints and `src/` convention.
- [ ] Complete configuration reference: build, targets, UI paths/theme, documentation.
- [ ] Interactive CLI, new/check/format/build/run/dev/test/ui commands and useful errors.
- [ ] Development cache, reload lifecycle, cancellation, cleanup and troubleshooting.
- [ ] Editor installation, completion, hover, signatures, diagnostics and target support.

### Language

- [ ] Values and all numeric widths, strings, booleans, nullable values and checked conversions.
- [ ] Variables, expressions/operators, blocks, conditionals, loops and pattern matching.
- [ ] Functions, callbacks, closures/capture, generics, interface inheritance and modules.
- [ ] Records, arrays, tuples, unions, narrowing, enums and collection ownership.
- [ ] Ownership, moves, borrowing, resources, explicit copy/clone and deterministic release.
- [ ] Tasks, cancellation, errors and concurrency; distinguish async syntax from runtime APIs.
- [ ] JSX, components, state, conditional mount versus retained hide, lists and event order.
- [ ] Rust bindings, ABI conversions and exact per-runtime capability diagnostics.
- [ ] Explicit unsupported syntax/semantics and migration from JavaScript/TypeScript.

### Standard library and services

- [ ] Every public std module and every public builtin receiver gets a mapped destination.
- [ ] Method references derive signatures and behavioral prose from the editor catalog.
- [ ] Each method presents arguments/defaults/errors, return/mutation, examples, ownership,
  boundary values and relevant method-level benchmark evidence.
- [ ] Strings, arrays, tuples, numbers, Math, JSON, Object, Map/Set and other collections.
- [ ] Bytes, exact decimals, dates/time zones/durations, UUIDs, schema and typed JSON.
- [ ] Filesystem/path/env/OS/time, crypto, signals, testing and result helpers.
- [ ] Router, middleware/closures/order, request/response, headers/cookies/state, client,
  forms/files and streams. Authentication policy stays in application code.
- [ ] SQL pools/transactions/rows/cursors and explicit driver constraints; reuse S18 evidence.
- [ ] Cache eviction/ownership, queues, workers and application-level durable SQL jobs.
- [ ] Shutdown, cancellation and cleanup recipes; never imply unsupported task transfer.

### UI foundations and authoring

- [ ] Targets/bootstrap, layout/breakpoints/scrolling, events, reactivity and resource lifetimes.
- [ ] Accessibility, keyboard/IME, focus, overlays, dismissal, reduced motion and contrast.
- [ ] Native headers, OS controls, drag exclusions, safe areas and platform limitations.
- [ ] Tokens, presets, light/dark mode, typography, live customization and theme generator.
- [ ] Base component API, props/state/events/composition, local variants and new-component recipe.
- [ ] Offline CLI installation, multi-select, local upgrades, dependency rewrites and attributions.

### Components, recipes, blocks and charts

- [ ] All 64 pinned components: overview, meaningful default/variant examples, exact code,
  installation, generated complete props, keyboard behavior, accessibility, limits and targets.
- [ ] Three recipes: Data Table, Date Picker and Typography, with actual dependency composition.
- [ ] Thirty pinned blocks: sidebar/dashboard/login/signup/preview families; responsive widths,
  source/dependencies, editable layout and application-owned data/policy.
- [ ] Chart families: area, bar, line, pie/donut, radar, radial, scatter and composed displays.
  Cover axes, scales, domains, legends, tooltip/focus, empty/error/loading data and exact decimals.
- [ ] Chart variants: grouped/stacked, positive/negative, dates/time zones, sparse/missing values,
  theme tokens, responsive geometry, keyboard access and a readable data-table alternative.
- [ ] Theme gallery/generator: real token editor, validation, contrast feedback, preview and export.
- [ ] A pending entry states its intended use and dependencies; it never shows a fake API or
  pretends to be installed. Route count is never used as a component-completion count.

### Performance, search and publishing

- [ ] Overall graphs and comparable native/compiled-Node snapshots from canonical SQLite.
- [ ] Per-method links and inline results, protocols, source identities and unmeasured states.
- [ ] Full-text search includes all prose/APIs/code, keyboard navigation and useful empty results.
- [ ] Rich Markdown, exact copy/highlighting, tables, images/assets, stable links and headings.
- [ ] Web/native parity, direct URLs, reading history, production export, HTML-first startup
  and executable loading. Preserve the startup responsiveness repair while adding content.
- [ ] Inventory every developer document before moving it; preserve evidence and old links.

## Execution order

1. Repair navigation/scroll/theme/example presentation across runtime, app and witnesses.
2. Expand source-backed reference destinations and publish explicit section overviews.
3. Complete the shared semantic components; then disclosure, selection and focus families.
4. Build chart geometry/data/accessibility, theme generator and real composed blocks.
5. Fill remaining tutorial/reference material and finish paired production/readiness gates.

Batch targeted verification after each complete section. Keep the dev preview and
installed editor current. Reuse existing SQL/resource/compiler evidence; do not
restart full matrices after individual controls. Update this checklist as work lands.

## Reviewed source migration

The existing Type aliases and Switch statements guides are user documentation,
not historical evidence. Their prose and executable examples now live under
`content/language/`. Original paths retain short links to the current guides;
existing handoff and evidence links are preserved. Other root-level documents
still need individual review before moving them.
