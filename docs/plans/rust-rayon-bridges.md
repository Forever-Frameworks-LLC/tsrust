# Local Rust and Rayon bridges

Implementation acceptance and first matched measurements are complete at `0cdd58bd`. The final benchmark import-path migration has passed verification; its timing record is pending.

Compiler acceptance covers 137 tests across project resolution/linking, retained
sessions, checked expression facts, existing workers/channels and offline exports.
The final dedicated bridge suite passes three tests, with expanded scalar/buffer,
Result<void, string>, stored-function, owned error-buffer, identifier-collision and
source-inclusion rejection witnesses. Heap array and owned string pointers remain
identical across round trips. Nested Rayon observes the selected two-worker pool;
source errors, backend panic and acknowledged cancellation preserve cleanup counts.

Editor acceptance covers 204 tests across project intelligence, incomplete source,
retained sessions, checker agreement, consistency, worker catalog/protocol and
bridge navigation. The final bridge suite additionally checks actual Rust definition
and reference ranges and retains Rust doc comments. The real-server typing-speed
bridge row passes all four modes. Live native signature changes add and clear caller
diagnostics. Fourteen dev checks pass, including Rust body-only edits and frozen
snapshot restoration. VS Code extension compilation and its existing tests pass.

Offline package export builds the readable Rust/Rayon example in debug and release
with an empty Cargo home after deleting its original sources and relocating the
package. Whole-project formatting preserves tokens and is idempotent. The matched
untimed Rust/native versus Node-after-tsc workload returns checksum 4791373295.
Original attempts and affected repairs are retained in SQLite under
`tsrust-rust-bridge-*.log`; these checks are not timing claims.

Import safe public free functions directly from a relative `.rs` file or an
existing path alias. The compiler derives checked source signatures from Rust;
no handwritten duplicate declarations or per-call `--extern` arguments are needed.
Direct calls execute on the caller. Existing `spawn` and `ThreadPool.task` accept
closures calling these exports. The later decision to remove the global pool
supersedes the older interop proposal: choose and own a ThreadPool explicitly.
Nested Rayon work in a pool task reuses that pool. A synchronous wrapper that
itself calls Rayon outside a task follows Rayon's global scheduler behavior.

The restricted boundary supports i64/int, f64/number, bool/boolean, String/string,
Vec<i64|f64|bool>/primitive arrays, &str, shared/mutable primitive slices, and owned
Result outputs. `&TaskContext` borrows this worker's explicit cancellation context;
Rust code polls `cancellation_requested()`. Return borrows, pointers, generics,
traits, arbitrary records, arbitrary crates and asynchronous exports are rejected.
Rayon remains pinned at 1.11.0. Wrappers are self-contained; nested modules,
source-inclusion macros and conditional export attributes are rejected. Rust
bodies compile with `forbid(unsafe_code)`. Local native source is trusted program
code, not a sandbox: it can perform standard-library I/O or deliberately leak data.

Owned arrays and strings transfer existing heap buffers without serialization or
cloning. Inline numeric arrays allocate a Vec when moved into Rust; static borrowed
strings allocate an owned Rust String. Shared slices/str borrow; mutable slices
retain exclusive source loans. Rust Vec/String results retain their allocations.
Result outputs map their original payloads into source Ok/Err. Native panics unwind
Rust cleanup and become `panic` worker completion; direct native panics terminate
the program after unwinding and are not source exceptions. Cancellation remains
cooperative and does not forcibly interrupt native code or imply join.

Build/dev tracks complete Rust source bytes through the project graph. Exported
native packages embed these bytes and pinned dependencies for offline relocation.
Imported editor completion, hover and parameter help use derived source contracts;
definitions map back to actual Rust source. Rust's own editor handles native bodies.

Channel capacity documentation now explains unread-message count, bounded buffering,
blocking send, nonblocking full, memory tradeoffs and that capacity is neither byte
count nor worker count.

Acceptance scope: native transfer pointer identity, borrowed/mutable calls, typed
Result errors, nested Rayon scheduling, cancellation/panic disposal, ownership and
unsupported-signature rejection, source edit invalidation, editor protocols, dev
watching, offline package export, formatted examples, and matched throughput.


## Measurement and cleanup evidence

SQLite record `benchmarks/threads/rust-bridge-throughput-2026-10-15T01-48-44-424Z.json`
retains three processes, eight warmups and 31 samples per variant/process (93 samples
per engine/variant), exact compiler/source/Rust/harness hashes and source checkpoint
`0cdd58bd`. The tree was clean during measurement. Every checksum is 4791373295.
On Apple M3, native serial/two/four-worker medians are
3.282708/1.717542/0.960291 ms, giving 1.91x/3.42x parallel speedups. Node-after-tsc
medians are 26.895666/14.003791/7.701292 ms, giving 1.92x/3.49x speedups.

The earlier equivalent source-only recurrence workload measured
3.511500/1.735416/1.052875 ms natively. The bridge workload shows no material
slowdown against that signal. The different source implementation and run conditions
do not isolate a per-call overhead number or prove general application speedup.
No optimization is justified by these results. This scalar workload does not measure
buffer transport; native heap-pointer assertions verify buffer transfer separately.

The optimized readable example returns `55` and `cancelled`. macOS `leaks --atExit`
reports zero leaks / zero leaked bytes for its dedicated worker, selected Rayon pool
and owned input cleanup. Source/backend error disposal is additionally checked by
exact Rust guard-drop counts. These checks do not prove arbitrary user-written
native code cannot leak or deadlock. Cold pool setup is unchanged and was not
remeasured; this bridge does not introduce a new scheduler or automatic pool.

Completed compiler/LSP regressions from the channel milestone remain retained.
This bridge pass runs affected project, native, editor and dev checks rather than
repeating unrelated complete suites. Other native platforms were not executed.


Final editor auditing found two hand-built unit-test ModuleSource initializers that
needed `rust: None`; 56 affected compiler unit tests now pass. The checked-in
benchmark import now names its actual local Rust file, matching ordinary workspace
checks as well as harness relocation. The installed CLI checks this workload and
runs the readable example successfully. The extension and release CLI/LSP consumers
are refreshed and hash-verified; the primary VS Code window was reloaded and reports
No Problems with Tsrust checking enabled. Native worker errors also retain the original
Rust Err payload through `unwrap`, pool completion and join; five guard drops prove
normal/error/panic/cancellation cleanup in the focused witness.
