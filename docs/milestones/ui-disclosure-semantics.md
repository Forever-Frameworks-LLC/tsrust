# Disclosure semantics and instance IDs

Implementation is `d0712025`; argument/signature witnesses are `48988790`.
Focused paired acceptance passes. The remaining source changes repair test-only
assertions and the independent browser observer, without changing runtime behavior.

## Behavior

Button `aria-expanded` and `aria-controls` accept constants or live nullable values.
Undefined removes explicit state/relationships. ID lists normalize ASCII whitespace;
other control characters reject. Invalid updates preserve prior metadata. These
props describe application state and do not create visibility or activation logic.
Links and non-button nodes reject disclosure metadata.

Browser creation and retained updates share the same attribute handling. Native
accessibility exposes expansion state, currently accessible controlled targets and
the applicable Expand/Collapse action. The action checks the current model before
activation, so repeated requests for an already reached state do not toggle it
back. Disabled and hidden buttons cannot activate.

`useId()` reserves an owned runtime-local string with checked exhaustion. Native
threads share one atomic counter; separate processes or Wasm modules do not.
Retaining the string during component construction preserves the ID across updates.
This is not a persistent key, secret or externally portable identity.

## Focused acceptance

- Three shared/native disclosure tests pass: validation and prior-value preservation,
  normalized/missing targets, expansion states/actions, disabled/hidden guards,
  repeated-request guards and retained drafts.
- Two ID tests pass: exact allocation, boundary exhaustion without wrapping and
  128 unique allocations across four native threads.
- Formatted generated-native execution checks the actual two-panel gallery and
  the exact documented useId example, including a program without UI nodes.
  Wrong prop types and extra useId arguments reject in `.tsrx` source.
- Two completion rows pass all four typing modes. ButtonProps and ButtonRenderProps
  agree with the compiler. Hover, zero-argument signature help, diagnostics and
  consistency counters pass against the installed server.
- The complete docs app checks cleanly. Content generation reports 166 pages,
  27 checked examples and 972 declarations.
- The actual dev browser matches six independent Node-after-tsc transitions.
  Enter and Space activation, independent expansion, stable unique IDs, current
  controls relationships, retained input nodes/drafts, disabled controls, accessible
  expansion state, 390/768/1280px bounds, theme changes and disposal pass. The
  screenshot was inspected. Hidden DOM descendants can detach individually; the
  witness checks their identity after reopening rather than requiring hidden
  parent/child attachment.
- Existing native window/input/lifecycle startup evidence from `f20e0aed` is reused.
  New native scene/action checks cover this section's changed behavior directly.
  No additional native window build or OS input claim is implied.

The original sandbox-cache failure, Rust test assertion lifetime failure and
browser observer failures remain in SQLite. CDP keyboard events now include their
text payload and explicit focus assertions. Only affected checks were rerun.
No full regression, SQL suite or timed performance pass was run.

## Consumers and evidence

The CLI and both LSP images include `d0712025` (compiler API 9). Global/editor LSP
SHA-256 is `2fdcdfcefa6650b69c761d82119d5fb08b549662bee9991313fa31567d09f0b1`.
Settings bytes and modification time were restored exactly. The actual dev preview
runs on port 4317 at `/docs/ui/foundations/disclosure-semantics`; its build reports
117553 ms. This is a diagnostic duration, not a benchmark; latency remains open.
Cold editor scheduling remains open as well.

SQLite independently verifies 29 closed records, 209284 original bytes, plus its
closure receipt. Exact decompression, byte equality, sizes, hashes, integrity and
foreign keys pass. Cleanup removes 10489378 bytes of section executables and an
unused native-window probe. Only the current preview log remains active.

## Continuation

Continue the six-export Empty component with its required paired typography,
border and descendant-icon utilities. Scoped context, event/ref composition,
focus scopes, transitions and full Collapsible remain separate required work.
The two panels are application composition examples, not a completed compound
widget. Continue the remaining component/chart/theme/block inventory without
another owner go-ahead.
