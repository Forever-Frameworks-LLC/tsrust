# Tsrust continuation

This is the only handoff. Read AGENTS.md, docs/agent-goal.md and the complete
B00–B20 checklist in docs/plans/stdlib-0.2.0.md. Full goal remains active;
v0.2.0 merge/publication is not approved. v0.1.2 is released.

## Current continuation — 2026-11-08

### Gallery as Kit docs mirror; API coverage tracking — 2026-11-09

Owner: the starter must mirror GPUI Kit's component docs (left nav, every Kit
example live with copyable .tsrx) and every component must cover GPUI Kit's
full public API, not just the docs. `tools/kit-coverage.py` inventories public
methods from the locked gpui-component/gpui-base sources (184 types, 1765
methods; gpui-component types shadow same-named gpui-base primitives) against
kit.json `covers`; `--check` gates completeness. Done so far: Button (all but
tooltip_with_action, which needs a std:ui actions/keybinding model shared with
menus), ButtonGroup (Button children), Toggle, ToggleGroup. Gallery: generated
from examples/gpui-starter/pages.spec by tools/generate-ui-gallery.py; custom
title bar (renderer now honors titlebar: 'custom'), tabbed Preview/Code cards,
Gruvbox theme set via new registerThemes/useTheme; std:ui utility colors now
follow the active Kit theme. Icons: all 1830 Kit catalog names (IconName is a
string; a literal union stalled rustc); programs naming non-default icons
register AllAssets. Display batch: Badge, Tag, Label, Kbd (command prop reads
Shortcut bindings), Separator, Skeleton, Spinner, Progress/ProgressCircle,
Avatar/AvatarGroup, Alert, Icon (svg, rotate), ShimmerText, Empty/EmptyMedia,
GroupBox/GroupBoxFooter, Clipboard and Link (disabled) are complete, with 15
gallery pages. Forms batch: Checkbox, Switch, Radio (also standalone),
RadioGroup and Rating are complete with 4 pages (coverage 374/1765). Styled Kit components in STYLED_KIT take
className as their own style refinement instead of a wrapper. Fixed: every
ScrollArea shared one scroll offset (Kit keys Scrollable state by call site
without an explicit id); gallery routes now each own a scroll area; sidebar
groups merge across the spec; utilities gained rounded-md and rounded-full.
Owner review fixes (examples must work when exercised and read like shadcn):
Kit text content comes from children (`<Label>Name <Text variant="secondary">
(optional)</Text></Label>`, `<Checkbox>Accept</Checkbox>`, `<MenuItem>Copy
</MenuItem>`); kit.json `textChildren` mirrors kit.rs `text_children_prop`
(generate-ui-kit.py checks agreement). Text gained `variant`. Alert takes
AlertTitle/AlertDescription; Empty takes EmptyHeader/EmptyTitle/
EmptyDescription/EmptyContent and EmptyMedia variant="icon". Form controls are
uncontrolled when `checked`/`value`/`selected` is unset (defaultChecked,
defaultValue, defaultSelected set the start). DropdownMenu is a plain button
that opens the menu; onClick or split makes a split button. Remote image src
loads through a reqwest HTTP client (gpui-http feature, selected for Avatar and
Image). Empty Avatar names no longer hide the placeholder icon. Colors: utilities
and Kit color props accept the Tailwind 4.1 palette (gamut-mapped to sRGB hex)
and arbitrary [#hex]/[oklch(...)] values. Every gallery page was exercised
(menus, split button, context menu, checkboxes, switches, toggles, ratings,
alerts). Inputs batch: Input and Textarea map GPUI Kit's Input/Textarea and
InputState/TextareaState (masking, masks, number formatting, regex pattern,
search and replace, rows/auto-grow, wrapping, tabs, focus/blur/submit events,
inline tokens); imperative calls go through controlInput(id, action) and
readInput(id, onRead) over the command queue. InputGroup is now shadcn-style
composition (Input/Textarea child plus InputGroupAddon > InputGroupText /
InputGroupButton, which opens a menu from MenuItem children); NumberInput gained
min/max/step (default 1), affix icons, sizes and defaultValue. Four new pages,
all exercised. tools/kit-coverage.py now matches braces lexically (its
test-module stripper had swallowed real impls) and maps InputBaseState aliases,
so the honest total is 701/2147. Still open on inputs: validate/step_by and
on_paste need synchronous TS callbacks; custom context menus and custom token
rendering. Select and Combobox are complete with pages: options are SelectItem
children (value + text label, disabled) optionally in SelectGroup with a
SelectLabel, or the items prop; SelectEmpty supplies empty-list text (Kit renders
it from &App, so text only); Combobox adds multiple, query, checkIcon and
ComboboxTrigger / ComboboxFooter slots. Language limits found: reactive props
must be scalars (array props are static; Combobox value is newline-separated)
and owned callbacks are callable only as parameters (kitOnList adapts
string[] events). Tabs (Tab/TabsContent by value,
prefix/suffix/empty-space slots, folder variant, menu), Tooltip (keys, command,
TooltipContent), Popover (open/defaultOpen/onOpenChange, anchor, offset, arrow,
right-click, overlayClosable) and Accordion (value/defaultValue,
AccordionTrigger/AccordionContent, styles) are complete with pages; the
Example card now uses value-based Tabs. Coverage 856/2147 (Tooltip's open
methods belong to the Plot chart tooltip). Owner direction 2026-11-09: finish a
page for every component, then add semantic search to the gallery, rebuild
Getting Started from the real docs as the single source of truth, and add an
animated showcase page (use the design-taste-frontend skill). Since then: Dialog and Sheet
(Trigger/Content/Header/Title/Description/Footer/Close parts), AlertDialog,
notify options and events, HoverCard, Collapsible, Breadcrumb, Pagination,
Stepper, DescriptionList, Slider, OTP, TimeField, Calendar, DatePicker,
ColorPicker, and a layout batch (Sidebar with nested items and item menus,
SidebarToggleButton, Resizable, Toolbar groups, Form/Field, StatusBar,
TitleBar close, WindowBorder) — each with exercised pages. Utilities gained
justify-end/around/evenly and items-end/baseline. Coverage 1300/2147. Remaining
types: Table/DataTable/List/Tree states, Command, Carousel, Diff, Editor,
Attachment, Bubble/Message/MessageScroller, Marker, Questionnaire, Settings,
Speech, AppMenuBar, the Plot tooltip, and input sync callbacks (validate,
onPaste, step_by) plus custom input context menus and token rendering. Then
semantic search, Getting Started from real docs, and the showcase page.
Next batch (2026-11-10): Command palette (groups, separators, shortcuts,
header/footer/empty, controlled query, loading, highlight vs select events),
Carousel (CarouselContent/CarouselItem/Previous/Next/Pagination, loop,
vertical, controlled `selected`), composable Table (header/body/footer/caption,
per-part className, align, colSpan, size) and List over a custom ListDelegate
(ListItem rows render their own children, ListItemSuffix, ListSeparator,
ListGroup sections, ListEmpty, search, confirmed/check icon, controlled
selection/query, hasMore/onLoadMore) and Tree (TreeItem text children,
icons, defaultExpanded, disabled, controlled `selected` that reveals hidden
items, onExpand/onCollapse, TreeMenu right-click menu with onContextMenu). JSX
children accept spread arrays (`{...rows.map((row) => <Row />)}`); a spread
that reads component state lowers to a reactive list slot (runtime
`Node::each`: transparent like branches, rebuilt on refresh, rows whose shape
is unchanged keep their identity and state; `visible_children` flattens
slots). Spec sections may define interfaces before the example function, and
generate-ui-gallery.py rejects unknown icon names (a typo rendered nothing).
Fixed while exercising: List section headers overlapped rows (Kit measures
header height from section 0), disabled List rows still reported, confirmed
rows drew no check (Kit needs an explicit check icon), Tree `selected` could not
reveal collapsed items, Carousel parts ignored className, Sidebar header/footer
`collapsed` needed Kit's Collapsible trait in scope. Open defect: in the gallery Table page, consuming a
read-only callback parameter's string field twice (`name={person.name}` and
`{person.name}` inside nested cells) made `tsrust check` run for minutes in
analyze_recursive_with_proofs instead of reporting callback-access; small
repros report the error in under a second, so the pages use clone() for now.
Reading a string field in a consuming position should copy rather than
require clone().
Second batch (2026-11-10): DataTable is composable (DataTableColumn children
with key/width/min/max/sortable/defaultSort/align/fixed/resizable/movable/
selectable/padding; rows as JSON; row, column or cell selection; sort, column
move/resize, right-click and DataTableMenu, empty/loading/load-more events;
controlled selectedRow). Chat family: Message parts (MessageAvatar/Header/
Content/Footer, MessageGroup, a Bubble inside follows the message side),
Bubble variants/BubbleContent/BubbleReactions/BubbleGroup, Marker parts,
loading style and shimmer, Attachment parts (AttachmentMedia src or icon,
AttachmentContent/Title/Description, AttachmentActions, tooltip, vertical,
size), AttachmentGroup edge fade, MessageScroller styling props. New Ticker
component (GPUI timer, onTick(frame), interval, running) drives animation.
Command gained CommandItem `value` and a Command-level onSelect(value).
Gallery: Getting Started pages are generated from docs/guides/ui/*.md and
docs/guides/tooling/desktop-apps.md (single source; ```tsx live Name``` blocks
compile into the gallery as live examples); title-bar search (⌘K) ranks
pages and examples over a generated index with concept synonyms ("modal"
finds Dialog); an animated Showcase page (60 Hz plot, 160-cell wave field,
spectrum, orbits, auto carousel). Fixes: `tsrust new --ui` template did not
compile against the current API (now checked by a scaffold test); runtime UI
errors exited silently with status 0 because GPUI exits inside its run loop
(now printed as Uncaught and exit 1); Plot colors ignored palette names;
arbitrary pixel offsets (left-[150px]) are supported. Coverage 1686/2147.
Remaining Kit types: Diff, Editor, Questionnaire, Settings, Speech,
AppMenuBar, Plot tooltip, SearchableListItemElement, input sync callbacks.
Known limits found: Kbd/controls inside Text or Button are rejected only at
runtime; lazy part children (DialogContent) cannot own a reactive list's
captures, so build rows in a helper function.

### Plot and Speech — 2026-11-09

Owner: wrap the whole GPUI surface (low-level included) and trust GPUI behavior;
verify our wrapper. Plot/PlotLine/PlotArea/PlotBars paint through GPUI Kit's plot
primitives (shared band x and linear y scale, axis labels, dashed grid).
SpeechButton (locale, waveform; onPartial/onFinal/onError) uses SpeechState with
SystemRecognizer; `gpui-speech` (Kit `speech`: cpal and platform audio, 36 locked
additions) is selected only when a program uses it, alongside `gpui-highlight`.
`tsrust package` writes NSMicrophoneUsageDescription and
NSSpeechRecognitionUsageDescription. `ui add` follows multi-item children lists.
Plot rendering checked; dictation itself not exercised. 112 components.

### GPUI highlighting, charts and setup — 2026-11-09

`gpui-highlight` feature (GPUI Kit syntect: +syntect, two-face, fancy-regex,
bincode, bit-set 0.8, bit-vec 0.8; existing pins unchanged) is selected only when
the emitted program uses CodeEditor (runtime marker); CodeEditor and Markdown code
blocks highlight. Chart gains radar, candlestick (open/high/low/close) and sankey
(links). Fixed: `setup --dependencies` fetched only the backend set, so fresh
machines could not build std:ui apps offline; it now fetches the GPUI set too, and
the starter's dev.sh runs setup first. Windows/Linux GPUI
runs remain unverified here. Tests: the test harness now defaults
CARGO_TARGET_DIR (repository target) and TSRUST_SOURCE_PACK_DIR (Cargo home packs)
like CI; four shared_sources tests had failed on every local run without them.

### GPUI CLI, Questionnaire, Svg and doc rewrite — 2026-11-09

Owner: no signing or published release; users run the source. The starter has
`tsrust.config.json` and `examples/gpui-starter/dev.sh` (builds this checkout's
compiler, then `tsrust dev`: ~20 s rebuild and restart on save; cold first build
~3 min). `tsrust new <name> --ui` copies the starter; `tsrust ui list|add` writes
editable wrappers (component plus its item components) into `src/components/`;
all 75 wrappers check and a generated project builds. Questionnaire/Question/
QuestionChoice (submit emits JSON answers) verified by input. Fixed: Svg/Path drew
nothing on GPUI; they now render as SVG images with resolved theme colors
(`Node::svg_markup`). Route, Link, navigate, back and CopyButton verified on GPUI.
The 27 std:ui IntelliSense entries and api.tsr comments no longer describe the
removed browser/Vello renderer; every std:ui entry now has a tsrx example, and
the LSP catalog tests accept tsrx fences and optional console examples for std:ui
(they had failed since std:ui returned). The runtime_build dependency-reuse test
only asserts a shared Cargo target when CARGO_TARGET_DIR is set (it failed on
every local run without it). 107 components.

### GPUI services, packaging, updates and starter — 2026-11-09

Branch `codex/gpui-ui`. Since the catalog pass: application services (notify,
window control, clipboard, file dialogs, extra windows, appearance, theme),
Icon/Animate, font utilities, i18n (`t`/`tWith`/`translate`/`translateWith`,
JSON catalogs, plurals, live locale switching, Kit strings follow), catalog
batch 2 (Markdown … ButtonGroup) and batch 3 (InputGroup, Diff, Marker,
Attachment/AttachmentGroup, DataTable, Settings pages/groups/items,
MessageScroller) and Dock/DockPanel (drag, stack and split verified): 104 generated components. Kit 0.9.0 has no GridLayout (docs
are ahead of the pinned crate). Not wrapped: Plot
(closure drawing API; Chart covers data plots), Questionnaire, Speech (needs the
`speech` feature and cpal), and code highlighting (syntect/tree-sitter features).
`tsrust package` (cli/app_package_cli.rs) builds release, writes the `.app`
(icon, Info.plist, app.json), optional codesign/notarytool/stapler, zip and DMG on
macOS, zip on Windows, tar.gz and .deb on Linux, plus SHA256SUMS. The std:ui
updater (tsrust-ui/src/update.rs; `appInfo`, `checkForUpdates`, `installUpdate`,
`relaunch`) reads GitHub `releases/latest` through system curl, verifies the
archive digest and macOS signing team, and swaps in place. Check verified against
the live API; install path verified up to the missing asset; packaged `.app`
launches. `examples/gpui-starter` is the demo (seven pages, en/fr, dark mode,
About & updates), verified with real input. Guide: docs/guides/tooling/desktop-apps.md.
Open: Developer ID identity and notarytool profile choice, a release repository
for the demo (owner approval before publishing), end-to-end install from a real
release, the remaining components above, CLI `new --ui`/`ui add`, and rewriting
the restored std:ui docs.

### GPUI Kit catalog pass — 2026-11-09

Branch `codex/gpui-ui`. One spec (`stdlib/ui/kit.json`, 62 components including
items and slots) drives `tools/generate-ui-kit.py`, which writes the typed `std:ui`
wrappers (`kit.tsr`, formatted) and their IntelliSense entries; `--check` guards
drift. The runtime has one generic Kit node (string props, live props through
text bindings, typed events `kitOn{,Boolean,Int,Number,Void}`); `gpui_kit.rs`
holds the renderer arms by pattern (display, controlled, Kit state entities,
overlays through a weak root handle, item containers). Compiler fixes found on the
way: literal and literal-union template interpolation (`TsrTemplateText`), and
text bindings over literal-typed getters. Native cache readers now wait up to 2 s
for short copies by other processes (dev reuse flake under parallel tests).
`experiments/gpui-kit-admission/tsrust/gallery.tsrx` renders every component
(screenshots checked). Lib 1541/47/25/6, tsrust-ui 108.
Open: event and overlay interaction proof (headless GPUI test platform, since OS
input injection is unavailable while the owner uses the machine); GPUI rendering
for Svg/Path, Link navigation, CopyButton and visibility callbacks; rewrite 27
restored std:ui docs that still describe the removed browser/Vello renderer; then
CLI (`new --ui`, `ui add/list`) and the starter.

### GPUI bridge: first slice — 2026-11-09

Branch `codex/gpui-ui`. Owner chose to restore the checked reactive lowering
(signals, computed, text bindings, branches, controlled inputs) rather than add
generic typed native callbacks first. `std:ui` keeps its name; `crates/tsrust-ui`
again holds the Node model, utility classes and theme without any renderer;
`stdlib/ui/gpui.rs` renders Nodes through one GPUI root view (Kit Button/Input
for unclassed controls, utility classes mapped to GPUI styles). `gpui-dependencies/`
is the backend manifest plus GPUI Kit with its own lock (892 packages, backend pins
unchanged). UI window programs keep the OS main thread. `check` hides bundled
`std:` contracts. Slice in `experiments/gpui-kit-admission/tsrust` matches the
Rust reference; startup 244 ms vs 253 ms median. Lib 1541/47/25/6, tsrust-ui 107.
Next: Textarea, focus/active states, Kit theme sync, then CLI (`new --ui`,
`ui add/list`), catalog and starter. Raise CI job timeouts to ~3 h with the next
CI change (owner: only test execution has the one-hour limit).

### Rust 1.99.1 pin — 2026-11-09

Branch `codex/gpui-ui`. The backend and repository pin move from 1.96.1 to exactly
1.99.1 (GPUI Kit needs post-1.96 APIs). Exact, not minimum: the rustdoc JSON reader
is format-specific, now format 61 with a regenerated trimmed shlex 2.0.1 fixture.
Active docs, scripts and tests moved; measurement records keep their recorded
toolchain. Also moved the ownership witnesses used by tests out of the deleted UI
experiments into `crates/tsrust-compiler/tests/fixtures/owner-handles`, and
recorded the GPUI bridge design in the platform plan. Local acceptance on 1.99.1:
clippy correctness; lib 1540/47/25/6; host targets acceptance, async_io,
native_dependency_bundle, project_commands, rust_adapter_generation, rust_bridge,
rust_data_boundaries, toolchain_installation, named_handler_specialization,
tracked_resources; seven pinned LSP targets; 134 runtime suites (415 s, cold cache).

### GPUI section: earlier UI removal — 2026-11-09

Branch `codex/gpui-ui`. Both earlier UI attempts are removed: `crates/tsrust-ui`,
`std:ui`/`std:ui-runtime`, UI analysis/emit, `ui_project`, the UI CLI, dev web
and docs app (`docs/src`, `docs/content`), the UI catalog, UI examples, tests and
benchmarks, and the React/Electron/Expo/shadcn traces. The `.tsrx` parser,
formatter and JSX syntax stay; JSX lowers to plain component calls. The native
dependency manifest dropped its UI crates (571 to 326 locked packages) and native
support always builds from the shared backend set. `tsrust ui`/`new --ui` report
GPUI Kit support in development. Fixed in passing: LSP snapshots record
`tsrust.config.json` as a loaded source (previously implied by UI config reads);
`testing_runtime` stubs the capture reset; four test helpers lex with the new
`tsrust_compiler::source_tokens`.

Local acceptance: compiler lib 1540, CLI 47, LSP lib 25 and bin 6; 134 runtime
suites (370 s, 0 failed); affected host targets (acceptance, interactive_cli,
native_cache, native_dependency_bundle, native_package_export, package_management,
project_commands, router_contract, testing_runtime, toolchain_installation,
url_features) and LSP targets (stdio, api_project, editor_consistency,
completion_matrix, checker_agreement, project_intelligence) pass; API audit PASS
at 1,583 identities (102 UI rows removed); clippy correctness, format check and
tooling tests pass. Next: toolchain minimum-version policy, then the GPUI bridge.

PR #16 CI fix `f324a669` (Windows `normalize` output, UTF-8 suite output, no `tsc`
in host tests) is pushed; its CI run is pending.

### Outstanding-issue fixes and v0.2.0 preparation — 2026-11-09

- Fixed: `captureOutput` now ends its capture in `finally`, so a throwing
  callback no longer swallows later output in the same test (the earlier
  error-effect failure was the missing effect-table entries, now present).
- Fixed: no compiler or library-test warnings remain (unused console imports;
  test-only runtime includes marked as such; `pin_project!` import annotated).
- Fixed: `tsrust dev` uses `sigaction` (persistent handler, explicit restore)
  instead of raw `signal()`, removing libc-dependent reset semantics behind the
  unexplained Linux dev-session exit failure.
- Fixed: HTTP server tests retried no rebind after shutdown; parallel tests can
  receive the freed ephemeral port. All five sites wait up to 1s.
- Doctests: both crates have zero; executable example coverage is the generated
  catalog suites, so CI does not spend a doctest build.
- Ledger: 1,685 identities PASS (five new testing identities reviewed).
- Version: tsrust-compiler 0.2.0, tsrust-lsp 0.3.0, extension 0.2.1 unchanged.
  Release notes and the status page are updated. All v0.2.0 plan boxes are checked.
- Local revalidation: compiler library 1,546 (twice), CLI 54, project commands 6,
  toolchain installation 25, acceptance 8, std:testing catalog suite 12.

### CI build-once restructure — 2026-11-09

Owner: CI must compile once per OS and run `tsrust test`; consolidation instead
of 1:1 is fine; no PRs/pushes until the restructure is done; pushes exist to
merge toward release. Plan: `docs/plans/ci-build-once.md`.

- New `std:testing` output capture: `captureOutput`, `startOutputCapture`,
  `finishOutputCapture` (runner resets unfinished captures; effect table updated).
- `tools/generate-catalog-suites.py`: 1,066 generated tests from verified catalog
  examples (42 host-only by rule), one directory per group.
- `tools/run-suites.py`: 133 suites, one native build each, PASS locally
  (cold 17.7 min, warm 301s). It found and fixed a real defect:
  `tsr_console_table` was private, so `console.table` failed in test programs.
  The rust-regex example's stale bindings lock was re-locked.
- `tools/host-tests.py`: 21 boundary targets. CI rewritten: lint, lib, build per
  OS (release artifact), suites and host per OS on that build, editor, CI gate.
  Removed: shard runner, platform runner/evidence chain, supporting-host timing.
- About 530 legacy per-case integration targets remain as local deep-regression
  witnesses, outside CI.

### Full runs 3–4 — 2026-11-08 21:45 UTC

- E0460 root cause: relocated vendored builds recompile shared crates
  (proc-macro2, cfg_if) in place in the shared target. Direct-rustc tests now
  link a private nix closure (`target/test-externs`); the CI libnix step was
  removed. JSON staging test: the runtime rlib is hash-named; test fixed.
  Relocated/doc budgets raised. Every shard has passed in at least one run; the
  last failures were cold-build timeouts.
- Owner flagged the hourly CI loop. Do not dispatch another full run without
  approval; one confirming run on the final commit remains the only
  full-suite item. Platform acceptance is complete (C9).

### C9 complete; full run 2 — 2026-11-08 20:00 UTC

- C9 42511473967 at 81f7beed: focused platform acceptance COMPLETE on all four
  hosts (61 targets each, zero failures). The Windows Ctrl+C fix worked. B00
  admission, B01/B03 Windows rows, B20 native acceptance and "finish B01–B19"
  are checked.
- Full run 2 42511475678: shards 0, 1, 3, 4 and 7 PASS; every shard finished
  within its hour (max 56 min). Remaining (all pass locally):
  native_buffered_file_backend E0460 (shared-target `cfg_if` rebuilt by tests
  that run their own Cargo builds), native_json_build staging,
  project_commands dev exit (status now printed), and the frozen-UI emit budget
  (40→120s). Next: full run 3; fix whatever recurs.
- Remaining plan: clean complete pass, release archives in the notice audit,
  prune, owner approval.

### First sharded full regression — 2026-11-08 19:30 UTC

- Run 42503200899 (8 shards, 2fd6e5e1-era HEAD on `codex/stdlib-0.2.0-full`):
  every shard finished within its one-hour job (slowest 54 min). Failures,
  diagnosed and fixed locally:
  - map-epoch `while(true)` loops emitted `while !stop && (true)`, so a function
    ending in such a loop failed rustc with `()` instead of i64. A literal-true
    loop with no break leaving it now emits `loop` (`breaks_out` uses resolved
    control targets). Test sources with genuinely dead statements after
    divergent loops were corrected (3 scoped-record cases).
  - Stale fixtures: exception_handling inlined runtime lacked `report.rs`;
    cache callback rejection is now `cache-payload`; `Option<T>` is supported
    (B19), so it is no longer a rejection case; json.test.tsr now expects the
    early 413; the empty generic call diagnostic is `array-type`.
  - Pretty reporter printed absolute assertion paths; sites now render relative
    to the working directory.
  - CI environment: crates.io `url` was absent offline (the workspace patches
    it), so it is now in `tests/dependencies`; a restored stale `libnix` broke
    direct-rustc tests (dropped before the shards run); "Complete regressions"
    was skipped after an earlier step failure (`!cancelled()` added); two cold
    budgets raised.
  - Local: all affected targets PASS, including scoped_record_composition 207
    and the map suites.
- C8 42503198395 sealed (Linux/ARM/Intel complete). The Windows helper died
  with STATUS_CONTROL_C_EXIT. It now ignores Ctrl+C process-wide and stays
  attached for one second after sending, so its own copy of the event is
  delivered while ignored.

### C6 review and measurement completion — 2026-11-08 16:45 UTC

- C6 42494556962 at 3972fa49: Linux, ARM and Intel macOS repairComplete=true
  (61 targets, packaged smoke PASS). Windows' only failure is the SIGINT witness:
  the owned console helper now dies on its own event, because it inherits the
  restored normal Ctrl+C processing. The helper now also sets the process-wide
  ignore. Sealed in LATER_RUNS (Linux 9e39f0fc…, ARM 5a4fc8a2…, Intel 98908cee…,
  Windows af07e6fb…).
- Ingested 240 entries (80 workloads × 3 engines); integrity ok. The page has
  1,165 comparisons; the generator uses the manifest `rustReference`.
- Owner: benchmark only Node-comparable operations, not integration wrappers.
  HTTP upload/middleware timing and the native HTTP refresh are dropped.
- Site 78 pages, links PASS (128 pages).
- Open: Windows SIGINT (C7), the sharded full regression run, package admission,
  B01/B03 Windows rows, prune, release approval.

### Plan completion pass — 2026-11-08 15:30 UTC

Owner: keep going until the whole v0.2 plan is implemented; CI matters but is
not the main goal. v0.2.0 merge/publication is still not approved.

- C5 42485215015 at a341f7ff terminal: Linux/ARM repairComplete=true (61 targets).
  Windows' only remaining failure is the real SIGINT witness (listener never saw
  Ctrl+C although the helper's GenerateConsoleCtrlEvent succeeded). Intel reached
  41 targets, then the relocated realtime case hit the 55-minute scope stop. All
  four results sealed in `LATER_RUNS` (Linux 33f3a296…, ARM 1be2637e…, Intel
  86cca64f…, Windows ef36ee03…); chain verified locally.
- Windows SIGINT: the CI launcher starts the test owner with Ctrl+C ignored, an
  attribute every child inherits even with CREATE_NEW_CONSOLE. The test owner now
  installs its own ignore handler and restores normal processing
  (`SetConsoleCtrlHandler(None,0)`) before spawning the listener. Unproven until
  the next Windows run.
- Generic sibling regression fixed: generic argument preparation retried only
  `generic-inference` errors, so `identity([])` aborted before the sibling
  `identity([7])` supplied T. A context-free `array-type` error is now deferred
  the same way. Witness `empty_array_generic_arguments_take_context_from_generic_siblings`.
- Host export ledger: `docs/plans/host-export-ledger.md`; verify-api-audit PASS
  for all 1,680 identities. Language inventory reconciled (80 probe-backed
  reclassifications, 1,355 disposition notes; generator links repaired).
- Probe defects fixed: `'a'.localeCompare('b','en')` panicked (receiver rewrite
  reached std:locale without preparing its records). Locale and regex receiver
  rewrites now share `prepare_receiver_binding`, which also reports arity by the
  public member name instead of `__stringCompare expects 3 to 3`.
- Notice audit: five Android/wasm-only backend crates had no license file in
  their archives; exact upstream MIT supplements are added by packaging and
  portable export (distribution tests 15 PASS).
- Full regression fit: CI `full_regressions=true` now runs eight parallel shards
  (`tools/regression-shards.py`, weights from the 965de83f run, every target on
  exactly one shard; shard 0 owns lib/bins/doc/format/lint). Not yet run.
- Measurement gap found: 80 registered runtime workloads (75 B02 byte codec/
  operation, 5 B01 file) and 16 B10 upload/middleware HTTP rows have never been
  measured; HTTP request/CPU native pins date from 2026-11-04.
- Measured (not yet ingested): all 80 runtime workloads verified (795s) and timed
  at 3×31 on native/node-tsc/rust-hand, results
  `benchmarks/runtime/results/2026-11-08T15-45-57-371Z.json` (36 codec) and
  `…T15-56-53-554Z.json` (44 operation/file). NEXT: `npm run store:ingest` in
  benchmarks/runtime, verify, add the runs to website/tools/performance-runs.json.
- B10 upload reference fixed: the hand-written Rust server never compiled
  (`form_urlencoded::Serializer` held across an await made `forward` non-Send).
  The serializer is now scoped; manifest pin updated; guard PASS. NEXT: run
  `--uploads --verify` then `--uploads --prepared`, the same for `--middleware`,
  then the native-only requests/CPU refresh (scripts http-*.sh pattern in this
  note's commands), ingest, update pins.
- C6 focused platform run 42494556962 at 3972fa49 dispatched (later chain
  42480586761 42485215015). Review all four artifacts when terminal; then
  dispatch `full_regressions=true` (eight shards) — same-ref runs cancel each other.

### C4 review and C5 dispatch — 2026-11-08 13:45 UTC

- C4 42480586761 at 3d6767c5 terminal: ARM and Linux repairComplete=true (all 61
  targets, zero failures). Windows reached all 61 targets with 118 new passes;
  remaining failures: the real SIGINT delivery witness and LSP async_tasks/
  native_threads/rust_bridge. Intel reached 41 targets, 17 new passes, then scope
  stop. All four results sealed in `LATER_RUNS` (Linux 720c3bd4…, ARM 377d65d1…,
  Intel 13a54555…, Windows e14c5b60…).
- Windows SIGINT root cause: the runner started test binaries with
  `CREATE_NEW_PROCESS_GROUP`, so every descendant inherited "ignore Ctrl+C" and the
  generated listener could never observe CTRL_C. Windows cleanup never used the
  group (owned handle plus exact PID/birth rows), so the flag is removed on
  Windows; POSIX sessions are unchanged. Guard test added.
- Windows LSP failures: tests used Unix-only absolute ids (`/async/main.tsr`,
  `file:///…`). The three files now derive ids from host-absolute URIs (Windows
  gains `C:`), unchanged on Unix; local async_tasks 30+3, native_threads 18+12,
  rust_bridge 8+8, service_sections 3+3 PASS (96.8s).
- Intel root cause: the compiler test harness counted time spent waiting for the
  serialized heavy-build lock against each command's own timeout (180s). The
  command budget now starts after the lock is held; the wait has its own 1800s
  bound. Platform scope 55 min/job 60 min are unchanged.
- Runner: ordered `LATER_RUNS` chain (`--later-repair-plan`, workflow input
  `platform_later_prior_runs`) requires every earlier seal unchanged. Python 27 PASS.
- C5 dispatched: run 42485215015 at a341f7ff (prior/second/third + later
  42480586761). PRs/Dependabot were empty before push. Do not dispatch another
  run while it is live; download all four artifacts when terminal, seal them in
  `LATER_RUNS`, and repair only diagnosed failures.
- Remaining B00–B20 work after C5: four-host package admission and B01/B03
  Windows rows from C5 evidence; the generic sibling-context regression above;
  making the complete compiler/LSP suites fit one hour (historical 11,754s case
  time plus per-target builds; settle repair saves about 1,000s; next levers are
  consolidating integration targets into fewer binaries and the 777s
  checker_agreement case); the per-export host ledger (1,247 inventory identities
  lack a review entry; `tools/verify-api-audit.py`); measured HTTP metric refresh;
  docs/help/install/update/archive audits; pruning; then release approval.

### Suite-time and catalog repairs — 2026-11-08 13:20 UTC

- LSP test `Session::settle` now stops 50ms after every open document's current
  version is published, keeping the old 900/400ms quiet window as fallback when a
  version publication is not observed. Assertions are unchanged. Actual local
  timings versus the 965de83f record: completion_matrix 341.5s (was 732s, 3 PASS),
  editor_consistency 36.5s (was 234s, 38 PASS), intelligence 176.1s (was 570s),
  checker_agreement 777.4s (unchanged 759s; not settle-bound, 6 PASS).
- Catalog: 65 non-UI entries had verified `examples` but no documentation fence,
  failing `backend_catalog_examples_match_native_output` at HttpClientOptions.ca.
  Each now shows its first verified example, formatted by the current compiler,
  with `Expected output:`. The test then PASSED alone in 679.3s.
- OPEN regression found, not yet fixed: `contextual_generic_array_calls_keep_checked_parameter_and_result_help`
  fails because `check` rejects
  `function identity<T>(value:T):T{return value;}function first<T>(left:T,right:T):T{return left;}function main(){const values:number[]=identity([1,2]);const selected=first(identity([]),identity([7]));console.log(values.length,selected.length);}`
  with `array-type: An empty array needs an array annotation or call/return context`
  at `identity([])`. Sibling generic-argument context no longer reaches the nested
  generic call. Candidate commits touching generic inference: 8b1d4e9f, 91428ba4,
  e339008f (2026-11-07). Bisect or trace `generic_call_work.rs` parameter contexts.
- C4 run 42480586761 at 3d6767c5: ARM and Linux SUCCESS, repairComplete true,
  all 61 targets, zero failures (new passes 6/5; retained 546/552). Results
  SHA prefixes 377d65d1f095 / 720c3bd41c0d. Intel/Windows pending at this entry.

### Resumed continuation — 2026-11-08 12:30 UTC

Owner resumed the goal. Hosted run 42474075588 at a7795eea is terminal FAIL on all
four hosts; all four artifacts were downloaded, SHA-sealed and reviewed. This
continuation repairs its diagnosed causes. Full B00–B20 stays incomplete;
v0.2.0 merge/publication is not approved.

- Actual C3 results (focused repair, not full-source acceptance): Linux 4 new
  passes (relocated logging 357.8s, relocated realtime 392.9s, nominal projection
  2.1s, LSP IO docs 11.8s); ARM 0; Intel 0; Windows 232 new passes across 22
  targets, reaching 41 of 61 targets before the 55-minute scope stop. Windows
  build-helper cleanup now works on the host. Results SHA256: Linux 3c28afa0…,
  ARM 849cf27d…, Intel 8ef7dc2d…, Windows 5f3be802…; each retains the C1/C2 seals.
- Root cause of almost every failure on all hosts: `Validating native support
  workspace …: URL patch payload is incomplete.` The Actions cache restores
  `target/tsrust-native` without some registered URL patch files, and
  `backend_sources::restore_missing` restored only the four base files. Repair:
  `native_url_patch::restore_missing` recreates only absent registered payload
  files under the existing workspace lock; changed files and links are rejected
  before any write. Unit witnesses cover partial/whole payload loss and changed
  files. This resolves the earlier unexplained remote ENOENT/missing-path class.
- Actual Windows stack receipt: relocated exported process `program.exe` (debug)
  exits 0xc00000fd with main-thread stack overflow. Reproduced locally: the same
  debug process example aborts under `ulimit -s 1024` (exit 134). Repair: an
  async entrypoint now runs the whole generated `main` (hook setup, result
  adapter, exit code) on a scoped thread named `main` with an 8 MiB stack; only
  `()`/`ExitCode` crosses threads. Local repro then exits 0 with identical output.
  The thread name keeps panic text unchanged. Witness:
  `async_entrypoints_run_on_an_explicit_main_thread_stack`.
- Remaining non-URL timeouts were cold per-command budgets on slow hosts: Intel
  `emit --package` for logging/realtime exceeded 60s (now 180s, matching IO);
  Intel IO relocated build and Windows logging relocated release build exceeded
  300s (all three relocated cargo builds now 600s per profile). Scope 55 min and
  job 60 min are unchanged.
- Runner/workflow: optional reviewed third input `platform_third_prior_run=
  42474075588` (`--third-repair-plan`) retains only C3's own successful cases at
  their a7795eea identity, and requires both earlier seals. Python 25 PASS 0.14s;
  workflow/release policy node tests 23 PASS.
- B19 optimization: equality between a record-union literal tag field and a
  string literal now selects variants (`match &(base) { ViewUnionN::Vk(_) => true,
  … }`) in both emitter paths instead of comparing text. Nested-option workload
  emits zero tag text comparisons; untimed oracle 505733888 matches. Not timed.
- Local actual checks at the working tree (shared target, inc0, jobs1, exact
  prior feature graph http-websocket+logging): compiler library 1544 PASS/8
  ignored; LSP library 27 PASS; CLI 54 PASS (requires explicit CARGO_TARGET_DIR,
  as on CI); schema_unions 14 incl. new witness, rust_bridge 8+8,
  owned_iterators 12, recursive_callable_results 16, generic_value_adapters 92,
  string_operators 6, async_tasks 30+3, async_io 17+4, process 12+4,
  task_ergonomics 14, service_sections 3+3, structured_logging 4+1 all PASS.
  The two relocated export cases need `TSRUST_SOURCE_PACK_DIR=target/portable-source-packs`
  locally, as CI sets it. Each compiler rebuild took about 430s.
- Next: commit, check PRs/Dependabot, push, dispatch one focused run with all
  three prior inputs. Continue remaining B00–B20 work locally while it runs.

### Owner-requested stop — 2026-11-08 11:14:58 UTC

The owner explicitly requested a handoff and stop. Pause the goal; do not continue
implementation or start more validation until the owner resumes it. Full B00–B20
remains incomplete; v0.2.0 merge/publication is not approved.

- Canonical writer: `/Users/seancotter/Desktop/dev/learning/tsrust`, branch
  `codex/stdlib-0.2.0`, HEAD `a7795eea73d631a9f4340c0c5d898a107ac047db`, pushed.
  Preserve the protected app worktree at
  `/Users/seancotter/.codex/worktrees/stdlib-platform-acceptance/tsrust`.
- Pending changes are ONLY `docs/agent-progress.md` and
  `docs/plans/stdlib-0.2.0.md`: actual OS leak/allocation findings and current CI
  evidence. Preserve them. No source edits or new builds followed a7795eea.
- Verified at stop: no local Cargo/rustc/native/leaks process, no live agents.
  Historical build PIDs and vanished sessions must not be restarted.
- Existing hosted run **42474075588** remains LIVE at exact a7795eea, all four
  hosts in bounded native-family/package acceptance. It was already dispatched;
  left running, not cancelled. No additional run was launched for this handoff.
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42474075588
- First action after resume: inspect this SAME run to terminal status, download
  its host artifacts, inspect actual failures and preserve successful scopes
  with their real identities. Prior two sealed inputs are42460717406@469 and
  42470181942@f46. Current repair retains both separately and is NOT full-source
  acceptance. Windows stack/signal phases and missing-support-path diagnosis
  remain unresolved until actual new host evidence arrives.
- Accepted local runtime/cache checkpoint7ff214ea: corrected one no-run build
  31.080s/697811784B; CLI54PASS; IO18 both profiles plus raw stdioPASS; shared55
  both profilesPASS. Existing compiler1544/LSP27/export proofs retained. Actual
  113 timing-history entries are committed, original93 samples each; unchanged
  Rust/Node controls must not be rebuilt or retimed without a diagnosed reason.
- Latest additional evidence (details below): all four password workloads on
  native/Rust have zero leaked blocks/bytes under macOS leaks at exit, using
  existing binaries. The original inside-runtime allocator rejection remains
  recorded. Current deep-data allocation count18 versus2066; collections5
  unchanged. Material collection timing drift and nested-adapter7x Rust gap
  remain explicitly recorded, not hidden by passing allocation observations.
- Remaining complete-goal work includes final backend binaries/all integration/
  doctest/editor/package/install/update/rollback/archive acceptance, public metric
  and inventory reconciliation, and outstanding plan requirements. Do not claim
  B00–B20 complete from the current focused scopes.
- Full-suite budget audit is READ-ONLY, no optimization implemented or run:
  old `/tmp/tsrust-965de83f-full-integration.json` has544 targets/4619 cases;
  non-UI-prefix509/4503 accumulated10530s case time plus6288s per-target build
  overhead. These are historical accumulated times, not a current wall-clock
  estimate. Do not launch a known oversized suite or split it into serial hourly
  chunks. One Cargo build and complete coherent coverage must fit one hour.
  Largest historical groups: native_package_export, checker_agreement,
  completion_matrix. `Session::settle` still waits900/400ms for quiet; replacing
  it was only considered, NOT implemented or validated. Any future change must
  preserve actual document-version/phase agreement and all existing assertions.
- Public metric refresh entry points located, not executed:
  `website/tools/generate-performance.mjs`,
  `website/tools/performance-runs.json`, and `website/package.json` reference/build
  scripts. Generator reads SQLite and explicitly curated Rust baselines. Add
  accepted current measurements with cached controls; do not fabricate new runs.
- No pushes/releases after a7795eea. Last prepush PR/Dependabot checks were empty.
  Check both again before any future push. Keep this as the single handoff.


Current source checkpoint a7795eea73d631a9f4340c0c5d898a107ac047db, pushed on
codex/stdlib-0.2.0. Runtime/cache repair7ff214ea is locally accepted; a7795eea adds
reviewed two-run evidence reuse and bounded cold-export command adjustments.
Working-tree changes after this checkpoint are plan/progress evidence only.
Hosted42460717406@469 and42470181942@f46 areterminalFAIL. Focused repair
42474075588@a7795eea is RUNNING on all four hosts; do not dispatch another run.
Full B00–B20 goal remains active. v0.2.0 publication/merge is not approved.

- ACTUALC3dispatch42474075588 atEXACTa7795eea, usingbothreviewedpriorruns.
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42474075588.
  LatestprepushPRs[]/Dependabot[], oldrunsallterminal. All4hostsnowinbounded
  nativefamily/packageacceptance. Scope55min inclusiveofsetup/job60min. Thisis
  focusedrepair, neverfullcurrent-sourceacceptance; publicationstillunapproved.
- OSleakdiagnosisusedEXISTINGcompiledprobesonly, zeronativecompilation:
  all4passwordworkloads×native/Rust report0leakedblocks/0bytes atprocessExit.
  Eightownedscopes1.071–1.590s, same120saggregate ceiling. The3additional
  measuredprogramsperenginehadexactacceptedbinarySHA verified beforeexecution.
  /tmp/tsrust-password-os-leaks.json. This doesNOTerase earlierstrictallocator
  inside-runtime zero-retentionfailure orprovezero livebackendallocations there.
- CurrentB18allocationdiagnosticsACTUALPASS4.213s/283885568B: threeobservations
  each retain0B. Deepdata18allocations/1728requestedB vsold2066/132800,
  removing2048temporaryrecordworkallocations. Collections5allocations/2400B,
  unchangedfromoldprofile. /tmp/tsrust-testing-current-allocations.json;
  benchmarks/runtime/diagnostics/2026-11-08T11-08-53-828Z-native-allocations.json.
  Noallocation-countcause forcollectiontimingdrift established; no controlretime.
- Secondfocusedrepair preparation: all4 C2 artifacts SHA/host/source/parentseal
  verified locally. ActualC2passesLinux2/Windows0/ARM6/Intel127 remainf46;
  C1retained447/0/540/399 remain469. Newoptionalexplicitsecondartifactinput
  preservesbothidentities, ignoresnestedretainedasC2 proof, andskipsONLYactual
  successfulok/noScopeStopcases; oldforcedC1controlsstillrequireactualC2pass.
  Fourhostoriginal61targetselection/guards/package+smoke unchanged; focused
  completefalse staysfalse. Python24PASS; actualall4sealedartifactsPASS.
- TerminalC2Intel47.6min (jobcap60min/scope55min): mostnewnativefailuresbare
  ENOENT plusIOpackageemit60s/logging+realtimeDEBUGCargo180s timeouts. Record
  /tmp/tsrust-repair-42470181942-mac-intel. Logging/realtimeCargo budgets now
  300s EACHprofile, IOpackageemit180s; onlypercommandcoldworklimits changed,
  scope55min/job60min stays. Unchangedlocallyacceptedpayloadsnotrerun.
- Nextremote decision: candidateafter7ffcheckpoint uses54CLI/IO18both+stdio/
  shared55both accepted, priorcompiler1544/LSP27/exportproofretained. Need
  Windowsactualstack/signal+successfulownedhelpercleanup and actualmissingpath
  receipt/resolution onhost caches; selectplatform_acceptance=true,
  platform_prior_run=42460717406, platform_second_prior_run=42470181942.
  OldequivalentrunterminalFAIL; no duplicateactivevalidation, nofullCI/release.
  CheckfreshPRs/Dependabot immediatelybeforepush. Publicationunapproved.
- Focused cache/build repair ACTUALPASS: exactpriorCargo featuregraph restored;
  onecorrectedno-run31.080s/697811784B, terminalbuild-finished success.
  CLI54PASS10.598s includingmissingfile/changedmetadata/symlinkcontrols;
  registeredIO18EACHprofile+rawstdioPASS193.470s; shared55EACHprofile
  PASS130.231s. /tmp/tsrust-partial-support-repair-corrected.json.
  Existing compiler1544/LSP27/relocatedexportsremainaccepted; notrerun.
- Rootmistakepreserved: firstrepaircommand omitted priorhttp-websocket/logging
  features andstartedredundantcompilerlibraryrebuild. Stoppedownsupervisor77436
  /Cargo77519, verifiedsettlement, thenusedEXACTpriorgraph. Bothattemptscharged
  sameoriginal1770sdeadline, noreset. /tmp/tsrust-partial-support-wrong-features-stop.json.
- Closed-outputcleanup removed791files343910222logicalbytes and recovered
  361938944observedfreebytes: aborted/closedloosecodegenobjects plusobsolete
  profilingbinaries/sources only. CurrentmanifestconsumersSHAverifiedunchanged;
  sourcepacks/shareddeps/currentdiagnostics preserved. Togetherwithprior
  8958775296B cleanup, observedrecoverythiscontinuation9320714240B.
  /tmp/tsrust-measured-repair-closed-output-cleanup.json.
- Allocation follow-up finished within ORIGINAL1770s deadline: native8
  iterator/adapter probesPASS5.829s; fourRustallocationcontrolsPASS1.607s.
  All3observations strictzero. Adaptercounts/bytes IDENTICALnative/Rust;
  nested4095allocations/77805B, so no allocationvolumecause for7.00x timinggap.
  /tmp/tsrust-post-allocation-gates.json and
  benchmarks/runtime/diagnostics/2026-11-08T10-37-09-708Z-native-allocations.json.
- Original37allocation scopeFAILED755.902s/588445688B atfirstpasswordprobe;
  noownedprocessesremaining.25syncprobes passedbeforefailure. Strictpassword
  rejection remains: recovered3samplesretain4376–4424B; matchedunchangedRust
  control4264–4376B. Backendstate/retirement plausible, leak/eventualcleanup
  unproved. Remaining3passwordallocationprobesunrun. Existingbinary-onlycounter
  recovery26rowsPASS2.151s afterEXACTemission/instrumentedsourcecomparison;
  initialrecoveryfailed .trim() mismatch BEFOREexecution, preserved.
  /tmp/tsrust-crypto-allocation-counter-recovery-v2.json;
  /tmp/tsrust-password-rust-allocation-control.json (4.299s scopedbuild/run).
- B18native-only originalprotocolPASS fivecorrectnesscases perworkload then
  8warmups/3processes/31samples, quietguardwaited forload<4. Native2groups93samples
  ingestedONCE at2026-11-08T10:38:29.075Z; cachedRust/Nodecontrolsunchanged.
  deepdata .011042ms versusprior .045500, 4.12xfaster/Rustgap1.53x. Collections
  .137666ms versusprior .070750, regression1.95x; no deferredrecordstack in
  emittedcollections, relevantMap/Set/NumericArray/helpersunchanged. Three
  processmedians .146625/.137666/.100292 drift; actualcauseunproved. Do not
  relabelregressionasnoise/pass orretimeunchangedcontrols. Planrecordsboth.
- Partialcache restoration repair nowLOCALLYPASSED: underexisting
  ownedworkspacelock restoresONLYmissingbasefiles; changedmetadata/symlinks
  rejected beforemutation. Contextualvalidationerrors includeactualpath.
  RemoteENOENTcachecauseremainshypothesisuntilcontextualreceipt. Loggingcold
  releasecommandbudget300s afterobserved180s compilationtimeout; scope55min/job
  60min unchanged. Nootherpassedexportsorbackendregressionsrestarted.
- Windows build-helper cleanup repair is in source: only code0 selected-test
  builds may terminate exact-owned background helpers and then require fresh
  empty-tree inspection. Runtime/test scopes still fail on unsettled children;
  no executable-name exception. buildCleanup preserves before/after evidence.
  Existing Python platform suite now23PASS0.125s. Hosted behavior is notyetproved.
- Public37 measurement ACTUALPASS219.179s/404946944B: all555 correctness
  comparisons passed before timing; original8warmups/3processes/31samples,
  quietguard enabled/systemallocator. Actual111 engine/workloadgroups93samples
  ingested ONCE with npm run store:ingest; SQLite integrityok. Canonical minimal
  histories timestamp2026-11-08T10:14:12.271Z. Preserve unchanged Rust/Nodecontrols.
  Raw disposable benchmarks/runtime/results/2026-11-08T10-14-12-271Z.json;
  /tmp/tsrust-current-public-37-measurements.json. B15/B16/B19 plan nowrecords
  actualmedians, including nestedoption7.00x Rustgap andrandomInt8.22x Nodegap.
- Allocation scope is CLOSED; no profiling/native process remains. See actual
  rejected/password and completed iterator/adapter evidence above. Do not
  restart25passed synchronous probes orremeasureunchangedRust/Nodecontrols.
- Focused42470181942 actualLinuxfailure: IO2/task1 instantfailures; logging
  scope319.804s endedwithunsettledchild. Windowsbuildcode0/748.391s blockedby
  genuinecurrentMSVCvctipdescendant, notoldPIDreuse. No nativeWindowscase ran.
  Artifacts /tmp/tsrust-repair-42470181942-{linux,windows}; agentsreviewlogs
  sourceonly. Keepownership/cleanup assertions; no blankettelemetryexemption.

- ACTUAL current compiler library1543PASS/0FAIL/8unchanged opt-ins ignored,
  96.406s/1352483320B. Current LSP library27PASS/0FAIL1.053s.
  /tmp/tsrust-final-extraction-test.json and /tmp/tsrust-current-final-lsp-library.json.
- Native family consolidation: task13, asyncIO18, network18, compression16,
  HTTP110 =175 registered cases. IO/network/compression pass BOTHprofiles;
  task/HTTP retain original default-profile protocols. Strict exact case names,
  original output/typed error oracles, source correspondence, cleanup and no
  unknown output retained. True stdio, no-runtime, cache, TLS, ABI and relocated
  export boundaries stay separate. All owned processes settled; no build live.
- Network BOTHprofiles ACTUALPASS16.694s/753188864B after closed native record/
  finite plain literal option result-discovery repair. Minimal real2file projects
  preserve listen alias and DNS conditionalalias/relay/undefined family; genuine
  void/wrong native argument/invalid literal remain rejected. Recursive-summary
  scalar eligibility unchanged. /tmp/tsrust-final-extraction-test-network-native.out.
- HTTP110ACTUALPASS10.447s/1289913592B, nativebuild8180.098333ms. HTTP emitted
  arguments ONCE; raw text reuses prepared operands, no abandoned conditional
  owner. Compression16EACHprofile ACTUALPASS25.503s/713097216B, nativebuild
  3513.819ms debug/20032.778ms optimized; worker7/5 outputs now correctly tagged.
  /tmp/tsrust-shared-native-context-functional.json.
- Existing6case testing fixture ACTUALPASS6.809s; FIVE child paths (TokioJob,
  async pool, sync pool, dedicated, scoped thread) retain exact case/source
  location/order, no orphan/duplicate output. Profile secondrun cachehit/native0.
  ONE ArcMutex collector shared across extractedruntime; Tokio poll/drop and
  sync worker/drop scope restored. Ended scopes reject late capture to existing
  visible unattributed reporting, never silentlydiscard or leak into next case.
- Complete runtimevariant factories reused by emission/dependency detection/
  extraction. Pure tests prove ONE runtime SCOPE/collector and actual selected
  modules, including async-only runners. No new dependencies or unsafe code.
  Non-runner runtime hooks remain passthrough. console_async.rs is committed at04f3df1f.
- Repair failures preserved honestly: originalbuild160.956s failed NEWtesthelper
  unwrap on InputDiagnostic(noDebug); corrected to render API. Sharedbuild
  399.154s then3guardsPASS/1FAIL: test wrongly expected bindingfeatureasync;
  actual async selection follows Task origins/runtime module. Secondbuild
  388.352s then3PASS/1FAIL: test string matcher missed pub/comment/mod syntax.
  Original1770s aggregate closed INCOMPLETE, no fake full acceptance.
  /tmp/tsrust-shared-native-context-repair{,-resume}.json and
  /tmp/tsrust-native-result-final.json retain failures.
- Final correction is cfg(test) ONLY: parses Rust AST for exact public module
  identity/visibility, not formatting. New600s gate rebuilt ONLYcompiler test
  harness226.206s (no tool binaries), corrected guardPASS2.617s, full library
  PASS96.406s. Earlier production consumers unchanged; native network is current.
- CI cache defect fixed: shared Cargo target alone did NOT share standalone
  native support. Platform runner AND CI env now explicitly use
  TSRUST_CACHE_DIR=target/tsrust-native; deliberate per-command isolation still
  overrides. Actual subprocess-environment regression + runner tests16PASS0.081s.
  No trigger/timeout changes;55min platform/60min job remains. Not dispatched yet.
- Contract catalog+guides now document direct imported aliases/stored callbacks
  versus unsupported stored generic all/extracted Iterator method values. Exact
  thread-call negativePASS. Public logging/realtime reference mapping/sitebuild
  PASS4.752s and128page rendered links/assets/anchors PASS1.605s (not deployed).
- Cleanup removed newly-created loose codegen objects only, preserving current
  libraries, metadata, binaries and shared caches. Exact count/bytes:
  /tmp/tsrust-native-context-object-cleanup.json. Prior net recovery oldhost
  1430007808B +obsoletebasepack766394368B remains separate.
- Generated minimal/unused vs ICU consumer build/size proof ACTUALPASS at04f3df1f:
  520872B vs2179640B (delta1658768B), exactoutputs7 and ICU4lines verified;
  first appcachemiss nativebuild2056.4ms/24974.8ms, warmhitsbothnative0.
  Existingstable sources, sharedtarget/cache, no directcontrols/11sample replay,
  no sampled-benchmark/SQLite claim. Temporarybinaries/emittedscratch removed.
  /tmp/tsrust-generated-minimal-footprint.json. B00feature-selection row closed.
- Public37 original30min scope CLOSED INCOMPLETE before timing after genuine
  preparation failures, no child/resource runaway and no accepted timing rows.
  Corrected9stream endindices,3Rust HKDFreference slice calls and4Argon2reference
  PHCstr calls; all29crypto+4iterator workloads then passed5independentcases on
  native/Rust/Node. FirstpureRustadapter failed unconditional injected Rayon;
  rootcompiler dependency-selection repair is required before timing can proceed.
  Preserved validated prepared artifacts with exact compiler/source/ABI/binary
  hashes, not a claimed measurement pass. No samples/SQLite rows were ingested.
  Attempts: /tmp/tsrust-public-iterator-crypto-adapter-{measurements,resume,final,
  password-repair}.json. Originaldeadline was never reset.
- Historicallaunchrecord (nowcompletedabove): Newpublic37 firstactualtiming atf46d7400 afterrealcompilerdependency
  repair. Existing29crypto preparednative/Rust/Node artifacts mustmatchNEWCLI
  emission/ABI/sourcebefore reuse; all37×5cases×3engines nowpassedindependent
  correctness before original8warmups/3processes/31samples began. No oldfailed
  scope reset or fabricated samples. Original publicpreparation gate remains
  incomplete; newsourcecandidate supportsall4pureRustadapterprograms.
  /tmp/tsrust-current-public-37-measurements.json; ingestONLYsuccessfulactualrun.
- Hostedfocusedrepair42470181942RUNNING atf46d7400, all4hosts; priorrun42460717406
  isexplicitinput withfixed4resulthashes. Failed/missing/new/unreachedcasesand
  reviewedforcedaffectedcontrols rerun; oldpasseskeeporiginal469identity.
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42470181942.
  LatestprepushPRs/Dependabotempty; no duplicatefullCI/publicationdispatched.
- Final focused host classifierPASS0.569s; crypto source/negative guards5PASS,
  platform budget/ownership/repair-evidence guards22PASS0.110s. Whole-project
  format0changes/tokenfingerprints0 anddiff-checkPASS beforecheckpoint.
- Explicit1MiB Rustthread frontendprobe ACTUALPASS: load/check/emit/drop current
  process example,226778emittedbytes; compile1.072s/execute0.550s. This islocal
  frontend evidence, notproof Windowsnative/runtime stackfailurefixed.
  /tmp/tsrust-process-frontend-stack-probe.json. Windowscommands/stagesnowlogged.
- Cleanup recoveredadditional2627801088bytes from64 source/CRC-provenobsolete
  host/cacheoutputs; currentacceptedartifacts SHAverifiedunchanged. Removed1156
  closedlooseRustobjects702278040logical/722604032observedfreebytes. Combinedwith
  earlier5608370176bytes consolecleanup, sessionobserved recovery8958775296bytes.
  Currentarchives/binaries/metadata/shareddeps preserved. Rootcache remainsat
  target/; do notpurge dependencygraphs ordelete swapfiles.
  /tmp/tsrust-platform-repair-{obsolete-artifact,loose-object}-cleanup.json.
- Priorf46 remotevalidation decision (supersededbyactualhostresultsabove): local1544compiler/27LSP/
 52CLI/4editorphases/IO18bothprofiles/rawstdio/pure+genuineRust/logging+realtime
  relocatedbothprofilesPASS. NativeWindowsstack/signal phases andcoldhosttimeouts
  remainunresolved. SmallestreviewedCI inputplatform_prior_run=42460717406 retains
  oldpasseswithOLDsourceidentity, listsfreshbinaries, rerunsallfailed/missing/new/
  unreachedandforcedaffectedcases, thenpackagesmoke. It isFOCUSEDrepair, nevera
  fullcurrent-sourcepass. Last4hostrun42460717406terminalFAIL; no liveCIrun.
  CheckfreshPRs/Dependabotimmediatelybeforepush; publicationstillunapproved.
- ACTUALcurrent compiler1544PASS/0FAIL/8unchangedopt-ins ignored96.012s;
  LSP27PASS1.099s; CLI52PASS3.196s. PurestdownedadapterO0/O3 PASS2.690s
  WITHOUTRayon; genuineRayon/TaskContextbridgePASS2.137s. Newdependencyselection
  factworkswithoutremovingactualRayonavailability. /tmp/tsrust-platform-coherent-repair.json.
- ACTUALcorrectedIO18 EACHprofile+stable rawstdio witnessPASS116.644s.
  30min aggregate thenexhaustedduringloggingexport; no scopepass claimed and
  remainingfield retainsPRE-killchildren. Liveinspectionconfirmedallownednative
  processesdead. Passedcompiler/editor/CLI/IO/adapterswerenotrestarted.
- Remainingexports continuedasdistincttrees underABSOLUTEonehourfromtheoriginal
  repairbuildstart, notanewhour. ACTUALloggingrelocatedBOTHprofilesPASS216.862s;
  realtimeBOTHprofilesPASS248.588s. Consolehelperrootnamespacefixnowhasactualraw
  portable/nativeproof. /tmp/tsrust-platform-export-continuation.json. Original
 30minscope remainsincomplete; theseactualexportchecksareseparatecompletedproofs.
- B00new/changed namespace+receivercontractaudit complete, includingbinding
  signatures/access/results, catalog/reference docs, aliasclasses andcancel/drop.
  Twoactualdocgapsfixed; no missingnewAPIidentityfound. Nativefourhostandhistorical
  pre-facadeprofile chronology remainseparateunchecked requirements.
- Coherent currentrepair build ACTUALPASS416.124s/3386036472B, oneCargo,
  terminal build-finished success; normalCLI+LSP+selectedharnessescurrent. Original
 1770s repairdeadline retained. Watchdogsetup firstfailed on UnixThursday being
  mistakenforISO(T); correctedformat detection,21Python testsPASS. Laterfocused
  repair-selection tests bringcurrentPython count22PASS0.092s.
- Current3compiler selection/worker guardsPASS2.143s. LSP4phaseIO firstrepair
  fixedcompletiondetail but nothover's checked-call syntheticheader; focused
  LSP-only rebuild11.583s/529908528B, correctedexistinghoverpath now4testsPASS
 12.619s. Originalfailedphasewitnessretained, no assertionweakening.
- IOsharedsuiteguardfirstrejected blanklines; exacttoken mapping nowignores
  formattingtrivia while preserving tokens/stringcontents. Thenactual19case run
  failedNDJSON because stored-call witness writesrawstdout, a genuineprocess/
  stdioboundary. Revertedthat registration tooriginal18; keepchecked-in stable
  sourceandexecuteoriginalrawstdoutoracle separately, no tempsource/no library
  interception. This corrects earliermisclassification, not anI/O runtimefix.
- Additional hostedterminal evidence: ARM544passing/Intel401passing/Windows174
  retained. Macs' I/O failures are180/300s setup/run timeouts without captured
  diagnostics, not proven EOF/loan failures; Intel55min budget exhausted in HTTP,
  so missing later cases are incomplete rather than independent defects. Global
  CI60min cap remains; setup-inclusive55min accounting patch nowpasses20Python
  tests with creation-ordered ownership changes included.
- ConfirmedWindows watchdog bug: old vctip1508 born07:56 predates08:42root7032,
  yet reused intermediatePPID1476 made it appear owned. Creation-ordered ancestry
  and exact identity kills now exclude old processes; Windows taskkill/T removed
  because it follows PID-only trees independently.20Python testsPASS0.092s.
  Five raw process cases didpass; seven have genuine stack overflows whose CLI/
  emitted-program phase is unlocated. No stack-limit increase or semantic pass
  inferred. Source fixturetime nowuses100ns exactWindows precision, retains1ns
  onotherhosts; rooted-path probe usesWindowsdrivequalified paths so onlyintended
  relative call sees injectedcwd failure. Nativeplatformrepairs remainunvalidated.
- Hosted469 Linux finished35min;551 exactpassing cases retained,4 realfailed
  scopes. Consolehook namespace breaks raw logging/realtime exports; editor native
  alias facts overwrite recovered hover shape; pure nominalRustadapter failsRayon
  linkage. Windows finished with174passing cases and additionalnative issues;
  full case/attempt evidence downloaded to /tmp/tsrust-platform-42460717406-{linux,
  windows}/. Macs stillrunning; no duplicate dispatch. No fullplatform pass claimed.
- Pending coherent source repairs: sharedconsole helper root aliases preserveONE
  collector for raw+extracted programs; keepnative alias signatures in checked
  generic-fact application. Docscontract audit additionally found7genericWebSocket
  type summaries and missing direct-only operation-value limits for signals/
  threads/delay/yield. Narrowdocs fixes prepared/applied; no newAPIbehavior.
- Public37 measurement first attempt FAILED before timing258.136s/488713952B
  at crypto_sha256_stream_262144. Actual native checked error exposed all9
  streaming benchmark sources passing length instead of bytes.view end index.
  Rust/Node peers/oracles were correct. Fixed9 Tsrust sources and exact hashes;
  rehashed bad-length negative now rejects it. Crypto guard3PASS0.701s;
  whole-project format0changes. No timing rows were produced/ingested.
- SAME original1770s scope RESUMED with1336.0s remaining, exact6 unchanged
  one-shotSHA native/Rust/Node prepared artifacts hash-validated/reused by existing
  prepared-prefix path. No budget reset, sample reduction or new runner.
  /tmp/tsrust-public-iterator-crypto-adapter-resume.json records deadline;
  original failure /tmp/tsrust-public-iterator-crypto-adapter-measurements.json.
  Source/helper/ABI/executable validation remains strict before reuse.
- Allocation profiler crypto hooks were missing despite existing async driver
  support. Added existing crypto source/reference validation, frozen runtime
  rechecks and diagnostic evidence. Strict zero-retention assertion UNCHANGED;
  no allocation pass or password-retention exception inferred before execution.
- ACTUAL stored realtime factory acceptance PASS162.370s: shared55 EACHprofile,
  strict original identities/outputs, cache hit witness native0. Existing host and
  oracle source hashes match their accepted builds; current stable suites are
  loaded from disk. Only test fixtures/examples changed, no production compiler
  code repair or new standalone case programs. Entire affected gate reused one
  original1770s deadline, including build/repairs/cleanup; no budget reset.
  /tmp/tsrust-observer-alias-acceptance.json retains failed attempts and passes.
- Current source adds B03 renamed/stored observer coverage to existing example
  and controlled process fixtures, plus9 checker-only invalid-argument/readonly
  controls.17catalog entries now map to15stable std:testing modules, one native
  runner with exact module/case/output identity and private output. Four existing
  path testsPASS; new catalog guard initially failed only a blank-line expectation,
  repaired and catalog15PASS2.629s. Original4passed scopes preserved.
- Current CLI unit binary52PASS2.626s after ONE26.004s affected-host build.
  Process11PASS initially; unchanged relocated export failed missing source-pack
  env, then ONLYthat scope PASS29.153s with current prepared packs. Do not infer
  a process implementation defect from that setup failure.
- Compression renamed/stored buffer/stream factories now execute inside existing
  cases without extra programs. Typed invalid-level assertion required evaluating
  fallible byte creation before the owner-taking call (correct existing compiler
  rejection, no weakening). Original failed cheap attempts retained. Current
 16casesEACHprofile PASS25.126s with strict old source/output oracles.
- Complete backend doctest command ACTUALPASS156.752s/2258586768B; both crates
  register0doctests. This closes command execution only, not executable examples.
  /tmp/tsrust-final-backend-doctests.json.
- Remaining public timing audit: B15four+B16twentynine+B19four workloads have
  ZERO public SQLite histories. Private admission profiles are different work;
  first public run must measure all3engines, then preserve unchanged controls.
  Existing8warmups/3processes/31samples and quiet-machine rules remain unchanged.
- ACTUAL new obsolete console-cache cleanup recovered5608370176bytes (71 exact
  entries;5592688011logical bytes). Embedded old console collector identifies
  obsolete source, not age/size. Namespace locks held; no native process active.
  Current worker-context runtime artifacts, stable build workspaces and shared
  dependency caches retained. /tmp/tsrust-obsolete-console-runtime-cleanup.json.
- Hosted FOURHOST validation RUNNING at469e8bb3:
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42460717406.
  Each host completed setup and entered bounded native acceptance. No full
  regression/publication run dispatched. Preserve local passed scopes; monitor
  terminal host results and actual packaged consumers before any acceptance claim.
- Remote validation decision: checkpoint includes only platform selector/CI/docs
  changes after5c3414e7. Local runner tests17PASS0.092s, release-policy9PASS,
  whole-project format0changes and diff-checkPASS. Hosted OS file/signal/process/
  TLS/export behavior and55-minute expanded fit cannot be proved locally. Smallest
  available workflow is CI platform_acceptance=true, four native hosts, no full
  regression/benchmark/publication dispatch. Previous equivalent full platform
  run42259225163 at7d67daf8 was cancelled;42279800898 is UDP-only supporting
  timing, not platform acceptance. Current PRs/Dependabot empty and no live run.
- Pending platform selection now includes45 compiler/16LSP targets (61 intended,
  64 validated Cargo artifacts); existing shared55, crypto24 and regex catalog
  programs are reused. No newly ignored tests or runtime benchmark replay. Fit
  on four hosts remains unmeasured under the unchanged55-minute aggregate.
- Full CI regression scope corrected to complete compiler/LSP lib/bin/integration
  and doctests, per development-workflow.md. Removed duplicate frozen UI runs;
  test threads3, jobs1/incremental0 unchanged. Job ceiling remains60minutes.
- B00 historical prototype audit found real pre-facade evidence for B03,
  B05–B17 and B19, with retained SSE semantics qualification. B01/B02 retained
  receipts are incomplete; B04/B18 retained profiling is post-facade. Global
  historical-order checkbox remains open; later measurements do not establish
  earlier chronology. No speculative prototype rebuild was launched.
- Defaults audit found no additional implicit v0.2 limits. Process spawn inherits
  stdio; run pipes output and closes absent input, matching spawn/output modes.
  Drop cleanup is an explicit documented ownership difference. Existing bounded
  date-format/Decimal/threadpool contracts remain intentionally preserved.
- NEXT: reconcile remaining B00contracts/
  defaults/feature clauses; final backend all-target/bin/doctest acceptance and
  FOURHOST native/package/install proof still pending. Check current PRs/
  Dependabot immediatelybeforepush; lastread empty/no live run. Do not dispatch
  before a concrete current scope that fits55min. Full means full_regressions=true.

## Earlier retained acceptance (superseded failures are not a work queue)

- ACTUAL PASS currentCLI publicfrontend declared/inferredfactory4controls plus
  genuineMissing_response error; originalservicefrontendPASS; inventoryPASS1680
  rows/671bindings/174types/43sourceexports/78retainedexamplewitnesses.
  /tmp/tsrust-service-unwind-repair-results.json.
- ACTUAL integratedservicePASS78.033s/1042023408B: ONEappbuild thenSAMEexecutable
  TLS/auth/sessionrevocation/persistentSQLite restart/forms/multipart/CORS/bodylimit/
  staticallowlist/redaction/SIGTERM/SIGINT. /tmp/tsrust-service-unwind-repair-service.out.
  NativeDarwin only; offlinePackagefalse, notLinux/Windows/exportproof.
- Main focusedharness launchfailed0.558s BEFORECargo becauseold /tmp/tsrust-watch.py
  vanished. Restoredthinwrapperusingrepo platform.bounded, no newwatchframework.
  ONLYpendingRustharness resumedinsideSAMEoriginal30minscope; actualPASS240.363s/
  3371849128B; exactprojectregressionPASS0.613s including4variants+realerror.
  /tmp/tsrust-service-unwind-host-tests-results.json and -d2e204380b48.json.
  Session66923/83979closed. No build live. Preserveallpassednativeapp checks.
- Current repairedsource retainsbeforeBOTHdiscovery→fullentries, restoresONLY
  failedtask'sdraineduncheckedemptyfunctions BEFOREpendingerrorretry. Diagnostics
  removed. Earlier03ec1538compiler/27LSP/Docnative/fullHTTP3passesretained; final
  wholebackend/platform acceptance stillpending. Do notclaimallv0.2done.
- B00newpackageaudit vs8efaaee:30native/31backenddirectadditions,17alreadylocked;
  44newregistryidentities checksumverified;42activehaveexactretainedadmission;
  2dormant. Currentbackendlock0vulns/0warnings; optionalUI warnings remainexplicit.
- Attribution/export05ea0131 ACTUAL PASS: existing CLI build26.545s/654639920B,
  terminal Cargo build-finished success; focused CLI notice test1PASS/0FAIL1.092s.
  Actual structured-logging portable export13.131s/57999960B, no native build:
  exact upstreamMIT1065B/SHAed60d479...b44 at root attributions/dependencies/,
  all23 vendored Valuable files match unchanged Cargo checksums; disposable
  export deleted. /tmp/tsrust-valuable-cli-build.json and
  /tmp/tsrust-valuable-export-result.json. JSfocused2tests remainPASS0.355s.
  No active Cargo/rustc processes at initial process inspection.
- Accepted alias/feature assertions from a3d9a82e plus narrow physical-default
  annotation repair: host build4.715s/413991704B; feature selection14rowsPASS1.564s;
  iterator private/extracted-method negativesPASS0.555s. Original shared native
  build failed10.462s with E0282, preserved in /tmp/tsrust-alias-feature-acceptance.json.
  Repaired deferred borrowed union owner now names INITIALIZER physical type;
  ordinary non-union defaults/ownership/evaluation unchanged. Existing unit guard
  retains physical enum check and verifies complete owner annotation.
- ACTUAL repair PASS /tmp/tsrust-alias-feature-repair.json: build385.896s/
  3387248888B, terminal Cargo success; focused physical-default guard1PASS1.067s.
  TWO existing std:testing files share ONE native runner:18PASS/0FAIL/0SKIP,
  native build/link2522.47175ms, check1275.527208ms, total setup4079.676458ms;
  test execution3.244125ms. Outer runner5.225s/392691712B, remaining[]/stopnull.
  Same original900s aggregate scope, not a new budget or full regression claim.
  Removed61 newly-created loose codegen objects23723224 logical bytes after
  all owned processes settled; kept libraries, metadata, binaries and shared caches.
  /tmp/tsrust-alias-feature-object-cleanup.json. No build active.
- B00 pin/wiring checklist reconciled with current54 common direct manifest
  declarations/lock identities, reserved-package regression and actual relocated
  exports. Broader package admission, optional-selection/minimal-program footprint
  and four-host acceptance remain open. Reuse already-passed regex/locale/process/
  network/compression/hostname selection controls; do not replay their nativeapps.
- Platform selector repair: all7 existing B01 filesystem targets now selected,
  46 intended integration targets/49 validated Cargo artifacts. Existing buffered
  backend harness includes actual fs.rs O0/O3 tests, including Windows exclusive
  open conflicts after explicit close/scope drop. Selector/watchdog unit tests
  ACTUAL PASS15/0FAIL0.093s. No native family rerun or new remote execution yet;
  55-minute aggregate fit and four-host execution remain pending. Guide corrected
  to describe one Cargo build and per-target pending-case batches.
- Owner permits profiling/additional test methods. Existing public test --profile
  separates check/emit/backend/cache/native build and per-case execution timings;
  use stable std:testing suites and cache, not independent programs per case.
  Single observations remain distinct from sampled benchmark claims. No new
  profiling framework or repeated accepted app builds were added.
- B20localintegratedapp rownowcompletedwithqualifiedMac evidence; nativefourhost/
  finalregressions/remainingB00review remainunchecked. Namedfs/HTTPdefaultmigrations
  documentedascompletedwithoutnarrowingparentcross-exportdefaultrequirement.
- Netcleanup oldhost1430007808B +obsoletebasepack766394368B. Currentbase/Shlexpacks
  protected; rawnewobjectcleanupseparate. Shareddepsretained. No push/publication.
- TERMINAL PASS current03ec4f13 repair: selectedbuild394.945s/3276963KiB,
  two focused compiler guards PASS, real documentation native case PASS119.67s.
  /tmp/tsrust-fixed-callback-repair-76986fc9793f.json.
  Complete compiler library1538PASS/8unchanged opt-ins96.549s; LSP27PASS0.559s.
  /tmp/tsrust-fixed-callback-family-libraries.json terminal-success. Outer public
  phase518.307s, library98.167s; stopnull/remaining[] both. Session65004 closed.
  No compiler rebuild currently running. Preserve all these accepted scopes.
- Integratedservice ACTUALfrontendFAIL0.559s BEFORE nativeapp build:
  /tmp/tsrust-current-integrated-service.json +.err. False `routes must return App`
  at examples/service-observability/main.tsr45; function explicitly returnsapp.
  Adding UNUSED exercise async try/finally calling routes reproducesfalseerror;
  same routes/body withoutunusedexercise checks. B06/B14 investigate consumed-body
  retry/checked-function invalidation, not an outdatedApp API. No speculative
  productionfix yet; do not inline/removeobservability or weaken service checks.
  No proven SERVICE_BINARY exists; afterfrontendrepair the existing harness needs
  ONEappbuild thenrestarts SAMEexecutable foractualservicecontracts.
- Existingpublic_api_inventory example build reused compiler/dependencies and
  finished4.158s; no compilerlib rebuild. Tool execution exposed private internal
  NativeTypeId lookup assumptions: PoolSubmission explicitly classified pending
  commit; IteratorCursor remains tool-only reconciliation gap. No completed current
  inventory/per-export acceptance claim. Root is only canonical writer.
- Second repair terminal: selected build PASS396.529s/3269491KiB; overall
 573.204s/3379566016B, stopnull, remaining[]. Session54291 closed; no build live.
 /tmp/tsrust-physical-carrier-repair-resume-6a381ec5ec6a.json.
- Positional cache key, coalescing context and owned-field replacement witnesses
 passed. Generic-layout fixture still failed because its base callback was owned
 rather than scoped; readonly-base fixture correction is pending native validation.
- Actual sharedHTTP std:testing suite PASS all3 cases: existing request streaming,
 real4097-byte route/syntax rejects, exact maxAge34560001/i64MAX headers and
 negative values/cleanup. One public runner/build; host58.71s including setup,
 not a comparable benchmark. Test now prints existing profile/build/test JSON
 events for future visibility. Do not rerun passed native cases for logging only.
- Documentation remains FAIL107E0308, all spread-boundary field mismatches; five
 positional call errors resolved. f858 physical-spread lookup did not fix actual
 failures and was removed, preserving canonical constructor repair. Tiny valid
 public JSX repro proves missing conversion at FIXED callback argument boundary.
 Reviewed replacement uses existing checked_carrier only after fixed invocation
 resolution; named/forwarded functions and methods retain physical specialization.
 Exact new uppercase JSX fixture frontend-check PASS before build.
- All1530 compiler/27LSP earlier library, rust_bridge8/nullish7/LSPbridge8 and5/6
 earlier docs passes retained. Actual generic fixture + new fixed-boundary guard
 and failed docs case next; complete libraries at this ownership-family milestone
 reuse the same build. Eight existing opt-ins remain unchanged, never forced.
- Three current production lock audits ACTUAL0vulnerabilities. Backend0warnings;
 workspace/native each four unmaintained +two yanked retained optionalUI packages.
 DB309392b18f6b17022f82c701f300433126901068, updated2026-11-06. Exact lock hashes,
 activation chains and qualifications are in backend-distribution-inventory.md.
 /tmp/tsrust-current-production-audit-summary.json. No warnings suppressed.
- Net obsolete host cleanup reclaimed1430007808B. Current selected artifacts and
 application dependencies preserved. Per-build new-object cleanup30068736B and
 130850816B are separate, not additional net historical recovery.
- Original30minute resume guard refused before launching another process when
 insufficient time remained. Original failed scope unchanged. Subsequent repair
 was independently capped30minutes AND first repair start+onehour. Next driver
 /tmp/tsrust-fixed-callback-repair-run.py keeps that same absolute one-hour ceiling.
 Do not extend it or claim budget exhaustion passed. After real doc acceptance,
 resume existing integrated service and remaining B00/B20/native platforms.
Only root writes canonical; app worktree owner WIP stays protected.
- ACTUAL PASS /tmp/tsrust-http-factory-portable-repair-422a7d392a0e.json:
  HTTP forwarder4, shared55 EACH debug/release plus filtered cachehit/nativeBuildMs0
  EACH, focused HTTP LSP3, middleware alias/native+relocated exports, real Shlex
  generated adapter+relocated empty-Cargo-home locked/offline exports at both profiles.
  Shared55 execution162.818459/67.557667ms; setup68631.7925/30725.667ms. Correctness
  observations, NOT comparable timings. No new skips/weakening/unsafe.
- Initial compiler build terminalPASS636.673s/4163284KiB; orchestration omitted
  LSP package and failed artifact assertion before tests. Original artifacts SHA
  retained. LSP-only alternate selection interrupted50.988s before compiler rebuild.
  Correct shared package selection reused compiler and added LSP14.488s/711952KiB.
  Resume terminalPASS400.573s/1668041368B,stopnull,remaining[]. All within SAME
  original1800s scope /tmp/tsrust-http-factory-portable-repair-scope.json,
  startedUnix1794096899.864186. Sessions53382/95717/24361 all terminal; no run live.
- Source fixes accepted: existing checked factory forwarding now includes stored
  secureHeaders/compress, plainMiddleware direct return and reentry-safe reserved
  Symbol dedup. Shlex bindingsresolve explicitly prepares --portable; export
  remains Cargo-free. All B10 API boxes now reflect actual passed evidence.
- B19 generator/customization/rustdoc57/fallback/editor3/3 retained19b; nominal
  nested String pointer/capacity moves O0/O3 retained19c. RealShlex paired with
  compiler-owned regex backend5/export passes at965 closes package-witness scope.
  Historical19b/19c nested bridge failure is now repaired: all8 compiler rust_bridge
  cases passed in the ownership scope. Preserve that pass; do not replay the
  already-passed adapter/native/export controls.
- Retain byte68 EACHdebug/release PASS /tmp/tsrust-nullable-storage-proof-front-30bc05cbad9c.json;
  actual10002node ABI/pointers/deepdrop/O0O3 PASS /tmp/tsrust-abi-witness-repair-c5252aa3173b.json;
  full compiler library1528+repairedHTTPpair =all1529runnable,8unchanged opt-ins;
  LSP27PASS /tmp/tsrust-closed-runtime-library-milestone.json plus
  /tmp/tsrust-http-body-limit-proof-28fed979d29f.json. Do not replay for harness edits.
- Cleanup quiet/locked /tmp/tsrust-closed-http-caches-cleanup.json recovered
  1603506176B actual:3source-proven obsolete native cache entries+closed old
  target/http-realtime-public-package. Current suites/shareddeps/controls retained.
  Per-build rcgu cleanup recovered1140387840B+96890880B+334708736B newly created
  objects; do not sum as net historical recovery.
- Reviewed pending harness changes reuse explicit validated TSRUST rather than
  rebuilding release compiler in run.mjs/profile-native.mjs. Logging allocation
  driver now supports existing checked Result ABI/required pin_project_lite and
  source guards. Pure focused checks6PASS; whole-project format0changes before
  latest evidence docs; repeat required formatter before source checkpoint.
- Existing coverage guide now lists245 added catalog namespace identities since
  preplan8efaaee21bcc;520 current backend namespace rows documented. This does NOT
  close receiver/source-only/per-overload/platform/default/advisory acceptance.
- ACTUAL B11 native measurement/ingest PASS: /tmp/tsrust-b11-fresh-public-native-sourcepack-resume.json,
  session24482 terminal0. First source-pack environment mistake failed1.288s before
  build; correctedresume used target/portable-source-packs/SAME original30minbudget.
  Emit14.153s,releasebuild130.698s/peak554713856B,publicprotocol198.282s/508788736B,
  ingest0.539s. Resume344.550s; no quiet bypass. Strict guard waited for Spotlight
  load13->below4, thenall6nativecases original3x31x128 +separate resourcepasses.
  SQLite18workload/engine rows/history18; each93samples/11904latencies.
  Native at2026-11-08T00:43:50.142Z, unchangedRust/Node at2026-11-07T10:50:14.417Z.
  WSbatch/nativeRust ratio0.91–1.04,Node0.83–0.92; SSE slower1.12–1.20Rust/
  1.06–1.10Node. DirectSSEbackendfloor lacks requiredpublicchannelproducerbridge;
  no hiddencontractweakening/optimizationclaim. Actualstore:ingest Added18entries.
  Buildpackage target/http-realtime-public-package nowclosed/disposable; controls
  and shareddeps retained. No indexing settings/path changes were made.
- Harness6purechecksPASS,whole-project format0changes/diffclean beforeddc61ed6.
- URL ACTUAL measurement/ingest PASS, current source75027f23:
  /tmp/tsrust-url-family-resume.json session33695 terminal0, protocol157.180s/
  peak571290008B,ingest0.558s,resume158.085s; SAME original30minscope as initial
  failed373.490s preparation. Initial10th TSquerysnapshot lackedDOM.Iterable;
  affected tsc --noEmit PASS0.201s, fixcheckpoint75027f23. Sealed first9artifact
  prefix receipt /tmp/tsrust-url-prepared-prefix.json: currentCLI/input/generated/
  binaryhashes +priorfailedscope; exactfrontendregenerationmatched, zerofirst9
  recompiles; existing5-shape correctness rerun, remaining2preparednormally.
  All11native/Rust/Node cases165correctness +3x31after8warmups; SQLite33rows/
  histories33,each93samples,at2026-11-08T01:00:48.357Z. Native authority/path
  scaling8192->65536 doubles1.85–2.11long/1.93–1.98short;65536medians1.645375/
  1.156375ms vsRust1.638458/1.188250,Node0.579333/0.497042. Boundedlinear
  corpus,notuniversalcomplexity/WHATWGclaim. B00URLinvestigation checkboxclosed.
  Generalnative/Rust components/query/setters1.06/0.92/0.96;Node0.66/0.71/0.12.
- Realtime closed-package cleanup /tmp/tsrust-b11-closed-package-cleanup.json
  recovered773849088B actual newly materialized vendor output; controls retained.
- B17 disabled-event allocation proofACTUAL: original profilebuilt30.313s then
  FAILED genericzero-retention validation. SAMEcompiledbinary,3freshprocesses
  eachn0/1/32/1024 all6alloc/0realloc/33656requested+peak/776retained+afterDrop.
  /tmp/tsrust-disabled-logging-first-call-counts.json +source/binaryseals in
  /tmp/tsrust-logging-family-resume.json. Independentchecksums +factoryneverinvoke;
  eventincrement0. B17checkboxclosednarrowly; NOT zero-whole-workloadretention/
  leakproof. Genericprofileassertunchanged/explicitlyFAILED, no probe rebuild.
- Matched logging3 firstattempt stopped32.924s at handwrittenRust ABI adaptation,
  beforetiming. Preserveauthored fnfunction0(n:i64,seed:i64) source/hash unchanged;
  reuseexistingreferenceexception pattern with explicitguard, nativeABI staysstrict.
  Focusedpure4PASS,format0changes. Resume SAME original30minscope aftercheckpoint;
  currentCLI/compilerRust unchanged. No logging timings/ingest claimed.
- Logging matchedsession70736 stopped31.879s on secondworkload's actual movedpath
  error; firstdisabledworkload prepared/allcorrectnesspassed. Repair ONLY benchmark
  sources: retain declaredLoggerOptions, narrowoptionalpath/readsamefieldafterclose;
  child typedJSONspan usesexistingnamedSpanFields alias,notunsupportedinlineobject
  type. BothsourcechecksPASS/focusedpure4PASS. Sourceoracles/fingerprintsupdated;
  Rust/Nodecontrols unchanged. Existingsealed-prefixguard nowalsoallowslogging
  family withfullsource/native/TS/hand hashes andfrontendregeneration. Sealed
  completeddisabledcase /tmp/tsrust-logging-prepared-prefix.json; no disabled
  program/probe/compiler rebuildneeded. Remaining2prepare normally.
- Logging sourcesession82879 stopped34.027s beforetiming: secondcontrolimports
  node:fs/os/path butfamilyomittedfromexistingNode typeRoots/types selector.
  One-linefamilyselectionrepair; BOTH filecontrol TypeScript --noEmit preflights
  PASS,whole-projectformat0changes/diffclean. Reuse SAME sealeddisabledprefix
  and SAME original30minscope; no compiler/probe rebuild or controlsource edits.
- B17 ACTUAL MATCHEDTIMING/INGEST PASS: /tmp/tsrust-logging-family-node-resume.json,
  session78133 terminal0,protocol164.558s/525921992B,ingest0.561s,resume165.554s.
  SAMEoriginal30minscope includingearlierprofile/wiring repairs; no quietbypass.
  All3workloads45correctness +3x31after8warmups; SQLite9rows/history9 each93samples,
  at2026-11-08T01:24:07.067Z. Native disabled/file/child0.009375/3.058417/
  5.668209ms; Rust0.005666/3.198125/6.186750;Node0.027625/3.622375/4.101875.
  Wholepipelines includecreate/close/checksum; child1.38xNodeslower retained.
  Zero-eventallocationdelta and776Bfixedfirstcallretention remainseparate,strict
  genericprofileFAILEDnotretroactivelypassed. Rust/Nodecontrols unchanged.
- ACTUAL affectedownership scope: /tmp/tsrust-final-ownership-acceptance-788f019d501b.json,
  session37998 terminal1/stopnull/remaining[]. SharedbuildPASS648.082s/4175300KiB;
  phase796.959s/4297971928B; cleanup1330298880B newlycreatedobjectsrecovered.
  All8compilerRustbridge (inclpreviousfailednestedtags/drops),7nullish,8LSPbridge,
  5of6documentationcontentPASS. B19firstcheckboxclosed; preserveallpassedcases.
  SoleFAIL authored_content_api_inventory_and_exact_examples_build_into_checked_native_data:
  RustmissingRecord261/290/... constructoraliases,247reportederrors/cascades.
- SAMEoriginalscope librarymilestone ACTUAL PASS, session36182 terminal0:
  /tmp/tsrust-stdlib-family-final-libraries.json,compiler1530PASS/8UNCHANGEDoptins
  106.487s/1262978632B; LSP27PASS0.566s/5915080B. No newbuild forlibraryruns.
  Scopeoriginalstarted1794101250.8819342,budget1800; completedwithoutoverrun.
- Reviewed pending rootfix /tmp/tsrust-object-spread-native-declaration.patch
  SHA1a50d8bff6371a29ee5f18afc42d4a6e25ce580727276111a088567a095a3756:
  object_spread_value_with constructorusesexistingnative_record_declaration(result),
  matchingdeclarationloop/plainliterals. Resultfieldlayout/coercions/orderunchanged.
  Existingnative_view_once_family_emits_real_alternative_lifetimes assert_family
  nowchecksnormalizedconstructornames/bothmutablemodes/singleinput/keyorder.
  SourcepeerreviewPASS; nativeDocfreshdiagnosticsstillrequired,notall247fixedclaimed.
- NEXT targeted source-repair scope,30min/5GB AND originalmilestoneabsolute1hceiling:
  buildnormalcompiler/LSP+updatedlibonce; runONLYupdatedscoped-family unit +failed
  documentationcase, preserve1530/27/8/7/8passes. OriginalscopeAretainsdocFAIL;
  don'treset/labelitpassed orrerunwhole4targets. Oldprepared command belowisNOTnext:
  prepared
  /tmp/tsrust-final-ownership-acceptance-run.py +gates/scopes/cleanup sameprefix.
  One shared compiler+LSP selectedbuild,explicitbothpackages+samefeatures,rootlibs
  buildcacheonlyNOTexecuted; whole rust_bridge/documentation_content/nullish_coalescing
  compiler targets +LSPrust_bridge. Includes lastfailednestedOption/Result/Vec tags/
  drops. Fixed30min/5GB/jobs1/incremental0/threads3; oldactualexecution~161s.
  Preserve1529library/27LSP/55/68/24/42passes; thisisaffectedblock,NOTfullregression.
  Existingfullintegrationoldtotals doNOTprove55minfit; oldbatcheddriverhascohort
  editorfilters andmustNOTbeusedunchangedforfullclaims.
- Remaining: B17 logging_disabled public
  allocation profile (existing driver, separate setup/event costs). Remaining
  B00 ledger/package/platform, B19nested integration, B20 apps/final regression/
  platform/measurements/release acceptance stay open. Integrated persistent API
  control tools/check-service.mjs supports SERVICE_BINARY reuse; otherwise build
  ONE app. Fullgoalactive; no push/merge/v0.2publication approved or claimed.

Latest accepted section — public55case native runtime; full v0.2.0 remains active:

- Current canonical checkpoint7c171bb8; uncommitted targeted compiler repairs
  are under bounded affected validation detailed below. Accepted runtime fixes are retained
  atc7e8eced. Only root writes canonical; app worktree edits stay protected.
- ACTUAL PASS: /tmp/tsrust-upgrade-handoff-4366908885f1.json. Selected fresh normal
  compiler CLI build terminal success,226.058s/peak2791138KiB. Shared public host
  public_registered_suites_share_one_native_build PASS; all55cases/5files,
  zero failures/skips/hooks. Exact logging10rows/order/concurrent request fields,
  realtime18cases,iterator32cases/catalog outputs and55names all verified by host.
  Execution155.242166ms; setup17567.628416ms (check8311.2005,emit348.149792,
  backend45.198542,cache0.302,nativeBuild8832.318167ms/cachemiss). These are test
  observations,not comparable benchmark measurements or SQLite runs.
- Final bounded run246.728s/peak2889995040B,stopnull,remaining[]. It and automatic
  cleanup fit /tmp/tsrust-upgrade-handoff-scope.json's original15minute ceiling.
  LSP was not rebuilt for backend-only fixes; existing editor passes retained.
- Accepted fixes: retain computed readonly inputs of checked Task factories and
  optional borrowed-union payload owners; preserve ordinary mutable union ABI.
  Enable Hyper serve_connection_with_upgrades for actual socket handoff. Stop
  close draining on acknowledged peer Close,keeping prior transport errors real.
  No TLS verification weakening,unsafe,hidden clones or raised API limits.
- Logging fixture corrected to Rust cancellation semantics: cancelled future drops
  owners; source finally only runs when reached. Explicit cleanup log follows
  awaited cancellation settlement in caller; original requested/cancelled asserts
  and exact ten-row oracle preserved. Public futures reference documents this.
- Earlier native executions50/55 then54/55 are preserved in prior gate JSONs;
  actual failures drove these repairs. Two filtered diagnostic reruns used the
  same stable55registered runner (18.255s/19.782s),not per-case native sources.
  Temporary stage logs removed. Existing tagged-literal,WebSocket literal,
  17core/12iterator/12editor receiver passes remain retained and unrerun.
- All source is whole-project formatted,diff--check clean. Cleanup THIS
  continuation recovered5172674560B actual free space: obsolete linked objects,
 105compiler/LSP executable consumers older than changed emitter source,and stale
  compiler metadata. Current consumers/shared dependencies/independent Rust-Node
  controls retained. Quiet processes/shared locks checked before deletion. Reports
  /tmp/tsrust-*-objects-cleanup.json and
  /tmp/tsrust-task-input-obsolete-executables-cleanup.json. Earlier1.02GB/5.71GB/
 491MB recoveries are separate; do not double count.
- Portable source setup PASSED offline15.1s. Corrected benchmark SSE callback to
  async before starting its producer Job; original runtime protocols unchanged.
  Public SSE/WebSocket server exports and release-builds offline PASS134.948s,
  peak553583384B. Existing Rust/Node controls remain unchanged/cached; exact Node
  ws8.18.3 archive materialized offline and package proof passes.
- Comparable measurement deliberately INTERRUPTED with zero network/resource
  groups: quiet guard saw sustained load12–14>=4 (Spotlight indexing and Codex
  renderer). No timings/SQLite ingestion claimed; no --allow-busy bypass. Driver
  /tmp/tsrust-b11-public-native-resume-measure.py ended -2,remaining[],404.275s;
  wrapper /tmp/tsrust-b11-accepted-public-resume-results.json. Optimized public
  package retained at target/http-realtime-public-package. Its normal program
  alias will need fresh validation/proof after exports overwrite that alias.
- Relocated offline exports PASS: /tmp/tsrust-upgrade-handoff-d970ae7bc9e7.json;
  realtime18cases at debug+release,empty Cargo home,local TLS fixtures,forbid unsafe;
  logging5cases/10exact rows at both profiles. Hosts247.074s/221.528s; total
 474.781s,peak884623928B,stopnull,remaining[],inside30min scope. Cleanup339607552B.
- Temporary TLS runtime paths validated: all55pass156.562333ms, but changing
  TSRUST_REALTIME_TLS_DIR every host invocation caused cache misses. Actual field
  comparison proved generated Rust(1988485B SHA0f836a623062...) and graph(103933B
  SHAdafb1961435e...) identical; only backend_sources application environment
  identity differed. Cache identity deliberately includes all build-script env.
- FIXED IN HARNESS,not weakened cache key: stable private temporary fixture path
  hashes workspace+six fixture contents; standard file lock protects staging.
  Runtime never reads Desktop certs. Reusing SAME compiler command with a filter
  proves cache hit/nativeBuildMs0,keeping initial55JSON rows intact for benchmarks.
  /tmp/tsrust-stable-family-6135303b69f6.json structured_logging PASS28.056s;
  all55nativecases147.86ms,then one filtered cached case. No arbitrary env excluded.
- All three prepared migrations below are now APPLIED+whole-project formatted:
 42collection cases(default),24crypto(debug/release),67bytes(debug/release).
  Original rejections/ABI controls/profile requirements retained. Formatter112files
  then0/idempotent; blank-only Rust trailing whitespace corrected. Acceptance
  remains pending; do not claim177former native launches are already replaced
  with accepted passes until actual new suites finish.
- Latest 30min family scope /tmp/tsrust-stable-family-run.py stopped MEMORY after
 45.355s at5233890872B. Structured logging/cache passed; collections failed native
  generic spread/join emission; crypto failed a source guard (export formatting
  inserts a blank separator); bytes check grew beyond5GB before native build.
  No higher limits/restarts. Cleanup quiet process check passed,recovered26673152B.
- Focused diagnostic ONLY: /tmp/tsrust-bytes-memory-profile.py paused checker at
 800408728B,sampled1s then killed it;2.483s total,no native compile/acceptance.
  Stack /tmp/tsrust-bytes-memory-profile.sample.txt43–67 proves CHECK phase:
  discover_record_callbacks -> prepare_record_child -> absent_nullable_record_annotation
  -> empty_environment_work -> absent_callback. empty_environment_work follows
  recursive Holder.next?:Holder without active/memo state,unbounded postorder
  frames; absent_callback identities themselves are cached. B14 reviewer owns
  finite recursive empty-environment repair; keep all67cases intact.
- Current source checkpoint c057307c preserves all147migration/export/cache files;
  it is NOT family acceptance. Uncommitted compiler repairs now APPLIED: collection
  spread/join373602cb...,crypto sourceguard336270ff... (public lexer-backed
  documentation highlight API used because syntax module is private),recursive
  empty environments3ec1e44c.... Root strengthened new mutual-view fixture with
  exact3/7/3 bytes in addition to lengths; bytes suite now68,all original67 intact.
- Recursive repair uses operation-local SCC traversal with record/map modes,
  finite reserve/remap for all component members,one full signature cache key,
  unchanged-layout reuse and already-absent callback reuse. No new global registry,
  depth caps or weakened ownership. Four focused private controls cover self,
  mutual both-callback,map/optional-map/maps-only,cache aliasing and idempotence;
  prior16k unchanged-chain/deep callback controls retained.
- Recursive-family gate TERMINAL: /tmp/tsrust-recursive-family-400197d26d77.json.
  Fresh lib/CLI/hosts build531.961s,peak4109828KiB; selected10empty/SCC/recursive
  unit controls PASS0.629s;55/cache PASS27.443s;collection42 PASS3.203s. Crypto
  source-count guard expected7 but actual8(seven field examples+shared header);
  fixed with independent8hash/9reference proof755c08c4.... Bytes now checks1.31s/
 113936KiB,then fails existing case14 native-view-call at child.read,not memory.
  End-to-end gates569.347s/peak4243298616B,remaining[];cleanup933388288B.
- Real failure-only metadata trace added under EXISTING TSRUST_TRACE_CARRIERS.
  CLI-only diagnostic build267.624s,then same68suite check2.121s/113311744B,
  original30min scope unchanged and actual work stopped before it expired.
  /tmp/tsrust-view-guard-diagnostic-results.json. All discarded narrow pairs were
  legitimate Result Err(95<-94 and101<-100);NO flow drops. Final holder106/child106
  bindings have NO loans. Therefore speculative guard-remapping patch5f827da...
  is explicitly NOT APPLIED; do not revive it as the current repair.
- Proven producer bug: named function native summaries defer discovering_results,
  but closure/family registration did not. Early callback summaries enable loan
  checking before named read/build factories publish owner data. Applied ONLY
  two matching discovery guards +one focused definitive-missing-owner control:
  /tmp/tsrust-native-discovery-summary.patch SHA04afd8c6b90a8d63903adfff9fe2400004288483f91e9907d301f6b44c9be621.
  No lifetime/narrow check relaxed. Crypto actualcount8guard fixed too.
- Native-summary scope TERMINAL: preflight60.369s; selected lib/CLI/hosts build
  PASS532.596s/peak4141412KiB; three native-view-callee controls PASS and55/cache
  PASS. Crypto now reaches real await-task error at imported catalog async main;
  bytes still rejects existing case14 child.read lifetime. Full gate567.01s,
  peak4288731352B,remaining[]; cleanup932052992B. Evidence:
  /tmp/tsrust-native-summary-42b94eaef168.json. Do not repeat passed scopes.
- Post-discovery frontend-only trace1.475s confirms final holder/child loansNone,
  no physical guard loss. /tmp/tsrust-native-summary-post-discovery.err.
- Imported async main root proven: parser synthesizes a synchronous executable
  main wrapper even for imported modules. Applied link-boundary normalization
  restoring imported Task factory, preserving actual entry wrappers. Named/default
  import and wrong-result-type checks added; not yet built/accepted. Patch
  /tmp/tsrust-imported-async-main-factory.patch SHA26a4df4883ffd07efbaa64279304f614aa4504a7d1f04242580cb55b876e6e87.
- Applied effects-only phase repair82a969c6: return-summary recording already
  skips access inference; named/closure/family registration now matches it. This
  prevents real callback summaries consuming intentionally absent inferred loans.
  Definitive missing-owner rejection retained; unit extension compile-preflighted
  but not independently replayed. Failure trace now identifies effects_only.
- Native-effects scope TERMINAL /tmp/tsrust-native-effects-75f59227bec7.json:
  preflight63.612s PASS; one CLI/three-host build227.184s PASS,peak2837810KiB.
  Crypto24cases PASS BOTH debug/release,all source guards retained; host36.294s.
  Actual debug setup10070.018ms/native9630.549ms/runtime2474.350ms; release
  setup22471.437ms/native22088.604ms/runtime138.365ms. Test observations only,
  not controlled benchmark measurements. Named/default async-import+wrong-type
  regression PASS0.573s. No library or previously passed suites replayed.
- Bytes advances past original case14 lifetime failure. New mutual-recursion
  witness fails frontend at root.next = second(read(right)): Expected Second |
  undefined,found Second. Preserve all68 and3/7/3 oracle; next repair is concrete
  physical-layout assignment compatibility,not the already-fixed missing loans.
  Total gates270.352s,peak2967212712B,stopnull,remaining[]. Quiet automatic
  cleanup123rcgu objects recovered433356800B; heavy slot free. Agent B14 reviewing
  new actual failure. Collections42,55/cache,10recursive/3callee units and portable
  exports retained; no new SQLite timings or full-goal completion claimed.
- Applied new checked-native-record eligibility patch b5fa0513 after source
  checkpoint2459529f. Exact nominal/generic arguments and nullish domains retained;
  recognizes physical scoped-native callable families with actual view lifetimes,
  rather than broadening aggregate ownership facts. Optional injection,nullable
  record compatibility and conditional join share this narrow condition. New parsed
  factory unit covers physical retention,wrong nominal and mismatched nullish domain.
- Native-carrier affected driver TERMINAL: /tmp/tsrust-native-carrier-abf5dc0317df.json.
  Preflight compiler+LSP tests PASS73.383s; selected CLI/LSP/lib/hosts build
  PASS547.739s/peak4109172KiB. Four focused compiler units PASS;27LSP library
  tests PASS; byte callable/editor agreement PASS; std_bytes25/28 PASS.
- Remaining bytes: all68 now pass frontend,then native_capture_storage.rs206
  panics at resolved native environment during EMIT. B06 investigates emitter.
  Two existing negative hosts(nullable recursive factory joins/mutable recursive
  stores) still reject,but incorrectly report missing Holder return +missing
  capture lifetime instead of required native-view-escape; B14 investigates phase
  consistency. Neither error assertion weakened; original cases/oracles retained.
- Affected gates568.461s,peak4225407168B,stopnull,remaining[]. Automatic quiet
  cleanup135rcgu objects recovered1007636480B. No further build live.
- Full compiler library milestone TERMINAL,NO Cargo build: reused exact source
  abf5dc0317df/binaryed18f6bb... under original30minute carrier scope. Actual
 1514PASS,4FAIL,8known opt-in probes/external-SQL ignores,zero filtered.111.546s,
  peak1256932096B,stopnull,remaining[]. Same exact eight ignores as prior baseline;
  /tmp/tsrust-native-carrier-abf5dc0317df-full-library.json and .out/.err. Preserve
 1514passes; do NOT replay full library after routine repairs.
- Four full-library failures diagnosed and repairs APPLIED:
  native_capture_storage panic shared with byte suite: physical lifetime queries
  traversed symbolic recursive fields omitted by native generic renderer. Reuse
  exact scoped_record_native_field predicate at all four query callers; source
  capture analysis remains full. No count fallback,cap or unsafe. ac982d44 patch.
  Dependency guard omitted6runtime names: futures-util,tokio-tungstenite,
  webpki-roots,pin-project-lite,tracing,tracing-subscriber. ce8a3ecc reserves all
  runtime names and tests bothschemas for all65native/54backend manifest entries.
  Two stale test expectations corrected by3f0bb5dd: actual Task result is canonical
  Callable contract; unreachable return after always-returning loop is now an
  explicit negative,while storage assertions stay intact. Root used err().unwrap()
  for CheckedProgram (does not implement Debug),avoiding a private-test compile bug.
- Existing byte negative errors traced to swallowed discovery failures consuming
  source bodies. c880cd6d restores ONLY active unchecked consumed functions from
  existing discovery_body_sources before resetting speculative stack. Definitive
  ownership rejection unchanged. New test proves original return/error survives.
- Runtime-family-repair driver TERMINAL: preflight72.963s PASS; selected build
 554.258s PASS/peak4126676KiB. Six of eight targeted compiler controls PASS:
  canonical Task contract,returning cleanup loop,all runtime pin reservations,
  native-capture three controls. New restore-body unit and existing recursive
  native-family Rust witness still FAIL; byte three hosts still FAIL; editor PASS.
  /tmp/tsrust-runtime-family-repair-faecbb5eb2da.json; gates574.75s,
  peak4259010848B,stopnull,remaining[];cleanup1005764608B recovered.
- Concrete remaining causes: function continuation drains function_stack BEFORE
  discovery receives the ignored error,so prior active-stack restoration could
  never repair failed bodies. Corrected44c4e4c restores unchecked consumed bodies
  from existing retained map; failing regression unchanged. Applied.
- Native count panic resolved; recursive native witness now reaches Rust E0425
  missing Record2: native declaration chose nominal ID for lifetime-free cycles,
  while Drop/Link/Array used concrete ID. Applied9904689e uses cyclic(record),
  matching the existing all-cyclic concrete-field invariant. Same patch omits
  ONLY unchanged nested array identity passthroughs for cyclic concrete storage
  from block tuple shadowing. The owning Map leaf panic was an ARRAY epoch caller,
  not a Map operation. Added failure-only function/binding/path/types metadata
  if that unobserved cyclic classification proves insufficient.
- CLI-first corrected-source driver TERMINAL: normal CLI/host build223.187s
  PASS/peak2764738KiB. BOTH byte ownership-negative hosts now PASS with their
  original native-view-escape diagnostics. Byte68 progresses past earlier emitter
  panics,then compiler STACK OVERFLOW before native compilation. No unit binary
  rebuild was attempted. /tmp/tsrust-runtime-family-frontend-f86a8673b358.json;
  total236.654s/peak2877837992B,stopnull,remaining[];cleanup400375808B.
- Actual profiling: /tmp/tsrust-byte-cyclic-stack-paused.sample.txt. Emit-only
  CLI paused at256590592B,one-second stack sample then killed(total11.309s).
 756samples show record_carrier_conversion repeatedly alternating Record/Union;
  no native program compilation or raised limits. Earlier two-second sample
  captured analysis only and was inconclusive; do not use it as root-cause proof.
- Applied480cb5a0: reuse existing absent_record_field guard to evaluate impossible
  record source once and match its empty required callback enum BEFORE expanding
  recursive sibling fields. Root retained INFERRED local type so boxed/RecordLink
  union payloads retain their actual physical representation; no forced RecordN
  annotation. Remaining cyclic conversions expose type IDs only under existing
  TSRUST_TRACE_CARRIERS. No new registry/cap/unsafe or relaxed ownership.
- Original runtime-family30minute scope is closed INCOMPLETE; both real byte
  negative failures are repaired,while byte68 native and two private units remain
  required. All its build/test/diagnostic processes are terminal. No pass claimed
  for stackoverflow and no reset of that scope. Next source-changed acceptance
  covers the remaining one byte host,then two private failures only after byte
  frontend/emission succeeds; preserve all earlier passes.
- Public status now lists actual source-build v0.2 feature families and links
  test/profiling guide; platform/package/performance/final acceptance explicitly
  remains incomplete. No publication/push/merge approved or performed.
- Byte-family closeout first stage TERMINAL: source e339008f/7864fd53ca4b.
  NormalCLI/host build PASS221.954s/peak2851442KiB;byte68 still STACK OVERFLOW,
  so private-unit build correctly NOT started. /tmp/tsrust-byte-family-closeout-
  front-7864fd53ca4b.json;total235.349s/peak2973078208B,remaining[],cleanup400326656B.
  Existing absent-record guard is sound but did not cover actual repeating pair.
- Exact trace /tmp/tsrust-byte-closeout-cycle-trace.err: source139→target140 once,
  source138→target140 repeated453432times; both nominal declaration70. Inhabited
  physical layouts need inspection; do not invent a general recursive converter
  or weaken scope without actual field/family evidence.
- LLDB diagnostic stalled before target launch (inferior688KB/suspended). Stopped
  after~50s; verified and killed owned debugger31843/debugserver31933/target31932.
  No debugger layout proof; no further debugger attempts. Public Analysis hides
  raw physical records,so an external probe would not provide private metadata.
- Carrier-layout diagnostic build TERMINAL: /tmp/tsrust-byte-carrier-layout-
  build.json PASS220.162s/peak2962182944B; actual Cargo build218.881s,cleanup
 400519168B. Native/user tests NOT run. --timings report shows compiler library
 197.23s(frontend13.91/codegen183.32),CLI18.42s,host2.33s; cached dependencies0s.
- Captured ONLY3layout rows then killed emit-only compiler after10.297s:
  /tmp/tsrust-byte-cyclic-layout-metadata.txt. Module52Holder is ORIGINAL byte
  case50 nullable-recursive-callbacks-join-factories-9 (reversed declarations).
  Source138readUnion86,next104;target140readUnion106,next107. Read is OPTIONAL,
  so source is inhabited. No safe nominal alias or absence shortcut exists.
- Applied reviewed bounded owned carrier worklist patch091ede6a (4files,new
  record_carrier_worklist.rs). Finite expression-local type-pair Enter/Finish
  tasks move records/unions/nested arrays with heap worklists,not runtime recursive
  calls. Existing union_move_payload/union_store_payload handle Link ownership;
  identical payloads transfer directly. Field order/key_order and partial-owner
  Drop retained; no cloning,unsafe,caps or globalregistry. Task generics use only
  source types,Node onlytarget,function either,avoiding zero→one unused lifetime.
- Existing recursive nativefamily witness extended with10002convertednodes on
  its EXISTING64KiB thread and O0/O3 loop; asserts actual distinctsource/target,
  CarrierTask generation,and every original first-owner pointer; retains second
  callback-owner pointer and escape negative. B14 reviewed exact091ede candidate,
  no blocker. Root frontend-only check caught test helper reusing moved first;
  fixed helper to THREE distinct Window inputs(empty(first),second,third). Same
  adopt specialization/probe intact. Equivalent source check nowPASS0.018s with
  currentCLI; Rust/private native validation still pending.
- Quiet old blocking-timeout fixture cleanup retained main.tsr/gate.rs/witness
  files,removed ONLY generated target after no-open-consumer/liveness/locks proof.
  /tmp/tsrust-blocking-fixture-target-cleanup.json recovered63438848B actual.
- Bounded-carrier first stage TERMINAL: preflightPASS68.714s; CLI/host build
  PASS223.337s/peak2887618KiB. Byte68nowpassesfinitegenerationbutstopsEMIT at
  StoredCallable149→StoredCallable0; privatebuildcorrectlyNOTstarted. No native
  pass. /tmp/tsrust-bounded-carrier-front-2b402dcab4c3.json;total235.843s,
  peak3019100936B,remaining[],cleanup400580608B.
- Actual parenttrace /tmp/tsrust-bounded-callable-boundary.err proves failure is
  NEWmutual module1Second224(read149,next205)→Second222(read0,next203). Earlier
  original module52/49 optionalrecursive conversions nowprogress. Source149is
  actualsecond(value) captured environment; no opaque boxing/empty-family shortcut
  justified. Candidate66e37187 remainsUNAPPLIED/unconfirmed.
- Proven registry cause: materialize_recursive normalizedSecond224toitsnominal
  beforeplanning,thenclonedSeconddeclaration(read0) whileonlypreservingFirstroot
  overrides. Appliedreviewed replacementd8383f7f: multi-member path retainsall
  suppliedphysicalnodeplans, exact-leaf/app coinductive candidate reuse, existing
  fullcomponentreservation/cache, ordinaryparameterregistration. Selfpathunchanged.
- Ghostretarget ONLYdirectnonoptionalCallbackIDconfirmedbyexistingAnalyzer
  absent_callback_payloads; two productioncallers passdisjointfieldpredicate(&dyn
  Fn,no newregistry). Optionalunions/unknowncallbacks doNOTalias. Peerreviewcaught
  optionalinhabited FirstGhost.nextSecond149 beingerasedbyRootSecond150; newunit
  assertsboth149+150,root148andinhabitedoptionalghost remainreachable. Otherunit
  covers148/149,backedge,repeatcache,concrete-inputfixedpoint/noarenagrowth,150distinct.
  B06peerreviewd838clear; no executionyet.
- Mutual-carrier first stage TERMINAL: preflight70.921sPASS; CLI/host build
 218.399sPASS/peak2895554KiB. Byte68advancestoEMITStored149→Callback72(verified
  emptyghosttarget),no nativepass/privatebuild. /tmp/tsrust-mutual-carrier-front-
  b70ec615d43b.json,total231.194s/peak2966901560B,remaining[],cleanup400171008B.
- Exactconsumerproof: start_record_work records.rs130 chooses recursive_parameter
  BEFOREliteralfields/defaults; resolved_prepared_call analysis/mod.rs8666 does
  samefornominalfactoryresults. Newmulti plannerregisteredALLmembers,sofirst148
  constructorregisteredrequiredemptySecond72 as soleSecondcandidate. LaterSecond
  selectsghost72. Wrapperdirectionisconsistentwithwrongselectedtarget; doNOT
  loosenNative149→absent72 conversion.
- Appliedbdc8ac2d filtersBOTHordinaryregistration sites usingexisting confirmed
  DIRECTabsentCbIDs. Layout/cache/linksretained; requireduninhabitedmembersnot
  recursiveparametercandidates. Rootexplicitlychecks!field.optional too; optional
  properties/unionsstayeligible. Regressionchecks first148ghostSecond72 retained,
  constructor/resultfallbackremainsnominalSecond1,thenliveSecond149isselected.
  Bothagentsconfirmedconsumerpath; no opacity/conversionworkaroundor66eguard.
- CURRENT /tmp/tsrust-live-recursive-run.py at7c171bb8, fixed30minuteacceptance: compiler/LSP
  testpreflight,remainingbyte68bothprofilesfirst,thenfailedrestoration/deepnative
  plusaffectedregistry/emptycontrols andfreshLSPbyteagreement. Preserve earlier
 1514lib/6focused/25byte/27LSP/2escape-negative/crypto24/collections42/public55/export
  passes. Source remainsnotaccepted until actualnative. No push/merge/publication.
- Quiet cleanup /tmp/tsrust-stale-worker-target-cleanup.json removed ONLY ignored
  generated targets from stdlib-bytes(native cache) and stdlib-path(rustc info)
  managed worktrees; source/WIP branches untouched. Recovered167149568B actual.
- Deterministic/customization-safe adapter generation actual PASS retained at
  /tmp/tsrust-utilities-http-repair19b-66af5f02808e.json. Only middleware positive
  and real-generated-shlex offline export hosts remain unproved in that pair.
- Additional cleanup /tmp/tsrust-obsolete-http-cache-cleanup.json pruned9cache
  entries containing replaced serve_connection(io,service) template under quiet
  build/cache locks; ACTUAL recovered2102931456B. No shared dependencies removed.
- Additional cleanup pruned6native cache entries with obsolete dependency identity,
  recovered155926528B; /tmp/tsrust-accepted-runtime-obsolete-cache-cleanup.json.
- NEXT: finish bytes mutual-layout assignment,then coherent compiler/LSP
  milestone and still-unproved B10 middleware/B19 shlex offline export boundaries
  LAST after compiler work. Middleware already has a genuine relocated example
  host; no new harness is needed. Public runtime TLS staging/migrations are done.
  Resume comparable timings only when quiet,retaining unchanged controls/proofs.
  Remaining full B00–B20 milestone/platform acceptance stays active. No push,
  merge or v0.2.0 publication approved by this local checkpoint.

Source migrations now APPLIED/formatted; acceptance pending for all three:
- B16: /tmp/tsrust-crypto-runtime-suite-migration.patch
  SHA538e29224c353b64560a90fdd060751f6ab21fa092e39a607b624cd6f18240ef;
  25old launches→24unique registered cases, debug/release batches,29catalogrefs/
  20verbatim examples.27paths; exact hashes/case map in matching -inputs/-cases.
- B14 collections: /tmp/tsrust-collections-stable-suite.patch
  SHAcac1163fcaf5b34739f6d9dd44922043df24cf32fb379eb8e74de5a0994ecc2b;
  37run sites→42programs→42registered cases; original bodies/comments,25checker
  sites/all8Rustc controls retained.44paths, matching -inputs case map.
- B02: /tmp/tsrust-bytes-registered-runtime.patch
  SHAc90267eb33bf5757710f5a806592177baca427199de300db489d4f74a8bd9eb3;
  67positive programs (24default-only,43bothprofiles)→one67case suite in both
  profiles. Original builders/negative checks retained; source bodies change only
  exported entry and final newline. Matching -inputs includes exact mapping.
All actual canonical apply-checks passed when prepared against0d090688; recheck
before applying. None touches current55cohort/core compiler. Format/source-check/
execution still required; never count prepared patches as accepted implementation.

B11 benchmark historical-root guard selfcheck PASS after0d checkpoint: both old
and current roots must match15sealed control/protocol files and actual Cargo
manifest/source paths. Private default unchanged. No controls retimed/relabelled.
/tmp/tsrust-b11-public-native-measure.py UNRUN, requires actual successful55case
host log/gate JSON and current CLI SHA before public release-server build,
protocol/native-only timings, cached-control reuse and npm run store:ingest.
Ensure portable source preparation prerequisite before emit package. Old actual
Rust/Node controls remain intact; no public native timings/ingestion claimed.

Prior cleanup5711986688B and491126784B remain separate from this continuation.
Full-goal next scopes remain B20 integrated applications using existing B10/B19
examples, affected acceptance/exports, actual matched measurements, B00 ledger/
default/minimal features, full regressions and four-host native packaging gates.
Future GPUI/Kit, .tsr client codegen/shared frontend/backend roots follow v0.2.0.
No push/merge/publication. Never redefine the goal around the current cohort.

## Earlier evidence (retained; current state above supersedes continuation advice)

## Current source checkpoint preparation — 2026-11-07

Canonical codex/stdlib-0.2.0 after a96b1d79. Previous turn made source and acceptance
progress, not completion. No build/test process is live. Full B00–B20 goal active;
no push/merge/publication. Source is formatted; native acceptance remains incomplete.

B17 integrated compiler/runtime/editor/reference/examples + three matched workloads.
Use stable std:testing suites, no standalone backend probes. All compiler/LSP test
harnesses TYPE-CHECKED with logging enabled: 67.028s, peak2181745904B, stopnull,
remaining[] (/tmp/tsrust-logging-unit-harness-typecheck.*); this includes cfg(test),
not just selected integration targets. Earlier selected cargo check --lib did NOT
cover unit bodies, so final preflight explicitly uses --tests. New pin-project-lite
PinnedDrop wrapper requires literal Pin type syntax, now corrected and checked.
Existing exact pin-project-lite0.2.17 direct edges, no new package/version/boxing.

Original public logging scope /tmp/tsrust-public-logging-scope-start.json:
started1794052719.0877151,3300s absolute. CLOSED INCOMPLETE; do not reset/restart
its oversized repair sequence. Last owned check finished with remaining[] before
55-minute deadline. No exports or timings claimed. Preserve actual passes below.
- First source2b6f build17.65s failed const parameter slice promotion; fixed.
- Source32d14 build229.715s failed runtime-test super import; fixed to self.
- Source4319 build204.330s failed Vec/reference assertion; fixed as_slice.
- Source1b4d build/gates380.247s, peak4184348840B, stopnull/remaining[]:
  seven passes/six failures. PASS file-append/scalars, interleaved contexts,
  Logger/LoggerOptions/LogError agreement,12 completion phases,module coverage,
  catalog reachability,negative sources. Cancellation bare-backend drop failed.
- Source223b build203.407s failed pinned-drop macro qualified type; fixed Pin.
- Source74d2d8ddc03c, attempt6 terminal231.012s, peak2871235288B,
  build-finishedtrue,stopnull/remaining[], four passes/three failures. PASS logging
  hover, unused-module/type-import/shadowing selection, both old callback-error
  guards. Unknown stdlog/iterator helper targets are repaired by preserving only
  trusted stateless wrapper symbols outside module-owner lift. Ordinary module
  functions retain program-owned capture storage (new structural witness unrun).
  FAIL two iterator cases: first typed catch48 remains unresolved, including
  effect-only nominal stored-field summary candidate (not accepted as a fix).
  FAIL logging suite first reached moved logger in a Once factory; fixture now
  creates/closes logger inside factory, reports actual error value through Channel.
  Direct same-suite check0.550s then reached unsettled-job at example48. Proven
  fixture gap: fallible readiness assertions occurred before cleanup try. Fixed
  source by guaranteed cancel+await in finally, then asserts requested/cancelled.
  Tracker consumes await operands BEFORE failure snapshots; no checker weakening.
  Corrected full suite has NOT been rerun. Runtime acceptance is still pending.

Actual docs traces used ordinary docs command, no native probe:
/tmp/tsrust-real-docs-carrier-trace.*3.158s pinned inhabitedRecord145/family9 being
replaced by absentRecord261 in a mixed union. Narrow direct-required-absent fold
repair applied, no populated-to-empty conversion permission. Fresh74d docs check
9.338s then advanced to Unknown example http-middleware (no carrier rejection).
Added actual docs/src/examples/http-middleware.tsr +config registration, formatted,
unrun. This is progress beyond former error, NOT full docs/native acceptance.

Unrun internal witnesses: cancellation/interleaved context after new drop owner,
trusted wrapper/module-state structural control, mixed-union fold controls,
nominal stored-field unknown/cycle controls and direct typed iterator diagnostic.
The diagnostic captured with OLD1b4d library binary0.556s remains
/tmp/tsrust-iterator-direct-effect-diagnostic.*. New trace adds closure captures
and actual call targets, but no new library test binary executed that trace yet.
Next milestone must include these guards, full corrected public suite, same-suite
relocated O0/O3 export LAST and matched timings with actual SQLite ingestion.
Do not replay unchanged accepted editor/negative scopes after fixture-only fixes.

Storage cleanup (locked/quiet, linked objects only) this turn:1120702464B +
469340160B reclaimed; previous profiling-turn906670080B separate, avoid doublecount.
Reports /tmp/tsrust-public-logging-before-repair-cleanup.json and
/tmp/tsrust-public-logging-closed-cleanup.json. target/debug14G/release3.7G;
preserve current consumers/libs and unchanged Rust/Node controls. Object pruning
is separate storage maintenance after terminal process cleanup (all remaining[]).

B11 coherent SOURCE-ONLY patch ready, NOT applied. Latest
/tmp/tsrust-b11-public-source.patch SHAb906740b56400aaddcd80bff21ea0f3b58c70640fa3cbc30611e0e01c9e45e83;
exact inputs /tmp/tsrust-b11-public-source-inputs.json. Central manifest proposal
/tmp/tsrust-b11-http-websocket-manifests-current.patch SHAc9518dc85506049fe7eb20c2af37cc44fa1c70f32224314274a062220eb0a628.
Includes compiler/runtime/editor/docs,18-case stable suite, strict JSON/exactnames,
prepared-native benchmark hook preserving old controls. Review caught E0716
parameter indexing, missing futures-util direct edges (all3manifests, exact
already-locked0.3.34 std+sink), bare bytes dependency (uses axum::body::Bytes now).
No locks/builds/formatter/measurements in worker. Root owns production locks and
must revalidate current baselines before integrating. Workers idle with preserved
isolated changes. Single writer canonical; one shared target/inc0/jobs1.

## Historical logging integration preparation — 2026-11-07

Canonical HEADa96b1d79, dirty source preserves owner progress/Owned predicate.
Integrated corrected complete B17 public patch3fc9507acc935fc653319477e754c0b68a0558285a15e43311449927e5d063e2
(excludes private admission), central19e6proposal then Cargo-managed offline locks.
Proposal lacked valuable cfg edge: offline metadata minimally adds valuable0.1.1;
all3locks now pass offline --locked resolution, old package versions preserved.
Logging compiler/runtime/editor/docs/examples +3matchedworkloads now in canonical.
Normal stable five-case std:testing suite first; relocated same-suite export is a
separate last gate. New unused-import/factory dependency-selection witness pending.
Root also integrated stable B15 registered iterator suite; changed wrapper to
stable canonical source/cwd (agent copied temp sources; root removed those copies),
retained ordinary app main +unnamed-catch emit witness +all existing assertions.
Existing carrier-rejection trace now opt-in CLI TSRUST_TRACE_CARRIERS=1, default
unchanged; use real docs command after fresh CLI, no standalone native reproducer.

Whole formatter6sourcefiles/24snippets/3fingerprints then stable0/0/0; Node2logging
source/oracle checks PASS. First selected build terminal17.65s/peak1440829088B,
stopnull/remaining[]: E0716 const EMIT borrowed ENABLED.parameters[1]. Root replaces
only that expression with same direct parameter declaration; no semantics change.
First evidence /tmp/tsrust-public-logging-budget.* and fingerprint2b6f675def43.
LIVE resume session54504; supervisor64615,driver64699,watch64712,CargoPG64713.
Do not duplicate/restart. Current source fingerprint32d14e03e52d..., build output
/tmp/tsrust-public-logging-32d14e03e52d-selected-build-artifacts.jsonl +build.log.
Watch /tmp/tsrust-public-logging-attempt2-budget.{out,err,json}; aggregate original
/tmp/tsrust-public-logging-scope-start.json start timestamp is in that JSON,
3300s absolute including firstfail/repair (never reset). Selected compiler logging
runtime3unitcases +normal suite/negative/unused-selection, stableiterator2cases,
LSP logging/catalog/12matrixphases/Logger,LoggerOptions,LogErroragreement.
One shared target/inc0/jobs1, --features tsrust-compiler/logging, threads3;
no offline exports in first pass. Full private units/old unaffected gates excluded.
B03 source-only suite patch integrated/corrected; B06 review finds no further proven
bug, bound method extraction contract source review pending. B14 realtime coherent
source still underway (runtime/compiler/catalog/suite/docs drafted; matched driver
uses explicit prepared-native mode preserving unchanged controls), no heavy lease.
No new public measurements ingested; all three new logging workload rows measured:false.

## Public test profiling accepted — 2026-11-07

New `tsrust test --profile` reuses existing build-stage timers, adds an optional
initial NDJSON build event, text profile on stderr, default JSON unchanged. Stable
examples/testing is one native runner: threefiles/sixcases. New host orchestration
checks duplicate/non-test flag rejection, default report, then profiled same
runner with cache=hit/nativeBuildMs0, six actual test events and summary. Full
focused scope TERMINAL27.691s, peak930432272B, stopnull/remaining[], build-finished
true; CLI/test-target build22.828s, host execution4.162s incl suite executions.
Actual profiled setup213.303166ms, test summary0.7395ms. These are correctness
observations, not protocol benchmarks/SQLite measurements. Artifacts:
/tmp/tsrust-test-profile-build.jsonl, /tmp/tsrust-test-profile-scope.{json,out,err}.
Text reporter sixpasses too; separately launched environments caused cache misses,
so an initial smoke assertion expecting cross-process cachehit was invalid. Repeat
plain scope2.126s, no stop/children, actual native/link1528.4ms; recorded rather
than hiding/restarting previous suite. Env hashes intentionally include build
script inputs; no cache-identity weakening. Both whole-project format passes
0changes, snippets0/fingerprints0; diff whitespace clean. Cleanup locked/quiet:
/tmp/tsrust-test-profile-objects-cleanup.json,147linked objects removed,
906670080freebytes reclaimed. Current compiler includes --profile; LSP unchanged.

B17 public source patch SHA619a6de1dd6e04899881189b3273a6b19489315533624ce8bad0c717cda946ea
passes git apply --check only, not yet integrated. Central proposal review caught
missing TOML commas; worker repaired and tomllib-validated all6proposed files:
/tmp/tsrust-logging-central.patch newSHA19e6e4b88a6af396aaf69bd58c8c675050d590aa3458d7c27b59e117afc96983.
Old f78b6proposal superseded. No production manifests/locks changed yet.
Fixture explicitOwned annotation remains pending acceptance; retain uncommitted.
B03 confirms docs span is receiver `props` (5chars), not `props.id`; actual source/
target record layouts require evidence before repair. No scalar widening guesses.

## Owner test-workflow correction — 2026-11-07

Use stable std:testing suites through tsrust test, one native runner for related
cases, same suite for export, unchanged Rust/Node controls reused. No further
private standalone backend probe builds. Owner explicitly permits testing-library
profiling/method extensions. Existing durationMs already reports test execution;
new --profile CLI separates discovery/setup/check/emit/backend/cache/native build.
Implementation + stable examples/testing six-case orchestration witness pending.
B17 frozen public source patch /tmp/tsrust-logging-public-source.patch includes
five registered cases and matched support, not yet integrated/validated.

Root late-factory gate is TERMINAL (session8471 closed), ac8bec50/3bbecb368457:
225.277s, peak2,934,641,248B, build-finishedtrue, stopnull/remaining[],2PASS3FAIL.
Both existing callback-error controls PASS. Named/unnamed iterator fixtures now
reach callback-escape on an unannotated named predicate. B03 independently proved
this fixture requires explicit Owned<Function<boolean,[value:Owned<TrackedResource>]>>
at creation; its resource-drop oracle expects a moved environment. Annotation
applied without weakening guards; no rerun yet. Docs still fails std:ui128,
actual differing owning callback record layout remains unresolved. Earlier
535.595s compiler build/5PASS4FAIL evidence preserved. Diagnostic scope original
1794048868.3530629/3300s closes incomplete; do not reset/replay its repair sequence.
A separate owner-requested public profiling feature scope will build the CLI and
run only the stable testing example suite and option checks; no old suite replay.

## Current state — 2026-11-07

Canonical: /Users/seancotter/Desktop/dev/learning/tsrust, codex/stdlib-0.2.0.
HEADac8bec50 (new specialization queue +uninhabited record/scalar compatibility repairs).
Last successful executed compiler consumer ac8bec50. Historical root session8471 (TERMINAL):
/tmp/tsrust-late-factory-repair-budget.*, same original diagnostic scope start
/tmp/tsrust-blocker-diagnostics-scope-start.json,3300s absolute including repairs.
This is affected integration-only refresh, no --lib/no replay of prior passing
B19/crypto/Task/LSP groups. Queue repair source-reviewed independently by B03/B14:
postponed catch retries can discover fresh specializations, which must drain
through the existing sequence before retry/finalization. Retry with no newwork
returns original unresolved error; no empty-family/template/effect suppression.
Named full native iterator fixture, unnamed-catch no-panic witness, existing
constructed/unknown callback-error controls and authored docs are selected.
New private named/unnamed unit variants await a later meaningful library gate.

Earlier root diagnostic ec11431e/a29c8df89d97 TERMINAL546.107s, peak4,116,551,776B,
build-finishedtrue, stopnull/remaining[],5PASS4FAIL. Fresh compiler-only build535.595s.
/tmp/tsrust-blocker-diagnostics-a29c8df89d97.json and budget.json. PASS fullnested
Rust adapter actual native, imported docs source-range unit, direct callback modes,
Job iterator ownership unit, literal scalar widening unit. Remaining UI traces
prove source records have registered empty TsrAbsentCallable field; new coalesce
uses Rust empty matches only for direct required ordinary-record absent fields
(!optional, no arrays/opaque/type-only guesses). New scalar compatibility mirrors
existing emitter rule all members must satisfy target.accepts_scalar_value.
Fresh diagnostic family22 trulyempty decl12 whilefamily23=[43] pure; source-only
unnamed-terminalcatch diagnostic input panicked all-functions-analyzed, proving
newfunction scheduling hole. Canonical acceptance fixture never weakened.


Current continuation: previous turn was concrete progress (native5passes, source
834f fix, actual5,357,232,128B cleanup), not a stalled/restarted build. Initial
process inspection found no live compiler/build/watchdog. Canonical dirty edits:
README v0.2 feature/example overview and docs_cli imported diagnostic renderer
(reuses project_cli renderer; new formatted source-range witness). Neither is
compiled; no canonical source checkpoint until whole-project formatting slot.

B17 heterogeneous retained-field extension also PASSED3.774s, peak270,368,768B,
stopnull/remaining[], exact old actual optimized rlibs/source/binary hashes.
/tmp/tsrust-b17-child-fields-result.json. Only6newgroups/93samples per generic/
erased shape after8; no replay36controls. Pointer/capacity and parent/child
Drop0whileheld→1afterfinalrelease, typedJSON and2request ancestryPASS. Erased
closure/nativegeneric allocations equal; signed+677B is warmed cache observation,
not an after-drop/leak claim. Backend choice opaqueLogger/Arc static-span closure
now has direct evidence; public integration remains UNVALIDATED.

B17 PRIVATE backend admission now PASSED in clean original logging tree3139162e.
Normal build/guards from attempt1 preserved; exact host-filtered metadata repaired
overapproximation without any source/runtime change or passed-normal rebuild.
Only unrun allocation/stress builds7.787s, profile5.212s, stopnull/remaining[].
Original55m deadline unchanged. All6processgroups/36timing invocations1116samples,
24allocationrows, original3x31after8; quietmax3.18359375. Disabled0alloc/retained;
format1036 andchild10252allocs equal directRust, retained217088/362496B are still
ownedoutputbuffers, not leaks. Correctness includes final sink/owner release.
/tmp/tsrust-b17-logging-attempt2-result.json and supervisor.json. Private results
are NOT public workloads or SQLite ingestion. Public B17 implementation underway
in same managedloggingtree newcodex/stdlib-logging-implementation at834f; only
old admission/logging restored from3139162e, original branch preserved. Source-only
opaqueLogger, static tracing scalar fields, lazy checked Once callback facade,
append-safe synchronous sinks, per-poll async instrumentation; no new generic
publicowner/dynamicmetadata/customformatter. Heterogeneous child probe pending.

B14 now owns existing managed realtime tree, CLEAN99c9admission preserved then
newcodex/stdlib-realtime-implementation at834f with only admission/realtime restored.
Existing full SSE/WS admission was found here; reuse it, no duplicated preparation.
B11 private admission TERMINAL35.455s profile, peak598,952,424B, stopnull/remaining[].
Originalstart1794048144.843 completed~8m40 including preparation/repairs, no reset.
All3correctnessgroups+2engines actualPending-send shutdownPASS.36timingrows×31=1116
samples,142848requestlatencies,36separateCPU/RSSrows, originalprotocols intact.
/tmp/tsrust-b11-profile-attempt8-result.json; historical terminal-success-with-
unresolved-sse-semantics / publicBackendAccepted:false retained honestly. Owner
Rust/backend contract selects exact Axum empty/CRLF behavior, not WHATWG parity.
B14 now publicsource-only implementation; no heavy lease. SSE/channel +explicit
sseWithProducer owns Job, converts post-header failure to existing HttpError and
drops typed E exactlyonce after settlement; detailed E handled inside producer.
WS uses reserved listener-Scope Job beforeupgrade and backendowned consuming split.
Compiler/editor/examples/matched/TLS/copy acceptance not yet executed.
Historical admission phases below describe preparation, not current live processes.
/tmp/tsrust-b11-attempt1.json and later attemptN-supervisor/out/err hold actual
handles. Cached exact Nodews8.18.3 installed+integrity verified; RustSec309392
no-fetch audit0vulns/warnings; hostmetadata/archive/source and optimizedactual
closure proofPASS. Actual-artifact features are checked subsets of metadata
optional/target upperbounds, exactrootfeatures. Original protocols unchanged.
SSE10framing/3lifetime controlsPASS; WS malformed/UTF8/limit/backpressure/cancel
controlsPASS. Delimiter/guard/invalid-close-fixture repairs preserved actual
passed scopes. Simultaneous-close fixture must move endpoints into join futures
so completed side drops its socket; no backendprotocol change. No timing yet.
Owner's explicit Rust/backend semantics select exact Axum empty/CR/LF behavior;
raw+WHATWG differences remain documented, no normalization/handmade encoder.
Private backend choice is NOT public generated facade acceptance. Root/B06 do
source-only work while B14 owns timing/build slot. No rootcompiler gate started.

All compiler/build/watchdog processes verified terminal before source edits and cleanup.
Repair19d is TERMINAL at6ebee54b/cffa06ffa9c3:236.293s total,
peak2,880,983,600B, stopnull/remaining[], build-finishedtrue;5PASS2FAIL.
Build224.246s; only compiler/affected integration consumers, no compiler --lib.
Results /tmp/tsrust-utilities-http-repair19d-cffa06ffa9c3.json; outer
/tmp/tsrust-utilities-http-repair19d-budget.json. Fresh normal CLI/LSP6ebee54b.
PASS: actual mapped startedJob identity→toArray→all nativeoutput3,3; nullable
iterator state and real ByteView escape/mutation guards; O0/O3 nullish fixture;
borrowed nestedResult payload discriminator read +mutation/move negatives;
Rust signatures/source ownership reject controls. Earlier crypto and nominal
pointer/capacity native passes retained.
Source-only after closure: shared commonfield receiver now reuses existing
conditional_arm(Read) owner staging instead of expr(Read), retaining temporary
function return through assertion borrowing. Wholeprojectformatter0/0/0; no new
redundant source-shape test, existing native E0716 witness remains required.
This one-line fix is NOT rebuilt/executed; CLI/LSP still6ebee54b.

Remaining FAIL: fullnested adapter now reaches Rust E0716 from borrowing
unit(none).kind/unit(some).kind temporary (B06 shared receiver staging repair);
authored documentation advances through prior coalesce panic to std:ui owned
callback record-family rejection (B03 exact source diagnosis). Typederror source
family22 remainsopaque from19c; do not rerun until provenance/repair changes.
New privateunits/familytraces are not executed; full family milestone remains open.
Middleware actual offline/native and generated shlex offline exports remain
required and UNRUN; no passes invented. Original19 scope CLOSED with cleanup at
~3240/3300seconds, no resource stop or surviving children, incomplete acceptance.
Do not reset or replay this same oversized selection as serial hour-long chunks.
Future checks require coherent new source changes and affected scope only.

Final quiet cleanup removed410linked rcgu.o leftovers:1,126,985,728 actual bytes
recovered (/tmp/tsrust-repair19d-objects-cleanup.json). This continuation reclaimed
5,357,232,128 actual bytes across three cleanup operations. Target is now18G
(debug13G/release3.5G), down from23G observed at this continuation's start.
All source/control/current consumers preserved; no cargo clean/refill.

Repair19c is TERMINAL: selected build549.707s, peak4,012,964KiB,
build-finishedtrue. Initial preflight used wrong task_diagnostics::tests prefix;
correct cursor_tests prefix discovered from actual --list. No rebuild: exact same
22e002fc/d2cd59ed7ca1 artifacts resumed direct execution40.789s,
peak795,459,584B, stopnull/remaining[],6PASS10FAIL. Results:
/tmp/tsrust-utilities-http-repair19c-d2cd59ed7ca1.json; original build outer
/tmp/tsrust-utilities-http-repair19c-budget.json, execution outer
/tmp/tsrust-utilities-http-repair19c-execution-budget.json. Original19start unchanged.
Fresh passes: all3remaining crypto native/catalog cases; middleware negatives;
nullable capture/nativecounter +realborrowescape/live-mutation guards; generated
nominal record O0/O3 pointer/capacity preservation. UI checks now progress to emit
record remapping (Alert Record107→54, Keyboard219→119); two new UI witnesses
need valid nominal/lifetime construction. Iterator Job diagnostic proves missing
direct-call origin mode; reviewed narrow source delta applied, not yet rebuilt.
Resource forEach typedcatch remainsopaque. NestedResult Option payload field
lookup rejects legitimate borrowedunion; B06 tracing checker+emitter repair.
All test processes terminal; no heavy lease live. No exports claimed passed.

Latest completed repair19b: source8c54386f, fingerprint66af5f02808e,
629.219s, peak4,140,701,720B, stopnull/remaining[], build-finishedtrue,
69PASS11FAIL. Exact results /tmp/tsrust-utilities-http-repair19b-66af5f02808e.json;
outer /tmp/tsrust-utilities-http-repair19b-budget.json. CLI/LSP and selected
binaries are fresh for8c54386f. The prior Argon2 verifier correction passed actual
module-password execution; optional CORS units, nullable iterator counter native,
named callback inference, literal-hint guards, Rust editor agreement and all
Task-array/pool transports passed. Preserve those completed scopes.

Reviewed source repairs now applied, execution pending: literal-to-scalar carrier
and nullish fallback mapping; Rust record-array collection through its concrete
alias; explicit generated Option seed tags including SomeUnit; borrowed password
options extraction (previous output silently selected all backend defaults);
project execution of imported crypto field examples; valid scoped ByteView callback
positive plus true escape/live-owner-mutation negatives. Password example now
asserts actual encoded custom cost and output length. Original repair19 start is
/tmp/tsrust-repair19-scope-start.json,3300s absolute including repairs and cleanup;
never reset it. Remaining unresolved: started Job iterator-map responsibility and
resource forEach typed-error metadata after a prior catch. B14 source-only diagnosis.
Required middleware native/offline and real generated shlex/offline exports remain
queued LAST after compiler checks; neither has been accepted by repair19b.

At verified quiet point, lock-protected cleanup removed3040 leftover rcgu.o files:
2,931,077,120 actual free bytes recovered. Report /tmp/tsrust-repair19-objects-cleanup.json.
Additional quiet-point cleanup validated543 exact obsolete runtime files from181
closed temporary workspaces, currentnamed/value/pathconsumer counts0 with1546
positive controls, preserving41live-source and26unknown/non-temporary roots.
Deleted543files; actual freebytes43953741824−42654572544=1,299,169,280.
/tmp/tsrust-runtime-refresh-applied.json. This continuation actualrecovery4,230,246,400B.
Prior cleanup recovered6,951,444,480bytes; do not count those again or claim this is
net growth reduction. Preserve linked consumers, reusable libraries and controls.
No push, merge, release or public timing/SQLite ingestion in this continuation.
Live remote read: no open PRs (including Dependabot); CI49 cancelled, CI50
startup_failure, CI51 success at794c839735cf is supporting-host UDP timing ONLY.
Its compiler/LSP/native-platform families were skipped, not full acceptance.
GitHub connector search unavailable; gh read-only fallback used. No workflow dispatched.
One writer per tree, one shared Cargo target, jobs1/incremental0; tests threads3.
Every scope <=1h, original55m aggregate and <=30m/5GB inner guards.

## Latest exact acceptance

All repair18/18b/18c/18d/18e belong to ONE original55m absolute budget, including
repair gaps; no reset. Final18e finished beforedeadline, not resource-stopped.
Private library witnesses changed after18c were NOT rebuilt/executed in18e;
they remain explicitly pending. Do not use stale library binaries to prove fixes.

- repair18 build-only301.95s: test Arc movedcounter error; fixed.
- repair18b c7cdaf14/f304ca82101c:881.223s, peak4,138,145,816B,
  stopnull/remaining[], build-finishedtrue;48PASS19FAIL.
- repair18c aaf1ab3a/d995e2bfd2d5:690.857s, peak4,140,570,576B,
  stopnull/remaining[], build-finishedtrue;42PASS13FAIL.
- repair18d build-only11.513s: Capture reference.clone producedVec<&Capture>;
  fixed using explicit copied7metadatafields, no new clone capability.
- repair18e895205bb/b2cedc3354d1:267.889s, peak2,919,469,664B,
  stopnull/remaining[], build-finishedtrue;19PASS7FAIL. Results:
  /tmp/tsrust-utilities-http-repair18e-b2cedc3354d1.json;
  /tmp/tsrust-utilities-http-repair18e-budget.json. No surviving child processes.

Preserved native passes: forms/multipart relocated vendored offline O0/O3 with
forbid(unsafe_code), uploads O0/O3, fixed digest/Hasher/HMAC/HKDF all readable
positions O0/O3, legacy owned mixedunion/Mutable default actualnative, four callable
record positives plus allnegativeguards,9Taskarray tests and5pool/error transports.
B16 lifecycle/typedargument/opaqueHasher controls pass; password export manifest
now selects crypto-password and full async pool support. Publicpassword execution
stillfails the above Argon2 &String call; d33fix pendingverification.

Preserved editor: crypto hover/signatures/7optionfields/2Hashermethods in4phases,
4crypto/CORS receiver checkeragreement, CORS newrow4phases; earlierHTTP2rows8phases
andcrypto4rows16phases actuallyobserved (oldmatrixdriver countbug corrected without
replay). Full affected editorhelp and catalogcompilerwitness PASS. Runtime/private
middleware construction/contextcompression/bodydrop/error/sourceorder controls pass;
new marker/prelude extraction assertion andoptionalCors privateunits awaitfreshlib.
Actual defaultTLS HTTP2 ALPN/privateCA/loopback transport passedrepair16.

## Superseded repair18 diagnosis (latest results above take precedence)

- Password runtime extraction, explicit crypto-password feature in nativeCLI AND
  standaloneexport, and asyncPool selection are now implemented; actualexecution
  progressed to admitted verifiergeneric API mismatch, fixedsourceonlyd33.
- Middleware positive example now stages await c.req.text() before c.text(body),
  and CLIchecks beforeportable setup. Actual typedCorsFactory=cors fails during
  asyncdiscovery: provisional function_value hardcodes Required/restfalse. Source8b1d4e9f repairs
  fallbackmetadata using sourceinitializer omission/rest and async typed-first
  regression; freshnative/privateproof pending. Do not add explicitundefined workaround.
- Iteratorretry now reaches strictEqual(error.code,'selected') but coarseString
  hint conflicts with alreadyboundliteralT. Source8b1d4e9f adds exactboundliteralhintdeferral
  preservingT and finalwrongliteral/typedString rejection; no blanketwidening.
  Freshnative/privateproof pending.
- Stored nullable current.next loses its selectednonnull fact after namedassignment
  and branchjoin. Rootlocalized: namedstore invalidatesselectedRHSfact, thenbranchjoin loses
  nonnullproof. Anyfix must normalizephysicalrecord to existingcheckedpayload
  withnominal/item/owned/no-view guards, notallow nullable.next. No guessedflowfix.
- Retainedkeyboard UI check+emit path repaired; actual documentation_content now
  reaches docs/src/components/ui/alert.tsrx ownedcallbackcarrier failure. B03 owns
  focusedprivatewitness/actualshape diagnosis, not UIfeature expansion.
- Freshprivate Taskmethod responsibility/contextualRead, namedcallbackgenericresult,
  PWruntimeextraction and UI/marker units remain required. Newtracker analyzes
  actual non-template polymorphicmethods instead of emptydeclaration bodies;
  preserves abandonment negatives. Namedcallbackinference preserves shadowing,
  explicitnon-generic guard and exactparameterchecking.

A separate <=60s CHECK-ONLY9probe diagnostic completed~0.4s after18/cleanup:
/tmp/tsrust-18e-literal-optional-check-probes.json. CLIhashb8c1278ba18a43991d4cd0046da6283ed7804bae31314f9139b85d8758a067a4.
Not an acceptance reset or native proof. Workers may do source/lightdiagnosis;
NONE has a heavy build/profile lease. Root alone edits canonical.

## Other dependency-ready work

B11 source audit confirms SSE/WebSocket public APIs absent; no acceptance claimed.
Prepared SSE backend admission only (UNRUN) in existing HTTP worker:
/Users/seancotter/.codex/worktrees/stdlib-http-admission/tsrust/benchmarks/runtime/admission/http-sse.
Review patch /tmp/tsrust-http-sse-admission-source.patch SHA256
5b29fc84d0247737a09116f76aadf1a34d0a24f372f434b4fbaadf4f36f8791b.
Exact file inventory /tmp/tsrust-http-sse-admission-source-inputs.json.
Pins existing Axum0.8.9/Tokio1.53.3/http-body-util0.1.5 stack; backend framing,
independent multiline/Unicode oracle, bounded channel/drop/KAT and18profilegroups.
README queues exact lock/metadata/build/profile commands, shared target/no downloads.
No heavy lease yet; profile first, then public Channel/Scope/compiler/editor/example/
matchedbenchmark implementation. WebSocket needs dependency admission plus real
upgrade-capable connection serving/scoped upgrade-task cleanup; not just a route.


B19 integrated sourcecheckpoint9ae0c2b9 after37fileexactoverlaycc8070 (private
admissioninputsunchanged). RecursiveOption/Vec/String/record/outputResult adapters,
move-onlynominalprojections, existingbindingsCLI selectedgeneration, hashguarded
regen/customization-safeeditableuncompiledstubs, realformat57reader/sourcefallback,
exactstd/coreDropalias/qualifiedselfguards, editor/docs/examples/4matchedworkloads.
No newproductiondependencies/manifests/locks. Rootreserves5admittedcrypto names.
NominalfixturelinksactualTsrust-emittedapp toseparatebackendrlib O0/O3; exactRayon
librarycomesfromselectedCargo fingerprint, notnewestmtime. Native/editor/export/
measurementUNRUN. TwoindependentNodeoracle/sourceguardsPASS0.238s;wholeproject
format5sources/4fingerprints then0/0/0. Alloldbenchmarkrows/protocolsunchanged.

Current19primaryscope12targets includesfreshprivate library witnesses, fullRustbridge,
2nonexportgeneratorcases, affectedRustLSP/nativecrypto/HTTP/Task/iterator/docs.
TWO requiredexports explicitlyqueuedLAST underSAMe19deadline: middlewarepositive
andrealgeneratedshlex relocatedoffline. Neither isclaimedcoveredbyprimaryscope.
Do not buildportableexports beforeplannedcompilerrepairs; fixesmayresumebudget.

Guardednullableassignmentflow89c integrated9ae; checksameownedknown/no-view
scopednominal+exactapplicationargs, onlyprovenrecord-onlyfacts normalize tocurrent
physicalpayload. Undefined/unknown writesdo notpublishfact. Actualcounter+5negative
unit/nativeexistingfixture pendingfresh19. ActualAlert privatecheck+emitwitnessand
failure-onlytraces887e integrated; no Alertproductionacceptanceguardchanged.

B19 privateprofile PASS47.454s:114groups/3534timings/114allocationrows, retained0;
/tmp/tsrust-b19-profile.json. Actualpinned1.96.1 rustdocshlexfixture PASS2.120s,
166166528B, stopnull/remaining[], format57; /tmp/tsrust-b19-rustdoc-lease.json.
B17 cleanloggingtree3139162e audit309392PASS; privateprofile stillUNRUN.
Prepared /tmp/tsrust-b17-logging-supervisor.py (original36groups1116samples/
24allocationrows31after8), requires explicitexclusivelease. Publiclogging notstarted.
B11 SSE/WebSocket andremainingB20 nativeWindows/Linux/macOS/package/install/full
regressions remain mandatory. No partialbundle substitutes forfullB00–B20 goal.

## Storage / validation order

Quiet lockedcleanup this continuation reclaimed2,832,830,464actualbytes:
3obsoletevendorpacks2,270,629,888B;15obsoletegeneratedworkspaces562,200,576B.
Reports /tmp/tsrust-repair18-obsolete-source-packs-cleanup.json and
/tmp/tsrust-repair18-obsolete-workspaces-cleanup.json. Currentvendorpack preserved;
unknownworkspaceidentity preserved. Cacheprograms23current/unknown retained.
Including preceding1,672,269,824Bobjects +2,446,344,192Bobsoletecache cleanup,
recordedactualreclaimed total6,951,444,480B. This is removedoutput, notnetdiskgrowth.
Keepcurrentdebugcompiler/LSP/libraries/controlbinaries/source/chats/toolchains.
No cargo clean/refill and no deleting currentdebugsymbols blindly.

Repeatedbuildroot proven:18c145dirtyunits/95rebuiltartifacts vs18b9nonfresh.
InterveningportableCargo builds made shareddependencies/buildscripts newer;
historical.d alsoholddeletedvendorpaths. Executecompiler/editor/directchecks first;
portablevendorexports LAST, withno plannedcompilerrefresh afterward. Sameprofiles/
target retained; no newcacheframework. Preserve successfulscopeevidence.

ExistingSQLite acceptedmeasurements remain: B03sevenworkloads93samples;
B14/B18nineworkloads27enginegroups93samples; I/O48histories13206samples;
compression36histories10044samples. B18focused24native accepted, deepassertnative
remeasurepending; B15fourreadyunmeasured. NewHTTPuploads10/middleware10/B16crypto29
matchedworkloads ready, independentoraclesPASS, actualpublictimingsUNRUN. Private
profiles are NOT publicmeasurements. Measure acceptedfamilies, reuseunchangedRust/
Nodecontrols, ingest actualtimings with npm run store:ingest. Fullregressions only
at meaningfulfeaturefamilymilestones; preservepasses/resumeunrun afterrealrepairs.

Next: finish provenoptional/literal/nullable/UI source repairs, reviewB19draft,
formatwholeproject, then ONE coherentlyscoped normalbuild+affectedproof withfresh
budget andexportsatend. Do not restart the expired18scope or call itpassed.
Check currentPRs/Dependabot beforeanypush. OldCI49cancelled; neverdispatchobsolete
330mworkflow. Publication stillunapproved. Postv0.2 GPUI/Kit signals/.tsrx, .tsrSDK
andsharedbackend/frontend roots follow docs/plans/gpui-application-platform.md.
