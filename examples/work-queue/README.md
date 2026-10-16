# Bounded work queue

Run `tsrust run examples/work-queue/main.tsr`. The example uses the existing
`std:futures.Channel`; no second queue wrapper or hidden shared payload is needed.
One producer moves 64 work records into a capacity-two channel. Two async consumers
borrow the receiver, validate their owned messages and return independent totals.
The producer then sends one stop message per consumer. FIFO delivery places work
before stops; settling all tasks proves processing is finished before close.

A full channel suspends send until space is available. Capacity bounds queued
message count, not payload bytes, active workers or pending producer messages.
Bound producer/worker counts as well. Workers are asynchronous; this example does
not promise CPU parallelism. `std:threads.ThreadPool` is the separate CPU-work API.

Closing rejects new sends and preserves buffered messages for draining. Receive
fails with `closed` after the buffer empties. A failed or cancelled send disposes
its transferred message; it does not return it for retry. Explicitly retain a
separate clone beforehand only when the payload supports cloning and retry needs
justify the copy. Receiver borrows must finish before mutating it with close.

A deadline around the whole task group cancels and settles outstanding work when
normal shutdown cannot finish. Cancellation drops suspended owned state; source
finally is not promised inside the cancelled task. Endpoint-bearing messages,
non-transferable resources, borrowed payloads, durable delivery, acknowledgements,
retry scheduling, and distributed queues are outside this in-memory contract.

The matched workload uses 64 messages, capacity two, two async consumers, eight
warmups and 31 samples in each of three processes. Its independent checksum is
64*65/2 = 2080. Node runs compiled TypeScript with bounded async queue operations;
its microtask scheduling differs from Tokio. Timing includes queue/task setup,
message construction, validation, handoff and settlement. Verification is untimed;
measurements belong to the shared S19/S20 milestone.
