# S14 Tokio tasks

Scope: Tokio runtime bootstrap, owned Task.start()/Job settlement, cancellation,
deadlines, delay/yield, race/allSettled and bounded asynchronous channels. Existing
dedicated threads, ThreadPool and Rust/Rayon APIs remain distinct. No detach,
Promise sharing, async arrows, networking or blocking-work adapter is added.

Implementation retains lazy construction and eager argument ownership. Local
borrowed futures remain supported; scheduling requires owned transferable inputs
and results. Error payloads retain their checked types. Final error-effect
validation rejects stale inferred transport contracts before native emission.

Started obligations follow fields, arrays, captures, returned jobs and lazy group
wrappers. Cancellation alone does not settle a job. Splice on Job-bearing arrays
is rejected until replacement and removal owners can be tracked independently.
Mixed group literals accept Tasks and Jobs with one result type; stored arrays
use one input type. No general callable-union array support is claimed.

Runtime witnesses verify nested cancellation, deadline cleanup, returned-job
ownership transfer, race loser cleanup, pending-message disposal, bounded-channel
backpressure, completed outcomes, worker overlap and repeated exact-once drops.
A three-party barrier proves both scheduled workers execute before either join.
Source finally is preserved during ordinary return/throw, but is not promised
when cancellation drops a suspended future. CPU work without suspension remains
cooperative; cancellation and deadlines may take longer while it runs.

Focused source acceptance: 23 existing async tests and 11 Tokio tests pass after
the empty-array final-effect validation repair. Runtime: eight tests pass. Catalog
examples execute natively after formatting/idempotence checks. Readable examples
are in examples/tokio-tasks. Public yield() naming and Task hover wording use
“returns” for values and explain scheduling separately.

Editor agreement passes for Task, Job, AsyncError and async Channel/Sender.
Typing-speed completion, editor consistency, complete family regressions,
measurement and consumer refresh remain pending at this implementation checkpoint.

Matched scheduled-batch verification passes for native and Node after tsc with
checksum 16777216. Its workload measures task scheduling overhead, not CPU speedup.
Both async workloads retain three processes, eight warmups and 31 samples per
process when measured. Verification is untimed and uses one sample.

Evidence is stored byte-exact in benchmarks/records.sqlite. Closed initial logs
use the tsrust-s14- prefix; failed attempts are retained alongside repaired runs.
Final milestone counts, measurements and installed consumers will follow below.
