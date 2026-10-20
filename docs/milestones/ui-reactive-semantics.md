# Reactive accessibility and component forwarding

Implementation is `984a3617`, static-size compatibility is `dd90289f`, captured
union emission is `67d6cca9`, and retained browser synchronization is `72d91de7`.
The installed CLI and active LSP include the final repair. Focused paired acceptance passes. Canonical evidence closure is recorded below.

## Behavior

`aria-label`, `aria-describedby`, `aria-hidden` and `aria-invalid` accept declared
live scalar bindings, including undefined. Local components and replacement
renderers can forward them without copying their evaluators. Each affected node
owns one metadata evaluator; state captures remain weak. Constants retain their
static path. Updates preserve the field value, node identity and component state.

SVG's automatic decorative/image role and hidden state follow its current label.
Explicit overrides remain checked. Size metadata stays static so existing Card
wrappers can continue cloning it. Identity, tags, explicit roles and slots remain
construction-time props. Authors transforming a live accessibility prop use
useProp rather than cloning a callable environment.

Metadata updates pass the existing naming, role and structure checks. Invalid
intrinsic metadata leaves the prior value intact. Callback failures propagate;
mounted trees use the existing error cleanup. Hidden ancestry is checked against
the final refreshed subtree, allowing a handler to disable controls and hide their
ancestor together. Hiding alone does not disable a control.

Button version 3 fixes dropped metadata and replacement props. Offline installation
in the docs project preserves the earlier Button byte-for-byte and creates
`button-v3.tsrx`. Existing imports remain unchanged; the new example and current
API reference explicitly select the new file. The lock records compiler API 8 and
the actual installer provenance. This is a forwarding repair, not full Nova Button
parity or event/ref merging.

## Focused evidence

- Four shared model tests and native accessibility pass. They cover names,
  description relationships, invalid state, retained input identity/value,
  unchanged variants, zero-change refreshes, bad metadata, expired state, hidden
  controls, callback errors and independent evaluator release.
- Formatted generated-native gallery and exact documented example execute in one
  program. The separately installed Button fixture retains/releases its owned
  metadata capture once and rejects a wrong getter result.
- A non-UI generated-native regression proves repeated mutable access through an
  owned captured union, absent/string alternatives and independent resource
  release. The emitter reborrows callback storage; ownership rules are unchanged.
- Existing string-prop wrappers, current Card source/native execution, offline
  installation, one-current-version catalog and compiler API compatibility pass.
- The two metadata completion rows pass four typing modes. Five affected receiver
  agreement rows, hover/diagnostics and zero consistency counters pass. The final
  installed-LSP witness passes; settings bytes/time are exact and global/editor
  server images match. Whole-repository cold scheduling remains open.
- Content generation reports 165 pages, 26 checked examples and 971 declarations.
  The complete updated documentation app checks without diagnostics.
- The actual dev browser matches four independent Node-after-tsc transitions.
  Names, descriptions, hidden decoration, nullable invalid attributes and SVG
  image/presentation defaults update and clear on retained DOM. Input draft,
  selection, focus and node identity survive; replacement/local-component
  forwarding, accessible descriptions, 390/768/1280px bounds, theme change and
  disposal pass. The final screenshot was inspected. The dev build reports
  111606 ms; build latency remains unresolved.
- The actual native route builds and remains alive/quiet for five seconds before
  termination and reaping. The subsequent browser-only synchronization repair
  does not change native behavior. This is bounded startup, not OS input evidence.

Original failed records remain part of the evidence: narrowed mutable forwarding,
captured-union Rust reborrowing, an outdated fixture stdout assertion, a missing
console declaration in the independent TypeScript reference and retained browser
attributes. Repairs rerun affected checks; accepted evidence is reused. No full
compiler/LSP regression, SQL suite or timed benchmark is implied.

## Continuation

Scoped context, event/ref composition, focus and expanded/controls relationships
still precede dependent compound controls. Continue remaining components, charts,
themes and blocks. Empty's pinned Base UI/Nova source and styles have been read;
tracking, relaxed line height, balanced wrapping, dashed borders and descendant
icon sizing need explicit paired support rather than silently ignored utilities.
Build latency, HTML-first loading, executable splitting and cold editor scheduling
remain open.

SQLite independently verifies 59 closed records (345821 original bytes), plus the
closure receipt. Exact decompression, bytes, sizes, SHA-256, integrity and foreign
keys pass. This includes original failures, the inspected screenshot, installer
and consumer receipts, closed preview logs, temporary source probes and pinned
upstream reference excerpts. Cleanup removes 102154245 bytes of closed native and
section-test outputs. Five current executable consumers, warm shared caches,
owner plans and the actual dev watcher remain. Only its current log stays loose.
