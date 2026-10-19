# Documentation application and complete component library

Status: active implementation plan, 2026-10-19. The owner requested this plan
before expanding the documentation application. Unchecked steps are required
work, not optional follow-ups. This is an iterative project; do not describe the
whole library as complete after building its shell.

## Decisions and authority

- Build one Tsrust documentation project for web and native desktop. The website
  is the public reading surface, but both targets have equal implementation and
  acceptance priority. Investigate native constraints before committing to shared
  APIs. Native headers belong in the first framework section.
- Start with offline full-text search across prose, APIs and examples. Semantic
  search is a later, separately designed addition; no embedding service or model
  download is required to read, build or search the documentation.
- Publish curated, reproducible snapshots exported from canonical
  `benchmarks/records.sqlite`. Benchmarks are part of the documentation: overall
  comparison pages and results embedded beside individual methods, such as
  `strings` / `trim`. They are not confined to a separate performance dashboard.
  Do not ship the full evidence database, move the benchmark harness, or execute
  benchmarks during documentation builds.
- Keep the approved Base UI + Nova baseline pinned to
  `daaf156ee00ed91cc7e6bdd937c1b4973152d141`. The current scope is 64 component
  entries, three recipes and 30 blocks in
  [the coverage inventory](ui-component-coverage.json). Inventory changes require
  an explicit source diff; an upstream release must not silently expand this task.
- Use only `std:ui`. Source stays editable and project-owned, under `src/app/`
  and `src/components/`. Inline `ui.theme` and explicit `ui.target` remain in
  `tsrust.config.json`. Use ordinary `.tsrx` application logic, JSX returns and
  automatic checked state/callback ownership. No public lifetime attachment,
  callback handles, duplicate reactive components, React runtime or WebView.
- Preserve current compiler/runtime ownership, cancellation, release and cleanup
  rules. Authentication examples contain application policy; the framework does
  not become an authentication product.

[UI_VISION.md](../../UI_VISION.md), the
[current handoff](../handoffs/compiler-to-roadmap-continuation.md) and
[development workflow](../development-workflow.md) still govern architecture and
acceptance. This plan supersedes earlier suggestions that the documentation app,
remaining UI families or native behavior are optional. Historical acceptance
evidence is retained; this plan does not erase outstanding checks.

## Existing support to reuse

The checked `.tsrx` pipeline, automatic component state retention, weak callback
captures, scalar text expressions, conditional mounting, initial nullable children,
text editing/selection/IME, semantic elements, utility styles, configurable tokens,
offline editable installation, versioned filenames, shared attributions, LSP and
interactive CLI already have focused evidence. These are foundations, not proof
of complete Base UI/Nova parity.

Native page scrolling, fractional text layout, persistent Cargo dependency
workspaces and deterministic executable-cache identities are accepted. Warm counter
starts observed at 1,394 and 1,556 ms are startup observations, not benchmark
statistics. Preserve those caches and records. Kbd/Separator still need the
outstanding paired browser evidence. Existing Button/Card versions must not be
relabelled Nova implementations without completing their actual contracts.

Live string-signal palette modes, retained `View.hidden` subtrees and checked
creation-time window options have focused paired acceptance at source `c8699512`.
The macOS custom header preserves OS controls and excludes controls/labels from
drag surfaces. Actual OS drag gestures, live chrome changes and custom headers on
other desktop platforms remain unaccepted. Live token-record editing, generic
reactive props, keyed lists, persistent routing, nested scrolling, overlays/focus
and integrated web build/dev remain framework dependencies. The new
`examples/ui-application-state` fixture preserves the older application fixture
and standalone counter owner files.

The next foundation section is accepted at `0948dca6`: bounded nested scroll
panes, viewport dimensions, checked `View.tabIndex`, focus reveal and retained
pane offsets. The real `docs/` project now has offline-installed local controls
and a paired initial shell. [Acceptance](../milestones/ui-documentation-foundation.md)
records targeted checks and the repaired browser CSS/offset ordering. This does
not complete steps 1–3: content classification, persistent navigation/keyed lists,
full reactive forwarding and the actual reference pages remain pending. A local
preview requested by the owner runs on `http://127.0.0.1:4317/`.

## Project and content layout

Use `docs/` as a separate runnable Tsrust project. Keep the standalone
`/Users/seancotter/Desktop/dev/learning/tsrust-counter` as a focused counter
reproduction; do not silently overwrite its edited controls to create the docs.

```text
docs/
  tsrust.config.json             # explicit web/native targets and project theme
  ui-components.lock.json        # editable bundled component provenance
  attributions/                 # deduplicated third-party notices
  content/
    getting-started/
    language/
    standard-library/
    cli/
    framework/
    ui/
      getting-started/
      foundations/
      components/<name>.md
      recipes/<name>.md
      blocks/<name>.md
      themes/
      authoring-components/
    performance/
  src/
    main.tsr                    # explicit application bootstrap
    app/                        # persistent layout and documentation routes
    components/
      ui/                       # actual editable catalog components
      docs/                     # Markdown, examples, API tables, search, code
    examples/                   # actual runnable examples displayed in pages
  public/                       # required local fonts, images and other assets
  README.md                     # contributor/developer documentation entry
  plans/, milestones/, handoffs/ # retained developer records, excluded by default
```

Generated content/search/highlighting/evidence assets belong under ignored build
output. Their declared source paths and hashes must allow reproducible rebuilding.
No duplicate hand-maintained Markdown/JSON/TSRX descriptions of the same page.
Keep application source and prose separate; a Markdown directory is not an
implicit module or a different UI routing convention.

## Ordered implementation

### 1. Inventory and establish the content contract

- [ ] Classify existing Markdown as current user documentation, design proposal,
  accepted evidence or historical record. There are currently 721 Markdown files;
  bulk moving them before classification would break useful references.
- [ ] Define stable page IDs, directory-derived routes, navigation groups, order,
  title, summary, target capabilities, status and optional example references.
  Reject duplicate IDs, missing assets, unsafe paths and unresolved references.
- [ ] Record each existing component's exact source version, exported pieces,
  declared props, behavioral limits and accepted target evidence. Extend the
  existing coverage inventory rather than creating a competing status ledger.
- [ ] Write the first content pages from verified behavior: installation,
  project structure, state/ownership, theming and existing component APIs.
- [ ] Establish a language-reference pilot for strings and `trim`, including the
  API-to-workload mapping. Reuse accepted measurements where present; record an
  explicit unmeasured state otherwise. Do not wait for the complete UI library
  before proving this documentation/evidence relationship.

Acceptance: every first-page claim maps to current source or accepted evidence;
the inventory accounts for all 64 components, three recipes and 30 blocks.

### 2. Finish the shared UI foundation required by the docs

- [ ] Support live theme mode and token changes without rerunning components,
  remounting examples, losing text/selection or resetting counters. Preserve
  nested theme scopes and reject invalid updates before replacing valid data.
- [ ] Complete checked reactive props and forwarding required by navigation,
  variants, class names, metadata and visibility. Conditional unmount and retained
  hiding must have distinct documented behavior. State remains owned exactly once.
- [ ] Implement nested scroll containers, viewport sizing and resize behavior.
  Share offsets across layout, paint/clipping, pointer targeting, text editing,
  keyboard focus reveal and accessibility. Bubble unused wheel deltas correctly.
- [ ] Establish window options, platform safe areas and native/custom headers.
  Retain real close/minimize/maximize/fullscreen behavior. Custom drag regions
  must exclude controls, disabled controls, labels and menus. Handle display
  scaling, resize, focus, keyboard access and errors without simulated OS buttons.
- [ ] Provide a working way to demonstrate native versus custom headers. Clearly
  distinguish creation-time options from supported live changes. Do not present
  a live toggle that merely restarts the application or resets its state.
- [ ] Complete persistent navigation with shared layouts, typed links, explicit
  not-found pages, web URLs/history/deep links and native route paths. Preserve
  appropriate route state, scroll positions and focus. Establish keyed list
  identity before using filtered navigation/results that can reorder.
- [ ] Wire these features through compiler checks, native and browser behavior,
  LSP completion/hover/parameters/diagnostics, examples and workload support.

Acceptance: one paired witness changes route, theme and viewport while preserving
an edited field and counter; separate witnesses verify intended unmount cleanup,
nested scrolling, header controls and drag exclusions. Unsupported platforms or
capabilities produce explicit diagnostics, not silent fallbacks. A macOS-only
prototype is not proof of universal native-header support.

### 3. Create the documentation shell as the first real consumer

- [ ] Scaffold `docs/` through the ordinary project/config/component workflows.
  Use the real CLI, explicit entrypoint and selected targets. Ensure all required
  project files, offline assets and attribution notices are present.
- [ ] Build a restrained, readable documentation layout: global section navigation,
  persistent component sidebar, article column, optional on-page contents and
  responsive compact navigation. Keep reading width deliberate and controls
  comfortably sized; test narrow windows rather than assuming desktop width.
- [ ] Add working search entry, light/dark controls and native-header controls.
  Keep header content draggable only where intended. Use the actual component
  library; do not introduce a separate HTML or native-only documentation UI.
- [ ] Give every catalog entry a stable navigation destination. Pending entries
  show exact status and dependencies; they must not look like working previews.
  Promote them to complete pages as their component family lands.
- [ ] Replace the counter-only presentation with reusable example cards in this
  project. Keep independent counters and conditional mounting as actual examples.

Acceptance: both targets open the same real pages, navigate by keyboard/pointer,
scroll independently, resize without jumping/cutoff, and switch themes. No
decorative or inert controls. This first shell is not final library acceptance.

### 4. Render Markdown and source code through shared UI

- [ ] Use a maintained CommonMark parser with explicitly enabled GFM extensions,
  not a line-splitting approximation. Inspect dependency reuse before admitting
  a new dependency. Pin and package it for offline builds.
- [ ] Parse Markdown during content builds into a typed document representation
  with stable heading IDs, source ranges, links and example references. Render
  the same representation through local Tsrust Markdown components on both
  targets. Browser rendering must not depend on native filesystem access.
- [ ] Support headings, paragraphs, emphasis, strong text, inline code, links,
  lists, blockquotes, rules, images, fenced code and GFM tables/task lists/
  strikethrough. Define relative-link/asset resolution, duplicate heading anchors,
  alt text, long lines, Unicode and malformed input behavior.
- [ ] Treat raw HTML and unsafe URLs as an explicit trust boundary. Documentation
  cannot execute arbitrary scripts or imports from Markdown. Unsupported syntax
  is diagnosed or rendered as literal content under a documented policy.
- [ ] Use compiler/editor tokenization for Tsrust/TSRX highlighting, including
  JSX closing tags, strings and async syntax. Reuse a maintained highlighter for
  other documented languages where needed; unknown fences remain readable text.
  Native rich-text runs must wrap/select/copy correctly across token boundaries.
- [ ] Preserve original source exactly for copy/download, independent of rendered
  highlighting and line numbers. Implement real clipboard actions with observable
  success/failure; retain keyboard selection/copy. No fire-and-forget failures.
- [ ] Bind interactive examples through a checked local registry of real `.tsrx`
  exports. Preview, displayed source and copied source reference the same files.
  Markdown content itself does not evaluate arbitrary code.

Acceptance: parser conformance/extension cases, safe-link rejection, exact copied
Unicode/code, accessible headings/tables and real web/native rendered pages.
Document supported syntax without claiming MDX or arbitrary HTML compatibility.

### 5. Implement offline full-text search

- [ ] Build one index from the typed content: titles, headings, prose, API names,
  signatures, descriptions and code. Include stable page/heading destinations and
  original text ranges for excerpts. Exclude historical evidence by default.
- [ ] Rank exact names and titles ahead of ordinary body matches. Define Unicode
  normalization, case folding, punctuation, multi-term and empty-query behavior.
  Preserve original text for display; avoid arbitrary regex execution.
- [ ] Provide keyboard opening/closing, result navigation, highlighted excerpts,
  clear empty states and accessible announcements. Selecting a result navigates
  to its heading and restores sensible focus on both targets.
- [ ] Keep indexing at build time, queries local and searches free of network
  dependencies. Measure representative index size/startup/query workloads at the
  documentation-platform milestone before introducing a worker or extra cache.

Acceptance: matched query/result fixtures for prose, props and code; offline
browser/native interaction; stale or partial indexes cannot silently hide pages.
Semantic search remains future work and must share these stable content IDs.

### 6. Establish the complete component-page contract

- [ ] Each page contains purpose, installation/imports, a working default preview,
  preview/source switching, exact copyable source and meaningful variants.
- [ ] Explain composition, every exported component, every public prop/default/
  error, controlled/uncontrolled behavior, callbacks, keyboard use, accessibility,
  customization, ownership and target differences where relevant.
- [ ] Derive checked signatures from actual source and reuse IntelliSense
  documentation. Keep manually written behavioral explanation alongside it;
  generated signatures alone are not documentation.
- [ ] Show relevant empty, disabled, invalid, read-only, loading, long-content,
  narrow-layout and custom-theme cases. Do not invent meaningless states for
  static components merely to populate a uniform checklist.
- [ ] Keep examples interactive and editable in local source. Switching source
  views or theme must not unexpectedly reset preview state. Validate every
  displayed runnable snippet with the current compiler and formatter.

Acceptance: the first completed family exercises this template end to end;
subsequent families extend real pages rather than accumulating undocumented APIs.

### 7. Complete the static and structured component families

- [ ] Finish Nova fidelity for existing Button/Card/Input/Textarea/Label/Kbd/
  Separator sources, preserving installed owner files and versioned upgrades.
  Reuse accepted behavior and finish only missing clone/target evidence.
- [ ] Implement remaining semantic content, button grouping and structured
  components. Finish shared rich text, layout or composition gaps once at their
  responsible layer rather than patching each preview.
- [ ] Finish editable form composition and basic stateful controls before using
  them in the theme editor or complex block examples.

Use the complete family assignments below. Each family includes sources, offline
installation, docs pages, LSP and paired behavior/accessibility acceptance.

### 8. Complete focus, overlays and composite navigation

- [ ] Establish reusable focus scopes, roving focus, modal/inert behavior, focus
  restoration, Escape/outside dismissal, anchored positioning, clipping, layering
  and nested-overlay ordering. Preserve capture cleanup when overlays disappear.
- [ ] Implement menus, popovers, dialogs, selects, comboboxes and commands using
  those primitives. Match Base UI interactions rather than only copying styling.
- [ ] Cover mouse, keyboard and platform text-input/IME interactions, scrolling
  anchors, resize/repositioning, disabled items and nested controls.

Acceptance: component-specific keyboard and accessibility cases plus nested
combinations such as a searchable select inside a dialog. Documentation search
may adopt Command only after its real behavior is accepted.

### 9. Deliver theme browsing, customization and generation

- [ ] Provide token reference pages and actual light/dark theme previews across
  representative content, forms, overlays, charts and blocks.
- [ ] Build a theme generator with editable semantic colors, paired light/dark
  palettes, radius, spacing and typography supported by the shared runtime.
  Explain defaults, aliases, color conversion/gamut and validation errors.
- [ ] Apply valid edits live without resetting the preview. Invalid drafts remain
  editable while the last valid theme stays active. Preserve aliases and numeric
  precision; do not silently clamp unsupported colors or round persisted tokens.
- [ ] Show contrast feedback for relevant text/surface/focus combinations without
  treating one ratio as proof of full accessibility. Support reset, comparison,
  import and exact export to the `ui.theme` section of `tsrust.config.json`.
- [ ] Export/copy a config patch by default. Explicit CLI/local saving previews
  changes and atomically updates only the selected theme fields, preserving
  project targets, unrelated settings and owner edits. A hosted website cannot
  silently write to a user's project.

Acceptance: customized tokens across both targets, invalid/cyclic aliases,
transparent-background rejection, extreme allowed scales, import/export round
trips and offline operation. No separate `ui.theme.json` convention.

### 10. Complete advanced components and recipes

- [ ] Finish media loading/decoding, motion/reduced-motion, progress, drag gestures,
  resizing, virtualized/long content where required and accessible announcements.
- [ ] Finish calendar/date selection using existing exact date/time APIs. Distinguish
  dates, instants and time zones; application locale/time-zone policy is explicit.
- [ ] Finish chart/data-table inputs, exact numeric conversions, null/empty data,
  meaningful labeling and nonvisual data access. Use existing checked data types;
  do not silently convert exact decimals or wide integers to lossy chart numbers.
- [ ] Deliver data-table, date-picker and typography recipes with all required
  dependency components, rich examples and documented extension points.

Acceptance: all assigned components and three recipes meet the component-page
contract; performance claims wait for their measured workloads.

### 11. Complete every block and layout gallery

- [ ] Install and render all 30 pinned blocks from real local component sources.
  Include actual source previews, dependency/file lists and copy/install actions.
- [ ] Provide viewport examples, light/dark/custom themes and working menus,
  sidebars, forms, tables and charts. Form submission/authentication uses clearly
  identified application examples, not a new framework auth layer.
- [ ] Package local licensed assets for offline use. No placeholders that depend
  on external images, fonts or an authenticated source download.
- [ ] Cover multiple layout compositions, empty and long data, narrow screens,
  resizable panels, RTL where supported and native-header combinations.

Acceptance: each block compiles/installs offline and works on both selected
targets. Dependency imports resolve to preserved versioned filenames. Gallery
metadata is not counted as block implementation.

### 12. Make custom base-component authoring practical

- [ ] Stabilize the primitives already used by real library components: semantic
  elements, state/bindings, focus, overlays, selection, slots/composition, styles,
  callbacks, accessibility and target capabilities.
- [ ] Document how to author a Button-like control and a composite popup from
  application-owned `.tsrx`/`.tsr` code, with a checked Rust adapter only when an
  actual platform capability requires it. Keep lifetime machinery internal.
- [ ] Extend offline catalog authoring/validation for user components, dependencies,
  files, attribution and metadata. Use the existing transactional installer and
  preserve-on-upgrade model rather than a second package/install system.
- [ ] Add generation/help only where actual authoring examples establish a stable
  contract. Explain target-specific implementations and errors explicitly.

Acceptance: a user creates, installs, edits, documents and tests an original base
component without modifying compiler internals or copying private runtime code.

### 13. Publish language documentation and curated benchmark evidence

- [ ] Migrate current language, standard-library, CLI and framework guides into
  the organized content directories, one coherent topic family at a time.
  Preserve authoritative distinctions between behavior, proposal and evidence.
- [ ] Validate links/anchors and update known callers before each move. Maintain
  explicit redirects for published/stable old routes without duplicating prose.
  Keep retained acceptance/historical evidence at its original path.
- [ ] Export selected accepted SQLite records with source hashes, compiler/native/
  Node versions, workload definitions, protocols/sample counts, units and run IDs.
  Build a deterministic, public-safe snapshot and verify exact source restoration.
- [ ] Explain language behavior using runnable examples alongside the relevant
  measured workload, methodology, distributions and limitations. Preserve slower
  results, failed historical runs and unmatched/unmeasured distinctions.
- [ ] Build Node-style reference sections for each documented language/container/
  resource type and standard-library module, with an on-page method/property
  index and stable anchors. A `strings` page contains a real `trim` section, not
  merely a link to a general benchmark page. Follow the method contract below.
- [ ] Embed method-specific charts/comparison tables beside their API explanation,
  using the same snapshot and filters as the overview graphs. Link each result to
  its workload, input characteristics and original measurement provenance.
- [ ] Build overview graphs by feature family, operation and input size, with
  visible runtime/toolchain/machine/run filters. Explain inclusion/exclusion and
  aggregation. Never average unrelated units or present selected fast cases as
  overall language performance.
- [ ] Add accessible charts/tables and useful filters without implying unrelated
  hardware, versions or measurements are comparable. Keep original benchmarks,
  drivers, oracles, registrations and SQLite as the canonical sources.

Acceptance: docs rebuild offline from curated exports without SQLite access;
every displayed measurement traces to exact original evidence. No new timing
run is required merely to publish an accepted existing measurement.

#### Language and method reference contract

Use [Node's API reference](https://nodejs.org/api/buffer.html#bufsubarraystart-end)
as a reading/navigation reference: recognizable method headings, typed arguments,
returns, behavioral explanation and examples. Write original Tsrust documentation
from checked behavior. The benchmark integration is an additional Tsrust feature.

A method section, for example `strings#trim`, contains:

1. **Purpose and signature.** What it does, receiver type, overloads and type
   parameters where applicable. Display checked public syntax.
2. **Arguments.** Names, types, required/optional status, defaults, valid ranges
   and errors. For a method without arguments, state that directly.
3. **Return and behavior.** Exact return type, receiver mutation, absence/failure
   behavior and useful edge cases. Put its concise ownership/borrowing explanation
   after observable behavior. Explain relevant native differences directly.
4. **Examples.** Copyable, compiler-verified source with expected results; relevant
   Unicode, empty input or boundary cases. Examples and editor help must agree.
5. **Measured performance.** Comparable Tsrust/native versus Node-after-tsc results
   for this method and workload. Show input kind/size, units, distributions or
   uncertainty supported by the original protocol, sample count and versions.
   Allocation data and direct Rust comparisons appear only when actually recorded.
6. **Evidence and limitations.** Link workload source/oracle, original run identity,
   compiler/source hashes and full methodology. Make mismatched semantics,
   historical versions and missing evidence visible. Show “No accepted measurement”
   when appropriate, never invented data, zero bars or an unrelated method's ratio.

Assign stable API IDs independent of file paths. Map workloads explicitly to one
or more API IDs and retain their input parameters and semantic scope. A workload
that includes several operations must not be mislabeled as an isolated method
measurement. Aggregate pages and inline sections resolve those same identities;
do not copy results into Markdown by hand. Preserve historical results when
current source changes; label their applicability rather than silently calling
them current. Native runtime throughput, browser performance, compilation time and
development startup are separate measurements.

First acceptance uses the strings/`trim` pilot plus an overview assembled from
the same curated records. Expand through arrays, collections, resources and
standard-library APIs as their current documentation is migrated. Benchmarks are
present throughout the reference even when a particular method is unmeasured.

### 14. Finish CLI integration and project-wide acceptance

- [ ] Integrate content generation, Markdown/highlighting and search invalidation
  into ordinary project build/dev. Finish actual web target build/dev instead of
  requiring users to discover a repository-only browser script.
- [ ] Reuse persistent dependency/executable caches with exact invalidation;
  content edits rebuild affected content, not every native dependency. Do not
  introduce arbitrary cache quotas or orphan development processes.
- [ ] Ensure help and interactive CLI expose documentation/component/theme
  workflows coherently, with compact rows and space outside selection lists.
- [ ] Finish keyboard, screen-reader, visual, resize, offline export/install and
  paired target acceptance for the integrated project. Refresh installed compiler/
  LSP consumers after accepted source checkpoints and retain exact settings.
- [ ] Run complete compiler/LSP regressions and accumulated matched measurements
  at meaningful feature-family milestones. Reuse completed evidence and rerun
  only affected failures. Import/verify closed records in SQLite and remove only
  verified duplicates; prune obsolete outputs and reap owned processes.

Acceptance: a clean checkout plus documented offline toolchain builds the docs;
new contributors can add a page/component/example without undocumented manual
steps. Public claims, component inventory, editor help and actual behavior agree.

## Complete component and block assignments

Assignments order dependencies; they do not assert implementation acceptance.
The machine-readable coverage inventory remains the source of per-entry status.
Every entry below requires its own useful page and examples, including exported
subcomponents. Dependencies can land earlier when the docs application needs them.

| Family | Required components |
| --- | --- |
| Semantic content | alert, badge, button, button-group, card, empty, item, kbd, label, separator |
| Layout and navigation | aspect-ratio, direction, resizable, scroll-area, sidebar |
| Structured content | accordion, breadcrumb, collapsible, pagination, table, tabs |
| Forms and stateful controls | checkbox, field, input, input-group, input-otp, native-select, radio-group, switch, textarea, toggle, toggle-group |
| Focus and overlays | alert-dialog, combobox, command, context-menu, dialog, drawer, dropdown-menu, hover-card, menubar, navigation-menu, popover, select, sheet, tooltip |
| Media, motion and feedback | avatar, carousel, progress, skeleton, slider, sonner, spinner, toast |
| Calendar and data | calendar, chart |
| Extended content | attachment, bubble, code-block, marker, message, message-scroller, questionnaire, trace |

Recipes: **data-table, date-picker, typography**.

Blocks: **dashboard-01; login-01–05; preview, preview-02, preview-03;
sidebar-01–16; signup-01–05**. Preserve exact inventory descriptions, pinned source
identities and dependency lists. Do not count a number range as permission to
skip an individual variant.

## Acceptance and execution discipline

Work in coherent sections across compiler/runtime/editor/application source before
running checks. Add focused witnesses during implementation. Tests verify state,
ownership, events and observable output; avoid tests that only restate code.

Every completed component family needs applicable evidence for:

1. Checked compiler behavior, diagnostics, formatting preservation/idempotence
   and generated native execution of documented examples.
2. Real web/native interaction, keyboard/focus, accessibility semantics and
   intended cleanup. Automated semantic checks supplement visual/platform review.
3. Default, dark and customized themes; long/localized text, relevant disabled/
   invalid states, narrow/wide layouts and scaling. Motion respects reduced motion.
4. Editable offline source, transactional dependencies, safe paths, idempotence,
   versioned upgrades and shared attribution retention in exports.
5. Complete IntelliSense signatures, completion, hover, parameter help and
   actionable incomplete-source diagnostics for the actual API.
6. Matched workload sources, independent oracles, source identities and observers
   ready for the next meaningful measurement milestone. Untimed is not measured.

Use one writer/build owner, `CARGO_INCREMENTAL=0`, one shared Cargo target and
the existing development workflow. Format the whole project before each source
checkpoint. Preserve unrelated owner files. Do not rerun accepted S18 database
suites unless relevant SQL or task-lifecycle changes require the explicit
three-driver runner. No archive directories, source snapshots or sweeping cleanup.

## Immediate execution order

1. Finish step 1's content/coverage contract and first migration map.
2. Finish step 2's live themes, visibility, nested scrolling and native header
   contract as one section, including editor/docs/witnesses. Navigation/list
   dependencies follow before building an unreliable docs sidebar.
3. Build the paired docs shell with real existing examples, then finish Markdown,
   code rendering and full-text search. Keep both targets usable while iterating.
4. Complete component families with their pages, using them to build the theme
   generator, advanced examples and complete block gallery.
5. Migrate public topic docs and publish curated evidence incrementally; finish
   authoring and CLI workflows as their actual contracts become established.

No additional go-ahead is required for these agreed steps. Ask only for material
unresolved behavior/product choices or actual blockers. Keep implementation status
and deferred measurement status separate, and update this plan when evidence
changes the dependency order.

## Reference material

Public references inform design; the pinned inventory controls implementation
scope and source fidelity. No authenticated GitHub connection is required.

- [shadcn component documentation](https://ui.shadcn.com/docs/components)
- [shadcn theming](https://ui.shadcn.com/docs/theming)
- [shadcn blocks](https://ui.shadcn.com/blocks)
- [Base UI accessibility](https://base-ui.com/react/overview/accessibility)
- [GitHub Flavored Markdown specification](https://github.github.com/gfm/)
- [Current UI application guide](../guides/ui-applications.md)
- [Canonical analytics storage](../../benchmarks/storage/README.md)
