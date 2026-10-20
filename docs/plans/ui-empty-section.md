# Empty component section

After disclosure semantics, port all six Empty exports from the pinned Base UI +
Nova baseline in `ui-component-coverage.json`. Reuse accepted View replacement,
ReactiveValue/useProp, live metadata, SVG and offline installer contracts.

1. Implement the shared utilities required by the exact source: dashed borders,
   tight tracking, scoped generic heading fonts, nonnegative margins, relaxed line height and `text-sm/relaxed`, `max-w-sm`, balanced
   text, descendant SVG pointer/shrink rules and default icon sizing that respects
   an explicit icon size. Preserve independent border widths and theme scaling.
   Native shaping, layout measurement, painting and selection must use the same
   text settings. Do not emit CSS-only utilities as paired support.
2. Add editable Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription and
   EmptyContent. Keep the actual `empty-icon` slot, default/icon variants, direct
   Link decoration and caller overrides. Preserve ownership and once-only render
   replacement. Keep application event handlers in application source.
3. Register one bundled current version and shared attribution. Install into the
   documentation app through the offline CLI; preserve old files/imports. Add API
   extraction, variants, composition, theme and constrained-layout examples.
4. Complete hover/completion/signature documentation for new utilities and props,
   formatting witnesses, native geometry/semantics and independent browser checks.
   Batch checks after the whole section is implemented; reuse accepted mechanisms.
5. Refresh consumers/preview only for changed executable code. Import and verify
   closed evidence in SQLite, prune temporary outputs and checkpoint acceptance.

Pinned references already preserved in SQLite:
`tsrust-empty-upstream.tsx`, `tsrust-empty-nova-reference.json` and
`tsrust-empty-style-paths.json`. Scoped context, event/ref composition and focus
scopes still precede dependent compound controls; Empty does not require them.


Implementation checkpoint scope: text slots intentionally use the Text contract,
including inline Link flow, rather than pretending View children have inline
layout. They do not expose replacement callbacks. Container replacement remains
once-only. SVG selectors accept pointer/shrink/numeric sizing; direct-Link selectors
accept decoration/text colors. Other descendant selectors reject. Browser descendant
sizes use the target's nearest theme variables, matching native scope resolution.
