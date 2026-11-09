Write backend services with TypeScript-style syntax and Rust ownership. This
release completes the v0.2.0 standard library: the APIs a backend program needs
from Node.js or Rust's standard library, translated to their direct Rust
equivalents and checked by the compiler and editor.

New and expanded modules:

- Files and bytes: `std:fs` open options, metadata, directories, cursors, links,
  temporary directories and text helpers; `std:bytes` codecs (UTF-8, hex,
  base64/base64url) and binary records; `std:io` readers, writers and copies
  shared across files, sockets, processes, HTTP bodies and compression.
- Host: `std:path` resolve/parse/format, `std:env` variables and directories,
  `std:os` parallelism and hostname, signals, `std:process` child processes,
  `std:net` TCP/UDP and `std:dns` system lookups.
- Web: `std:url` WHATWG URLs and query parameters; HTTP client timeouts,
  redirects, proxies, uploads and forms; source-owned middleware; SSE and
  WebSocket servers and clients.
- Text and data: reusable `std:regex`, an explicit-locale `std:locale` subset
  (collation, segmentation, number formatting), gzip/deflate `std:compression`,
  and more collection, numeric and date operations.
- Work and coordination: owned lazy iterator pipelines, task/timer/channel
  helpers with cancellation tokens, and `std:crypto` hashes, HMAC, HKDF, random
  bytes and Argon2 password hashing on explicit CPU pools.
- Operations: `std:log` structured logging with request context, and
  `std:testing` hooks, deep assertions, rejection checks, output capture
  (`captureOutput`, `startOutputCapture`/`finishOutputCapture`) and profiling.
- Rust packages: practical adapters for locked Rust crates, including generated
  nominal record projections.

Behavior changes from v0.1:

- File reads and writes no longer impose a 64 MiB ceiling. Use the optional
  `maxBytes`/`maxEntries` controls for an explicit bound.
- HTTP clients follow the selected backend defaults: no total request timeout,
  bounded redirect following and proxy discovery. Response readers no longer stop
  at 1 MiB; set `bodyLimit`, `timeoutMs`, `followRedirects` or `proxy` to opt in.
- HTTP servers no longer cap forms at 128 fields, multipart bodies at 128 parts,
  configured body sizes at 64 MiB, deadlines at one hour, routes at 4096 bytes or
  cookie lifetimes at 400 days. Configured limits stay in application source.
- Async `main` runs on an 8 MiB main-thread stack on every host.

Every public export has a reviewed ledger entry. Every non-UI catalog entry has an
example that runs natively against its recorded output. The performance page
compares Tsrust with hand-written Rust and Node.js after tsc across the accepted
v0.2 workloads, plus HTTP text, JSON, parsing, SQLite and CPU rendering. These
are workload-level results, not universal application speedups.

The distribution includes the `tsrust` CLI and `tsrust-lsp` language server, plus
the separate VS Code extension. Applications expose HTTP APIs; bring your own UI
and generate TypeScript or Rust clients with `tsrust api generate`.

Use `tsrust new service --app` to create a backend project, `tsrust new hello` for
a console program, or `tsrust` for the interactive launcher.

Documentation: https://forever-frameworks-llc.github.io/tsrust/

Download the archive matching your host, verify it against `SHA256SUMS`, and put
both `bin/` executables on `PATH`, or install a managed copy with
`tsrust toolchain install --root NEW_DIRECTORY`; managed installations update with
`tsrust toolchain check` and `tsrust toolchain update`. Cargo-owned installations
update through Cargo. Install the separate VSIX in VS Code. Native compilation
requires Rustup with Rust 1.96.1 and a system linker; archives do not include Rust.
Linux archives target glibc systems (built on Ubuntu 24.04), not static musl.

Tsrust is experimental. Language and library APIs may change between minor
versions. No platform developer signing or Apple notarization is provided.
Large-workspace and multi-hour language-server memory behavior is not yet
verified. The original Tsrust UI is frozen and is not part of this release.

See the installation and release guides in the tagged source for prerequisites,
verification commands and test coverage.
