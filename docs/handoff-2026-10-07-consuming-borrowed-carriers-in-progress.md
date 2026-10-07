# Consuming borrowed conditional carriers in progress

This work extends the accepted scoped borrowed family at commit 520aa7c. It is not
an accepted gate or a completed Gate173. Preserve the 15 protected owner fixtures
and all existing implementation edits. No new source checkpoint has been committed.

The checked family now joins consuming invocation access. Its scoped contract is
CallbackKind::Once, distinct from OwnedOnce. Native emission consumes the selected
concrete environment and provides OnceCallback adapters. Repeated calls report
closure-consumed. Owning callback transfers and scoped returns remain rejected.

Four focused compiler/native tests pass. They cover owned string capture transfer
while borrowing a local array, consuming versus reusable alternatives, defaults,
joined failures, repeat-call rejection and scoped escape rejection. An instrumented
native witness verifies eight original record owners dispose exactly once after
invoked and abandoned selections at optimization levels 0 and 3. Formatting round
trips precede native checks. One focused editor test passes for signature help,
hover and result-chain documentation.

A callback-parameter witness exposed a public syntax gap: Once<Function<...>> is
OwnedOnce, so it correctly rejects a borrowed environment. The failed original
check remains in the diagnostic evidence. The owner chose existing Once syntax first. Follow
docs/once-callback-environment-plan.md for retention inference and preserve existing
owned callback returns and capture behavior. Do not weaken the ownership checks to make the parameter witness pass.

Remaining acceptance includes environment retention inference and its source/native/
editor checks, callback failure adaptation and an explicit source checkpoint. The bounded compiler batch
passes 121 tests, including conditional_expressions, callable_fields and all eight
depth canaries. Focused borrowed unit controls pass 22 tests; the complete editor
crate passes 333 tests. All nine closed logs are imported under
runtime/diagnostics/consuming-borrowed-carriers/ in benchmarks/records.sqlite,
with byte-exact readback before deleting loose copies. SQLite quick_check passes.
Consumers still represent the previous accepted checkpoint; do not refresh them
from this unaccepted source.

Continue with AGENTS.md, docs/development-workflow.md and
docs/conditional-callable-carriers-plan.md. Preserve the broader remaining scope:
scoped aggregate/builtin transport, polymorphic selection/replacement and the
point-sensitive environment restoration bridge. No timing is due at this stage.

## Matched direct-call support

The matched generic_value_adapters workload now retains 132 observations. The new
observation consumes the selected record environment while borrowing the original
array. Both alternatives update their own record call count from that array length.
The returned record contributes its value and call count to the retained result.
The independent oracle computes the selected branch directly. Source guards cover
the construction, selection, consumption and retained fields; a mutation dropping
the call count is rejected. Manifest fingerprints match both current sources.

consuming-borrowed-carriers.test.mjs checks five original correctness cases against
the independent oracle with native execution and Node after TypeScript compilation.
The TypeScript target remains ES2022 with ES2023/DOM libraries, matching the existing
harness configuration. Native generated warning noise exceeded the initial harness
buffer; generated warnings are suppressed while errors remain captured. The first
TypeScript harness omitted the required ES2023 library, then was corrected. Original
failures and green checks are retained in SQLite. No timing measurements were run.
Whole-project formatting changes one workload with one token-preserving fingerprint
refresh. This support is ready; acceptance and consumers still await retention inference
and remaining callback-boundary checks.

## Once decision and regression investigation

The owner chose existing Once syntax first. No additional wrapper is authorized
as the first approach. See docs/once-callback-environment-plan.md. A blanket scoped
parameter resolution experiment passes the direct/conditional failure witness but
breaks two existing callback_moves owned-retention tests. It is reverted; existing
parameter ownership remains intact. The next change must infer environment retention
separately from consuming invocation, preserving both scoped calls and owned escape.
New Once source tests remain unaccepted. Source checkpoint 3a7eaf1 preserves consuming
carrier work and the original parameter red; later investigation is not committed.
The pending syntax question is resolved; no further owner answer is required.

Restored callback_moves passes ten tests; the positive owned-retention control
passes. Scoped conditional calls remain red. Direct-arrow input now also requires
post-call array reuse and fails with transferred: current ownership admission
moves the array rather than preserving its borrow. Retain both reds. Investigation
logs have exact verified SQLite readback under once-environment-admission.

## Retention kernel continuation

The isolated callable_environment_demand.rs kernel now passes six focused controls,
including recursive forwarding and a 10,000-binding chain. Unknown/missing evidence
cannot certify scoped admission. An existing source index capture/shadow identity
test passes. Formatting changes zero files. Exact red/green logs are imported and
verified under runtime/diagnostics/once-environment-demand/ in SQLite.

Continue from docs/once-callback-environment-plan.md. Connect the kernel to complete
source use/capture/destination facts using the existing effect_bodies lexical index,
then resolve concrete Once parameter environment requirements without breaking
owned retaining factories. Preserve both scoped-call reds and the positive owning
retention witness. The kernel alone does not admit new source behavior. Gate173
remains open. Use explicit Git paths and preserve the 15 owner fixtures. No timing
or consumer refresh is due before source integration acceptance.
