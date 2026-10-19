# UI documentation foundation — 2026-10-19

Accepted implementation: `0948dca6`, following source checkpoint `4710cde2`.
This is a focused application-foundation section, not completion of the component
library or documentation application plan.

## Delivered behavior

- `docs/` is a standalone project scaffolded through the ordinary offline UI
  installer. Both targets are explicit; theme tokens, editable local components,
  locks and shared attributions are present. The bootstrap is `src/main.tsr`.
- Bounded `overflow-auto` panes and viewport dimension utilities work in native
  layout and browser CSS. Native wheel deltas bubble from inner panes, while
  layout, rounded clipping, pointer hits, editor origins and accessibility use
  the same offsets. Focus reveals descendants through nested clips.
- `View.tabIndex` accepts 0 or -1, with source-order Tab traversal, pointer/
  accessibility focus and pane keyboard scrolling. Invalid ranges and interactive
  ancestry reject. Compiler props and actual LSP hover remain consistent.
- Hidden panes retain model state, DOM identity and scroll offsets. Removed
  subtrees retire their scroll records. Browser acceptance exposed restoration
  before sizing CSS, which clamped offsets to zero; CSS now returns first.
- The shared initial shell includes live neutral light/dark themes, retained
  counter/draft state, a custom macOS header and independent sidebar/article
  panes. It labels the remaining documentation work explicitly.

## Focused evidence

Canonical records are in `benchmarks/records.sqlite`, under
`tsrust-docs-scroll-` and `tsrust-docs-preview/` paths. Original failed records are
retained alongside successful repairs. Thirty-eight records (1,953,274 original
bytes) are imported and independently restored exactly; SQLite integrity is `ok`.
Source/consumer receipts include exact identities. Verified loose copies and the
superseded browser output are removed; active preview assets remain runtime
consumers.

Six native scrolling/editing cases pass across the original batch and one affected
repair. Compiler acceptance checks formatted project source, generated execution,
emission and rejected prop types. One actual-server LSP case checks prop/utility/
window help and diagnostics. The real browser observer checks retained DOM,
focus, draft, counter and offsets; wheel and keyboard scrolling; viewport/narrow
layout; region, Kbd and Separator semantics; and disposal. Counter/draft assertions
reuse the independent Node-after-tsc application oracle.

A generated macOS docs window initializes, stays alive for five seconds and is
reaped with its dev supervisor. It uses the existing shared counter dependency
workspace. Launch observations are untimed acceptance, not performance results.
Five executable consumers are refreshed; both editor settings files are restored
exactly. Existing acceptance at `c8699512` is reused. No full compiler/LSP,
SQL, timing or benchmark matrix is repeated.

## Running preview and remaining work

The owner requested a persistent local preview. One Python server remains on
`http://127.0.0.1:4317/`, serving verified browser output from
`/private/tmp/tsrust-docs-preview`. Refresh to see published section updates.
This is an intentional runtime consumer, not an orphan build/watch process.
Temporary acceptance browsers, servers, native programs and supervisors are reaped.

Persistent routes, keyed lists, generic reactive props, live token-record edits,
source previews, Markdown/code rendering, offline search and generated API/evidence
pages remain required. The sidebar currently describes available controls; it is
not implemented routing. Kbd/Separator base semantics are observed in this shell;
complete contextual/variant browser acceptance remains separate. Actual OS drag
gestures, live chrome changes and custom chrome on other desktop platforms remain
unaccepted. Continue the active plan without another permission checkpoint.
