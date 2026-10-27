# Missing native linker diagnostic

`build` and `run` require both the pinned Rust backend and the platform's native linker. The compiler can find rustup under the Cargo home even when the editor or shell omits that directory from PATH. Finding rustup does not make a missing native linker available.

When Rust reports that its selected linker was not found, Tsrust now reports:

> Native linker unavailable. Install the platform build tools and ensure the linker is on PATH. check/emit do not require a linker.

The diagnostic includes Rust's actual backend error, including the linker name. No executable is produced or cached after the failure. Install the platform build tools or restore the linker directory to PATH before retrying. The compiler does not download or install those tools. `check` and `emit` still work without a native linker.

Other generated-code rejections retain the compiler-defect message and now include the backend's error details. A missing linker is recognized from Rust's explicit `error: linker ... not found` diagnostic, rather than from every failure that mentions linking; genuine generated-code errors are not relabeled as setup failures.

## Evidence

- `target/editor/missing-linker-red.log`: the real pinned backend with an empty PATH originally produced the misleading compiler-defect message.
- `target/editor/missing-linker-acceptance.log`: all 23 acceptance tests pass, including the real missing-linker witness, missing-backend setup, debug/release execution and pinned backend selection.
- `target/editor/missing-linker-generated-rejection.log`: the additional generated-code rejection test passes and retains E0308 details without the missing-linker message. Together the retained runs cover all 24 current acceptance tests.
- `target/editor/missing-linker-build-debug.log` and `missing-linker-build-release.log`: refreshed compiler/editor consumers.

The exact baseline is `archive/missing-linker-diagnostic-start`. This is a CLI diagnostic repair, not a new runtime operation or numbered feature gate. No new workload or timing is needed to compare a failed toolchain launch; existing benchmark sources and SQLite measurements remain unchanged. Gate 127 remains the next feature gate.
