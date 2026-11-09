<!-- tsrust-docs
{"id":"reference-standard-library-io","group":"standard-library","order":21,"status":"partial","targets":["native"],"api":[{"receiver":"std:io","name":"reader"},{"receiver":"std:io","name":"writer"},{"receiver":"std:io","name":"stdout"},{"receiver":"std:io","name":"stderr"},{"receiver":"std:io","name":"stdin"},{"receiver":"std:io","name":"read"},{"receiver":"std:io","name":"readInto"},{"receiver":"std:io","name":"readExact"},{"receiver":"std:io","name":"readAll"},{"receiver":"std:io","name":"write"},{"receiver":"std:io","name":"writeAll"},{"receiver":"std:io","name":"flush"},{"receiver":"std:io","name":"copy"},{"receiver":"std:io","name":"lines"},{"receiver":"std:io","name":"readLine"},{"receiver":"std:io","name":"close"},{"receiver":"std:io","name":"readStdin"},{"receiver":"std:io","name":"readLineStdin"},{"receiver":"std:io","name":"body"},{"receiver":"std:io","name":"Reader"},{"receiver":"std:io","name":"Writer"},{"receiver":"std:io","name":"LineReader"},{"receiver":"std:io","name":"Stdin"},{"receiver":"std:io","name":"IoError"}]}
-->
# Asynchronous byte I/O

`std:io` adapts owned files, buffered file handles, byte buffers and HTTP bodies
into `Reader` and `Writer` handles. `reader` consumes `File`, `BufferedReader`,
`Bytes` or HTTP `Body`; `writer` consumes `File` or `BufferedWriter`. Existing
unread or pending buffered bytes remain in order. `stdout()` and `stderr()` make
asynchronous output writers. `body(reader)` transfers a reader into a backpressured
HTTP body, with producer errors propagated to its consumer.

Every asynchronous operation returns an explicit `Task<Result<T, IoError>>`.
Awaiting completes the operation; inspect the Result or use `unwrap` deliberately.
`IoError` contains `code`, `operation`, `message` and optional `osCode`.
Imported aliases and stored operation values use ordinary checked wrappers.
They preserve lazy task construction, optional arguments and parameter loans.

`read(input, maxBytes?)` returns owned Bytes from one short read (8192 bytes by
default). `readInto(input, destination)` writes into an existing `Bytes` or
`ByteViewMut` prefix and returns its byte count. Bytes after that prefix remain
unchanged. A zero-length destination returns zero without reading; otherwise a
zero count is EOF. Readonly views and overlapping mutable loans are rejected.
`readExact(input, destination)` repeats short reads until the destination is full;
EOF before that point returns `unexpected-eof` and leaves the completed prefix
observable. Both the reader and writable destination stay exclusively borrowed
through actual settlement, including cancellation cleanup.
Await these borrowed operations directly. To start a Job, move owned resources into
an async function and await the Job; started Tasks cannot retain caller loans.

`readAll(input, maxBytes?)` reads until EOF. There is no default total-byte cap.
An explicit bound is nonnegative and must fit the target; overrun detection
consumes at most one extra byte and returns `limit`, without returning partial
accumulated Bytes. Completed input progress is not rolled back.

`write(output, bytes)` may accept a short prefix and returns its count;
`writeAll` repeats until all input is accepted, failing with `write-zero` if the
backend stops making progress. Both retain the writer and readable input loans
through settlement. `flush` reports buffered output failures; it is distinct from
filesystem durability synchronization. `copy(input, output, bufferSize?, maxBytes?)`
uses one reusable positive-sized working buffer, default 8192 bytes, and returns
the copied byte count at EOF. It does not flush or close either handle. With an
explicit total bound it writes the allowed prefix, then reports `limit` if an
additional byte exists; that detection byte is consumed. Working-buffer sizes
are not total-stream caps.

`lines(reader, maxLineBytes?)` consumes a reader and returns
`Result<LineReader, IoError>`. `readLine` returns a UTF-8 string without LF or its
immediately preceding CR, an empty string for an empty line, and `undefined` for
exhausted EOF. An unterminated final line is returned; split UTF-8 sequences are
retained across chunks. Malformed text returns `encoding`. The optional raw-byte
line bound includes terminators; overrun consumes the prefix and one detection
byte, then returns `limit`. The default imposes no line-byte cap. Cancelled line
reads retain their accumulated prefix and unread read-ahead buffer.

`close` consumes Reader, LineReader or Writer and reports release, flush or
shutdown failures. Drop releases resources automatically, but only explicit close
observes those failures. Blocking-backed file and standard-output work runs away
from executor workers. Cancelling a task does not interrupt a system call: scope
drainage waits for actual completion and resource cleanup. An unfinished staged
read remains available for the next read; successfully completed earlier chunks
and writes are not rolled back. Frame storage is retained for HTTP reads; copy
and HTTP output do not accumulate an entire stream.

`stdin()` is deliberately separate and synchronous. `readStdin` returns a short
owned Bytes read; `readLineStdin` follows the same line/EOF and explicit-bound
rules and returns a synchronous Result. Both block the calling thread. There is
no asynchronous stdin Task or hard cancellation guarantee for idle input.

Child stdin/stdout/stderr pipes and owned TCP halves use the same Reader/Writer
contracts. A full `TcpStream` also supports `read`, `readInto`, `readExact`,
`readAll`, `write`, `writeAll`, `flush` and either endpoint of `copy` without
splitting. Exclusive loans reject copying a stream into itself. For independent
concurrent read/write owners, call `stream.split()`. Full-stream
`stream.shutdownWrite()` sends write EOF; synchronous consuming `stream.close()`
releases both sides. The async `std:io.close` overload does not accept full TCP.


Compression endpoints from `std:compression` compose through `read`, `readInto`, `readExact`, `readAll`, `write`, `writeAll`, `flush`, `copy` and `close`. Their concrete overloads preserve `CompressionError | IoError | SpawnError | ThreadError<undefined>`; ordinary endpoints retain `IoError`. `copy` does not finish or close a compression stream. Explicit `finish` returns the consumed transform's original base endpoint; compression `close` finishes and closes it. A single explicit CPU worker/slot can serve both ends of a decode→encode pipeline because no worker is held during async I/O. See [Compression](compression.md) for byte policies, flush/finish behavior and ownership.

Copy supports every pairing of Reader, full TcpStream and CompressionReader with
Writer, a distinct full TcpStream and CompressionWriter. Any transform endpoint
selects the compression error union; other combinations retain IoError only.
See `examples/compression/tcp-pipeline.tsr` for a complete TCP/codec request/reply.
