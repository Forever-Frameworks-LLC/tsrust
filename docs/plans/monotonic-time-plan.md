# S09 monotonic time

Status: focused compiler/editor/example and matched untimed acceptance complete.
Installed consumer refresh is pending; family measurements remain pending S09.

`std:time` exports `monotonicNow(): number`. It takes no arguments and returns
nonnegative fractional milliseconds from a process-local origin initialized on
first use. Subtract readings within one process to measure elapsed time. Values
are not wall-clock timestamps or portable between processes; equal consecutive
readings are allowed, resolution is platform dependent, and suspend accounting is
unspecified. Floating-point precision decreases for long durations. There is no
resource handle, ownership transfer, explicit close or receiver mutation.

The backend uses the pinned Rust toolchain's `Instant` and `OnceLock`; generated
programs require no extra dependency. The runtime is separately reusable by the
native build cache. Backend behavior follows the [Rust Instant contract](https://doc.rust-lang.org/std/time/struct.Instant.html), including platform clock limitations.
This slice does not add wall time, dates, sleeping, timers or asynchronous scheduling.

The initial missing-export witness is retained. Focused public checks pass at O0
and O3 for finite, nonnegative and nondecreasing reads, subtraction and aliased
function values. Extra arguments and integer result assignments are rejected.
Hover, signature help and import completion pass; the catalog example is checked,
formatted twice and executed at O0/O3. Records use `tsrust-monotonic-time-*` in SQLite.

Matched workload `std_time_monotonic_now` retains one boolean observation per clock
read: finite, nonnegative and no earlier than the preceding read. Its independent
oracle expects those contract properties, not equal timestamps across processes.
The workload includes one setup read plus n measured-loop reads, validation and
result retention; no pure clock-call latency claim is made. Native and Node-after-tsc
pass ten untimed cases, including n=0 and n=20,000. Two registration/source-guard
checks reject constant observations, skipped iterations and omitted clock reads.
ABI adapters, main/profile driver guards, report category, source hashes and oracle
identity inputs are registered. Formatting preserves tokens and updates the source
fingerprint; the formatted workload passes the same checks.

Three public/feature-selection checks and the expanded O0/O3 runtime-separation
witness pass. Runtime support is absent for an unused clock import and remains
reusable after an application-only edit. Records use `tsrust-monotonic-*` in SQLite.
Refresh consumers next. Timing stays pending the S09 synchronous-host family milestone.
