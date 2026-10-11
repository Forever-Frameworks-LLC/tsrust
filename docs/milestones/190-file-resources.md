# Gate 190: consuming file resources

Focused implementation acceptance is complete. This is the first synchronous file
slice within S09, not acceptance of the broader host family. S07 complete regressions,
matched measurements, performance review and installed-consumer refresh remain open.

## Requirement evidence

| Requirement | Implementation and evidence |
| --- | --- |
| Owned opaque identity and explicit consuming close | `native_types.rs`, `fs_bindings.rs`, `fs_emit.rs`; File has no public storage fields, clone, key or thread-transfer capability. Close accepts an owned File and returns Result<void, FsError>. |
| Nonthrowing fallback and failure ownership | `stdlib/resource.rs`, `stdlib/fs.rs`; one release path, no close retry, ownership remains consumed on Err. Private backend and native lifecycle tests cover success, failure and unwinding. |
| Useful resource and structured errors | Public open/read/close, bounded cursor reads and FsError with operation/category/message/optional OS code. No implicit flush or sync. `std_fs.rs` and executable read-close fixture verify native behavior. |
| Exactly one release across source control flow | `native_file_cleanup.rs` instruments generated public source; 18 actual file owners per run, eight runs each at O0/O3, exact order and no retained descriptors. Normal/early exit, replacement, Result failure, closure disposal, throwing captured owner, whole/partial finally return and close failure are covered. |
| Source ownership and capabilities | `std_fs.rs` rejects reuse/double-close, Err-arm reuse, forged/cloned/compared/inspected handles, File map keys and readonly-handle close/read. Thread eligibility remains false; this does not accept a thread API. |
| Editor and documentation | Five builtins.json entries and ten O0/O3 example executions; hover, parameter help, import completion and four source-range diagnostic controls in project intelligence/diagnostics tests. |
| Isolated filesystem tests | Test helpers create private directories and remove them. Native descriptor observation uses a separate process and never closes a possibly reused descriptor. |
| Matched workload readiness | `native_file_read_close` and `native_file_scope_drop` have TypeScript counterparts, closed-form byte-sum oracle, source guards, source/input hashes, Result ABI support, report category and runtime/profile integration. |

Compiler paths are relative to `crates/tsrust-compiler/src` or its tests directory;
editor paths are under `crates/tsrust-lsp`. Detailed commands and retained evidence
are indexed in `docs/b11-resources-bytes-acceptance-map.md`.

## Matched workload protocol

Each iteration opens a private regular file, reads `(i + seed) % 64 + 1` bytes,
observes the returned byte count and sum, and releases the file. Input is 4096 bytes:
256 repetitions of ASCII `0123456789abcdef`. The oracle uses arithmetic rather
than executing either workload. Each operation denominator is one complete lifecycle.

The explicit workload calls consuming close; the scope workload relies on native
drop. Node uses try/finally for deterministic cleanup in both counterparts. The
measured domain is successful regular-file operations. Close-error behavior is
separately verified through native instrumentation, not claimed as a Node timing
comparison. Temporary input creation is outside timed bodies. Warm filesystem cache
behavior is expected; these workloads do not measure durable writes or cold storage.

Both harnesses create and clean private inputs. Normal and failed-process exit
cleanup is tested. Source and fixture mutations reject. Native and Node-after-tsc
agree on five untimed cases per workload. Final evidence is SQLite record
`runtime/results/2026-10-11T08-39-16-390Z-verify.json`, with `verifiedOnly: true`
and no execution samples. An earlier verification remains preserved separately.

## Platform and remaining milestone

Native behavior is verified on macOS. Linux close semantics were reviewed but
execution remains unverified. Other platforms have an explicit backend guard.
Resources remain uniquely owned; source sharing or handle duplication is not added.
The private instrumentation API is not published.

Next run the S07 compiler/editor regression milestone and accumulated matched
measurements under the existing protocol, review comparable regressions and
allocations, optimize only with evidence, then refresh installed consumers.
