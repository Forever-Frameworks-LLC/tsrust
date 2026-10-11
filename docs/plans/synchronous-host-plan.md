# S09 synchronous host APIs

Status: read-only file resources are accepted in S07. The owner-prioritized
`new` and `dev` CLI milestone is accepted on macOS, including installed consumers.
Bounded full-file access is implemented with focused native/editor and matched
untimed and installed acceptance; the broader S09 family remains pending.

The create, write and writeAll slice is implemented. The private backend now has
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
tests pass. Records use `tsrust-file-write-benchmark-*` in SQLite. The installed
CLI passes all sixteen cases at d76217cc; CLI/LSP consumers include the slice.
The owner-window reload remains tracked in the project CLI plan. Timing remains
pending S09.

readFile(path, maxBytes?) and writeFile(path, bytes) now provide bounded whole-file
access. Reads default to a 64 MiB ceiling, validate limits before opening, detect
excess with at most one additional byte, and tolerate short/interrupted reads. Writes
validate the 64 MiB input bound before truncating and accept owners or views. Both
close their handles and report close errors when no earlier operation failed.
Eight backend tests, public binary/limit tests and existing write/cleanup witnesses
pass. Ten catalog examples execute at O0/O3; 85 project IntelliSense tests pass.
A third matched workload uses readback and an independent byte oracle. All 24
untimed native/Node cases and six registration tests pass. Full-file bridge emission
was repaired for programs without an explicit File type. Original records use
`tsrust-full-files-*` in SQLite. Eight installed CLI/LSP consumers were refreshed
from 562a6c00, and the installed CLI passes all 24 untimed cases. The active
`tsrust_test` editor server was restarted and shows checking enabled with no Problems.
Buffered files now have focused implementation acceptance, executable examples,
matched buffering/readback support and eight installed native/Node untimed cases.
See [buffered-file acceptance](buffered-files-plan.md).

Environment lookup has a tested read-only backend and public native/editor
implementation plus matched workload support and ten untimed native/Node cases.
Installed acceptance remains pending. See [environment lookup](environment-plan.md).
Complete environment installed acceptance, then monotonic time foundations, auditing
the existing path/fs/os contracts. Define
focused API batches before implementation and run the family milestone only after
the public surface, examples and matched workload support are complete.

Memory mapping is under discussion with the owner, not an accepted API. A separate
std:mmap module with owned mappings and scoped byte views is the proposed direction.
Ordinary file-backed maps cannot promise safety against external modification or
truncation; read-only and copy-on-write do not remove this constraint. Settle the
external-file safety contract before implementation. Writable mappings and flush
semantics require separate acceptance.
