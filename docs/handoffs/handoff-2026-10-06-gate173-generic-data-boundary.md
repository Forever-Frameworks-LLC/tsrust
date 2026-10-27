# Handoff: deferred generic data boundary

Continue from 4147395 and this red checkpoint. Read AGENTS.md,
docs/development-workflow.md, docs/plans/generic-function-values-plan.md and
handoff-2026-10-06-gate173-generic-contexts.md. Contextual specialization is verified;
unannotated polymorphic values, inferred generic factories and gate173 remain open.

No compiler source changed here. Two additional generic_value_signatures native
tests fail at the explicit pending-value diagnostic. Their committed TypeScript
counterparts pass strict tsc and Node execution. They cover T[] element inference,
mutable int/string caller arrays, one captured counter across methods, defaulted
fixed prefixes, explicit undefined and homogeneous generic rest packs. Do not
remove these or the three original polymorphic red witnesses during implementation.

The concrete missing boundary is in materialize_closure_method_types: its current
substitution handles callable slots, not unresolved generic data positions. The
native closure_method emitter also selects receiver access from global invocation.
The physical plan now records both boundaries. Next implement explicit deferred
source metadata, original-template solving and concrete data method contexts, then
integrate one environment with sound per-method access and ownership flow. Keep
ordinary void checking strict. Do not replace the final scope with contextual-only
values or allocate an environment per argument type.

Preserve four owner fixtures untouched and uncommitted: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr, map-group-by/documented.tsr. Explicit
Git paths; one agent in the tree. Hourly checks remain paused. Reuse the documented
Cargo target with CARGO_INCREMENTAL=0 and offline locked Cargo. Node is
/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Consumer refresh and broad
checks were unnecessary because compiler source did not change. Previous accepted
contextual evidence remains in SQLite. Use --verify for routine benchmark checks.

Both original logs were imported and verified byte-for-byte in SQLite before
removing loose copies. Generated Node scratch and 2 obsolete test executable
were removed; library caches and five consumers remain. Tests may relink.
