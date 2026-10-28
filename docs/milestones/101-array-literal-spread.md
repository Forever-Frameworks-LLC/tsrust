# Gate 101: array-literal spread

Status: focused implementation and documentation accepted; full regressions and timing remain deferred to a completed feature-family milestone.

The implementation accepts dense homogeneous array spreads such as
`[0, ...values, 1]`, using the existing checked spread construction path shared
with `Array.of`. It evaluates each segment once, from left to right, and stages
its elements before evaluating the next segment. Scalar elements are copied;
owned strings, records, nested arrays and callable environments transfer ownership.
Borrowed owned values cannot be implicitly cloned. Strings and arbitrary iterables
are not array spread sources.

Initial native fixtures cover source-order mutation, nested literal spread inside
call spread, contextual empty arrays, generic boundaries, strings, records, nested
arrays, boolean arrays and number inference. Negative fixtures reject non-arrays,
mixed element types and moved-source reuse. Each native fixture executes at O0/O3.
See `target/editor/gate101-literal-spread-focused.log`.

Snapshot: `archive/gate101-array-literal-spread-start`.

## Focused acceptance

Ten compiler integration tests cover all dense element families, callable capture
and invocation contracts, null/undefined arrays, nullable source narrowing,
contextual empties, scalar/Number inference, ownership rejection, source-order
mutation and nested literal expansion inside calls. Seven native fixtures and the
published guide fixture execute at O0/O3. An imported generic record/function
fixture passes at O0/O3 in the project suite. Logs:
`target/editor/gate101-language-and-matrix.log` and
`target/editor/gate101-project-and-callable.log`.

The editor checks complete and incomplete spread sources and inferred result types.
Array and Array.of catalog documentation now matches supported constructors and
spread/callable element behavior. All executable catalog examples pass native
verification. Logs: `target/editor/gate101-editor.log` and
`target/editor/gate101-catalog-examples.log`.

Ten matched benchmark pairs cover dynamic scalar expansion, multiple ordinary and
spread segments, Boolean, String, nested arrays, complete record fields, null and
undefined, IEEE signed zero, stored functions and source-order mutation. All five
correctness cases per pair pass on native and Node after TypeScript compilation.
The retained result observer checks complete shape/content. Independently written
literal cells and source mutation guards pass. Earlier manifest objects are
unchanged. Untimed proof:
`benchmarks/runtime/results/2026-10-04T16-28-49-150Z-verify.json`;
`target/editor/gate101-paired-verify.log` and `gate101-registration.log`.

The guide is [array-literal-spread.md](https://forever-frameworks-llc.github.io/tsrust/language/array-literal-spread/). Coverage is
registered under syntax.spreadArray and regenerated without measurement claims.
A partial checkpoint regression ran under the previous policy, reached its 15-minute
bound, and resumed only unrun groups. The user then superseded fixed five-gate
checkpoints with implementation-first focused TDD and completed feature-family
milestones. The heavy continuation is stopped; completed checks remain retained in
`target/editor/checkpoint101/{regression,resume}.log`. No timing was launched.
The 71-workload selection is retained as a deferred readiness plan, not a command
to resume heavy work. Gate101 comparisons remain unmeasured.

Active fixture directories are descriptive: `fixtures/call-spread` and
`fixtures/array-literal-spread`. The reported formatter gap after `...` is fixed,
installed and visually verified. Supplemental renamed-fixture/formatter checks
pass (10 + 5 + 8 tests). The user's added `length:` label in the language example
is preserved with matching expected output.
