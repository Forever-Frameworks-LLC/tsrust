# Handoff: callable effect index WIP

Read AGENTS.md, docs/development-workflow.md, docs/handoffs/handoff-2026-10-07-lexical-generic-transport.md and docs/plans/captured-call-effects-plan.md first. Accepted lexical transport is 399a1c4. Current Git HEAD adds a tested effect-index prerequisite; it does not accept captured invocation effects or complete Gate173.

The isolated generic_call_effect_bodies module owns named and arrow body snapshots keyed by source declaration span. It retains declaration BindingId values and resolves uses through capture-plan provenance or lexical ranges. Hoisted named captures work before the binding's textual initializer. Local and parameter shadowing retain different declarations. Planned uses and ordinary declaration records agree on identity.

Two formatted-source unit tests pass; the failing stub and corrected runs are retained in SQLite with exact readback. The index is registered but production validation does not call it yet. No analyzer dispatch, editor behavior or executable consumer changes occur. Do not claim the eight invocation-effect reds repaired. No benchmark or timing work is required for this indexing-only WIP; the accepted 124-observation family remains intact.

Next wire identity into source flow, then implement ordered normal and exceptional invocation summaries. Select bodies from saved callee contracts and their alternative declaration provenance after argument evaluation. Preserve uncalled declarations, saved aliases, lexical shadowing and restoration controls. Model recursion with a finite worklist and synchronous collection callbacks with their actual invocation rules. Keep unknown origins conservative and never introduce shared capture cells or implicit clones.

Preserve all fifteen owner fixture edits, use explicit Git paths and format the entire project before checkpoints. Reuse the warm Cargo target with incremental compilation disabled. Keep SQLite canonical. Existing five executable consumers and installed CLI remain the accepted 399a1c4 image. No whole-editor or depth rerun is claimed for this unused index module; run the required suites when validation/dispatch wiring changes.
