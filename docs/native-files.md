# Owned file resources

Import `File`, `FsError`, `open`, `read` and `close` from `std:fs`.
`open(path)` borrows a string path and opens an existing file for reading, returning
`Result<File, FsError>`. It does not create, truncate or write files. Relative paths
are resolved against the process working directory. Opening consumes no path owner.

`read(file, length)` mutably borrows the file, reads from its current cursor and
returns `Result<Bytes, FsError>`. Length is a required integer from zero through
67,108,864 bytes (64 MiB), inclusive. Zero returns an empty owned buffer without
I/O. A short read or EOF succeeds with the bytes actually obtained; this is not an
exact-length read. Repeated reads advance the same cursor. Length validation and
reservation happen before I/O, and interrupted reads retry. Returned bytes are
independent owners and can outlive the file.

`close(file)` consumes the file and returns `Result<void, FsError>`. Consumption
also happens on `Err`: later use and double close are compile-time errors. Closing
does not flush data to durable storage and is not retried after an OS close error.
If an owner leaves scope without explicit close, deterministic disposal releases
the handle once and suppresses close errors. Use explicit close when its error
must be observed. Moves and owning callbacks transfer cleanup responsibility;
early returns, failure and `finally` preserve lexical cleanup.

`FsError` exposes `code`, `operation`, `message` and optional integer `osCode`.
Codes include `not-found`, `permission-denied`, `invalid-input`, `interrupted`, `is-directory`,
`would-block`, `io`, `length-range` and `allocation`. OS-derived messages vary by
platform. The `operation` identifies `open`, `read` or `close`. Validation and
allocation errors need not have an OS code.

`File` cannot be constructed as a record, cloned, compared, used as a map key or
implicitly shared. Reading needs mutable access; closing needs ownership. The
compiler rejects consuming a readonly borrow. Bytes and views have no close API.

The [read/close example](../crates/tsrust-compiler/tests/fixtures/native-files/read-close.tsr)
expects an `input.bin` containing bytes `[0, 65, 255, 66]`. It verifies zero-length
reads, rejection of a negative length, cursor advancement, a short read, EOF and
explicit close. It prints nothing when all assertions pass. The
[cleanup fixture](../crates/tsrust-compiler/tests/fixtures/native-files/cleanup.tsr)
is exercised with isolated files and instrumented exact-release checks.

Native integration is verified on macOS. Linux close semantics have been reviewed,
but Linux execution is not claimed. Other targets are guarded. File writes, seeking,
metadata and buffered streams remain part of the broader S09 host roadmap.
See [S07 acceptance](milestones/s07-resources-bytes.md) for correctness and measurements.
