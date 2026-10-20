# Shared vector rendering

Continue after accepted Badge checkpoint `35689f60`. Component icons and charts
need one checked vector model rendered by browser SVG and native Vello. Reuse the
installed geometry dependency and existing ownership/binding machinery. No browser
JavaScript bridge or image registry.

## Scoped contract

- Public Svg and Path primitives, standard viewBox/path data, fill/stroke/cap/join
  controls, explicit accessible/decorative images and normal project utilities.
- Preserve path source. Reject malformed/nonfinite geometry and unsupported paint
  values. Normalize viewport transforms consistently; clipping and aspect policies
  must be explicit. Do not claim exact GPU/font pixels across platforms.
- Owned Path children belong to Svg. Reject invalid parent/content combinations;
  no HTML fallback. Ordinary reactive d expressions update retained path identity
  through weak scalar bindings, with independent cleanup observation.
- Browser namespace/attributes, native layout/paint/accessibility, compiler/package
  integration, complete LSP contracts, editable icon/chart examples and independent
  geometry/transition witnesses land before focused validation.

Strict parsing uses pinned svgtypes 0.16.1, which reuses Kurbo 0.13.1.
Kurbo alone accepts invalid trailing input. Keep the original failed witnesses.
Native arc tolerance requires nonzero radii above 0.00001 units; auto/fit sizing
and the defer aspect flag remain explicitly unsupported.

## Acceptance

- [ ] Runtime model, validation, ownership and reactive path updates.
- [ ] Paired browser/native painting, layout, clipping and accessibility.
- [ ] Compiler/runtime packaging, public API and editor agreement/help.
- [ ] Shared docs/examples and independent untimed Node reference support.
- [ ] Batch affected acceptance, refresh consumers/preview, SQLite and cleanup.

Gradients, filters, masks, arbitrary DOM/CSS and interaction policy are outside this
first vector section. Required context, event/ref composition and focus/overlays
remain separate foundations; finishing vectors does not complete those contracts.
