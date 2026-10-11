# Buffered file I/O: S09 implementation batch

Status: implementation pending. Continue the existing owned-resource contract.

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
