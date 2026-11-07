<!-- tsrust-docs
{"id":"reference-standard-library-fs","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:fs","name":"SeekOrigin"},{"receiver":"std:fs","name":"File"},{"receiver":"std:fs","name":"FsError"},{"receiver":"std:fs","name":"open"},{"receiver":"std:fs","name":"read"},{"receiver":"std:fs","name":"close"},{"receiver":"std:fs","name":"create"},{"receiver":"std:fs","name":"write"},{"receiver":"std:fs","name":"writeAll"},{"receiver":"std:fs","name":"readFile"},{"receiver":"std:fs","name":"writeFile"},{"receiver":"std:fs","name":"BufferedReader"},{"receiver":"std:fs","name":"bufferReader"},{"receiver":"std:fs","name":"BufferedWriter"},{"receiver":"std:fs","name":"bufferWriter"},{"receiver":"std:fs","name":"flush"},{"receiver":"std:fs","name":"OpenOptions"},{"receiver":"std:fs","name":"FileTime"},{"receiver":"std:fs","name":"Metadata"},{"receiver":"std:fs","name":"DirEntry"},{"receiver":"std:fs","name":"openFile"},{"receiver":"std:fs","name":"stat"},{"receiver":"std:fs","name":"lstat"},{"receiver":"std:fs","name":"fileStat"},{"receiver":"std:fs","name":"readDir"},{"receiver":"std:fs","name":"mkdir"},{"receiver":"std:fs","name":"mkdirAll"},{"receiver":"std:fs","name":"rename"},{"receiver":"std:fs","name":"copyFile"},{"receiver":"std:fs","name":"removeFile"},{"receiver":"std:fs","name":"removeDir"},{"receiver":"std:fs","name":"removeDirAll"},{"receiver":"std:fs","name":"canonicalize"},{"receiver":"std:fs","name":"readLink"},{"receiver":"std:fs","name":"hardLink"},{"receiver":"std:fs","name":"appendFile"},{"receiver":"std:fs","name":"readTextFile"},{"receiver":"std:fs","name":"writeTextFile"},{"receiver":"std:fs","name":"truncate"},{"receiver":"std:fs","name":"syncAll"},{"receiver":"std:fs","name":"syncData"},{"receiver":"std:fs","name":"createTempDir"},{"receiver":"std:fs","name":"seek"},{"receiver":"std:fs","name":"position"}]}
-->
# Fs

Read and write host files on Windows, macOS and Linux. Unique File handles support
explicit open options, cursor reads/writes, seek/position, truncation and explicit
synchronization. Directory entries and metadata are independent owned snapshots;
FileTime preserves exact signed epoch seconds and normalized nanoseconds.

Whole-file reads have no application-size default ceiling. Use optional maxBytes
or maxEntries when a caller needs a bound. Text reads validate strict UTF-8 and
preserve BOM/newline bytes. Paths, overwrite behavior and I/O errors follow the
host; non-Unicode output fails explicitly. Temporary directories are exclusively
created and transfer cleanup responsibility to the caller.

In v0.2, ordinary file reads and writes no longer impose v0.1's implicit
64 MiB ceiling. Explicit bounds, exact integer ranges and host I/O failures
still apply.

The generated public filesystem reference supplies all signatures, argument help,
ownership rules, platform behavior and executable examples from the shared API
catalog.
