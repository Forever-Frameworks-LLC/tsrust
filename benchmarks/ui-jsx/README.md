# JSX syntax witness

Run `TSRUST=/path/to/tsrust node benchmarks/ui-jsx/verify.mjs`. The runner compiles
Tsrust JSX and TypeScript JSX with an explicit local factory, then checks both
outputs against a fixed independent string oracle. It records exact source hashes
and removes temporary binaries. This is untimed syntax/lowering evidence, not UI
rendering, browser interactivity or a throughput measurement.

`verify-counter.mjs <built-counter-site> --presentation-only` checks centered
button text, rounded styling and stable font/height metrics across viewport widths
in real Chromium. It skips the previously accepted input/lifetime interaction pass.
The regular invocation includes both presentation and interaction checks. Build
fresh artifacts with `tools/build-ui-web.mjs`; do not relabel an older artifact as
current. These checks report observations and source identities, not throughput.

`verify-semantics.mjs <built-site>` checks the semantic UI example in real Chromium,
including headings, named regions, live roles, separator orientation, hidden
content, disabled accessible names and disposal. Its independent TypeScript oracle
runs after tsc. The native renderer has matching AccessKit assertions. These are
untimed checks; no component-family timing result is implied.

`verify-composition.mjs <built-site>` checks composed content, integer bindings,
optional handlers, a reactive disabled control without a handler, actual listener
ownership, accessible names, alignment and disposal against composition-reference.ts.
It records untimed observations and source identities; it does not claim timing.

`verify-layout.mjs <built-site>` checks the shared grid example against an independent
Node-after-tsc geometry oracle at three viewport widths. Native Taffy checks cover
the same custom spacing, grid placement and spans, plus preferred/min/max sizing,
percentage widths and per-side padding precedence. These checks are untimed.

`verify-colors.mjs <built-site>` checks alpha hex, OKLCH conversion, alpha modifiers,
semantic aliases, border/ring colors, ring scope reset, hover and keyboard focus.
Its independent double-precision Node-after-tsc oracle uses CSS Color 4 matrices;
native scene checks separately verify the premultiplied RGBA8 encoding boundary.
Source theme strings remain unchanged. These are untimed color/semantics checks,
not a GPU screenshot or a claim of identical rasterization between backends.

`verify-forwarding.mjs <built-site>` compares the owned-prop/metadata example with
`forwarding-reference.ts` after tsc. It checks small/default scopes, descendant and
own-class conditions, live DOM metadata, accessible content and disposal. This is
an untimed prerequisite witness, not acceptance or timing for a Nova component.

`verify-card.mjs <built-site>` compares all seven editable content parts against
`card-reference.ts` after tsc. It checks default/small and custom numeric spacing,
caller overrides, edge/corner styling, header action layout, typography, rounded
pointer clipping, accessibility and disposal. Image edge rules, inline-size
containment and named font assets remain unsupported; this does not claim full
Nova Card parity or timing.

`verify-state.mjs <built-state-site>` checks the paired `examples/ui-state`
component against `state-reference.ts` compiled with TypeScript. It covers all
four scalar payloads, exact integers above 2^53, independent instances, text and
disabled updates, keyboard activation, accessible controls and disposal. The
native inspector additionally asserts string snapshots and IEEE-754 storage.
This is untimed acceptance; it is not a throughput measurement.

Editable text controls: build `examples/ui-inputs/src/main.tsr` for web and run
`node benchmarks/ui-jsx/verify-inputs.mjs <site>`. The untimed harness compares
committed application transitions with `input-reference.ts` after TypeScript
compilation, using a real browser for typing, selection preservation, composition,
controlled/uncontrolled values, disabled/read-only state, labels, descriptions,
required/invalid state, responsive Nova styling and disposal. It records exact
source hashes and closes its server/browser. Native model and layout witnesses
cover Parley selection, IME cancellation, undo/redo, scrolling and AccessKit state.
These checks do not measure performance or accept specialized input types.

Build the same project's `src/failures.tsrx` as a second entry and run
`verify-input-failures.mjs <site>` to check throwing derived getters and change
handlers. Each failed mount reports its own error, releases the owned tree and
ignores later events; explicit disposal removes the remaining host elements.

`verify-nullable.mjs <built-site>` checks `examples/ui-nullable-children` against
`nullable-reference.ts` after TypeScript compilation. Null and undefined produce
no placeholder elements, adjacent text stays joined, sibling order and accessible
button names agree, and two present nullable components retain independent live
state. Disposal removes every mounted node. The native inspector separately
observes resource release for present, null and undefined results. This is untimed
initial-branch composition; dynamic branch replacement is not claimed.


`verify-computed.mjs <site>` compares computed scalar text in Text/View/Button
with `computed-reference.ts` after tsc. It covers arithmetic, imported helpers,
string operations, nullable text, exact numeric literals, stable node identities,
accessible names and disposal. Build `examples/ui-computed-text/src/failures.tsrx`
and use `verify-computed-failures.mjs <site>` for readable getter failure and
independent-mount cleanup. These are untimed observers, not throughput claims.


`verify-conditional.mjs <site>` compares nested lazy mounting with
`conditional-reference.ts` after tsc. It checks selected-branch state, fresh state
after removal, sibling/control identities, focus/selection/IME, transparent row
layout, retired events, new-node themes, accessibility and disposal. Build
`examples/ui-conditional-mounting/src/failures.tsrx` and run
`verify-conditional-failures.mjs <site>` for lazy-construction errors and independent
mount cleanup. Both observers are untimed and close their isolated processes.

Semantic content: build `examples/ui-content/src/acceptance.tsr` with the bounded
Wasm builder, then run `node benchmarks/ui-jsx/verify-content.mjs <site>`. The
observer checks two independent light/dark/scaled mounts against
`content-reference.ts` after tsc, including keycap geometry, separators, theme
colors, accessibility, live child bindings, DOM identity and disposal. This is
untimed acceptance support; no performance measurement is claimed.

## Application modes and visibility

`application-reference.ts` is the independent Node-after-tsc state-transition
oracle for `examples/ui-application-state`. Build that shared entry with
`tools/build-ui-web.mjs`, then run `verify-application.mjs <site>` for untimed
browser acceptance. The observer checks palette changes, nested scopes, retained
field DOM/focus, hidden preview state/accessibility, responsive header, read-only
source and disposal. It records exact source hashes. Native model/layout/header
hit witnesses cover the same section in `tsrust-ui`. No timing is claimed.

## Documentation Alert and positioning

Run the docs project's actual `tsrust dev --target web --port 4317` preview, then
create an empty output directory and run:

```sh
node benchmarks/ui-jsx/verify-application.mjs /tmp/alert-observation docs-alert http://127.0.0.1:4317/
```

This untimed observer checks the five editable Alert examples, retained state and
focus, independent theme modes, role exposure, text contrast, narrow-width bounds
and disposal. `alert-reference.ts` supplies independent CSS box-model equations
after TypeScript compilation. The native `positioning_tests` print the same four
observed boxes across flex/grid and three display scales. Compare their captured
log with `node benchmarks/ui-jsx/verify-positioning-native.mjs <native-log>`.
Neither witness establishes GPU pixel equivalence or actual OS gestures.

`verify-application.mjs <site> reactive-props` checks the paired
`examples/ui-reactive-props` project against `reactive-props-reference.ts` after
TypeScript compilation. It observes live literal props, classes, variant metadata,
derived text, forwarded handlers/children, stable nodes/focus and disposal. Native
model tests cover matching transitions, validation errors and independent release
observation. These are untimed correctness witnesses; no new throughput result is
claimed.

`verify-application.mjs <output> docs-badge http://127.0.0.1:4317/docs/ui/components/badge`
checks the six Badge palettes, live/nullable variants, caller padding, replacement
Link/Button semantics, node retention, narrow layouts and disposal against
`badge-reference.ts` after TypeScript compilation. Native flow witnesses cover
Unicode selection, inline link ranges, inherited whitespace, editable values and
local/ancestor decoration boundaries. These are untimed correctness checks.

The vector section uses `vector-reference.ts` for independent viewport mapping and
six-value chart paths. `verify-application.mjs <output> docs-vectors
http://127.0.0.1:4317/docs/ui/foundations/vector-graphics` checks the actual guide's
SVG namespaces, retained paths, data transitions, accessible images/buttons,
responsive bounds, theme stroke and cleanup. Run `verify-vector-native.mjs
<native-vector-log>` against the native viewport witness to compare its actual
corners with the TypeScript-after-tsc oracle. No timing is claimed.

The lazy-child section uses `lazy-children-reference.ts` for independent constructor
order and eight interactive states. `verify-application.mjs <output>
docs-lazy-children http://127.0.0.1:4317/docs/ui/foundations/lazy-children` observes
absent initial content, repeated fresh child state, parent DOM/draft retention,
responsive layout, accessibility, theme changes and disposal. It records source
hashes. `verify-lazy-native.mjs <compiler-log>` compares actual generated-native
construction order with the compiled TypeScript reference. Native std:testing
resource assertions independently verify unused, consumed and failed factories.
All these checks are untimed; no JavaScript GC cleanup parity is implied.

The `docs-reactive-semantics` scenario uses the independent Node-after-tsc
transition model and actual dev gallery. It checks live names, descriptions,
nullable invalid state, hidden decoration and SVG default roles,
with retained field identity/draft/selection/focus, replacement and local-component
forwarding, accessibility, three viewport widths, theme changes and disposal.
Run `node benchmarks/ui-jsx/verify-application.mjs <output> docs-reactive-semantics
http://127.0.0.1:4317/docs/ui/foundations/reactive-semantics` after the completed
section builds. This is untimed correctness evidence, not benchmark timing.
