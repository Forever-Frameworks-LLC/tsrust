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
check remains in the diagnostic evidence. The owner has been asked to choose an
explicit ScopedOnce<Function<...>> contract or scoped Once parameter acceptance
with owned storage/return rules preserved. Do not implement either interpretation
until the owner replies. Direct invocation work is independent and has progressed.
Do not weaken the ownership checks to make the parameter witness pass.

Remaining acceptance includes the selected public contract and its source/native/
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
refresh. This support is ready; acceptance and consumers still await the contract
choice and remaining callback-boundary checks.
