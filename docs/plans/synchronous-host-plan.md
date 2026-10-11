# S09 synchronous host APIs

Status: read-only file resources are accepted in S07. The owner-prioritized
`new` and `dev` CLI milestone is accepted on macOS, including installed consumers.
S09 public file-write integration is next.

The next file slice is create, write and writeAll. The private backend now has
focused tests for truncation, exact binary content, read-only descriptor failures,
64 MiB write limits, interrupted/short writes, write-zero and failure after partial
progress. Six backend tests and the generated O0/O3 cleanup witness pass. Original
red and green logs are retained as `tsrust-s09-file-write-*` in SQLite. Public bindings and bridges now expose create, write and writeAll, including borrowed
Bytes, ByteView and ByteViewMut inputs. Focused native tests cover truncation, exact
bytes, explicit/scope close, read-only errors, retained handles and ownership rejections.
All eight file catalog examples execute at O0/O3, and 84 project IntelliSense tests
pass, including view alternatives and partial-write documentation. The initial public
witness exposed the required error-effect registration; it was added. A rejection
witness expected the wrong diagnostic wording and now checks error[transferred].
Original logs are retained as `tsrust-public-file-writes-*` in SQLite. This slice
has matched native/TypeScript write and writeAll workloads registered with independent
readback oracles, exact output-file checks, ABI adapters, source guards, current hashes
and isolated cleanup. Sixteen untimed native/Node-after-tsc cases pass; five registration
tests pass. Records use `tsrust-file-write-benchmark-*` in SQLite. Installed consumer
acceptance remains required before this slice closes. Timing remains pending S09.

Continue S09 with bounded full-file access, buffered readers/writers, environment
and monotonic time foundations, auditing the existing path/fs/os contracts. Define
focused API batches before implementation and run the family milestone only after
the public surface, examples and matched workload support are complete.

Memory mapping is under discussion with the owner, not an accepted API. A separate
std:mmap module with owned mappings and scoped byte views is the proposed direction.
Ordinary file-backed maps cannot promise safety against external modification or
truncation; read-only and copy-on-write do not remove this constraint. Settle the
external-file safety contract before implementation. Writable mappings and flush
semantics require separate acceptance.
