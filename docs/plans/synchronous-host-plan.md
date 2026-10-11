# S09 synchronous host APIs

Status: read-only file resources are accepted in S07. The owner-prioritized
`new` and `dev` CLI milestone is accepted on macOS, including installed consumers.
S09 public file-write integration is next.

The next file slice is create, write and writeAll. The private backend now has
focused tests for truncation, exact binary content, read-only descriptor failures,
64 MiB write limits, interrupted/short writes, write-zero and failure after partial
progress. Six backend tests and the generated O0/O3 cleanup witness pass. Original
red and green logs are retained as `tsrust-s09-file-write-*` in SQLite. These methods
are not public yet: native bindings, byte-view alternatives, bridges, catalog docs,
public native/editor fixtures and matched benchmark support remain required.

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
