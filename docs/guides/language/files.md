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

## Opening options and file metadata

`openFile(path, options)` accepts an `OpenOptions` record with optional `read`,
`write`, `append`, `create`, `createNew` and `truncate` booleans. Each omitted or
undefined switch is false. `{}` selects no access and fails with `invalid-input`.
`createNew` atomically creates an absent path and overrides `create`/`truncate`;
creation requires write or append access. Invalid combinations fail before
truncation. Permissions, umask, sharing and append behavior follow the host.

`stat(path)` follows the final symbolic link. `lstat(path)` inspects the link
itself. `fileStat(file)` reads metadata through an open plain `File` without
moving its cursor. Each returns an independent `Metadata` snapshot with `isFile`,
`isDirectory`, `isSymlink`, `size`, `readOnly`, and optional `modified`, `accessed`,
`created` timestamps. `readOnly` is the host permission flag, not an effective
access check. Directory/link sizes and timestamp precision depend on the host.

A `FileTime` contains `unixSeconds: int` and `nanoseconds: int`. Seconds round down;
nanoseconds range from 0 through 999999999. One nanosecond before the Unix epoch
is `{unixSeconds: -1, nanoseconds: 999999999}`. There is no calendar-year bound.
An unsupported timestamp reads as `undefined`; other metadata failures return
`FsError`. Sizes or timestamp seconds that cannot fit `int` return `integer-range`.
Snapshots do not refresh when a file changes.

## Directories and path operations

`readDir(path, maxEntries?)` returns owned `DirEntry` records with `name`, `path`,
`isFile`, `isDirectory` and `isSymlink`. Enumeration is eager, nonrecursive and in
host order. Hidden entries are included; dot and dot-dot are omitted. Entry type
checks do not follow links. Paths retain the supplied directory spelling and may
remain relative. Non-Unicode names/paths return `not-unicode`; no lossy replacement
occurs. No iterator handle escapes. Directory contents can change during and
after enumeration.

Omitted or undefined `maxEntries` adds no application cap. An explicit nonnegative
limit that fits native storage fails with `length-range` on the first excess
entry, returning no partial array. Zero accepts an empty directory. Invalid limits
fail before opening the directory.

| Operation | Observable behavior |
| --- | --- |
| `mkdir(path)` | Creates one directory; parent must exist; existing entries fail. |
| `mkdirAll(path)` | Creates missing parents; existing directories succeed; partial creation can remain on error. |
| `rename(path, destination)` | Uses host rename/replacement semantics; cross-volume failure has no copy fallback. |
| `copyFile(path, destination)` | Copies bytes and host permission bits, returning copied byte count; failure can leave partial output. |
| `removeFile(path)` | Removes one file/link; link targets remain intact. |
| `removeDir(path)` | Removes one empty directory; nonempty directories fail. |
| `removeDirAll(path)` | Recursively removes the requested tree without traversing encountered directory symlinks; partial removal can remain on error. |
| `canonicalize(path)` | Resolves an existing path to host absolute spelling and follows links; Windows may return an extended-length prefix. |
| `readLink(path)` | Reads stored link target text without resolving it; relative targets remain relative. |
| `hardLink(path, destination)` | Adds a name for existing file storage; destination must not exist; host filesystem restrictions apply. |

Do not pass source/destination names for the same file to `copyFile`, including
hardlink aliases: host copy behavior can truncate source data. No transactional
rollback, timestamp preservation or implicit recursive deletion is promised.
Filesystem error text/OS codes vary across platforms. Stable categories include
`already-exists`, `not-directory`, `directory-not-empty`, `cross-device`,
`unsupported`, `integer-range`, `not-unicode` and `invalid-utf8` as applicable.

## Text, truncation and explicit synchronization

`readTextFile(path, maxBytes?)` reads strict UTF-8, preserving BOM/newline bytes.
The optional bound counts bytes, using the same no-default-cap and exact-bound
contract as `readFile`. Invalid UTF-8 returns `invalid-utf8`; read/decode errors
take precedence over close errors. `writeTextFile(path, text)` writes exact UTF-8
bytes, creating/truncating the file with no added newline. `appendFile(path, bytes)`
creates if missing and appends owned bytes or the selected byte-view range.
Interrupted/short writes retry, but a multi-write append is not guaranteed atomic
against competing writers. Failure can leave partial writes. These functions
close internally and perform no durability synchronization.

`truncate(file, length)` requires a mutable plain `File` and nonnegative length.
It discards trailing bytes or extends with zeros without moving the cursor.
`syncAll(file)` requests data and metadata synchronization. `syncData(file)` may
omit metadata unrelated to retrieving data; some hosts implement the same work as
`syncAll`. Neither closes the handle, moves the cursor, flushes a separate buffered
writer nor synchronizes containing directory entries. Host durability guarantees
still apply. Buffered files are deliberately excluded from these plain-file APIs.

The complete [filesystem utility example](../../../../examples/filesystem-utilities/main.tsr)
creates a tree, writes/truncates UTF-8 text, copies/renames a file, lists owned
entries and removes its tree. It prints `5`, `he`, `7`, then `3` on separate lines.

## Temporary directories

`createTempDir(parent?, prefix?)` exclusively creates a temporary directory and
returns its owned absolute path. Omitted or undefined parent uses the host
temporary directory; a supplied parent must already exist. Omitted or undefined
prefix retains `.tmp`; an empty prefix is allowed. Six random characters are
appended, using backend collision/randomness defaults. Prefixes cannot contain
NUL or either path separator, or start with a drive prefix. Invalid prefixes
fail with `invalid-input` before accessing the parent. Host filename and path
length restrictions still apply.

Permissions use platform defaults and umask; this does not guarantee a private
0700 directory. Non-Unicode output returns `not-unicode` before automatic cleanup
ownership is transferred. On success the caller owns cleanup: dropping the path
string does not remove the directory. Call `removeDirAll(path)` when finished.
The selected backend attempts automatic cleanup if path conversion fails.

## Plain-file cursors

`SeekOrigin` is the literal-string alias `"start" | "current" | "end"`.
`seek(file, offset, origin)` requires a mutable plain `File`, an exact integer byte
offset and an explicit origin. It returns the resulting absolute byte position.
Start offsets must be nonnegative; a negative start returns `offset-range` before
I/O. Current/end offsets are signed; before-zero and unsupported positions follow
host seek errors. Ordinary files allow seeking beyond EOF; subsequent reads can
return empty. Seeking alone does not change contents or extend the file.

`position(file)` queries the current byte position without moving the cursor.
It also requires a mutable plain-file borrow, following the seek interface.
Neither API accepts buffered handles: their cached data would need explicit
invalidation. Neither flushes, synchronizes durable storage or adds an application
offset cap. Returned positions must fit `int`; if a successful host seek reaches
an unrepresentable position, `integer-range` is returned after the cursor moved.
The file remains open after success and failure.
