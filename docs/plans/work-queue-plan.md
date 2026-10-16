# Bounded work queues: S20

The existing S14 Channel/Sender, owned tasks, groups, cancellation and deadlines
already supply the required bounded in-memory queue operations. S20 adds complete
service-oriented composition and acceptance rather than duplicating these APIs.

The source worker example transfers records through a capacity-two queue, uses
one producer and two borrowed async consumers, shuts down with FIFO stop messages,
settles all work, and verifies close/drain and rejection after close. The matched
native/compiled-TypeScript workload retains a 3-process, 8-warmup, 31-sample
protocol with an independent 2080 checksum and current source fingerprints.

A deterministic backend poll must prove a full queue suspends another send.
Independent counters and allocation identity must prove cancellation disposes only
the blocked message, close preserves buffered ownership, drain transfers it, and
failed sends dispose transferred inputs once. Reuse accepted S14 group cancellation
and worker cleanup evidence; preserve all existing ownership restrictions.

Focused implementation acceptance passes: source worker execution, four ownership
rejections, independent backend backpressure/identity/disposal, task cleanup,
editor hover/completion and matched native/Node verification. Twelve focused
task/runtime tests and the record-origin remapping witness pass. Whole-project
formatting and workload fingerprints are current.

Mixed started/borrowed task groups exposed two compiler defects: an empty failure
set was treated as an explicitly thrown undefined value, and closure compaction
left record fields and interning keys pointing at old origins. Both are repaired
without changing task settlement or transport policy. A negative guard preserves
validation of an actual thrown undefined.

Raw `tsrust-work-queue-*` acceptance and the workload verification record are
retained in SQLite, including initial failures and focused repairs. The [combined S19/S20 milestone](../milestones/s19-s20-owned-coordination.md)
now passes complete regressions and measurements. Continue S21 service dependencies.
Durable/distributed delivery, retry scheduling and acknowledgement protocols are
outside this bounded in-memory scope.
