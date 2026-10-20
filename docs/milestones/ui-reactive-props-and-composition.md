# Reactive custom props and replacement composition

The scoped section passes focused compiler/runtime, installed-editor, optimized
Wasm, live dev-guide and native application startup acceptance.
This section does not complete the component-library inventory.

## Delivered contract

`ReactiveValue<T>` accepts a scalar or an owned checked getter. Component authors
adapt it with a direct `const value = useProp<T>(...)`; callers use ordinary JSX
expressions. `computed<T>(...)` defines a derived getter. Both expose `get()` and
reevaluate on each read, without memoization or component reruns. Their weak
bindings preserve node identity and release evaluators with the owning tree.

Supported values include exact ints, numbers, booleans, strings, enums, literal
unions and nullable scalars. Records, containers, resource results, setter calls
and resource cloning are rejected. Consumed optional props must be restored to
`undefined` before forwarding their whole record. Ordinary spread/rest ownership
rules remain unchanged.

Intrinsic `className` and `data-variant` bindings validate updates before replacing
valid state. Existing component prop aliases retain string defaults. Editable
components opt into live styling explicitly; an SDK update does not silently turn
previously installed string props into reactive descriptors.

`View` and `Button` accept one replacement factory through `render`. It receives
owned props once, including children and handlers, with no recursive render field.
Returning its node adds no wrapper or focus stop. Replacements forward the
behaviors they need. This does not supply arbitrary event/ref merging, context,
focus scopes or the remaining Base UI composition contracts.

## Editable Alert

Alert version 2 uses the same public authoring contract for its variant and all
four subcomponents' caller classes. Its replacement receives the computed styles
and metadata. Compiler API 4 prevents an older SDK from installing unsupported
source. The previous immutable payload remains intact.

An actual offline installation in `docs/` preserves `alert.tsrx` and configuration
byte-for-byte, adds `alert-v2.tsrx`, and retains both lock records. Existing
application imports remain under application control. The live example imports
version 2 explicitly; previously accepted examples retain their version 1 imports.
SVG icons, arbitrary descendant CSS, text balancing and full Base UI event/ref
composition remain required work.

## Focused evidence

- Compiler/native witnesses cover exact and nullable values, scalar-union results,
  source evaluation order, replacement elements, forwarded callbacks/children,
  absent owning callback environments, invalid contracts and old string props.
- Runtime witnesses independently observe live classes/variant metadata, node and
  handler identity, invalid updates, recursive reads, expired weak bindings and
  evaluator release. Existing fallible-record and borrowed-callback rejection
  witnesses continue to pass.
- Editor witnesses cover computed receivers, precise result types, completion,
  hover, signatures, local Alert props and replacement help. Declaration text
  removes export markers; public alias metadata now comes from original source.
  Incomplete, header and checked completion all retain the public render-prop
  identity. The displayed callback contracts compile and match checked contracts.
- Optimized Wasm matches all five independent TypeScript transitions. Rendered
  colors, group selectors and title weights change and restore; empty variant
  values remove their DOM attribute. Stable identities, focus, disabled activation,
  three viewport widths, accessibility and disposal pass.
- API extraction preserves inherited generic fields and documentation even when
  unused trusted facades have no concrete specialization. The documentation
  reference inventory and executable catalog examples pass.

Browser checks exposed stale class/variant DOM attributes; synchronization now
updates only nodes with those bindings and avoids unchanged class writes. The enum
probe exposed ambiguous Rust method resolution for string-enum ordering; explicit
trait dispatch repairs it, and all five affected enum tests pass.

The final CLI is built from `274a014e` and installed atomically. Installed-LSP
protocol checks pass from `a485dca3`; the two backend repairs do not change its
public/editor contract. The active editor loads that binary, with settings bytes
and modification time restored exactly. The real dev guide has 162 destinations,
20 checked examples and 961 API declarations. Those counts do not claim library
completion. The formatted native guide starts quietly for five seconds and is
terminated/reaped; this is not an OS gesture or normal-close release claim.

Only the requested dev watcher and editor remain. Test browsers/native processes
are reaped; approximately 1.45 GB of disposable build/test/site outputs are pruned.
Warm dependency and runtime caches are retained. Build latency and cold editor
scheduling remain unresolved; no startup improvement is claimed.

The TypeScript transition oracle and paired UI observers are untimed. No new
performance measurement or full-family acceptance is claimed. Existing SQL,
resource, reading, scrolling, theme and window evidence is reused; unrelated
regression matrices and database suites are not repeated.

## Durable evidence

SQLite retains 138 original records totaling 1,986,734 bytes.
Original logs, failed attempts, source probes, consumer hashes, exact settings
restoration, cleanup and the native entry are verified in canonical SQLite under
`tsrust-reactive-props-*` and `docs/target/reactive-props-native.tsr`. Closed loose
copies are removed after independent byte/size/SHA-256 checks and database
integrity/foreign-key checks. Only the concise active preview log remains loose.
