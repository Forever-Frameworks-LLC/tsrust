# Lazy child composition

Implementation checkpoints are `71f6e6d6` and `2ec53b01`; corrected ownership
witnesses and documentation are `482e86d5`. The installed compiler and LSP include
the implementation. Focused paired acceptance passes; canonical evidence closure is recorded below.

## Behavior

`LazyChildren` is `Owned<Function<Child[]>>`. A declared function component whose
children field accepts an owned zero-argument array factory receives nested JSX
as that factory. Inherited and concrete generic props work. Attributes evaluate
before the parent call; descendants construct only when called, in source order.
Each invocation constructs fresh content. Ignoring or dropping the factory does
not construct descendants. Existing eager arrays and explicit factories retain
their behavior. No memoization, public owner handle or extra node is introduced.

Single-use factories use `Once<Function<Child[]>>`, preserving the existing single
ownership-wrapper rule. Reusable callbacks cannot consume captured nodes. Existing
consuming array conversions remain supported and transfer ownership. Contextual
array results now support empty, mixed-union and child literals in explicit arrow
factories. Stored or indirect components require explicit factory props; automatic
between-tag lifting currently applies to declared top-level functions.

## Focused evidence

- Formatted generated-native assertions cover deferred/eager evaluation order,
  ignored and repeated factories, single-use captured nodes and arrays, independent
  resource release and partial construction failure cleanup. Rejection witnesses
  cover reused resources, repeated once calls, use after transfer, wrong props and
  simultaneous explicit/nested children. The documented example executes.
- Independent Node-after-tsc output matches the native construction trace. These
  are untimed correctness checks, not throughput measurements.
- The three completion rows pass all four typing modes. Receiver agreement,
  invocation help, source documentation, inherited prop hover, current diagnostics
  and zero consistency counters pass. The installed-LSP witness also passes.
- Content generation reports 164 destinations, 25 checked examples and 971 API
  declarations. The updated docs application checks without diagnostics.
- The actual port-4317 dev gallery passes eight independent state transitions.
  Parent draft/count survive closing; reopened child state starts fresh; unrelated
  parent updates retain the child DOM. Theme changes, accessibility, 390/768/1280px
  bounds and root disposal pass. The screenshot was inspected.
- The actual native documentation app and a disposable entry for this route build.
  The route remains alive and quiet for five seconds, then terminates and is reaped.
- CLI and both LSP consumers were atomically refreshed. Global/editor LSP images
  match. The active server restarted; settings bytes and modification time are
  exact. The dev build reports 120977 ms; build latency remains unresolved.

The first fixture invocation hit a filesystem sandbox restriction and is retained
with its permitted successful rerun. Original incorrect array-rejection and missing
editor-import assumptions are retained alongside corrected checks. No full
compiler/LSP regression, SQL suite or timed benchmark is implied.

## Remaining work

This is a child-construction contract, not a completed disclosure component.
Context, event/ref composition, focus, expanded/controls relationships and overlays
still precede dependent Base UI/Nova parity. Continue the remaining component,
chart, theme and block inventory without another owner approval. Native startup
does not prove OS interaction or cross-platform GPU pixel equivalence.

SQLite independently verifies 39 closed records containing 245858 original bytes,
plus the closure receipt. Exact decompression, bytes, sizes, SHA-256, integrity and
foreign keys pass. Original failures, the browser screenshot, consumer receipt and
closed vector-preview log are retained. Disposable native route outputs remove
44692653 bytes. The current actual dev watcher and warm shared caches remain.
