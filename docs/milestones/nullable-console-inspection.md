# Nullable console inspection

## Behavior and acceptance

Console inspection accepts printable nullable payloads directly. A present payload uses normal inspection; absent alternatives print null or undefined. The compiler checks nested records and arrays for printable values and still rejects callable or void payloads. Owned values are borrowed, not transferred by logging. Direct pop/shift call temporaries retain their owning storage for the console call. Inspection retains the existing depth and array-length limits.

The console catalog documents direct nullable logging with a verified pop example. Compiler/native tests cover the reported shorthand/typed mapping example, populated and empty pop, repeated owned-payload inspection, Copy nullable values, nullable record fields, owned array payloads, and a temporary conditional string. Native output is verified at optimization levels zero and three.

## Focused evidence

- target/editor/nullable-console-repaired.log: 12 console tests and four contextual-arrow tests passed.
- target/editor/nullable-console-boundaries.log: expanded nullable console boundary fixture passed at both optimization levels.
- target/editor/nullable-console-hover.log: reported source has no editor diagnostic; lastUser remains string | undefined and console hover describes nullable inspection.
- target/editor/nullable-console-editor.log: expanded console catalog/signature discovery passed.
- target/editor/nullable-console-diagnostics.log: updated nullable rejection checks passed; previously unsupported console cases and gate-97 nullable callable cases no longer count as rejections.
- target/editor/nullable-console-observers.log: canonical nullable transport/observers passed after the reusable callback fixture explicitly borrowed its callback. Parenthesized ternaries followed by a colon are distinguished from arrow headers.
- target/editor/nullable-console-build.log: compiler and language-server binaries rebuilt.

## Outstanding evidence

Editor compatibility follow-up: stale nullable-console rejection assertions in intelligence.rs now verify successful checking and console.log hover, while nullable Number conversion remains rejected. All 79 intelligence tests passed in target/editor/nullable-console-compatibility-fixed.log. The same run exposed a separate project fixture using `Hidden<T>=>` in an arrow return annotation; spacing it as `Hidden<T> =>` preserves its private nominal identity assertions and passes the affected check (target/editor/generic-arrow-fixture-repair.log). Adjacent generic-close/arrow token parsing remains a compiler follow-up, not a fixed language capability. Previously passed groups were retained, and remaining project-session, rest, stdio and workspace groups passed in target/editor/nullable-console-compatibility-remaining.log. The interrupted script did not refresh its successful-fingerprint cache.

The broader nullable_unions run in target/editor/nullable-console-focused.log exposed a separate owned nullable closure-capture lowering failure: ownedCallback's captured string|null is emitted as a string slot, producing Rust type/view errors. This suite is not fully accepted. Preserve its failure for investigation; console-focused acceptance does not establish general nullable closure correctness.

No runtime timing was performed. Full regression/performance remains at the scheduled compiler checkpoint; nullable console workload timing is pending and must not be described as measured. Existing console runtime formatting/allocation contracts remain unchanged; this work connects tagged values to that inspection path.

Resolved later: the owned nullable closure-capture lowering failure is fixed and the full `nullable_unions` suite passes; see [callable and depth regression repairs](./callable-regression-repairs.md).
