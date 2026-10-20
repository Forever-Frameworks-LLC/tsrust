# Reactive semantic properties

Continue after accepted lazy-child checkpoint `703d4fb6`. Keep static metadata
cheap and update retained native/browser controls without replacing identity.

## Contract

- Support live `aria-label`, `aria-describedby`, `aria-hidden`, `aria-invalid`
  and `data-size` using the existing declared ReactiveValue contract.
- Keep identity, element tags, roles and slots construction-time values. SVG's
  existing automatic decorative/image policy follows its current label unless
  explicitly overridden. No arbitrary ARIA is accepted.
- Evaluate one metadata binding per affected node. Preserve static fast paths,
  weak state capture, validation, callback errors, cleanup and live variants.
- Refresh native accessible names/descriptions/validation and browser attributes
  together with styles. Preserve field drafts, focus and node identity.
- Include forwarding/replacement props, absent values, invalid updates, hidden
  control rejection, native observers, actual gallery and editor documentation.

## Acceptance

- [ ] Shared runtime, compiler facade and formatted ownership/error witnesses.
- [ ] Local component forwarding, paired gallery and independent reference.
- [ ] Completion, hover, parameter help, diagnostics and documented examples.
- [ ] Batched targeted checks, installed consumers, preview and SQLite closure.

Expanded/controls relationships, event/ref merging, context and focus scopes remain
separate contracts. This section does not label pending compound controls complete.
