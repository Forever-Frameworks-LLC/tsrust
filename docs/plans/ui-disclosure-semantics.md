# Disclosure semantics and instance IDs

Continue after accepted reactive-accessibility checkpoint `f20e0aed`.

- Add live, nullable Button `aria-expanded` and `aria-controls`. Keep ordinary
  button keyboard activation and checked replacement forwarding. These props
  describe application state; they do not create a disclosure controller.
- Pair retained browser attributes with native expanded state, current controlled
  targets and guarded Expand/Collapse actions. Disabled/hidden controls do not
  activate; repeated requests for the current state do not toggle it back.
- Add `useId()` as an owned string allocator with checked exhaustion. Calls are
  unique within one runtime, including native threads. Keep the returned value in
  component construction; it remains stable while that component is retained.
- Use two independent retained disclosure examples with generated IDs, keyboard
  activation, disabled state, independent drafts and cleanup. Keep open/close logic
  in the .tsrx application. This does not claim Base UI Collapsible parity.
- Finish formatting, native models/accessibility, generated execution, editor
  signatures/help, independent browser reference and consumer/SQLite acceptance
  as one section. Reuse prior metadata, ownership, SQL and resource evidence.

Scoped context, event/ref merging, focus restoration, transitions and the full
Collapsible component remain separate work. Do not silently add partial widgets
to the installable inventory.
