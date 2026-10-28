# Gate 130: nested array-rest targets

Status: focused compiler/native, editor, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

An array rest target may itself be an array or object pattern. Rest is copied once, then the nested target binds positions or properties under ordinary access rules. Nested rest can request another independent copy. Tuple targets preserve remaining position types. Nested target positions accept defaults. Array-length object targets work. These patterns are supported in declarations, parameters/arrows and for...of.

The outer rest target must remain last and cannot have a default or trailing comma. The existing nesting limit of32 remains enforced across rest targets. Record rest still requires an identifier. Ordinary selection access and lexical borrow lifetimes are unchanged; the explicit rest copy does not introduce implicit clones elsewhere.

See [the syntax guide](../../website/src/content/docs/language/destructuring.md).

## Evidence

- Initial snapshots: `archive/nested-rest-patterns-start`; matched support: `archive/nested-rest-patterns-benchmark-start`; before-refresh consumers: `archive/nested-rest-patterns-consumers-start`.
- `target/editor/nested-rest-patterns-red.log` preserves the focused unsupported-syntax failure. `nested-rest-patterns-final-native.log` passes3 native/compiler checks using std:testing assertions. Cover nested array/object targets, copy independence, parameter defaults, tuple types, arrows, for...of, invalid placement/default and depth rejection.
- Adjacent rest/spread23 passes in `nested-rest-patterns-current.log`. Higher-order function values6 and type syntax8 pass in `nested-rest-patterns-first.log` after the parser change.
- `nested-rest-patterns-editor.log` verifies selected string hover and original nested-name definition. The guide example passes native and independent Node assertions with empty stdout in `nested-rest-guide-verification.json`. Executable fixtures independently pass Node in `nested-rest-patterns-node.json` and `nested-rest-arrows-loops-node.json`.
- Debug/compiler/LSP/editor and release consumers rebuild and refresh successfully in `nested-rest-patterns-final-debug.log` and `nested-rest-patterns-final-release.log`.
- The first runtime harness run records337 passes and6 executable failures during a consumer replacement race. Original evidence is preserved. All6 failed titles pass in the10-check affected-group supplement `nested-rest-patterns-harness-repaired.log`; `nested-rest-patterns-harness-audit.json` accounts for all343 original checks. No full restart. The development workflow now requires finishing refresh before execution checks and atomic replacement of existing executable consumers.

## Matched support

`nested_rest_patterns` retains nine cells per iteration across nested array rest, array-length object rest, typed tuple rest and a parameter pattern with a missing-position default. Its independent oracle computes expected scalar/string values directly without executing destructuring. Current source hashes, complete-retention guards, whole-scenario denominator, registration, runtime/allocation-profile hooks, oracle fingerprints and report category are ready on the existing complete-shape ABI.

Both registration/oracle checks pass in `nested-rest-patterns-registration.log`. Five current-release native/Node-after-tsc cases pass untimed in `benchmarks/runtime/results/2026-10-05T11-13-06-980Z-verify.json` and `target/editor/nested-rest-patterns-verify.log`.

Coverage separates accepted implementation from pending value-family measurement. No timing, allocation profiling or complete compiler/LSP regression is claimed. SQLite measurements remain unchanged. Next gate is131.
