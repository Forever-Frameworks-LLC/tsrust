# Continuation acceptance checklist

Baseline: `7c956bd7`, inspected on Windows on 2026-10-26. This checklist reconciles
the latest handoffs with the governing plans. It does not replace their detailed
contracts or turn historical acceptance into Windows acceptance.

## Required sections

- [ ] Resolve the reported long-lived LSP memory pressure. Inspect retained inputs,
  invalidation, interrupted scans and foreground/background lifetimes; reproduce
  under explicit limits and verify any repair without reducing coverage.
- [ ] Finish Table acceptance from `1085b9c8`: three exact compiler witnesses,
  two exact editor witnesses, real offline installation/lock registration, generated
  content, paired browser/native behavior and complete public documentation.
- [ ] Complete documentation plan sections 1–6: content classification and versioned
  coverage, live themes/reactive composition/native headers, navigation/shell,
  Markdown/source rendering, offline search and complete component-page contracts.
  Reuse accepted records; unchecked plan items require reconciliation, not blind reruns.
- [ ] Complete sections 7–8: all pinned static/structured families and reusable
  compound context, event/ref composition, focus/overlay/composite navigation.
- [ ] Complete sections 9–12: theme customization/export, advanced components,
  all three recipes, all 30 blocks and practical custom-component authoring.
  The fixed inventory is 64 components at the approved pinned upstream revision.
- [ ] Complete sections 13–14: public topic migration, curated SQLite-backed method
  and overview comparisons, CLI integration, HTML-first loading/executable splitting,
  paired accessibility/platform acceptance and meaningful family regressions/timing.
- [ ] Reconcile compiler completion ledger B/D rows against newer milestone records
  and actual source. S01–S20 have later accepted scoped evidence; S21 service-family
  regression/performance acceptance remains pending. Do not restart B11–B13 from
  stale table cells. Audit D2 type features, D3 classes/this, D4 generators/iterators
  and D5 BigInt/Symbol individually before declaring those families complete.
- [ ] Audit remaining approved roadmap and UI vision requirements against source
  and acceptance maps. Record each remaining concrete deliverable before its section.
- [ ] Finish required complete compiler/editor regressions and matched measurements
  at feature-family milestones, review comparable regressions, retain exact SQLite
  evidence, verify current consumers and prune obsolete generated outputs.

Each implementation section requires checked diagnostics/ownership, formatting
token preservation/idempotence and formatted behavior, generated-native examples,
complete editor help/agreement, applicable real web/native interaction, offline
installation/export and benchmark support with independent oracles. Timing remains
separate from untimed correctness. Unavailable platform checks remain pending.

## Explicit exclusions and decisions

Semantic search and the optional browser JS/TS bridge are deferred. UI vision
section 18 file routing is a proposal with unresolved decisions, not authority to
invent routing semantics. Decorator semantics require a separate owner decision.
Compiler ledger policy exclusions and undecided section C features remain excluded.
Table's explicit portable limits remain documented; they do not imply full DOM/CSS
compatibility. No arbitrary TypeScript compatibility expansion is authorized.

Public release publication requires explicit version/outcome authorization. Local
checkpoints are normal; hosted validation requires a recorded exact commit and
specific uncertainty that local checks cannot resolve.
