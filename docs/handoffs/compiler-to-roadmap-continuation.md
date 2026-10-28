GitHub preparation — 2026-10-26: the owner requested public README/docs, MIT licensing and automated CI/releases. See [the release guide](../guides/releases.md) and [local acceptance](../milestones/github-distribution.md). This work does not resume the paused UI implementation or resolve LSP memory pressure. No editor/preview restart or local Rust rebuild was performed; hosted CI/release execution awaits a GitHub remote.

Paused at the user’s explicit request — 2026-10-20: processes are stopped and temporary builds removed because of severe memory/swap pressure. Read [the pause and memory handoff](compiler-pause-memory-pressure.md) first. Source checkpoint `1085b9c8` is saved; Table SDK/LSP repairs remain unverified. Investigate memory before restarting the editor, preview or broad checks. All earlier continuation instructions below are superseded by this pause.

Active implementation — 2026-10-20: continue [the paired Table section](../plans/ui-table-section.md). The eight-export source, native shared columns, checked spans/scope/state, browser transport, offline bundle, editor help and three docs examples are implemented. Initial native/model checks and the constrained-width/hidden-row repair pass. Generated-native, LSP and actual-browser acceptance remain in progress; Table is not yet accepted. Nested portable tables reject explicitly until recursive constraints are implemented. Reuse latency/Kbd evidence and keep the real watcher on port 4317. No further go-ahead is needed.

Current paired component section — 2026-10-20: [Kbd 2 and Markdown frames](../milestones/ui-keycaps-and-markdown-frames.md) pass focused native, installed-LSP and actual dev-browser acceptance. The source checkpoint is `dae8cb59`; the following acceptance checkpoint includes the measured recursion repair. Recursive Markdown dispatch falls from 173,216 to 17,040 Wasm stack bytes without a stack-limit increase. The Kbd route renders its table and three working examples. Existing source/imports remain preserved on upgrade; the new docs example uses `kbd-v2.tsrx`. Browser checks cover retained props/state, themes, icon sizing, contrast, responsive layouts and exact source copy.

Continue the actual pinned component APIs and paired docs. Scoped compound context, event/ref composition, focus/overlays, remaining components/charts/themes/blocks, HTML-first loading and executable splitting remain open. Kbd coverage stays partial for its explicit portable limits. Reuse accepted evidence and batch affected checks. Port 4317 is the real dev watcher; current CLI/LSP consumers and the active editor include the performance repair. No further go-ahead is needed.

Current performance repair — 2026-10-20: [dev and workspace latency](../milestones/compiler-dev-and-workspace-latency.md) records focused acceptance. Real warm docs rebuild is 14,870 ms; emitted Rust remains byte-identical across the runtime-support scan repair. Actual repository LSP scanning is 133,894.5 ms cold and 1,068.0 ms warm, with 238.8 ms foreground hover and zero stale/member counters. These are diagnostic observations, not matched benchmarks. Five consumers and the active editor are refreshed; settings bytes/time are exact. Port 4317 remains the real dev watcher. The in-app tab was reloaded from an obsolete foundation screen onto the shared landing.

Continue the pinned Base UI + Nova component contracts and paired docs. Kbd 2 source and upgrade/native witnesses are in progress; do not describe that section as accepted yet. Full compound context/event/ref composition, focus/overlays, remaining components/charts/themes/blocks, HTML-first loading and executable splitting remain open. Reuse accepted evidence; batch affected checks and continue without another go-ahead.

# Compiler continuation: UI resource lifetimes, data types, then S19–S21

Current section — 2026-10-20: documentation navigation passes
[focused paired acceptance](../milestones/ui-docs-responsive-navigation.md).
Exactly one page is painted selected, section highlighting is separate, and small
screens use Browse docs with a full-width article. Shared responsive hidden display
preserves ownership while removing layout/input/accessibility. The actual browser
passes five viewport widths, visible clicks, both palettes and console/mount checks.
Dev polling/assets distinguish server restarts. CLI and active editor include the
`bf9e9c92` contract. Landing section functions repair a measured Wasm stack trap;
no stack limit was raised. Port 4317 remains the actual dev watcher.

Empty's six exports and paired typography pass
[acceptance](../milestones/ui-empty-and-typography.md) through `d72c8f6f`.
Coverage remains partial: Text slots expose TextProps, not full DOM replacement/
event/ref APIs. Continue actual pinned shadcn contracts, scoped context, event/ref
composition and focus scopes before dependent compound controls. Keep remaining
components/charts/themes/blocks paired. Workspace-wide LSP scan cost, initial web
loading, general compiler temporary-frame growth and dev latency remain open.
Reuse accepted evidence and batch targeted checks; no new go-ahead is needed.

Previous accepted section — 2026-10-20: disclosure semantics and runtime-local instance IDs
pass focused paired acceptance. See [acceptance](../milestones/ui-disclosure-semantics.md).
Live Button expanded/controls metadata forwards through local components. Native
accessibility actions use current state, resolve visible targets and guard repeated,
disabled and hidden requests. useId has checked exhaustion and native-thread
uniqueness. The two retained panels pass actual dev-browser keyboard, draft,
identity, accessibility, theme and responsive checks. Existing native window/input
startup evidence is reused; changed native scene/action behavior is checked directly.

CLI and installed/active LSP include `d0712025` (API 9); test/signature witnesses
are `48988790`. Workspace settings bytes/time are exact. Content reports 166 pages,
27 checked examples and 972 declarations. Port 4317 serves the actual dev watcher;
current gallery is `/docs/ui/foundations/disclosure-semantics`. SQLite verifies
29 closed records and cleanup removes 10489378 bytes. Preview build latency
(117553 ms) and cold editor scheduling remain unresolved.

Continue [Empty's six-export section](../plans/ui-empty-section.md), including its
required paired styles. Scoped context, event/ref merging, focus scopes,
transitions and complete compound controls remain required. Expanded metadata
alone is not a completed Collapsible. Continue remaining components/charts/themes/
blocks; reuse accepted evidence and batch checks. No further go-ahead is needed.

Accepted reactive accessibility — 2026-10-20: reactive accessibility passes focused compiler,
native model/accessibility, generated-native, installed-LSP and actual dev-browser
acceptance. See [acceptance](../milestones/ui-reactive-semantics.md). Names,
descriptions, hidden and invalid states refresh without replacing controls. SVG
image/decorative defaults follow live labels. Retained DOM now updates/removes
semantic attributes. A shared captured-union reborrow defect is repaired without
changing ownership rules. Static size metadata remains compatible with Card.

Button 3 forwards metadata/replacement props. The docs installer preserves the
previous Button exactly and adds `button-v3.tsrx`; existing imports stay unchanged.
There are 165 pages, 26 checked examples and 971 declarations. The installed CLI
and active LSP include `72d91de7`; settings bytes/time are exact. The actual dev
preview is `/docs/ui/foundations/reactive-semantics` on port 4317. Browser-only
repair reuses accepted native evidence. No full regression, SQL or timing pass is
implied. Build latency and cold editor scheduling remain unresolved.

Continue scoped context, event/ref composition, focus/disclosure/overlays and the
remaining component/chart/theme/block inventory. Expanded/controls relationships
are still required. Empty's exact pinned Nova source/styles have been read;
tracking, relaxed leading, balanced text, dashed borders and descendant icon sizing
need explicit paired support. Do not claim unsupported utilities are implemented.
Reuse accepted evidence and batch focused checks; no owner go-ahead is needed.

Accepted lazy children — 2026-10-20: lazy children pass focused generated-native,
installed-LSP and actual dev-browser acceptance. See
[lazy-child acceptance](../milestones/ui-lazy-children.md). Ordinary nested JSX
becomes an owned array factory only for a declared child-factory contract. Eager
arrays remain eager; resource order, single-use transfer, partial-failure cleanup,
fresh reopened content and retained parent state pass. Content reports 164 pages,
25 checked examples and 971 API declarations. The installed CLI/LSP include
`2ec53b01`; corrected witnesses are `482e86d5`. Settings bytes/time are exact.
The actual dev gallery is `/docs/ui/foundations/lazy-children` on port 4317.

Continue scoped context, event/ref composition and focus/disclosure/overlays;
independent components can proceed alongside those foundations. Then finish the
remaining component/chart/theme/block inventory. The deferred-panel example is
not a completed disclosure. Build latency, cold editor scheduling, HTML-first
loading and executable splitting remain open. Reuse accepted evidence and batch
focused checks; no additional owner go-ahead is needed.

Shared Svg/Path rendering is accepted through `7b82bcfa`; see
[vector acceptance](../milestones/ui-vector-rendering.md). It supplies checked
retained paths and paired native Vello/browser SVG, not full chart/SVG parity.
Vector cleanup removed 2778752550 bytes of disposable outputs.

Accepted Badge section — 2026-10-20: editable Badge and shared text flow pass focused
native/compiler, installed-LSP and actual dev-browser acceptance. See
[Badge acceptance](../milestones/ui-badge-and-text-flow.md). Six palettes,
live/nullable props, once-only replacement controls, Unicode selection, text
wrapping/decoration, Link predicates and offline installation are covered.
The actual native docs route builds and passes bounded startup. There are 162
destinations, 21 checked examples and 966 API declarations. The CLI/LSP use
`61a0d5bf`; settings bytes/time are exact. The live port-4317 gallery is
`/docs/ui/components/badge`.

Continue context, event/ref composition, focus/disclosure/overlays and remaining
components/charts/themes/blocks. SVG icons and transitions remain missing from
Badge; this is not full Nova parity. Build latency and cold editor scheduling
remain open. Reuse accepted evidence and batch focused checks after complete
sections. No additional owner go-ahead is needed.

Accepted reactive props — 2026-10-20: scalar reactive custom props, derived getters and
View/Button replacement composition pass focused compiler/runtime, installed-LSP,
actual browser and native startup acceptance. See
[acceptance](../milestones/ui-reactive-props-and-composition.md). Editable Alert
version 2 uses live variants/classes and replacement rendering; the offline
installer preserves earlier source/imports. The actual port-4317 dev guide at
`/docs/ui/foundations/reactive-props` demonstrates both custom props and Alert.
There are 162 destinations, 20 checked examples and 961 API declarations.

The CLI is current at `274a014e`; the public/editor contract is current in the
installed and active `a485dca3` LSP. Settings bytes and modification time are exact.
Browser metadata synchronization and string-enum ordering repairs pass affected
checks. Full context, event/ref merging, focus/disclosure/overlays and the remaining
64-component, three-recipe and 30-block inventory are still required. Continue
independent components and those foundations before dependent charts/themes/blocks.
Build latency and cold editor scheduling remain unresolved. Reuse accepted reading,
resource, SQL and runtime evidence; no additional go-ahead is needed.

Current owner direction (2026-10-19): follow the
[documentation application and complete component-library plan](../plans/documentation-and-component-library.md).
Create a separate runnable docs project under `docs/`, with organized Markdown,
real examples, code rendering, all 64 pinned components, three recipes, 30 blocks,
theme generation and custom-component authoring. Web/native have equal acceptance
priority; resolve native headers early. Search starts with offline full-text;
semantic search follows later. Publish curated SQLite exports without moving the
benchmark harness or exposing the full canonical database. Benchmark documentation
includes overview graphs and Node-style per-method reference sections, with
arguments/returns/examples and inline comparable results (strings/trim is the first
pilot). Both views use the same curated evidence and explicit unmeasured states.
Existing counter owner files and developer evidence paths remain preserved.

## Current reading and reference section — 2026-10-20

The owner's reported scroll, selected-navigation, theme-link, light syntax and
preview defects are repaired through `8803a208`. See
[reading/reference acceptance](../milestones/documentation-reading-and-reference.md)
and [the full section audit](../plans/documentation-experience-audit.md).

There are 160 destinations and source-derived references for 101 public receivers;
941 API declarations do not mean 941 new or reverified runtime implementations.
Generated payloads are readable UTF-8 data, with shared decoders and bounded page
construction. The native bundled path repair is `3239330f`. Focused compiler,
installed-LSP, native models, actual browser and real native schema-route startup
pass. Consumers and active editor are current; already open landing/catalog
versions publish empty diagnostics. Reuse accepted SQL/resource/runtime evidence.

The real dev preview stays at port 4317. An expanded-content diagnostic before
final copy/layout polish records 262.7 ms readiness and a 149.7 ms maximum event-loop
gap; no controlled timing claim is added. Cold
whole-repository editor scans remain slow. Keep HTML-first loading, executable
splitting and editor background scheduling explicit. The small-screen reading
layout is usable; compact mobile navigation disclosure remains required.

Resume generic reactive custom props/base composition and the focus/disclosure
families before claiming the remaining Base UI/Nova components, charts, themes and
blocks complete. The audit lists the required material and intended presentation
for every section. The old HTML design reference has not yet been aligned with the
new shared landing. No additional owner approval is needed.

## Current startup responsiveness repair — 2026-10-19

The owner's startup priority is accepted through `583c097d`: the original fresh-page
event-loop gap of 2,196 ms falls to 108.3 ms, with readiness at 172.1 ms in the normal
cold-page diagnostic. Retained lazy Route factories construct articles only on first
visit and preserve visited state/DOM/scroll. Default theme validation is cached once
with independent returned values. The escaped 1.63 MB catalog program becomes a
47.7 KB dispatcher plus checked page-sized factories; search loads on first query.
See [startup acceptance](../milestones/ui-documentation-startup.md).

The actual native documentation route, seven native lifetime/theme cases, formatted
compiler example/contracts, content tamper/reuse/cleanup, installed LSP and actual
browser wheel/navigation/state/search/theme/deep-entry/disposal checks pass. Live
gzip transport preserves exact decoded bytes, MIME, HEAD and revision behavior;
5.42 MB raw Wasm transfers as 1.86 MB. At 4× CPU slowdown and 1.6 Mbps/150 ms, the
event-loop gap is 417 ms; the cold network transfer still takes 9.5 seconds. These
are diagnostics, not controlled benchmarks. HTML-first hydration and real executable
splitting remain required work; Route defers construction, not binary download.

The final real dev preview stays at port 4317 with current CLI/LSP consumers and
restored editor settings. Resume the shared landing and required component library,
including generic reactive props, base composition, overlays/focus and richer paired
motion. Keep loading/first-paint work explicit rather than claiming production web
readiness from this responsiveness repair. Reuse accepted UI/resource/SQL evidence.

## Current continuation after bridge discussion — 2026-10-19

Resume the shared landing and component library with paired web/native behavior,
live dev preview and current LSP. The optional browser JS/TS bridge is explicitly
[deferred](../plans/optional-browser-javascript-bridge.md). It requires ordinary
imports, no server/native JS runtime, explicit target implementations and immediate
editor/compiler diagnostics for every configured target down to individual exports.
Do not implement it as part of the landing or silently drop unsupported behavior.

The landing uses the intended GitHub repository
`https://github.com/Forever-Frameworks-LLC/tsrust` and GitHub Releases. Omit Discord
and the unverified curl installer. Use actual language/CLI APIs and accepted evidence;
do not publish invented size/performance claims from the earlier design brief.
Shared visibility callbacks and HTTPS links pass focused acceptance through
`0f56fbfc`; see [landing acceptance](../milestones/ui-shared-landing-and-visibility.md).
The current compiler/editor follow-up is `b13c831a`, described below; the
preview remains at port 4317. Continue richer component pages
and the required Base UI/Nova families; the HTML design reference does not establish
sticky scene/animation parity in the shared renderer.

## Current component galleries and compiler repair — 2026-10-19

Continue after `b13c831a`. Six component routes now have seven real previews and
exact copyable sources, with previews before the reference text. Card/Input prose
is cleaned up; Textarea, Label, Kbd and Separator have authored behavioral pages.
Dark error/button contrast exceeds AA. Native component assertions, the actual
native Card docs route, focused LSP help and live browser interactions pass.
See [gallery acceptance](../milestones/ui-component-documentation-gallery.md).
The full pinned library remains required; these are not full Nova parity claims.

The direct-declaration source-call repair preserves emitted gallery and full-app
Rust byte-for-byte. Indirect calls retain contextual analysis and ordinary
ownership checking remains. Installed consumers and active editor are refreshed.
The current dev server uses this compiler and reports a 15,893 ms initial build
with warm dependencies. The earlier unchanged-source build reported 253,046 ms;
these are observations, not a controlled performance benchmark. Reuse the accepted
browser evidence and current warm web/native workspaces.

The AspectRatio section is implemented through `438444bf`, with shared positive
finite ratio utilities, a required-ratio editable offline component and an authored
page with seven live examples. Focused compiler/native, installer, ownership, LSP
and real-browser checks pass. See [AspectRatio acceptance](../milestones/ui-aspect-ratio.md).
Its exact pinned upstream source and independent geometry oracle remain in SQLite.
Construction-time ratio support is partial; reactive custom props and intrinsic-width
cross-axis sizing remain unsupported rather than silently approximated.

Continue generic reactive custom props and base composition, overlays/focus, the
remaining pinned component families, blocks and theme generation. The landing's
shared visibility chapters still need the richer paired scroll/transition behavior;
HTML design-reference motion is not shared-renderer parity. The optional JS bridge
remains deferred with mandatory per-export editor/compiler availability across
every configured target. Reuse accepted galleries, editing, resource and SQL
checks; run focused checks after each complete implementation section.

## Rich inline docs and actual web development server — 2026-10-19

The preview at `http://127.0.0.1:4317/` now runs
`tsrust dev --target web --port 4317`, not the earlier Python static server.
The installed CLI supports the same command. Web dependencies/package workspaces
persist and successful publications reload automatically; errors retain the last
successful app. Current browser assets are from `6bbacdbe` and remain byte-identical
through the compiler-identity refresh. No fresh build is needed to view progress.
The live supervisor is still the `6bbacdbe` process; the installed CLI includes the
new proof-reuse repair. Restart the supervisor when its compiler/runtime needs to
change; do not claim its next build already uses that repair.

Rich inline Markdown, configurable syntax colors, exact source copy with retry and
disposal, category navigation, cleaner search and the revised landing pass the
focused live-browser observer. Native inline layout/accessibility and focused LSP
help, including standalone/custom/inline theme JSON, pass through `19d1d634`.
Formatted generated native inline execution passes. Installed CLI/LSP consumers
are refreshed and editor settings restored; the running editor maps the new inode.
Closed evidence is imported and independently restored/verified in SQLite.

The shared native docs app now builds and passes its bounded five-second launch;
it emits no output/errors and is reaped. This does not add OS gesture/pixel claims.
The duplicate-proof repair through `a8f63f4c` passes focused source-proof isolation,
recursive fallback and formatted native project assertions. Reuse these checks.
All five CLI/LSP consumers are current through `a8f63f4c`, with exact settings
restoration and the active editor inode verified. The dev server is idle and
intentionally remains running; its accepted assets are held in memory. Both current
native runtime workspaces stay warm. Obsolete bundle workspaces, executable variants
and the byte-verified old static preview files are removed.

See [current acceptance](../milestones/ui-documentation-inline-and-web-dev.md).
Startup is still slow in the debug compiler (297,033 ms initial, 271,651 ms after
compiler-identity refresh); do not describe dependency persistence as complete
frontend caching or measured performance. Most Base UI/Nova component families,
assets, external links, keyed routing, overlays/focus and custom base composition
remain required. No broad regression, benchmark or SQL rerun was triggered.

## Landing, actual documentation navigation and search — 2026-10-19

Accepted source: `d1870833`, following `a5531211`. The preview now separates `/`
from `/docs`. Refresh the existing browser tab. Every catalog destination has a
real anchor; page content, offline text/API/code search, history, direct/Unicode
heading entry, pending labels and not-found behavior pass the scoped real-browser
observer. Search stays outside the scrolling index; retained article scroll and
home counter state survive route changes. Initial heading focus now runs after
mounting. Page-cache exit releases mounts and persisted return launches a fresh root.

The shared block renderer keeps list/table/link semantics. Compiler project/native
execution, native ownership/accessibility and focused LSP help pass. The same native
docs route passes a bounded launch; this adds no OS gesture/pixel claim. Five
consumers are current; settings are exactly restored and the active editor maps the
installed inode. See [navigation acceptance](../milestones/ui-documentation-navigation.md)
for scope, immutable evidence and cleanup. No full matrix, timing or SQL rerun.

Continue required rich inline text/code/asset rendering and complete Base UI/Nova
families, overlays/focus, blocks and theme editing. The current fixed retained
catalog does not complete keyed/dynamic routing or general route lifecycle policy.
Do not represent pending component pages as implemented components. Preserve the
live preview, current content outputs and warm dependency caches; reuse accepted
checks. The earlier content/foundation sections below remain historical evidence.

## Checked documentation content and child forwarding — 2026-10-19

Content, literal JSON decoding and cached CLI preparation pass focused acceptance
through `522cdf9c`. The generated 103-page catalog contains nine authored pages,
53 checked API declarations, exact source/examples and explicitly pending pinned
library destinations. CommonMark/source ranges, imported inherited prop origins,
invalid-publication preservation and immutable dev snapshots have witnesses.
All four literal JSON scalar families preserve checked precision/ownership;
mixed shape unions remain unsupported. Build/run/dev prepare declared docs content;
checks/LSP remain read-only. No benchmark or SQL suite runs during content builds.

Explicit JSX child arrays now forward with `children={values}`. Syntax conflicts,
evaluation order, editor hover/completion and four typing modes pass through
`3a049e7f`; ordinary owning-array rules remain unchanged. Keep the unrelated
coalesced-array receiver inference limitation visible in readiness notes.
The shared ExampleCard publication at `15b533a3` passes actual browser retention,
exact read-only source, accessibility/narrow layout, native inspection and a bounded
native launch. The existing preview URL serves `docs-content`; refresh it. This
launch reuses accepted native interaction behavior and adds no OS gesture claim.
Five consumers are refreshed from `3a049e7f`; both editor settings files are restored.
Untimed native/Node workload support is repaired at `a09aa41e`; it remains unmeasured.
[Content acceptance](../milestones/ui-documentation-content.md) records exact evidence
and cleanup. Closed records and obsolete executable variants are removed; preserve
the current browser assets, generated catalog and warm runtime dependencies. Continue the required rich text,
keyed identity/navigation, full-text search and complete component families. These
are required work, not conditional follow-ups. Reuse accepted scrolling, mode,
resource/cancellation and database evidence.

## Documentation shell and nested panes — 2026-10-19

Accepted source: `0948dca6`, following checkpoint `4710cde2`. The standalone
`docs/` project now contains editable offline-installed controls, explicit paired
targets and the first actual shared shell. Bounded panes, viewport sizing,
`View.tabIndex`, focus reveal, wheel bubbling and retained scroll state pass
focused native/compiler/editor/browser acceptance. Browser hiding exposed CSS/
offset restoration ordering; the repair preserves actual DOM and scroll state.
The generated macOS window initializes and is reaped. Five consumers and both
editor servers are refreshed with exact settings restoration.

See [foundation acceptance](../milestones/ui-documentation-foundation.md).
One requested preview server intentionally remains at `http://127.0.0.1:4317/`;
refresh for verified section updates. Other acceptance processes are reaped.
Continue content classification, keyed identity/persistent routing, Markdown/code
rendering and offline full-text search. The shell explicitly marks those features
pending; it does not claim complete component pages. Preserve accepted checks;
no full matrix or SQL rerun is due merely for this section.

## Live application state and window setup — 2026-10-19

Accepted source: `c8699512`, after initial checkpoint `5d068a07`. The previously
unverified section now passes targeted compiler/native execution, eight native/
model cases, editor help/diagnostics, an independent Node-after-tsc/browser observer,
formatted-source checks and a bounded generated macOS window launch. Five executable
consumers are refreshed from the accepted source; both editor settings files are
restored byte-for-byte and their servers restarted. No full regression, SQL suite
or performance matrix was repeated.

`withTheme` accepts component-local string signals for live light/dark modes,
preserving children, field values and browser focus. Explicit nested scopes remain
independent, including transparent branches. `View.hidden` retains state/callbacks/
identities while excluding its subtree from layout, input and accessibility; JSX
removal still disposes its branch. `WindowOptions` validates creation dimensions
and macOS custom chrome with OS controls; clipped drag surfaces exclude controls,
including disabled controls and labels. Browser hosts retain their window chrome.
Live titlebar changes, other desktop custom chrome, OS drag-gesture automation and
live token-record editing remain pending. The new example is
`examples/ui-application-state`; the pre-existing `ui-application` fixture and
standalone counter owner files are preserved.

Acceptance also repairs unique composite-result inference into union contexts,
native retained-boolean callback adaptation and hover/definition on typed literal
keys such as `WindowOptions.width`. Ambiguous generic payloads still reject.
`dev` now honors `TSRUST_CACHE_DIR`, matching build/run/cache commands. Its restart
witness passes. The first generated window attempt exposed the ignored override
and was stopped/reaped; its new private cache was removed. The repaired launch
uses the counter's existing dependency workspace and reaps its program after five
seconds. These are untimed acceptance observations, not new performance claims.

Thirty-two exact successful/failed records, source identities, generated package/
browser inputs and consumer receipts use `tsrust-ui-application-` paths in SQLite.
Independent decompression restores every byte and the integrity check passes;
verified loose copies and owned package/site outputs are removed. Continue the paired documentation app
from the implementation plan: native/web pane scrolling and navigation, organized
Markdown/code rendering, real component pages and offline search. General reactive
props, keyed routing and the complete component/block families remain required.

## Installed corrections and startup cache repair — 2026-10-19

Owner's selector correction is installed: compact option rows and padding around
complete lists in rich single/multiple/search/target and plain selectors. Four
focused prompt checks and seven terminal witnesses pass. Full UI component notices
now live once per notice in project `attributions/`, with short source references
and lock hashes. Explicit offline migration preserves component bodies, page,
theme and config. Sixteen installer cases, migration/package export, current
Card/text/content installation and terminal navigation pass. The standalone
counter is migrated, checked and formatted; package/web exports retain notices.

Source `1a653745` also repairs unstable callback error-layout naming. Raw failure
order made unchanged UI source emit different Rust and miss the executable cache;
layout sharing now uses existing canonical checked payload alternatives. Reversed
failure-order/native execution and four fresh compiler-process witnesses pass.
No ownership/error contract, cleanup or transport rule is relaxed.

Native window scrolling and fractional text measurement pass 31 focused cases.
Persistent dependency workspaces preserve Cargo artifacts across runtime source
revisions, with exact identities, serialized writers and explicit clean handling.
The cold GPU build timed out at 240 seconds; resuming the same workspace finished
in 98,987 ms. A later 5,778 ms runtime-cache hit exposed the executable-identity
bug above. After repair, unchanged native starts take 1,394 and 1,556 ms, both
executable-cache hits with zero native/link work. These are bounded warm startup
observations, not a statistical benchmark or cold-start guarantee. Both owned
programs are reaped. Five executable consumers match rebuilt source and both live
editor servers restart with exact settings restoration. Semantic content/run help
and literal fallback regression also pass. No full compiler/LSP, SQL or benchmark
matrix was repeated. Exact original logs/failures use `tsrust-ui-` records in SQLite. Sixty-six closed
records are imported, independently restored byte-for-byte and verified with
SQLite integrity checks; verified loose copies are removed. Eleven obsolete
test executables (932 MiB) are pruned with all five consumers unchanged. No owned
dev, native test program or build process remains; the two editor servers stay live.

Continue the requested documentation application using the standalone project:
component navigation, detailed examples/API/variants, preview and copyable code,
live themes and native/custom window chrome with accessible drag/control behavior.
It is not implemented yet. Current screen remains the counter. Finish missing
reactive props, nested scrolling, component families and paired browser/native
acceptance instead of treating them as optional. Kbd/Separator browser evidence
remains pending; prior compiler/native/editor checks are accepted and reusable.

## Prior source checkpoint notes — 2026-10-19

The standalone counter is becoming a reusable documentation/showcase application:
component navigation, preview/code cards, detailed props/variants, live themes and
configurable native/custom window chrome with accessible controls and dragging.
Repair startup caching, page scrolling and resized multiword text first; see
[the native foundation checkpoint](../plans/ui-window-cache-foundation.md).
Thirty-one native cases and dependency reuse/lock/clean witnesses pass. Standalone
startup/GPU acceptance, editor refresh and the documentation application remain
required work. No broad regression or SQL run is triggered by this checkpoint.

Component licenses now use shared project `attributions/` files with short relative
source references. Offline installation deduplicates full notices, records paths
and SHA-256 hashes, and retains immutable template identities. Explicit
`tsrust ui attributions [--dry-run]` migrates recognized legacy headers while
preserving edited bodies and original provenance. Source package exports and the
bounded web builder retain notices. Sixteen installer cases, public migration /
package export, current Card/text/content installation and affected terminal
navigation pass; original staging-failure evidence is retained with its repaired
witness. Consumer and standalone refresh are pending.

Owner clarification supersedes the row-padding statement below: lists now have
space before/after the complete options, with compact individual rows. Single,
multiple, searchable, target and plain fallback selectors share this behavior.
Four focused prompt checks and all seven terminal witnesses pass; previous
row-padding evidence remains historical. Consumer refresh follows this checkpoint.

## Current terminal component installation — 2026-10-19

All selectors now use padded rows with a smaller visible item count in short
terminals. `tsrust ui add` selects several components with Space and installs one
combined transaction with Enter. Catalogs, browsing and scaffolds use one current
version per component/source variant; scripts can pass names or current exact pins.
Older payloads validate existing records but are not new installation choices.

Upgrades preserve existing files and application imports, adding filenames such as
`card-v3.tsrx`. Newly generated bundled source imports the dependency's recorded
filename. Parsed import rewriting leaves comments, ordinary strings and existing
source alone. Idempotence, collision preflight and transactional cleanup remain.
See [terminal installation acceptance](../plans/ui-terminal-installation-acceptance.md).
Thirty-seven CLI unit, seven terminal and five affected UI project witnesses pass;
no full compiler/LSP, SQL or UI rendering matrix was repeated. Continue the Nova
component families using the standalone counter as the visible working project.

Accepted source checkpoint: `80355ccd`. Five executable consumers are refreshed;
both live editors restart with exact settings restoration. Installed catalog and
counter dry-run/check/format checks pass without source/config/lock writes. The
counter preview adds versioned current files alongside its owner files. SQLite
retains exact logs, identities and cleanup receipts. Seventy-six obsolete outputs
(2649.7 MiB) are removed; all five consumers remain unchanged and no owned test,
build or browser processes remain. Do not rebuild removed acceptance outputs merely
to refill caches.

## Current conditional UI mounting — 2026-10-19

Local-state JSX ternaries now mount lazy nested branches. Selected components keep
state until removed; remounting creates fresh state. Transparent slots and empty
anonymous text create no layout/AX wrapper. Retained controls keep identities,
focus, selection and IME; removed controls/editors release normally. Browser
listeners belong to the mount, and native paint/AX use persistent owned-node IDs.
Ancestry and depth rules are checked again after replacement.

See [conditional mounting acceptance](../plans/ui-conditional-mounting-acceptance.md)
and examples/ui-conditional-mounting. Seventy-one distinct runtime cases, two new
compiler witnesses with explicit rejection reasons, computed/nullable compiler
checks and conditional/computed editor help pass. Three fresh Wasm Chromium
observers agree with independent Node-after-tsc oracles; existing input behavior
survives the delegated event path. A fresh offline locked native export presents
through three resizes. Its dependency build timed out once before window launch;
the same target/package resumed without repeating completed checks and passed.
No full compiler/LSP matrix, SQL or performance timing was repeated.

Original failures/repairs and source identities use `tsrust-ui-conditional-*` in
SQLite. The internal text-run ABI is exported for generated Rust consumers but
has no public Tsrust factory. Local-state conditional component results represent
live slots even when visually absent; precomputed/scalar-prop conditions retain
snapshot behavior. Generic reactive props, arbitrary prop-record state reads and
keyed lists remain pending. Continue Field composition and remaining Nova families,
layouts/blocks, persistent routing and integrated CLI web build/dev. Do not treat
these as optional or describe the complete library as accepted.

Accepted source checkpoint: `2f610a65`. All five executable consumers are refreshed
from that commit; the standalone counter's live editor server restarted with its
settings restored exactly. Installed conditional/computed editor witnesses pass.
The working user demo is `/Users/seancotter/Desktop/dev/learning/tsrust-counter`,
with its screen in `src/app/page.tsrx`. It now demonstrates computed text and
conditional mounting while retaining the owner's original counter controls,
editable button, theme and lock. Its project check, formatting and installed-server
open-document diagnostics/hover pass. Compiler-repo examples remain focused
acceptance witnesses; continue visible demo updates in the standalone project.

## Current computed UI text — 2026-10-19

Supported scalar JSX expressions now refresh text after handlers in Text, View
and Button. Arithmetic, strings, scalar nullable branches and declared top-level
or imported scalar helpers work with directly declared component state. Private
getters reuse text runs and weak state captures; no public attachment API or
duplicate component is required. Numeric children preserve exact int/number types.
This supersedes the earlier bare-numeric rejection. Bare booleans remain errors.

See [computed text acceptance](../plans/ui-computed-text-acceptance.md) and
examples/ui-computed-text. Two new/seven affected compiler witnesses, computed and
existing scalar/nullable editor help, three prop agreements, four affected runtime
witnesses and two fresh Wasm Chromium observers pass. Browser observations agree
with an independent Node-after-tsc oracle, including preserved application errors,
independent mounts, accessible names, unchanged control identities and disposal.
Original failures/repairs use `tsrust-ui-computed-*` in SQLite. No full matrix,
SQL, new GPU smoke or timing was repeated.

Accepted source checkpoint: `39c97cad`. Installed CLI/LSP, workspace server override
and both shared-target executables are rebuilt from that commit. The live editor
restarted; installed computed-text help and computed-project/counter checks pass.
Counter source/settings remain unchanged. SQLite retains consumer hashes, exact
record restoration and cleanup receipts. Both generated sites and 117 obsolete
build outputs (1434.7 MiB) were removed; all five consumers remain unchanged.
No owned build/browser/test processes remained after acceptance.

Continue dynamic conditional mounting and Field composition, then the remaining
Nova component/layout/block families, keyed editing and persistent routing.
Integrated CLI web build/dev remains required. Inferred helper returns, generic
child contracts and arbitrary prop-record reads are still snapshots; do not claim
general reactive props or full library parity.

## Current nullable UI composition — 2026-10-19

Null/undefined children and stateful `Node | null | undefined` component results
now pass focused compiler/native/editor and fresh Wasm Chromium acceptance.
Absent results create no placeholder and preserve lexical callback-before-owner
cleanup. Present components retain independent live state. Nullable owning arrays
still reject cloning/borrowed forwarding; bare numeric/boolean children remain
unsupported. Contextual match blocks retain the complete union rather than only
its present payload, with an ordinary source match regression too. See
[nullable composition acceptance](../plans/ui-nullable-children-acceptance.md).

Three compiler witnesses, nullable help, scalar help, input-event consistency,
three prop agreements and the independent Node-after-tsc/browser observer pass.
Reuse previous unchanged renderer/GPU/runtime evidence. Originals and repairs use
`tsrust-ui-nullable-*` in SQLite; no SQL, full matrix or timing was repeated.

Accepted source checkpoint: `b1d1f255`. Installed CLI/LSP and the workspace
server override are refreshed. The active editor restarted; installed nullable
help plus both nullable-project and standalone-counter checks pass. Counter
source/settings are unchanged. SQLite retains consumer hashes and verified cleanup
receipts; generated sites and minimized debugging fixtures were removed.

Continue dynamic conditional mounting/derived text and Field composition, then
complete remaining Nova components/layouts/blocks, keyed editing and persistent
routing. Integrated CLI web build/dev is also a concrete gap: current browser
output uses the repository's bounded Wasm builder. Do not document an unimplemented
`tsrust build --target web` command or treat these requirements as optional.

## Current editable text controls — 2026-10-19

Portable Input/Textarea and local Nova input/textarea/label v3 sources now work on
both targets. Controlled/uncontrolled editing, weak derived value/readOnly props,
borrowed change snapshots, Unicode selection/IME, clipboard, undo/redo, scrolling,
accessible labels/descriptions/state and scoped responsive styles are implemented.
Use `examples/ui-inputs`; no public handle or dummy handler is needed. See
[text-control acceptance](../plans/ui-editable-controls-acceptance.md) for exact
boundaries. Specialized inputs, value-change details, form integration and full
Label composition remain pending; all three coverage entries stay partial.

Focused acceptance reuses 63 distinct runtime witnesses and six compiler witnesses,
with event documentation/consistency, eight receiver agreements, two typing rows
and native-alias type help. Fresh Wasm Chromium transitions match Node-after-tsc;
separate failure mounts release independently. Nested dark disabled/invalid colors
and custom spacing/radius pass. Fresh generated native export builds offline/locked
and presents/resizes on the GPU. Failed attempts, repairs and exact identities use
`tsrust-ui-input-*` in SQLite. No SQL, full regression pass or new timing was run.

Accepted source checkpoint: `6281f73c`. Both installed tools and the workspace
LSP override are refreshed; the active editor server restarted. The standalone
counter source/settings remain byte-identical, and both project checks plus the
installed event-editor witness pass. SQLite holds the consumer hashes and cleanup
receipts. Obsolete outputs reduced the shared target from 19,065 MiB to 12,195 MiB;
all five consumers and useful dependencies remain. No owned test processes remain.
Some removed build variants will rebuild on demand; no warm-up pass was run.

Continue Field composition and its nullable/dynamic child prerequisites, then the
remaining complete Base UI + Nova component/layout/block families, keyed editing
and persistent routing. Reuse accepted input/IME and ownership evidence. Do not
replace missing parts with inert wrappers or treat remaining UI work as optional.

## Current shadcn continuation — 2026-10-18

The owner selected current Base UI + Nova. The pinned inventory contains all 64
Base UI source components, three composed documentation recipes and 30 Base UI
blocks, including their block dependencies. See
[coverage](../plans/ui-component-coverage.json). Pending entries are not installed
stubs or accepted clones. Existing immutable Button/Card v1/v2 remain intact.
Card v3 now supplies seven working portable content parts; its explicit media,
containment and font-asset gaps keep full Nova parity pending.
Continue composition, layout/theme capabilities, real controls and complete source
families across both targets. No further planning approval is required.

The first prerequisite section is implemented: quoted record keys and exact JSX
aria-/data- names; semantic Text/View tags; accessible names, heading levels,
regions, alert/status content, separators and hidden-control validation. Empty
children default to empty arrays. Native semantic content is separate from its
accessible name, preserving paragraph/code/live-role bodies and stable action IDs.
Invalid naming on generic/paragraph/code/presentation elements fails explicitly.
The editor completes exact names, inserts bracket access for quoted members and
keeps fields/hover/navigation while source is incomplete. Constructor semantic
failures retain the facade's readable-string behavior and consume failed trees.

Focused acceptance reuses completed passes: all eight JSX witnesses, eleven UI
compiler witnesses, runtime ownership/theme/accessibility checks, UI editor help,
three UI consistency checks, the semantic typing-speed row in four modes and all
five UI prop-type agreements. Only affected failures were repaired/rerun. Real
Wasm Chromium DOM/accessibility/disposal agrees with the independent Node-after-tsc
oracle. Native conformance changes have affected semantic checks; the final
formatted generated-native example also checks readable invalid-naming errors.
Whole-project formatting proves token preservation/idempotence; final pass changes
only the new facade formatting. SQL, broad regressions and timing were not rerun.
Original failures and exact pinned upstream inputs use tsrust-ui-semantics-* and
tsrust-shadcn-* in canonical SQLite. No independent screen-reader session or full
component-library acceptance is claimed. Use examples/ui-semantics as the focused
primitive witness, and preserve the standalone counter owner's source.

## Current owned child composition — 2026-10-19

The second prerequisite section implements `Child = string | Signal | Node` for
View/Button, span containers, mixed text/subtrees and optional button handlers.
Adjacent text bindings share one owned run. Buttons validate phrasing descendants;
nested controls/flow elements fail and release all handler environments once.
Hidden decoration stays outside accessible names. Passive buttons remain focusable
without retaining a dummy callback. Reactive disabled predicates still refresh.
Native composed labels use centered flex layout and preserve one action target.
Browser listeners exist only for actual callbacks and disappear on disposal.

Existing owned string/Node arrays, including nullable forwarding props, move into
broader tagged child arrays without cloning. Element conversion preserves order,
exact integer values and resource ownership. Borrowed/mutable widening, narrowing
and implicit numeric conversion fail. Shared clone validation now traverses union
array leaves; inferred native array callbacks retain their checked identity rather
than requiring users to import extra public type names. The focused native signature
unit helper now maps only the two visible aliases its witness actually exercises.

Focused acceptance includes 24 native runtime checks, compiler UI/ownership and
exact-array witnesses, UI editor hover/diagnostics, the composed-span/import rows
in all four typing modes, ViewProps/ButtonProps checker agreement and three UI
consistency checks. Reuse passed checks; fixture-only failures and original logs
remain preserved. Real Wasm Chromium agrees with the Node-after-tsc oracle for
mixed children, state, disabled predicates, listener ownership, accessible names,
centering and disposal. The existing paired counter passes fresh-artifact browser
pointer, Tab/Enter/Space, disabled-control, stable-node, resize and disposal checks.
Examples live in examples/ui-composition; formatting checks
apply to all source. No timing, SQL or full regression run is implied.

Computed strings/trees still snapshot at construction; only direct Signal children
and disabled predicates bind updates. Numeric/boolean/null children, dynamic child
reconciliation, rich inline Text, editable controls, overlays and Base UI render
composition remain real pending capabilities. This section does not accept any
pending Nova component/block as complete. Continue shared layout/theme capabilities
and complete source families without another approval; preserve immutable legacy
payloads and the standalone counter owner's source.

## Current shared layout foundation — 2026-10-19

The layout section implements grid/flex display, equal tracks, the two portable
written track lists, starts/spans, per-side padding/gaps, preferred/min/max sizes,
fractions, item alignment, wrapping and grow/shrink. p/px/py and size expand into
independent properties so later source classes win correctly. Sizes 2/4/6 retain
sm/md/lg; 1 uses sm/2, 3 averages sm/md and other numeric sizes use md/4 per unit.
Native root preferred widths now survive window layout; max constraints still
apply. Text leaves reject container display/track/wrap utilities explicitly;
composed Buttons and Views can use them. Item dimensions/padding remain valid on
Text. Conditional method compilation removes UI dependency warnings without blanket
suppression; generated host and Wasm runtimes preserve their required methods.

The shared utility provider owns validation, CSS/native values, documentation and
theme references. Completion/hover describe real behavior and navigation reaches
all contributing spacing fields. No new field/member types changed, so the
accepted UI checker-agreement evidence remains applicable.

Focused acceptance: five new layout/guard checks, the existing style merge check,
two native presentation checks and the composed-button geometry check; generated
native std:testing example and invalid-token witnesses; new/existing theme editor
checks; grid completion in four typing modes. The editor fixture now actually
scopes its imported theme, matching the API being navigated. Original failed build
and fixture records are retained. Real Wasm Chromium grid placement, spans, resize,
custom spacing, accessible region and disposal agree with the independent
Node-after-tsc oracle at three widths. Fresh counter artifacts retain presentation;
reuse the earlier complete counter input/disposal pass.

See examples/ui-layout and the layout guide for exact supported sizes/limits.
Arbitrary CSS, breakpoints, named lines, subgrid and scrolling remain unsupported.
Alpha colors, Nova typography, clipping, structural selectors and complete Nova
source components/blocks remain pending. No SQL, full regression or performance
run was requested by this prerequisite section. Preserve the earlier evidence and
continue implementation rather than repeating passed suites.

## Current checked paint colors and surface rings — 2026-10-19

The paint section adds alpha hex and checked in-gamut OKLCH, canonical semantic
aliases, integer color-opacity modifiers and surface ring width/color utilities.
Both targets consume one resolved sRGB float representation; imported theme strings
stay unchanged. OKLCH lightness/alpha accept 0–1 or percentages, chroma is
nonnegative/unitless, hue is unitless 0–360. Other functional syntaxes, missing
components, hue/chroma units, invalid ranges and unsupported gamut fail. Tiny
conversion roundoff is retained, not clipped; native scene encoding explicitly
quantizes premultiplied sRGB to RGBA8. No HDR/wide-gamut, transparent-window or
pixel-identical rasterization claim is made. Background must resolve opaque.

Color /N multiplies existing alpha for integer percentages 0–100. Completion offers
common percentages after slash without multiplying ordinary completion lists.
Surface rings use independent width/color and reset per node; parent ring colors
do not leak into children. Legacy ring remains an explicit outline whenever its
state matches. Surface rings and focus outlines stay independent. Native opacity
bounds include known child overflow and style outsets rather than clipping rings
to the button box. Square stroke corners retain square joins.

Focused acceptance passes eight color/contrast/encoding checks, disabled overflow,
existing theme behavior/validation and keyboard focus. The generated-native
std:testing example checks rendered styles and original data, with invalid utility
witnesses; the affected compiler theme check passes. New/existing editor color and
theme checks pass, including unsaved gamut failure. Color-opacity completion passes
all four typing modes. Real Wasm Chromium matches the independent double-precision
Node-after-tsc oracle for alpha hex, OKLCH, multiplication, aliases, border/ring
colors, scope reset, hover, keyboard focus, accessible names and disposal. The
native witness checks actual encoded scene data, not GPU readback. Default text
and focus pairs have light/dark contrast checks; arbitrary custom themes still
require accessibility review. Reuse earlier geometry/input/ownership evidence.

Source/oracles live in examples/ui-colors and benchmarks/ui-jsx. The exact official
CSS Color 4 input and closed original records use tsrust-ui-colors-* in canonical
SQLite. No new public field/member types changed, so accepted checker agreement
remains applicable. No SQL, complete regression or performance pass was rerun.

Continue the actual Nova source families. Card has seven parts and requires
metadata/structural selectors, numeric spacing scope, typography and edge/corner
styles; these must be generic renderer capabilities, not runtime Card switches.
Keep original MIT attribution and immutable v1/v2 payloads. No pending component
or block is accepted by these prerequisite sections, and no further planning
approval is required.

## Current owned prop forwarding and structural conditions — 2026-10-19

Extended local props can move through JSX into a narrower declared component
interface. Only excess spread fields are discarded, with native destruction once;
explicit unknown attributes and missing required fields fail. Omitted optional
fields hold undefined. Sources evaluate once in order, including overridden fields;
exact integer values survive. Ordinary object spread/rest retains checked-copy
semantics. Record spread storage planning now respects existing JSX move markers.

Shared data-size metadata and bounded data/descendant/named-group/own-class
conditions drive both targets. Predicates compose by conjunction, exclude self for
ancestor/descendant matching, and preserve source order per property/state.
Browser predicates have zero extra specificity. Mount scope surrounds each complete
selector and every ancestor condition, avoiding cross-mount matches. Multiple named
groups can match the same ancestor without imposing an ordering between groups.
Native matching borrows context without cloning owned descendants.

Focused acceptance includes native predicate/metadata/layout checks, exact-value
JSX output, formatted rejection controls with diagnostic assertions, generated-native
independent release observations, editor forwarding/hover/chained completion, all
five prop agreements, both new typing rows in four modes and three UI consistency
checks. The initial rejection fixture incorrectly failed before its intended rule;
it now uses named records and asserts the actual diagnostic. Original records remain
preserved. Fresh Wasm Chromium agrees with the independent Node-after-tsc oracle for
small/default scopes, spacing, descendant predicates, own classes, chained groups,
live DOM metadata, two-mount isolation, accessible content and disposal. The browser
caught incorrect ancestor scoping; the repaired witness reuses unaffected evidence.
Final whole-project formatting preserves tokens and is idempotent.

Examples/oracles live in examples/ui-forwarding and benchmarks/ui-jsx. Closed records
use tsrust-ui-structural-* in canonical SQLite, including original failed controls
and browser runs. No SQL, broad regressions or timing was rerun. Existing counter
input/ownership acceptance remains applicable. This is an accepted prerequisite,
not acceptance of any pending Nova component or block. Continue Card's real source
family; numeric spacing scope, typography, edge/corner styles, clipping, media and
inline-size containment remain concrete requirements rather than inert markers.

## Current typed scalar UI state — 2026-10-19

`signal(initial)` now infers int, number, boolean or string. `Signal<T = int>`
preserves existing integer annotations. Strings read/write independent snapshots;
integers retain all 64 bits, numbers retain IEEE-754 values and direct Text bindings
use ordinary language conversion (including -0 displayed as 0). Arrays, records,
nullable values, callbacks and resources are explicitly rejected as scalar state.
No public handle/owner helpers or duplicate reactive components were added.
The returned tree keeps the single owner; Text and callbacks retain weak bindings.

Empty-props JSX now supports named zero-parameter components. Attributes, children
and ordinary `Component({})` calls retain their arity errors. The paired example
uses two independent `<Counter />` instances and local application callbacks.
The LSP substitutes scalar types into get/set help and preserves documentation on
the generic public alias. Compiler facts expose the source-level scalar read rather
than its hidden Result call, including distinct generic specializations.

Focused acceptance covers four compiler witnesses, all 17 existing UI witnesses
(reusing the earlier unaffected passes), 32 shared runtime checks, four scalar
editor types, public alias help, four receiver agreement rows, state consistency
and all four string-result typing modes. The first matrix report captured missing
answers despite a successful command status; its original is retained. The fixed
row passes in assertion mode. Native renderer package checks offline/locked.
Fresh Wasm Chromium observations match the independent Node-after-tsc oracle for
both instances, text/disabled updates, keyboard activation, exact large integers,
special number strings, accessibility and disposal. Initial setup, harness and
canonical-path failures remain original evidence under `tsrust-ui-scalar-*` in
SQLite. No new timing, SQL pass or full compiler/LSP milestone run is claimed.

Continue real editable controls using the pinned Base UI Input contract and Nova
source already saved in SQLite (`tsrust-ui-editable-input-upstream.json`). Reuse
Parley editing/selection/composition rather than hand-written Unicode editing.
Input/textarea/field semantics, native IME, overlays, lists and the remaining
complete source families/layouts/blocks are still required work.

## Current generic alias foundation — 2026-10-19

Generic non-record aliases now accept type parameters and trailing defaults.
Substitution preserves lexical parameters, callable access, exact integers and
resource ownership across modules. Defaults can refer to earlier parameters;
missing/extra arguments, cyclic aliases and oversized expansions fail explicitly.
Record/interface/function parameter defaults remain unsupported. Data borrowing
stays on function parameters rather than aliasing an access wrapper.

The LSP uses a compiler query for generic alias expansion, retains imported
private record/native members, shows defaults and follows definitions. Focused
acceptance covers five compiler witnesses, six existing alias regressions,
imported/local editor behavior, a checker agreement row, hover/typing consistency
and all four typing modes for the new array-alias row. The formatted project
executes with exact 9007199254740993, native-cache aliases and independent resource
release observation; clone/reuse rejection remains strict.

The existing matched type-alias workload now exercises generic aliases/defaults
without changing its result, oracle, operation counts, ABI or timing protocol.
Four untimed native/Node-after-tsc cases agree; registration guards pass.
Whole-project formatting proves token preservation/idempotence. Original initial
build failures and all completed evidence use `tsrust-generic-alias-*` in SQLite.
No full regression, SQL rerun or new timing is claimed.

Continue typed UI state and editable controls using this foundation. This is a
language prerequisite, not component-family completion. Base UI + Nova remains
the selected baseline; the remaining component/block inventory stays required.

## Current Card content port and shared surface styling — 2026-10-19

card@3.0.0 installs all seven editable content parts with the pinned Base UI + Nova
MIT attribution. Default/small cards, descriptions, actions, content and footers
work on both targets. This is a partial content port, not full Nova parity:
image primitives/edge selectors, inline-size containment/container queries and
named heading-font assets remain unsupported and explicit in source/help/coverage.
Header is an ordinary grid, so intrinsic layouts can differ from containment.
Do not publish inert markers or call the remaining library/blocks complete.

Generic renderer capabilities implement inherited numeric spacing declarations
[--NAME:--spacing(N)] and p/gap references. Values resolve in their declaring theme
scope; metadata conditions and source overrides work. Missing references resolve
to zero; interaction-state declarations, aliases/arithmetic/cycles fail. text-sm,
numeric weights, leading ratios and generic font families share their values.
Explicit leading wins over a size default. radius.xl is optional and falls back to
lg; present null, negative and out-of-range values fail. Native corners scale
together and asymmetric borders use elliptical padding curves. Independent sides
and corners merge in source order. overflow-hidden clips paint and pointer targets
at physical display scale while preserving source/keyboard/accessibility order.
Native scrolling/automatic focus scrolling remain unsupported; necessary controls
must not be placed entirely outside a clipped region. Cubic curve approximation,
platform font selection and rasterization do not imply pixel identity.

API 2 accepts API 1 locks/payloads while preserving exact ABI/toolchain checks.
New installation records retain each payload's actual minimum API, so adding old
Button v2 beside Card v3 remains supported. Existing v1/v2 payload bytes are intact.
The installer/catalog remains offline and does not overwrite local components.

Focused acceptance: five new native style/geometry checks and six affected existing
paint/presentation/opacity checks; native Card inspector and offline installation
with full license/old-version compatibility; styled/plain/JSON catalog witness;
Card/source/utility editor help; ThemeRadius and affected ThemeConfig agreement;
font and radius completion rows in four modes. Reuse earlier ownership, input,
structural and prop agreement evidence. The null-radius boundary was tightened
and only its affected unit was rerun. A generated native GPU package passes cargo
check offline without launching a window. Fresh Wasm Chromium matches the independent
Node-after-tsc style oracle for all seven parts, default/small/custom spacing,
caller overrides, header layout, typography, rounded click clipping, accessible
heading/region/buttons and disposal. All examples pass project formatting and
formatted native execution. No SQL, broad compiler/LSP regression or timing rerun.
Original records use tsrust-ui-card-* in canonical SQLite; exact generated native
package inputs are preserved there. Source/oracles: examples/ui-card and
benchmarks/ui-jsx. Full-family/performance acceptance remains separate.

Continue implementing concrete missing primitives and real source families without
another go-ahead. Editable state/IME, focus/overlays, render composition, media,
containment, remaining semantics and layouts/blocks are required work. Inventory
state support before editable controls; use the pinned Base UI API and existing
Parley editing facilities where practical. Preserve the single std:ui facade,
automatic ownership and app-local callbacks; no public handle/ownState/add plumbing.

## Current terminal UI correction — 2026-10-18

The owner rejected the numbered command dump as incomplete interactive CLI work.
It now uses grouped, styled keyboard screens, a compact wordmark, searchable
offline component browsing/details, Back navigation, editable input and native/web
multiselect. Pinned Cliclack/Console provide terminal controls; ordinary command
dispatch and installer ownership remain. Redirected/JSON output stays plain,
color policy is honored and TERM=dumb retains a numbered fallback. This supersedes
the numbered-only interaction below; reuse accepted non-presentation behavior.

Six focused CLI witnesses pass, reusing five prompt-guard passes and rerunning only
the repaired keyboard witness. Two prompt unit checks and existing help/usage
checks pass. Real terminal acceptance covers arrows, search, browser details/back,
project creation/check/formatting, exact component installation, web-only and paired
target selection, colors/NO_COLOR/forced color, narrow displays, unattended guards,
Esc/Ctrl+C and restored input flags/cursor. A prompt-wide input guard fixes the
redraw cancellation race; the test ignores only kernel-owned PENDIN status, not
input behavior flags. Original failed runs remain in SQLite under tsrust-terminal-ui-*.
Whole-project formatting changes zero files/fingerprints. The installed CLI is
atomically refreshed and manually exercised from the standalone counter through
UI browsing and clean cancellation. No SQL, rendering, full regression or timing
suite was rerun; independent Windows terminal acceptance is not claimed.

## Current interactive CLI correction — 2026-10-18

Bare `tsrust` previously errored despite the owner's request for interactive CLI
workflows. It now opens a terminal command menu using existing command dispatch,
with guided `new` destination/type selection and exact bundled component selection
for `ui add`. Explicit commands remain scriptable; redirected bare invocation is
nonblocking, and JSON/dry-run/unattended UI installation never prompts. Existing
target selection and installer ownership rules remain; choosing creates no files.
Four focused CLI checks pass, including real terminal project/UI creation,
cancellation, EOF, invalid-answer retries, source/format checks and unattended
guards. Prompt unit coverage and existing help/usage checks pass. Whole-project
formatting changes zero files/fingerprints. Native rendering, SQL and full-family
regressions are reused rather than rerun. Evidence uses `tsrust-interactive-cli-*`
in canonical SQLite. See [the project CLI guide](../../website/src/content/docs/tools/cli.md).

## Current HTTP editor correction — 2026-10-18

The owner reported missing hover on `ctx.req.queries` and questioned Hono parity.
Hono does expose `queries(name)` for repeated parameters. The editor lost native
field identity when Context imported Request indirectly. Shared field rendering
now retains identity; typed, generic, aliased and inferred nested access passes
focused hover, completion and parameter-help checks. Both typing-speed rows,
editor consistency and Context/Request checker agreement pass. See
[the query acceptance note](../plans/http-query-acceptance.md). No-argument record
overloads and the documented encoded-name ordering difference remain open parity
gaps, not accepted 1:1 behavior. Reuse native query evidence for this editor-only
repair; no SQL, rendering, full regression or performance rerun is needed.

## Current application configuration update — 2026-10-18

The owner requested inline `ui.theme` in `tsrust.config.json`, source conventions
under `src/app` and `src/components`, explicit boolean ui.target choices, interactive selection without flags,
and first-component setup through ui add without a separate
initialization command. This supersedes external theme files/root-level app paths
in historical entries below. Legacy external theme paths remain readable.
Complete this as one compiler/CLI/editor/examples section; preserve source edits,
configuration fields, exact offline identities and accepted rendering evidence.
See the application guide for explicit installation and read-only import diagnostics. Imports, build/run/dev
and editor checks must never install configuration or components.

Focused acceptance passes: ten installer/target witnesses, four CLI application
witnesses, three compiler theme witnesses and eight UI editor witnesses. Reuse
those accepted results; the focused web-only export repair also passes. Real PTY
selection retries an invalid choice and creates no files before selection. Web-only
new scaffolding checks/formats successfully. Native CLI rejects web-only window
entries; web package export defaults to ui-web, and the bounded web builder rejects
native-only configuration before writing output. Embedded remains unsupported.

The standalone counter is migrated to src/app and src/components with its page
bytes and theme values preserved exactly. Check, format, generated native build and
its screen test pass; renderer support is reused. The matched untimed Node-after-tsc
oracle agrees with generated HTML from inline configuration. Rendering/input,
lifetime and SQL evidence from c0eeb633 remains applicable and is reused; no broad
regression or timing matrix is triggered. Closed records use tsrust-ui-config-* in
canonical SQLite, including the original editor failure and repaired diagnostics.

## Current owner direction: complete UI application workflow — 2026-10-18

The owner stopped local tuning in favor of a researched, coherent UI system.
Follow [the application-system contract](../plans/ui-application-system.md),
UI_VISION.md sections 6/18/19 and the existing ownership decisions. Themes,
configurable editable component source, immutable bundled offline installation,
compatibility/lock records, shared styles and LSP are one implementation section.
They are missing product requirements, not optional later polish. Keep Caveman
updates concise and Ponytail's minimum complete solution; no repeated planning
approval is needed. Do not reopen the selected GPU or bounded Wasm decisions.

The application section now implements project-owned light/dark theme JSON,
semantic token scales, editable local Button/Card source, exact bundled v1/v2
payloads, offline dependency installation and compatibility/provenance locks.
`new --ui` uses that same installer. JSX prop spread transfers owned callbacks and
children; direct signal-reading `disabled={count.get() < 5}` predicates use weak
captures and refresh after handlers. No public lifetime plumbing is required.
See [the application guide](../guides/ui-applications.md) for supported utilities,
range checks, source preservation, explicit upgrades and unsupported cases.

Compiler and editor acceptance covers derived conditions, owned prop forwarding,
configured/unsaved theme errors, lock incompatibility, inherited props/variants,
class completion/hover, definitions, checker agreement and editor consistency.
Installer witnesses cover exact dependency closure, custom destinations, edited
source/lock preservation across compatible compiler changes and refusal to
silently overwrite upgrades. Shared native layout/paint and theme precision checks
pass. The untimed Node-after-tsc oracle agrees with generated native HTML tokens
and component sizes. Existing lifetime, cancellation, grammar and SQL evidence is
reused; no full regression or performance matrix was started.

Real Chromium acceptance passes derived disabled updates (0, 5, 105), stable DOM,
dark semantic colors, local Button size/padding, rounded keyboard focus, focus-over-
hover priority, stable font/height across 640/800/480/640 widths and disposal.
The generated Wasm is 396,689 bytes plus 15,619 bytes of JavaScript glue,
uncompressed; these are payload observations, not speed measurements. UI package
export now permits the pure JSON module needed by theme serialization while still
rejecting native service/file/task modules in the bounded browser tool.
Generated native GPU acceptance passes the same state transition through actual
Accessibility button actions and closes the window cleanly. Pixel inspection is
not claimed. The first new support build took 270.7 seconds; the standalone build
reused renderer support and spent 3.3 seconds in native compilation/linking. These
are diagnostic samples, not controlled benchmarks. Installed compiler/LSP consumers
are refreshed; real standalone protocol checks include local props, classes, clean
diagnostics, invalid setter reporting and diagnostic clearing. The standalone
GPU app independently passes 0–5–105–104 while the second counter stays at 0;
its application-owned screen test and formatting/check commands also pass.

Runtime artifacts no longer have implicit disk size/count quotas. Explicit
`cache info`/`cache clean`, identity checks, atomic publication and private copies
remain. Earlier 255.7-second cold/115-ms warm/3.586-second edited-build observations
are historical diagnostics, not controlled benchmarks or current measurements.

Closed records are byte-verified in canonical SQLite under `tsrust-ui-contract-*`.
One earlier theme-agreement rerun accidentally overwrote its loose original log;
only a retrieved diagnostic excerpt survives for that attempt. The archive labels
that excerpt explicitly. Earlier failures and subsequent complete repaired logs
remain intact. Do not claim full original-byte preservation for that one attempt.

The standalone project retains its owner's Decrement handler, two independent
counters and exact derived disabled condition. Continue with the editable-list
family (string/boolean state, text editing/IME, selection, keyed insertion/removal
and deterministic disposal), then persistent layouts and typed routing. These are
required product work, not optional counter polish. The GPU and bounded Wasm
choices remain settled. No full UI-readiness claim is made.


## Standalone UI project and installed editor — accepted 2026-10-18

Created `/Users/seancotter/Desktop/dev/learning/tsrust-counter` with `tsrust new`.
The generated config retains `src/main.tsr` and `target/tsrust-counter`; the source
now uses the accepted paired counter, with a rendering test and local editor
settings selecting installed executables. Opened this directory as a separate
VS Code root. The installed LSP passes real protocol checks for clean `.tsrx`
diagnostics, Signal get/set completion, setter hover, an invalid setter argument,
and diagnostic clearing after correction. The editor has one idle server for this
root. Whole compiler-fixture workspace scanning had delayed open-file diagnostics;
the speculative scan opt-out was discarded, not shipped.

The standalone test exposed the same typed-record formatting borrow defect in the
`std:testing` callback adapter. The shared adapter now borrows the error payload.
The focused throwing-record regression and actual standalone screen test pass.
Installed compiler refreshed. Existing UI, browser and native acceptance is reused;
no full regression, SQL or timing suites rerun. Closed original failures and repaired
checks are verified in SQLite under `tsrust-state-*`. Continue the unfinished UI
scope listed below; this small project is not a full UI-framework readiness claim.


## Direct UI state bindings and captures — accepted 2026-10-18

`Signal` is used directly in JSX and callback bodies. The compiler creates
non-retaining bindings at use and closure creation; the returned component tree
retains its single state owner. Public `add`, `handle`, `SignalHandle`, `SignalText`
and `ownState` are absent. State exposes `get()` and `set(value)`; the `.tsrx`
application handler calculates the increment. Disposed access throws `UiError`.
Checked integer overflow occurs before mutation and preserves the previous value.
The formatter preserves conventional parenthesized multiline JSX returns.

Focused acceptance passes: compiler UI witnesses, generated nested callbacks and
overflow preservation, repeated bindings, branch/loop captures, parameter shadowing,
private API rejection, nine runtime witnesses (including independent destruction),
JSX layout/idempotence, editor hover, UI completion at all four typing modes,
Signal checker agreement and editor consistency. The generated Wasm counter passes
real Chromium pointer/keyboard, independent-state, disabled, stable-node, targeted
text, disposal and late-click checks against the updated TypeScript oracle.
Generated native GPU presentation and bounded close also pass. Native OS input
injection and screen-reader sessions remain unverified. Wasm is 231,840 bytes plus
15,601 bytes of JavaScript glue, uncompressed; this is not timing evidence.

Original failures and repaired checks are retained in SQLite under `tsrust-state-*`.
The native callback error adapter repair borrows typed error payloads while
formatting them; the generated-button witness prevents recurrence. Earlier static,
SQL and full-family evidence is reused. No broad regression or performance run was
triggered. State remains integer-only; constructor aliases, state factories, state
creation inside anonymous callbacks, general reactive props, dynamic trees,
text input/IME and async UI remain open. Named local functions retain ordinary
checked capture rules; automatic non-retaining capture currently applies to arrows.


## Paired interactive counter — 2026-10-18

The owner selected the bounded Wasm browser backend. The generated counter now
passes real Chromium pointer, disabled-button, Tab, Enter and Space checks, two
independent instances, stable node identity, targeted text updates, explicit mount
disposal and late-click rejection. Generated native GPU presentation/close passes;
shared state, native keyboard/action dispatch, AccessKit actions and independent
capture destruction have focused Rust witnesses. Native OS input automation and
screen-reader sessions were not performed. These limits are not visual parity claims.

The owner rejected separate reactive components and manual state attachment.
Ordinary `Text` now accepts mixed literal/SignalHandle children. Public `SignalText`
and `ownState` are absent. Returning a component subtree automatically transfers
its direct local signal owners through ordinary checked moves. Multiple owners and
branch returns are covered; premature moves and rebindable owners reject. Constructor
aliases/anonymous callback state are explicitly rejected. Broader state factories,
reactive properties, text input/IME, dynamic trees and async UI remain open.

See `examples/ui-counter/README.md` and `tools/build-ui-web.mjs`. The Wasm artifact
is 217,823 bytes plus 15,601 bytes of JavaScript glue, uncompressed. These are
artifact sizes, not timing measurements. The typed independent counter oracle,
source hashes and original repair logs are retained in SQLite under
`tsrust-counter-*`. Compiler UI tests (4), runtime witnesses (8 plus independent
capture-drop witness) and focused editor help pass. Reuse static/JSX and SQL evidence.
The browser harness fixes concern target selection, Chromium key text and profile
cleanup; their original failures remain recorded. No full compiler/LSP, SQL or
performance matrix was rerun.


## Generated static UI and counter continuation — 2026-10-18

`std:ui` now connects checked `.tsrx` components to the shared owned renderer.
`Text`, `View`, disabled `Button`, `document` and `run` have public documentation.
HTML-only output excludes native GPU dependencies. Native exports use the existing
pinned renderer dependencies. The shared screen passes generated semantic HTML
checks and an actual bounded GPU presentation/close. This does not claim pixel,
screen-reader or interactive acceptance. Node ownership/private-runtime checks and
JSX highlighting checks pass; editor hover acceptance is recorded with this section.

The owner selected WebAssembly for the paired interactive counter, not as a
permanent browser-backend commitment. The Wasm target and digest-verified matching
wasm-bindgen tool are prepared. Continue component-owned state, retained checked
handlers, weak handles, targeted bindings, pointer/keyboard input, disabled behavior
and cleanup on both targets. The static renderer and syntax evidence should be
reused. No SQL or full regression rerun is required for this section.


## JSX compiler integration — 2026-10-18

Named JSX components now lower into ordinary checked calls with props records and
ordered children arrays. `.tsrx` discovery, configured CLI execution, import
resolution, dev watching and editor attribute help are integrated. Focused native,
editor and matched TypeScript witnesses pass; see [JSX integration](../plans/jsx-integration.md).
The formatter preserves JSX islands verbatim. This is syntax acceptance, not paired
rendering or counter acceptance. Continue checked renderer bindings, generated
static output and the paired interactive counter without another go-ahead.


## GPU-first UI foundation — 2026-10-18

The owner selected Winit + Taffy + Vello. `crates/tsrust-ui` now provides a bounded
owned static tree with shared utility meanings, HTML escaping, native text/layout,
GPU window rendering and AccessKit semantics. Native API checks, three focused
checks and a bounded real window presentation/close pass. See its README for exact
limits. The Rust-constructed example does not count as generated Tsrust UI.
Continue `.tsrx` parser/checker/lowering, project/CLI/editor integration, then the
paired generated static fixture. Preserve the static/interactive distinction and
accepted lifetime evidence. Renderer selection is resolved; JS/WASM selection is
still unnecessary for this noninteractive slice.


## Named HTTP query helpers — 2026-10-18

`c.req.query(name)` and `c.req.queries(name)` return owned optional text/arrays
without consuming the request body. Scope and Hono differences are explicit in
[acceptance](../plans/http-query-acceptance.md). Continue remaining ordinary HTTP
helpers/composition and the paired static `.tsrx` UI slice. The UI review remains
read-only; this thread remains the sole writer/build owner. The owner selected GPU-first Winit + Taffy + Vello on 2026-10-18. Continue the
paired static slice; static output does not require choosing JS/WASM yet.


## Process arguments — 2026-10-18

`std:env.args()` now exposes owned arguments with checked Unicode errors. Native,
editor, executable documentation and untimed native/Node witnesses pass. See
[acceptance](../plans/process-arguments-acceptance.md). Continue ordinary HTTP query
and response helpers, composition/error hooks, then the open UI milestones.
Generic inheritance and CLI argument acceptance do not establish framework parity.


## Generic interface inheritance — 2026-10-18

Explicit generic interface bases now support local/imported definitions, aliases,
multiple/chained bases, optional and nested fields. Wrong arity, cycles, duplicate
fields and unknown unused arguments reject; nominal ownership is unchanged.
Compiler-derived editor fields include project and incomplete-member recovery.
Focused acceptance: six interface tests, six retained-project-session tests,
three interface editor tests, agreement for InheritedItem, its four-mode completion
row, and 17 editor consistency checks. The unchanged interface oracle passes eight
native/Node untimed cases; no timing or SQL rerun. See
[acceptance](../plans/generic-interface-inheritance-acceptance.md).
Continue CLI usability, ordinary HTTP APIs and UI foundations under the owner's
readiness correction below; this closes only generic interface inheritance.


## Owner correction — language and framework readiness, 2026-10-18

Continue basic language completeness, CLI usability, Hono-style API coverage and
UI implementation. These requirements are not conditional on selecting another
service workload. Accepted S21 sections do not establish overall readiness.
See [the current source-backed inventory](../plans/language-framework-readiness.md)
for concrete gaps and implementation order. Reuse accepted evidence; batch targeted
verification by complete section. Preserve Router naming and application-owned auth.



## Current continuation — SQL cursors and application jobs, 2026-10-16

Checked `Pool.cursor`, `Cursor.next` and `Cursor.close` now pass focused runtime,
generated execution, ownership, editor and untimed native/Node acceptance. The
explicit SQL runner passes all three drivers and downstream SQL/HTTP/migration
checks; accepted runtime and exact-domain evidence was reused on repair. The await
emitter no longer applies a second borrowed-union conversion. See
`docs/plans/async-sql-acceptance.md` and SQLite evidence under `tsrust-cursor-*`.

The owner selected application-level durable jobs using existing SQL pools, not
`std:jobs`. `examples/durable-jobs` implements SQLite claims, leases, bounded retry,
deduplication, stale-worker fencing and recovery. Native execution and independent
persisted-state inspection pass, as does the matched 128-job native/Node workload.
See `docs/plans/durable-jobs-acceptance.md` and `tsrust-durable-jobs-*` evidence.
This example supports SQLite only; external effects still need their own idempotency.
Source checkpoint: `dd85a30d`. Installed CLI and both editor servers are refreshed;
the active job example has no workspace problems with checking enabled.
Rust-analyzer remains disabled. Consumer hashes and cleanup proof are retained
under `tsrust-sql-jobs-*` in SQLite. Cleanup reclaimed 37.78 GiB of obsolete
workspace, cross-target, test and native outputs while preserving all five current
consumers. Reuse the single shared target; deleted caches are intentionally cold.

The concrete cursor/job follow-ups are implemented. Keep remaining scope practical:
The owner correction above supersedes the earlier conditional scope for streaming,
HTTP/UI and language work. Redis/brokers remain demand-driven. The S21
full-family/performance checkpoint remains separate and unrun under the owner's
current targeted-verification direction. Do not repeat accepted SQL/S18 or S19/S20
work. Preserve Router naming and application-owned authentication policy.


## Current continuation — producer stream bridge, 2026-10-16

`std:http.streamChannel(Channel<Bytes>, status = 200)` now transfers bounded
application-produced chunks into a response. Focused runtime, compiler, editor,
formatting and untimed native/Node verification pass; evidence is in SQLite under
`tsrust-producer-*` and the HTTP producer record. This bridge preserves producer
scope: it does not detach request children. Callback-owned producer lifetimes and
SQL cursors remain the next streaming work. See `docs/plans/http-acceptance.md` for
bounds and unsupported cases. Reuse accepted S21 and SQL evidence; no full-family
regression/performance run was added.

## Current acceptance and continuation — 2026-10-16

Checkpoint `c82daa25` adds typed Router application/request state, cookie helpers
and the service route rewrite. The native metadata/body boundary is repaired:
body ownership remains exclusive while request/response metadata can be shared
across suspension. All 16 affected HTTP runtime checks, five ownership rejection
witnesses, focused editor checks and integrated macOS service acceptance pass.
Old App retained-state examples are preserved; the new example is
`examples/http-state/router.tsr`. The v3 native/Node workload passes untimed verification. The revised service
and relocated compiler/editor package pass offline Linux aarch64 acceptance.
Installed CLI and both LSP consumers are refreshed; the active editor reports no
workspace problems with checking enabled. Rust-analyzer remains disabled.
Implementation acceptance is complete for this scoped Router/service section.
The S21 performance/full-family checkpoint remains separate and unrun. Original failures and intermediate
checks are retained byte-for-byte in SQLite under `tsrust-router-*`.

The public router is named `Router`, not `Hono`; Hono is only the API reference.
The existing server lifecycle is accepted. Ordered/scoped middleware, multiple
callbacks, inline async arrows and conditional short-circuits now execute in the
focused native example. The matched untimed workload passes. Original evidence
is in SQLite under `tsrust-hono-*`; the Unicode runtime witness was corrected to
observe Unicode through a response body instead of an ASCII-only header accessor,
and that affected witness now passes its repaired rerun. No full regressions or
new timing were run. The typed state, cookie and service rewrite continuation is described above. Keep application authentication policy out of the framework.
Installed CLI and both editor servers now include Router/async arrows; the active
editor reports no Tsrust errors. The cookie-loop TextMate bug is repaired and
visually verified after reload. Rust-analyzer was restored to disabled for this
workspace. Context form/multipart bridges pass integrated macOS service acceptance.

Owner correction supersedes the authentication wording below: implement a
Hono-style public HTTP API over the existing Axum backend. Prioritize request
context, routing/middleware, request and response headers, cookie helpers, checked
request-local state and safe shared application state. Authentication, sessions,
user models and authorization policy belong to application code. The existing
service authentication example is an application witness, not a framework API or
a substitute for these primitives. See `docs/guides/language/http-and-hono.md` and the current
scope in `docs/plans/service-readiness-plan.md`. Preserve accepted ownership,
cancellation and cleanup rules. Finish this API section before further deployment
verification; reuse completed evidence and batch targeted checks.

SQL migrations and recovery now pass focused acceptance. `Pool.migrate` keeps its
checked pool borrow, validates versioned files and history, and closes failed or
cancelled migration connections through tracked cleanup. All three drivers pass
failure/recovery and independent lock-release observation; ephemeral SQLite inputs
reject before migration. The full explicit SQL runner, relocated offline package,
editor checks and matched untimed history workload pass. Exact failures, repairs,
consumer hashes and cleanup proof are in SQLite under `tsrust-migrations-*`.
Installed consumers are refreshed; the restarted active editor reports no problems.
Continue S21 observability and scoped HTTP/security/authentication, then actual
Linux/offline/signal and integrated service acceptance. Do not repeat completed
SQL or S19/S20 suites without a relevant change. See
`docs/plans/service-readiness-plan.md` for contracts and remaining work.

Shared cache integration now passes focused native/compiler/editor acceptance:
`SharedLruCache`, explicit `share`, owned `getCloned`/`peekCloned`, unchanged local
cache loans, retained HTTP access and independent destruction. Constructor catches
now receive their missing failure checkpoint. Matched scalar checks pass untimed
in both engines. CLI/LSP consumers are refreshed and the restarted editor reports
no problems. Exact original failures and repaired passes are in SQLite under
`tsrust-shared-cache-*`. Continue remaining S21 observability, HTTP/security/authentication and actual Linux/offline/signal
acceptance. Do not repeat completed S19/S20 regressions or timing.

S21 retained read-only HTTP state now passes focused acceptance: owned configuration
and SQL pools, routes/middleware/fallback, suspension and exact-once destruction,
compiler/editor agreement, all three SQL drivers, and matched untimed HTTP state
checks. CLI and both editor consumers are refreshed; the active editor reports no
problems after restart. Native support builds now preserve `CARGO_BUILD_JOBS` to
respect the caller's memory limit. Exact logs and workload records are in SQLite
under `tsrust-http-state-*` and `benchmarks/http/retained-state-*-verify.json`.
Continue remaining S21
requirements in `docs/plans/service-readiness-plan.md`. Full S21 acceptance and
its timing milestone remain open. Do not repeat accepted S19/S20 regressions.

Exact decimals now pass focused macOS acceptance: `std:decimal.parseDecimal`,
exact bounded arithmetic, checked conversions, explicit cloning and worker
transport, editor agreement/help, 13 native catalog examples, offline package
export and matched untimed money checks. See `docs/guides/language/data-type-coverage.md`.
Installed CLI/LSP consumers are refreshed; open decimal examples report no
problems after the active server restart. SQLite holds `tsrust-decimal-*` evidence.
Current CLI SHA-256: `e1678a081b74795801123e9da220e818bff42a1a0358ef2c2632fbb97f0fa7d7`.
Current LSP SHA-256: `49b5cb23b43f9169af491876e522563bbd6fa123486e93b2576e8fdc891c0ca1`.
The `std:date` calendar section now passes focused acceptance: validated Date,
parseDate/formatDate, date-fns-style immutable arithmetic, bounded formatting,
editor agreement, 15 catalog examples, offline export and matched untimed date-fns
workload. The original fixture failure and repaired pass remain in SQLite under
`tsrust-calendar-*`. Installed consumers are refreshed; VS Code shows no problems.
Temporal values also pass focused acceptance: Time, Instant, Duration and TimeZone,
exact nanoseconds, checked epoch conversion, explicit DST gaps/folds and named/fixed
zones. Chrono-TZ 0.10.4 pins IANA 2025b; later zone-rule changes are not included.
Four compiler/export checks, 31 native docs examples, runtime/editor checks and
matched millisecond workload pass. Current installed consumers include this section.
UUID focused acceptance also passes: strict canonical text, independent exact bytes,
fallible entropy, ownership/worker transport, nine native documentation examples,
offline export and matched untimed workload. Installed consumers include UUID.
Explicit checked Rust width/byte/Option/tuple adapters now pass native/compiler
and editor acceptance. Direct unsupported signatures still reject. SQL domain
witnesses now pass all three drivers, including exact decimal, date/time, UUID,
JSON and nullable storage. Pinned Rust/date-fns/Decimal.js declaration inventories
are recorded. Full data-type family regressions and targeted integration repairs
now pass; see `docs/guides/language/data-type-coverage.md` for retained original failures and final
evidence. The five accumulated workloads are measured at `b911a378`: 930 samples
with checksum agreement, exact records and derived summaries in SQLite. Native
medians are lower in these scoped first baselines. Do not repeat this full pass.
The local `std:cache` API now passes focused compiler/runtime/editor acceptance.
Normal reads handle undefined without `!`; ownership, resource destruction and
checked TTL/capacity errors pass. See `docs/milestones/s19-local-cache-acceptance.md`
for the conservative lifetime boundaries and exact evidence. Matched LRU
verification passes; timing is not yet performed. Installed consumers include the
cache API and the restarted active editor has no problems. The serialized loading
integration now passes focused cancellation, closed-client, independent drop and
Node counterpart checks. Cache matrix/agreement gaps and two native emission
defects are repaired; see the current section in `docs/plans/cache-plan.md`.
Bounded loading/shutdown now passes with pre-registered clients, explicit completion,
closed-reply handling and a group deadline. Owned Task capture metadata and deferred
exception assignment are repaired with focused ownership/drop witnesses. S19
implementation acceptance is complete within its serialized-loading limits.
S20 bounded work queues now pass focused native/compiler/editor acceptance, including
backpressure, exact cancelled-message destruction and close/drain. Mixed task groups
exposed repaired empty-failure and record-origin compaction defects; see
`docs/plans/work-queue-plan.md`. Installed consumers are refreshed and the active
editor reports no problems after restart. The combined S19/S20 owned-coordination
milestone now passes: 4,194 compiler and 731 editor tests, separate doctest drivers,
and 372 measured cache/queue samples. All 18 sandbox socket failures passed exact
unsandboxed reruns; do not repeat the full suites. See
`docs/milestones/s19-s20-owned-coordination.md`. Continue S21 implementation now
using `docs/plans/service-readiness-plan.md`. No new permission checkpoint is needed. The owner prefers date-fns: use small
named immutable functions and familiar arithmetic/formatting names, while keeping
checked ranges, precision and explicit DST ambiguity handling. The bounded Rust/SQL
boundary witnesses are accepted; proceed without another go-ahead.

The first data-type section, `std:numeric` (`4760e58d`), now passes focused compiler/runtime,
editor and matched-workload acceptance. It preserves original int/number union
inputs and rejects loss for toInt/toNumber/toFloat32, validates signed/unsigned
8/16/32/64-bit ranges within int storage, and parses strict exact decimal integers.
Full u64 above i64::MAX and 128-bit values remain explicitly unsupported. See
`docs/guides/language/data-type-coverage.md` for measured workload scope. SQL REAL precision repair
also passes explicit three-driver acceptance, including precision/underflow
rejection and unchanged automatic rollback. S19 is next; no new go-ahead is required.


The checked `std:testing` resource facility and synchronous UI lifetime contract
are accepted on macOS arm64. Compiler source is `3c23f7b8`; the UI representation
repair is `5540a836`. See [the current L01–L26 mapping](../../experiments/ui-tsrust-owner-handles/CHECKED-RESOURCES.md)
and `release-matrix.json::checkedContinuation`. Historical external projected Drop
rejection remains intentional. Do not restore a separate public testing-resources
module or treat synchronous success as async/worker UI readiness.

Passed: 7 resource compiler tests, 56 testing-library tests, 4 callback-error tests,
the native-effect unit test, all existing UI feasibility/identity checks, and final
native demo exit 0. Added subtree-removal/FIFO-payload coverage passes its affected
formatted native rerun. Final editor suites pass: checker_agreement 2,
completion_matrix 3 and editor_consistency 9. The initial agreement run imported
all bundled testing code into every unrelated probe; its original log/sample are
retained. Scoping that import to the resource row preserves every receiver and
member check; the repaired full agreement suite passes in 234.55 seconds.

Installed CLI `~/.cargo/bin/tsrust` and both installed/configured LSP consumers were
atomically refreshed. VS Code's Tsrust server was restarted; the already-open
resource example and workspace Problems panel report no problems. Installed CLI
execution and installed LSP resource hover/signature help pass independently.
The user's rust-analyzer workspace extension remains disabled.

Prior SQL CLI SHA-256: `296f5d1ba4c6ac93a49925d7ba97870b2d36179e8dd9e1241dd47d74d984c19e`.
Prior SQL LSP SHA-256: `8cf43bc480b94629114fccafdf0e9bbaebd072d9faeb0c5bd7065d541599a0b7`.
Both consumers were refreshed again for numeric acceptance. The active editor
server was stopped when inspected, then restarted through its status item.
Both already-open numeric fixtures initially reported no workspace problems.
Subsequent consumer inspection found repeated background-thread stack overflows
in the active editor (SIGABRT at 19:20:25, 19:31:47 and 19:35:36). The faulting stack identifies workspace scanning of nested HTTP calls, not the
numeric fixture. A real-server regression reproduced the failure using the
existing HTTP loopback example. The single analysis worker now has a named,
bounded 8 MiB stack instead of the platform default. Four real-server fixture
checks, five workspace-scan checks and six server lifecycle/version checks pass;
installed-server fixture checks also pass. The editor was restarted and open
documents again show no problems. This fixes the witnessed finite call nesting;
it is not an unlimited-recursion guarantee. Resume the domain types below.
Reuse `/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0` and
`CARGO_BUILD_JOBS=1`. The restart made the initial build cold; no separate target
was introduced. Closed diagnostic records are byte-verified in
`benchmarks/records.sqlite` under `tsrust-resource-*`, including original failures,
repairs and interrupted runs. Reuse accepted S18 SQL evidence; SQL/task lifecycle
code did not change in the resource section.

Continue the [data-type coverage inventory](../guides/language/data-type-coverage.md), then S19–S21,
without another go-ahead. Discovery now includes pinned TypeScript 5.9.3 and a
bounded standalone rust-analyzer 0.3.3081 run; the latter shuts down after probing
and does not re-enable the editor extension. The Tsrust inventory handles native
aliases and explicitly identifies the internal non-importable TestRegistry.
The bounded domain/width coverage is accepted; unsupported wider source types
remain explicitly documented.
SQL REAL narrowing is now checked: finite values that round or underflow reject
with precision. All three drivers passed with retained/resumed evidence; no need
to repeat this suite until subsequent SQL or task-lifecycle changes require it.

One writer/build owner remains in this tree. Preserve the owner's six unrelated
untracked UI plan files. The UI review chat may review readiness read-only until
an explicit write/build window is coordinated.

## Restart checkpoint — 2026-10-15, resource observer implementation

The owner requested a restart-safe stopping point during resource acceptance.
Resume the priority order below; no data-type, S19, S20 or S21 implementation has
started in this pass. The practical-service requirements below are durable.

Implemented `TrackedResource`, `trackedResource()`, `resourceId()`,
`closeResource()`, `releaseCount()`, `releaseOrder()` and `liveCount()` in
**std:testing**, as requested by the owner. Native bindings remain private in
std:testing-runtime; do not restore the temporary std:testing-resources namespace.
See [resource contracts](../guides/resource-lifetimes.md) and
`examples/resource-lifetimes`. Native Drop releases an allocation before recording
its integer identity; the observer never retains a resource. Record replacement,
array removal, callback capture, explicit close, local async suspension, failure
unwinding and abandoned lazy-task cleanup pass. Worker, channel and started-task
transport reject explicitly. External Rust projected-data Drop rejection remains.

Focused results: 6 tracked-resource tests, 6 Rust-bridge tests and 3 native-resource
lifecycle tests pass. Final focused editor completion (four typing modes) and
hover/signature-help tests pass. The full checker-agreement suite passed against
the earlier temporary namespace; the final std:testing full editor pass was stopped
at the owner's restart request. Do not claim final full editor acceptance. Resume
checker_agreement, completion_matrix and editor_consistency on the final source;
retain completed evidence. A mistaken direct invocation selected an older editor
test binary and ran zero tests; the correct newest binary then ran and passed the
new help test. This is recorded rather than counted as acceptance.

**Demonstrated UI blocker:**
`experiments/ui-tsrust-owner-handles/witnesses/tracked-release.tsr` passes active
callback self-removal and exact-once release, then fails the assertion
`automatic child release must precede parent`. Owner scope destruction bypasses
its explicit child-first cleanup; current registration insertion order destroys
parent captures first. No owner representation repair has been applied. The UI
review chat confirmed that reversing handlers alone is insufficient because drain
removes/re-appends entries. Group component state and captures into a lifetime unit,
with an extractable callback slot if state must remain accessible during dispatch;
maintain descendant-first destruction independently of display order and FIFO.
Monotonic descending component IDs are a candidate only while reparenting is absent.
Add post-delivery, nested-mount, same-key replacement and sibling-reorder cases.
Resource-bearing state and queued payload acceptance remains pending; define rejected
enqueue ownership and stale/consumed/owner-drop payload cleanup explicitly. Do not
claim B2 or async UI readiness from the primitive. Preserve existing identity rules.

This is a source checkpoint, **not a completed resource acceptance gate**. Remaining:
final editor suites, testing-library/error-effect cross-checks, full resource/UI
acceptance, benchmark applicability review, and consumer refresh after acceptance.
No timed workloads or S18 three-driver suites were rerun. The changed shared error
metadata is limited to new private testing functions; SQL/task lifecycle is unchanged.
Installed CLI/LSP and configured editor consumers remain at the prior accepted
checkpoint. Shared debug binaries contain this work; rebuild from Git if needed.

Original logs are byte-verified in `benchmarks/records.sqlite` under
`tsrust-resource-*.log`; interrupted passes and original failures are preserved.
Whole-project formatting passed, then changed zero files/fingerprints on repetition.
Six unrelated untracked owner plans are preserved. Active test processes were stopped
for restart. Reuse the shared Cargo target with incremental compilation disabled;
removed test artifacts may need rebuilding. Do not launch a full historical pass.

## Current handoff — 2026-10-15, after S18 acceptance

This section supersedes all historical “next”, “pending”, consumer-version and build-window
statements below. Follow the [current roadmap priorities](../plans/standard-library-roadmap.md).

1. Finish checked resource lifetimes for UI, including a real destructor and independent
   non-retaining release audit. Read the roadmap acceptance contract and the B2
   `experiments/ui-tsrust-owner-handles/RESOURCE-RELEASE.md`, `release-matrix.json` and
   `RESOURCE-RELEASE-REVIEW.md`. Preserve Drop rejection for projected external records.
   Coordinate readiness with “Review UI vision and roadmap”; compiler acceptance alone
   does not close the UI matrix or establish worker readiness.
2. Complete the roadmap's required data-type coverage pass. Map existing support first,
   then implement practical gaps for UI, Rust/SQL boundaries and S19–S21. Include exact
   numerics, dates/times, UUID, JSON, binary and nullable values in the boundary inventory.
   Require explicit conversion/ownership contracts and compiler/native/editor witnesses.
   Record unsupported cases honestly; do not expand into an unlimited type-system audit.
3. Continue S19 caches, S20 queues and S21 integrated acceptance autonomously after the
   priority work. The owner explicitly authorized continuation, not another planning stop.

S21 now requires practical service readiness, not a minimal demo. Follow the
[service requirements and coverage methodology](../plans/standard-library-roadmap.md#practical-service-readiness-and-coverage-methodology):
authentication, safe shared state, versioned migrations/recovery, redacted
observability, scoped HTTP/security contracts and actual Linux/container/offline/
signal/shutdown acceptance. Defer the polished MVP/demo, not executable witnesses.
Calendar/date/time coverage is a first-class prerequisite; monotonic clocks are
not calendar support. Compare pinned TypeScript and Rust language-tool inventories
with Tsrust and prove behavior through native/compiler/editor witnesses. Implement
in coherent sections and retain accepted evidence. The same roadmap section orders
post-S21 follow-up and names deferred scope.

### Completed work: reuse evidence, do not restart

S12 workers/pools/channels/bridges and package management, S13 async checking, S14 Tokio,
S15 HTTP, S16 TLS, S17 SQLite and S18 async SQL have accepted implementations within
linked scope. The roadmap rows and individual acceptance documents define limitations.
No global pool remains: use owned `ThreadPool`; `spawn` stays dedicated.

S18 checkpoints: `1b4a1639` implementation, `1eeaac6e` nested cleanup repair,
`cf3aa83c` acceptance documentation. See [SQL acceptance](../plans/async-sql-acceptance.md)
and [guide](../guides/async-sql.md) for exact checks and limits. PostgreSQL, MySQL and
SQLite passed separately, including cancellation, rollback, acquisition recovery,
owned rows and readable examples. A cancelled drain now retains nested cleanup until
settlement. Preserve this regression and SQL pool/transaction loan behavior.

Compiler/editor focused suites, offline export, installed consumers and untimed
native/Node checksum parity passed. Full regression and timed benchmark matrices were
not rerun for S18. Do not relabel untimed evidence as measurements. Original logs,
failures, repairs and benchmark records are byte-verified in `benchmarks/records.sqlite`;
closed loose copies and temporary Docker containers were removed.

Database suites are ignored by default. Run `node tools/check-async-sql.mjs --docker`
when SQL runtime/dependencies, task lifecycle, SQL borrowing or SQL emission changes;
unrelated documentation or source changes do not require rerunning them. The script
provisions and removes its own three-driver acceptance resources. Cancellation can
wait for in-flight queries; a cancelled/disconnected commit has indeterminate outcome.
Migration orchestration, streaming and compile-time database schema checking remain
deferred. The shared SQLite dependency is now SQLite 3.51.3 via rusqlite 0.39.0 for
SQLx compatibility; synchronous SQLite correctness passed after that change.

### Current consumers and exclusive workspace

Installed CLI `~/.cargo/bin/tsrust` and LSP `~/.cargo/bin/tsrust-lsp`, plus the configured
`target/editor/debug/tsrust-lsp`, were refreshed from `1eeaac6e`. The final LSP bytes
passed the installed completion matrix; the active VS Code workspace showed no problems.
Do not rebuild/restart them merely to resume this handoff. Refresh after new accepted API
changes. Reuse `/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=1`, with Node v24.11.1 first on PATH. Inspect current status and confirm
exclusive checkout/build ownership before edits; historical UI validation windows
below are no longer current reservations.

Preserve all unrelated owner UI plans, including the six currently untracked files:
`ui-b4-compiler-repair-first-task.md`, `ui-identity-enforcement-first-task.md`,
`ui-lifetime-contracts-resource-observation-first-task.md`, `ui-resource-release-first-task.md`,
`ui-state-lifecycle-first-task.md` and `ui-tsrust-owner-handles-first-task.md` in `docs/plans`.
Follow AGENTS.md and the development workflow for whole-project formatting, focused
acceptance, source checkpoints, evidence import and cleanup. No archives or snapshots.

## Historical continuation and acceptance notes

The following dated notes preserve decisions and evidence. Their old scheduling,
consumer hashes and unfinished-work statements are superseded by the current handoff.

## Owner priority, updated 2026-10-14

The owner approved a bounded practical-performance review and a return to product
features. Extreme-nesting optimization is deferred. This supersedes earlier
instructions to finish every nesting audit, eliminate every quadratic path, or
achieve universal benchmark parity before S12. Historical measurements remain
valid for their original inputs; they are not current acceptance requirements.

Do not turn this continuation into an open-ended compiler cleanup campaign.
Fix correctness failures, crashes at practical depths, and measured costs that
block a representative workload. Keep existing depth witnesses and diagnostics.
Do not raise worker stacks or silently remove resource checks.

## Accepted work and current evidence

- S01–S11 are accepted within their documented scope. The
  [S11 acceptance](2026-10-13-s11-acceptance.md) retains its historical evidence.
- Parser/checker/emitter worklists, disposal, constructor and spread repairs are
  implemented across multiple checkpoints. Use the
  [pipeline history](../plans/compiler-nesting-pipeline.md) and
  [source layout](../guides/compiler-source-layout.md) for implementation details;
  old open-item statements must be checked against later commits.
- Schema `safeParse`/`safeParseJson` expose `{ success, data } | { success, error }`.
  Compiler narrowing, editor hover/completion, examples and workload migration are
  accepted. Do not restore the old Result-match surface.
- Program-scoped test registry: `7c7b5bc0`. Callbacks own eligible local captures
  and borrow module state only while invoked. Registry disposal precedes module
  storage disposal. [Design and acceptance](program-scoped-test-registry.md).
- Nested concat overflow: `fc82d9db`. Receiver and argument work is scheduled on
  the shared expression stack; earlier operand loans stay active in source order.
  Depth-100 string/array witnesses execute on normal stacks; depth 200 preserves
  the existing admission diagnostic. All 18 affected compiler tests, 10 editor
  agreement tests and 20 native/Node correctness cells pass.
- Allocation fixtures/adapters: `263ff644`. Seven migrated fixtures, 19 adapter
  tests and 189 native/direct-Rust/Node executions pass. Native allocation and
  disposal records are in `tsrust-allocation-syntax-acceptance-2026-10-14.json`.
- Rust warnings: `00f996c8`. Obsolete resolver entry points/import removed;
  test-only query facades remain test-only. Live generated-runtime APIs retain
  behavior and gain meaningful backend coverage. Workspace/all-target checking
  reports no warnings/errors; 219 affected test executions pass. VS Code reports
  No Problems. Evidence: `tsrust-rust-warnings-acceptance-2026-10-14.json`.

Original acceptance and failed attempts are stored in `benchmarks/records.sqlite`.
Registry, concat and consumer refresh records use respectively
`tsrust-program-registry-acceptance-2026-10-14.json`,
`tsrust-concat-acceptance-2026-10-14.json` and
`tsrust-consumer-refresh-2026-10-14.json`.

## Practical performance decision

See [the bounded performance review](../reviews/2026-10-14-practical-schema-performance.md)
for current representative timings, allocation evidence, comparability limits and
priorities. The historical 31-row S11 review has 27 slower native medians, not the
28 claimed in its prose. Some references omit validation work, return existing
objects, or construct simpler errors. They are handwritten references, not Zod
measurements and not a universal language-performance ranking.

Use fused `safeParseJson` when validating raw JSON directly. The separate owned
`safeParse(JSON.parse(...))` path intentionally constructs an intermediate dynamic
tree. Preserve both APIs and their ownership contracts. Do not remove safety checks,
add hidden clones/shared wrappers, or weaken reference observations to win a ratio.

Keep timing and implementation acceptance separate. No new optimization is
required solely because a specialized reference wins a microbenchmark. Profile
representative applications before changing general schema machinery.

## Deferred work and trigger to reopen

1. **Quadratic fragment assembly in extremely deep expressions.** Recorded example:
   roughly 4.8 seconds at 32,768 nested Set constructors. Programs containing many
   ordinary functions scale linearly in the recorded checks. Reopen for measured
   compile-time/memory impact in actual generated source or an explicit untrusted
   compiler-service requirement. This is not a blocker for S12.
2. **Quadratic nested array-literal spread checking.** Retain witnesses; reopen
   when representative source makes it material.
3. **Broader architecture audits and representation limits.** Preserve known
   limitations and diagnose practical failures. The historical audit inventory is
   not an instruction to exhaust every possible shape before feature work.

The match/default-cleanup milestone completed consolidated compiler (4,008 passed,
7 ignored) and LSP (679 passed, 1 ignored) regressions. Initial failures and affected
repairs remain in SQLite. Do not repeat this full pass for routine edits. Differential
suites require pinned Node v24.11.1 first on PATH.

## S12 owned workers

The owned `spawn`/consuming `join`/explicit `detach` slice is implemented across
compiler, runtime, editor and benchmark support. See [scope and acceptance](../plans/owned-thread-acceptance.md)
for the direct-handle lifecycle boundary and evidence. Broader S12 remains open:
Bounded channels and multicore throughput are accepted; restricted local
Rust/pool integration is implemented. See [bridge acceptance](../plans/rust-rayon-bridges.md)
for its supported boundary and current checks. Broader Rust package mapping remains
separate work. Do not repeat public spawn/join groundwork or treat untimed checks as timings.

Preserve already settled decisions:

- `spawn` starts immediately. `join` observes completion; otherwise owning-scope
  cleanup joins and discards completion. `detach` explicitly opts out. No hidden
  clone, automatic detach or implicit shared ownership. Global `unwrap` propagates
  the original Result error through ordinary try/catch.
- Local Rust modules and supported wrappers belong alongside S12 before worker
  APIs harden. Automatic lazy default pooling for explicit threaded interop is
  approved; evaluate Rayon and reuse compatible pools. Direct calls stay direct.
  Queue/admission/shutdown contracts remain material design work. Arbitrary
  crates/macros/traits/lifetime-heavy APIs are not implicitly authorized.
- Async tasks are lazy, uniquely owned and single use. Calling transfers inputs;
  awaiting consumes. Start/cancel/timeout direction is approved. Never-started
  tasks warn; abandoned started jobs error. Cancellation alone does not settle a
  job or guarantee source finally execution. Detailed combinator contracts remain
  open; `std:futures` naming remains a recommendation.
- Cache reads borrow. Explicit clone creates independent supported values.
  Expiration occurs during operations; outstanding used borrows block mutation.

Later dependency order remains S13 async checking, S14 runtime/task lifecycle,
S15–S16 HTTP/TLS, S17–S18 SQLite/SQL pools, S19 cache, S20 queues, S21 integrated
service acceptance. This review does not implement or accept those features.

## Work discipline and consumers

Follow `AGENTS.md` and [development workflow](../development-workflow.md). One
agent works in this tree. Inspect status and preserve owner UI/interop plans.
Use Git checkpoints, not archives or snapshots. Format the entire project before
source checkpoints; preserve tokens, idempotence and executable fixture behavior.

Reuse `/private/tmp/tsrust-gate100-editor-target` with `CARGO_INCREMENTAL=0` and
`CARGO_BUILD_JOBS=1`. Keep normal worker stacks. Preserve exact original evidence
in SQLite, verify restoration/integrity, then remove closed loose copies. Prune
obsolete test/native outputs without deleting owner editor caches.

Five executable consumers are the installed CLI/LSP, the workspace override
`target/editor/debug/tsrust-lsp`, and shared-target debug CLI/LSP. Consumer refresh
now uses formatter source checkpoint `9fb8036f`, with atomic installed
release replacements and matching SHA-256 hashes. The primary VS Code server was
restarted and its workspace reports No Problems. The second `tsrust_test` window
was not rechecked in this pass; reload it if it still uses an older server.
Request both `--bin tsrust` and `--bin tsrust-lsp` when selecting Cargo binaries
explicitly.

The current pass adds value-producing matches, scalar literal patterns and named
record variants. See [match acceptance](../plans/match-values-acceptance.md).
It also makes worker cleanup the default for `spawn` and exposes global `unwrap`;
see [default cleanup](../plans/default-worker-cleanup.md). Compatibility spellings
remain available. Family checks and all three matched workload measurements pass.

Owned `ThreadPool` instances replace the global `pool` singleton and generic
`Pool` name. `new ThreadPool()` supplies system-based defaults; explicit options
set workers and capacity. Constructor failure throws SpawnError without unwrap.
Task, stats, shutdown, cancellation and cleanup retain the accepted ownership
rules. A shut-down instance cannot restart; a new instance has an independent
lifecycle. See [pool acceptance](../plans/worker-pool-cancellation.md). Migration
compiler/runtime/editor and offline-export acceptance passes, with zero leaks in
the cleanup corpus and current matched benchmark measurement. Release consumers
are installed and hash-verified; the restarted primary VS Code server reports
No Problems. Both installed-CLI examples produce their expected output.

Required next S12 work: dedicated-worker channels. Preserve owned transfer and
typed completion. Cancellation remains cooperative, with join acknowledging
completion; there is no forced thread termination. ThreadPool container storage is
rejected until lifecycle support exists. No async/deferred keywords or parallel
iterator API were added. The full family regression remains separate from this
focused pool/cancellation acceptance.

## Empty-delimiter formatting acceptance

Checkpoint `9fb8036f` keeps empty `{}`, `[]` and `()` inline, including gaps
containing blank lines. Comments retain their previous formatting. Acceptance:
29 formatter tests, 240 lexable fixtures / 84 callable contracts, 5 LSP formatting
tests and 8 fingerprint guards pass. Whole-project formatting changed 65 files
and refreshed 37 token-preserving fingerprints; a repeat changed zero files or
fingerprints. Installed release formatting is idempotent and its native probe
prints `0`. The primary editor server was restarted with checking enabled.
Evidence is stored under `tsrust-empty-delimiters-*.log` in SQLite. Unrelated full
regressions were not repeated.

## S13 async ownership continuation — 2026-10-15

S13 compiler/editor support is accepted: lazy `Task<T>`, consuming `await`, eager
argument/default evaluation, retained ReadOnly/Mutable loans, generic transfers,
source errors and normal finally handling. Readable examples live in
`examples/async-tasks`. Full regression evidence and the matched lazy-task
measurement are in [async acceptance](../plans/async-tasks-acceptance.md).

The next async section is S14 Tokio scheduling/I/O and started-task lifecycle.
S13's local future driver supplies no scheduling, timer or cancellation API.
Dropping a future releases owned resources without promising source finally
execution. Returning a child task retaining borrowed data is currently rejected
with an actionable source diagnostic; await it and return its value instead.
Nested owned tasks remain supported. Preserve these boundaries and the existing
thread ownership/cancellation rules when integrating Tokio.

## S14 Tokio completion — 2026-10-15

S14 adds owned scheduling with `Task.start()`, required Job settlement, cancellation,
deadlines, delay/yield, race/allSettled and bounded asynchronous channels. Compiler,
runtime and editor acceptance passes; see [Tokio acceptance](../plans/tokio-tasks-acceptance.md)
and [usage guide](../../website/src/content/docs/library/futures.md). The active editor was restarted with
current Task documentation and no open-document diagnostics.

Keep `await yield()` as settled. No external pause/resume or priority API was
requested. Cancellation remains cooperative; dropping suspended futures releases
resources without promising source finally execution. Dedicated threads, pools
and Rust/Rayon bridges remain separate. S15 HTTP is the next roadmap dependency;
networking and a blocking-work adapter were not added in S14.


## S15 HTTP completion — 2026-10-15

S15 is accepted: Axum routes/middleware, owned request/response/body/continuation/
server, reusable Reqwest clients, validated bodies, demand-driven forwarding,
disconnect and bounded graceful shutdown. See [acceptance](../plans/http-acceptance.md)
and [guide](../../website/src/content/docs/backend/http.md). Source af0cbba8 includes the final async disposal
repair; release CLI and both LSP paths are refreshed and VS Code restarted.

Consolidated acceptance: 4,111 compiler tests and 700 editor tests pass, seven
existing compiler tests ignored. HTTP measures 5.39 ms native versus 7.99 ms Node
per 128 sequential loopback requests. Async review repaired owned receivers,
retained loans, temporary disposal and multi-input task lifetimes; it removed one
body-chunk copy without redesigning scheduling. Exact evidence remains in SQLite.

S16 TLS listener configuration and certificate fixtures are next. Shared handler
state/providers, producer streams, WebSocket/SSE and UI runtime integration remain
separate. Preserve explicit ownership, cooperative cancellation and cleanup limits.


## S16 TLS completion — 2026-10-15

S16 is accepted: `app.listen(address, { cert, key })`, `new Client({ ca })`,
mandatory certificate/hostname/time verification and shared HTTP/TLS cleanup.
Zero-argument Client and cleartext listeners retain their behavior. Source
checkpoints are `b0656298` and `7510b254`; see [TLS acceptance](../plans/tls-acceptance.md).
All 59 affected compiler/runtime/export tests and 15 editor tests pass. HTTPS
measures 5.98 ms native versus 6.78 ms Node per 128 requests over HTTP/2 reuse.
Installed CLI/LSP consumers are refreshed; VS Code shows current TLS help and no
problems. SQLite holds original logs, failed attempts, hashes and all 186 timing
samples. No full unrelated regression matrix was repeated after S15.

The B4 repair agent has the exclusive validation window for
`/private/tmp/tsrust-gate100-editor-target`: `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=1`, Node v24.11.1 first on PATH, loopback escalation when needed.
TLS has no remaining builds, timings or consumer refreshes. Preserve the active
shared target until B4 finishes. The UI briefs remain owner work. S17 SQLite is
the next standard-library roadmap item; no work on it started in this pass.


## S17 SQLite and B4 workspace repair — 2026-10-15

S17 is accepted at `91e6b937`, with indexed editor completion repaired at
`b255009e`. B4 is merged at `e33af7db`; its missing callable access-summary entry
caused the workspace panic visible even when an empty/comment-only file was open.
The full workspace now checks without panic or errors. The remaining `unused-task`
warning on an awaited greeting is a pre-existing false positive, verified with the
old installed compiler; do not describe it as intentionally discarded work.

See [SQLite acceptance](../plans/sqlite-acceptance.md) for exact checks, consumer
hashes and measurement provenance. New examples are `examples/sqlite/people.tsr`
and `examples/sqlite/rollback.tsr`. Compiler ownership rejects closing a borrowed
connection, committing with live transaction statements, handle cloning and reuse.
Runtime checks cover scope/error rollback, failed commit, SQLite automatic rollback,
strict data conversion and lock cleanup. Borrow provenance through `unwrap` is fixed
for existing byte views too.

All affected compiler/runtime/export checks and 18 editor checks pass. Installed
CLI and LSP binaries are refreshed; the active VS Code server was restarted and
current SQLite hover verified. Matched prepared transactions measure 0.031084 ms
native versus 0.105708 ms Node per 128 rows; versions differ, this is an in-memory
whole-stack baseline. SQLite retains exact evidence and all 186 timing samples.

UI identity feasibility now has a separate managed worktree and the quiet
check/emit/rustc/format window. It uses the verified `b255009e` release CLI; any
compiler dependency requires a minimized witness and coordinated ownership first.
Preserve the five untracked owner UI plan files. S18 async SQL remains pending;
this pass adds no ORM, database pool, migrations or async SQLite adapter.
