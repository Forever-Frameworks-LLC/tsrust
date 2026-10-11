# Buffered file I/O: S09 implementation batch

Status: backend, public bindings, type alternatives, IntelliSense and executable
examples implemented. Matched workloads and installed-consumer acceptance remain
pending; this batch is not yet accepted as a complete feature.
Continue the existing owned-resource contract.

Expose BufferedReader and BufferedWriter in std:fs. Constructors consume a File;
there is no second handle or shared cursor. Construction errors release the incoming
handle. Capacity defaults to 8192 bytes and must be between 1 and 1048576 bytes.
Document this transfer explicitly, including failure behavior.

Use the existing read, write, writeAll and close names with native type alternatives:
read accepts File or BufferedReader; writes accept File or BufferedWriter. Byte
arguments retain their Bytes/ByteView/ByteViewMut alternatives. Reads return owned
Bytes, including empty EOF, with the existing 64 MiB operation ceiling. Buffering
may read ahead, so the original File cannot remain available concurrently.

flush borrows a BufferedWriter mutably, sends pending bytes to its file and reports
errors. It does not synchronize durable storage. A flush error retains the writer
and its remaining buffer for an explicit retry. Consuming close attempts one flush,
then releases the descriptor even if flushing fails; the first error takes precedence.
Do not let BufWriter destruction silently retry after a failed close flush. Automatic
scope cleanup uses the same release path but cannot report failures; recommend
explicit close when errors matter. Closing discards any remaining buffered bytes
when flushing fails. There is no detach/unwrap API in this batch.

Acceptance covers transfer/reuse rejection, capacity bounds, buffering/read-ahead,
short/interrupted operations, explicit flush, flush failure followed by close,
no duplicate flush/release, scoped cleanup and native O0/O3 execution. Complete
signatures, hover/help/completion, executable examples, and matched native/Node
workloads before marking the batch accepted. Family timing remains deferred until
buffered streams, environment and monotonic time foundations are complete.

Private backend evidence: `tsrust-buffered-backend-red.log`,
`tsrust-buffered-backend-green.log`, `tsrust-buffered-backend-native.log`,
`tsrust-buffered-backend-native-repaired.log` and
`tsrust-buffered-backend-format.log` in SQLite. The native check compiles the
resource, file and buffered backends together at both optimization levels. Five
buffered witnesses cover read-ahead, deferred writes, explicit flush/close,
interrupted and partial writes, retained bytes after failure, zero progress,
scope cleanup and descriptor release on constructor/close errors. The first
integration attempt exposed a missing test-only module guard; the repaired run
passes. This is backend preparation, not public feature acceptance or timing.

Public integration adds `bufferReader`, `bufferWriter`, `flush`, `BufferedReader`
and `BufferedWriter`, with existing operation names accepting the corresponding
buffered owners. Native bridge traits preserve concrete unique ownership without
shared handles. A formatted executable witness writes a borrowed byte view through
a typed mutable writer, flushes, closes and reads through a typed mutable reader.
Eight rejection witnesses cover transfer, cloning and incorrect access. Existing
file-write, full-file and O0/O3 cleanup tests pass. All 15 file catalog examples
execute at O0/O3. The 86 project-intelligence tests pass across the original run
and a focused repair of an obsolete unbuffered-only signature expectation.

Public evidence is stored in SQLite under `tsrust-buffered-public-*`,
`tsrust-buffered-editor*` and `tsrust-buffered-signatures.log`. Failed test attempts
are retained: initial missing exports, then two assertions using outdated diagnostic
wording. No timing measurements or installed buffered-file consumer refresh are
claimed. Next complete matched native/Node buffering workloads and registration
guards, then refresh and smoke-test consumers before accepting this batch.
