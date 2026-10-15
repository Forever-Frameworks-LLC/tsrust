# Local Rust and Rayon bridges

Focused implementation acceptance passed; matched timing is pending.

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
Original attempts and affected repairs will be retained in SQLite under
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
