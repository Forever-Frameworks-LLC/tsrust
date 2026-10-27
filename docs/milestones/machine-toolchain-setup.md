# Explicit toolchain inspection and preparation

Implemented `tsrust doctor [--json] [--native]` and `tsrust setup --toolchain [--yes]`.
Doctor reports the exact pinned rustc/Cargo, host OS/architecture and optional minimal
compile/link/execution result. Application dependencies remain explicitly unchecked.
Doctor performs no network acquisition. Setup requires consent before invoking
rustup's exact-pin minimal-profile installation with --no-self-update; no default,
override, profile-setting or installer-script commands are issued.

Acceptance on Windows x86_64:

- Bin unit test 1/1: explicit flags, duplicate/unknown rejection and exact upstream
  command construction; no default-changing commands.
- CLI integration tests 2/2: isolated missing RUSTUP_HOME reports unavailable tools
  without downloading them; noninteractive setup requires --yes; a compiled local
  rustup process fixture verifies success, pinned-version verification, failed-install
  propagation and no verification step after installer failure. No live toolchain
  acquisition or global configuration changes occurred.
- Actual `target/debug/tsrust.exe doctor --native --json`: exit zero, Rust/Cargo
  1.96.1 found, minimal host compile/link/run passed, networkUsed false. Application
  dependency readiness remains unchecked.

Logs are disposable under target/setup-toolchain-current.log and
target/doctor-native-current.json. The integration build took 10m31s; this is a
local latency observation, not a matched benchmark measurement. Embedded dependency
payload removal/shared Cargo-cache migration is the next distribution priority.
The compiler's test-only UnionArena::len helper is now cfg(test), removing its
normal-library dead-code warning. Existing authoring-only UI feature warnings remain
separate unfinished backend-boundary work.

This milestone does not implement runtime source-cache preparation, a CLI/LSP
installer, release trust, self-update, portable-export migration, cross-platform
acceptance, the server-owned API registration replacement, or a docs UI startup fix.
The owner's editor Problems list has not been certified clean. No release, push,
package publication, benchmark measurement or installed-tool replacement occurred.
