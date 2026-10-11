# S09 contract audit

Audit source: `51ac0b8c` plus the documented read-range/platform clarification.
S09 covers synchronous host foundations. Extended filesystem, process/subprocess,
wall-clock/calendar and asynchronous timer surfaces remain assigned to their existing
H02/H03/H12/S14 gates. This audit does not claim those exports are implemented.

The public catalog has 28 exports across the five namespaces below (23 functions
and five types). Checked bindings, runtime implementations, catalog examples and
focused fixtures supply the contracts. All APIs retain their documented explicit
ownership; no hidden shared file owners or view copies are introduced.

| Export | Arguments/defaults and behavior | Errors, ownership and boundary evidence |
|---|---|---|
| `fs.File` | Private unique descriptor; scope release, explicit close | No clone/literal/inspection; S07 resource lifecycle and native file cleanup |
| `fs.FsError` | Stable code/operation, explanatory message, optional OS code | Message text is not stable; native file catalog |
| `fs.open` | Required borrowed path; existing read-only file, cursor zero | Host resolution/permissions/NUL errors; native resource lifecycle |
| `fs.create` | Required borrowed path; write-only create/truncate | Missing parent/permission errors; preserves existing permissions; native file writes |
| `fs.read` | File or BufferedReader, required 0–64 MiB length; zero avoids I/O | Short read/EOF succeed, interrupted calls retry; mutable borrow and owned bytes; full/buffered file fixtures |
| `fs.write` | File or BufferedWriter, borrowed Bytes/view, 0–64 MiB | Partial count, no rollback, input unchanged; native file writes |
| `fs.writeAll` | Same receiver/input bounds; accepts complete input | Short/interrupted retry, zero-progress error; native file writes |
| `fs.close` | Consumes File/BufferedReader/BufferedWriter, including errors | No close retry; writer flush before release, first error wins; native file cleanup/buffered files |
| `fs.readFile` | Borrowed path, optional maxBytes default 64 MiB, range 0–64 MiB | Reads through EOF; bounded excess detection, closes on success/failure; native full files |
| `fs.writeFile` | Borrowed path and bytes/view, at most 64 MiB | Validates size before truncation; writes fully and closes, no atomic replacement/durability promise; native full files |
| `fs.BufferedReader` | Private unique descriptor plus owned buffer | Read-ahead; unread bytes discarded at close; no clone/share; native buffered files |
| `fs.bufferReader` | Consumes File, capacity default 8192, range 1–1048576 | Invalid capacity/allocation errors still consume and release file; native buffered backend/files |
| `fs.BufferedWriter` | Private unique descriptor plus owned buffer | Explicit close reports flush failure, scope cleanup discards errors; native buffered files |
| `fs.bufferWriter` | Consumes File, same capacity contract as reader | Does not change access permissions; invalid construction releases file; native buffered backend/files |
| `fs.flush` | Mutable BufferedWriter; retains handle | Removes written prefix on failure, preserves unwritten suffix; no durability synchronization; native buffered backend |
| `path.join` | Zero or more borrowed string segments; empty result becomes dot | Lexical target-platform behavior, no filesystem access/containment guarantee; path join fixtures |
| `path.basename` | Borrowed path, optional case-sensitive suffix | Independent string, target-platform separators; native path cases/catalog |
| `path.dirname` | One borrowed path | Independent lexical parent; lexical-path fixtures |
| `path.extname` | One borrowed path | Independent extension, documented dotfile/terminal-dot cases; lexical-path fixtures |
| `path.isAbsolute` | One borrowed path | Root-syntax boolean, no filesystem query; lexical-path fixtures |
| `path.normalize` | One borrowed path | Independent lexical normalization, no symlink resolution; lexical-path fixtures |
| `path.relative` | Two borrowed paths; empty/relative inputs resolve from cwd | Independent lexical result, documented platform/drive boundaries; relative-path fixtures |
| `os.platform` | No arguments; executable target OS name | Unsupported selectors fail native compilation; OS target mapping tests |
| `os.arch` | No arguments; executable target architecture name | Same target-selection boundary, not physical-host detection; OS identity fixtures |
| `os.endianness` | No arguments; BE or LE for executable target | No state mutation; OS identity fixtures |
| `env.EnvError` | Owned code/message record | InvalidName/NotUnicode; no secret value in diagnostic; environment backend/catalog |
| `env.get` | Borrowed name; owned string or undefined | Empty remains distinct from missing; rejects empty/= /NUL names and non-Unicode values; environment fixtures |
| `time.monotonicNow` | No arguments; process-relative fractional milliseconds | Equal readings allowed, platform resolution/suspend behavior, no handle; monotonic-time fixtures/catalog |

File operations are implemented only for macOS/Linux targets; this family acceptance
executes on macOS. Other file targets receive a native compile diagnostic. Filesystem
fixtures use disposable directories, not a runtime capability sandbox. Normal host
permissions and symlink resolution apply. Path tests exercise explicit platform
algorithms; that does not certify complete native Windows filesystem support.

Audit correction: `read` hover omitted its actual numeric bound and zero-length
behavior. The catalog now states both, plus the existing file-platform limitation.
No runtime semantics changed.

## Family measurement scope

Retain accepted lexical-path/path-relative and S07 read/close measurements. Measure
nine pending registered workloads: `native_file_write`, `native_file_write_all`,
`native_file_full_write_read`, `native_file_buffered_write_read`, `std_env_get`,
`std_time_monotonic_now`, `std_os_platform`, `std_os_arch`, `std_os_endianness`.
Use native and Node-after-tsc with the unchanged manifest sample counts (three
processes, eight warmups, 31 execution samples and 11 startup samples). Preserve each
completed workload record independently so a later failure cannot erase progress.
Review rates and source-matched historical signals; do not relabel old workloads as
newly timed. Complete regressions and consumer/editor follow-ups remain required
before S09 family acceptance. No S09 family acceptance is claimed by this audit.
