# Gate 168: scalar unary plus

Unary plus accepts scalar inputs and scalar unions through the existing reserved
Number conversion. Numeric unary plus retains its int/number behavior. Number also
accepts scalar unions, including enums, and borrows the active payload. Arrays,
records and functions remain type errors. See [Scalar unary plus](../scalar-unary-plus.md).

The lowering helper runs before checked expression dispatch and returns before
recursive checking. Numeric operands retain their original expression form and
recursive path. Union emission matches the borrowed active alternative and evaluates
its source once. Checked Number result metadata supplies editor types.

Number constructor documentation now lives in builtins.json, with signatures,
parameter help, behavior, errors and an executable example. Old nullable-conversion
rejection assertions were replaced with positive contracts. Function-conversion
errors now identify the unsupported input type directly.

Red witnesses were committed before implementation. Final focused native/static
tests and all eight depth canaries pass (72 checks); the whole editor crate passes
266 checks. Native assertions also cover numeric and string enum unions and reject
non-scalar unions. The documented Number example executes with identical native
and Node output. Registration and report checks pass (17 checks), and all five
matched native/Node cases pass untimed. The matched workload retains seven scalar
conversion observations. Its independent oracle constructs expected decimal text
and boolean/nullish outcomes without using Number or unary conversion. Source
fingerprints, denominator guards and execution/profile/report hooks are registered.
No timing or routine full compiler regression is requested; measurement remains
pending the meaningful language milestone.

The TypeScript counterpart uses erased any assertions for nullish unary plus,
because strict TypeScript rejects those expressions even though JavaScript defines
their runtime conversion. Its string copy is String(text), corresponding to native
explicit clone(text); the operation and retained values remain matched, without an
allocation-parity claim.

Acceptance logs are retained in SQLite with read-back verification. Verification
records are imported as runtime-result records, checked by reading back their exact
bytes, and their loose copies are removed. Five executable consumers are refreshed
from the shared Cargo target. Generated native/editor example outputs are pruned
before handoff; accepted source remains reproducible from commits.
