# Files and byte limits

Import file operations from `std:fs` and byte storage from `std:bytes`.
File operations use the host filesystem on Windows, macOS and Linux.

```tsrust
import { readFile } from "std:fs";
import { length } from "std:bytes";

function main() {
  const bytes = unwrap(readFile("input.bin", 1048576));
  console.log(length(bytes));
}
```

The example accepts an input file up to 1 MiB and prints its byte length.
A larger file returns `FsError` with code `length-range`. `unwrap` throws the
original error so ordinary `try`/`catch` can handle it. An exact-limit file succeeds;
a zero limit accepts an empty file. Limits are checked while reading, including
when the file grows.

`readFile(path)` and `readFile(path, undefined)` read through EOF without an
application-size ceiling. The result owns the bytes, so the file is closed before
the call returns. Keep an explicit `maxBytes` when the input should be bounded.
Available memory and native storage still constrain allocation. Whole-file reads
are unsuitable for an unending input; use `open`, repeated `read` calls and `close`
for incremental processing.

`read(file, length)` requests at most the specified nonnegative number of bytes.
Short reads and empty EOF results succeed. A zero request performs no I/O.
`write` may accept only part of its input; `writeAll` retries until all bytes are
accepted or an error occurs. Byte owners and views are borrowed for writing.
Neither function imposes an additional operation-size ceiling.

`bufferReader` and `bufferWriter` transfer the file into a uniquely owned wrapper.
The default capacity is 8192 bytes. A buffered reader can read ahead; each read
returns at most its current buffered bytes or one refill. Buffered writes can
succeed before reaching the filesystem. Use `flush` or explicit `close` to observe
errors while sending pending output. Closing a buffered writer still releases
the file if flushing fails. These operations do not promise durable storage.

Explicit `close` consumes the handle even when it reports an error. Scope exit
also releases it, discarding cleanup errors. Unix close errors can be reported;
Windows uses the safe standard file destructor, whose OS close errors are not
observable. Buffered flush errors remain observable on Windows.

## Migrating from v0.1

The previous file implementation imposed 64 MiB ceilings on reads and writes,
and `readFile` used 64 MiB when its second argument was omitted. In v0.2,
ordinary file operations have no added size ceiling. Existing calls remain valid,
but callers relying on the old whole-file limit should make it explicit:

```tsrust
const bytes = unwrap(readFile("input.bin", 67108864));
```

There is no equivalent implicit write limit. Check the intended byte count before
calling `writeFile` if a destination must not be changed by oversized input.
Writes can leave partial output on failure; `writeFile` is not an atomic replace.
Service request/body policies are configured separately and are not changed by
these file defaults.
