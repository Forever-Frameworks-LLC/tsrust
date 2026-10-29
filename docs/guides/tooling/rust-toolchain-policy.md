# Rust toolchain policy

[Compiler architecture](../language/compiler-and-modules.md) · [Development workstream](toolchain-and-development.md) · [Repository setup](../../../README.md)

## Current decision: pin both compilation stages

Use Rust **1.96.1**, the installed release used to validate the current compiler and ownership corpus. The compiler is written in Rust, and it also generates Rust; both stages need a deliberate version policy.

| Stage | Selection |
| --- | --- |
| Building and testing `tsrust` | Root `rust-toolchain.toml` selects 1.96.1 with the minimal profile, rustfmt, and Clippy. |
| Compiling generated `.rs` programs | `tsrust build/run` explicitly invokes `rustup run 1.96.1 rustc`. |
| Analysis and Rust emission | `tsrust check/emit` and the analysis library require no external compiler. |

The CLI verifies the selected backend's reported version before compiling generated code. It fails with setup instructions if the backend is missing or does not match. It does not fall back to ambient `rustc`, modify the user's default toolchain, or automatically download a compiler.

The CLI finds Rustup on `PATH` first, then in `$CARGO_HOME/bin` or, when
`CARGO_HOME` is unset, `$HOME/.cargo/bin` (`USERPROFILE` on Windows). This allows
VS Code launched from the desktop to run programs when its inherited `PATH`
omits Cargo's bin directory. The same pinned backend is used in every case.

The backend version is compiled into the CLI as `RUST_BACKEND_VERSION`. A test checks that it agrees with the repository pin. This ensures an installed CLI retains its backend requirement when run outside this repository.

Backend programs have two explicit compilation profiles. `tsrust build/run` defaults to `-C opt-level=0`; `--release` selects `-C opt-level=3`. Both pass `-C overflow-checks=on`. These settings are independent of the Cargo profile used to build the compiler executable. The release profile matches the runtime benchmark optimization level and preserves the language's checked arithmetic and bounds behavior. `check` and `emit` do not accept `--release`, since they do not compile a native program.

The retained native UI has a separate development profile (application level 1
and dependency level 2). It is outside the active backend product scope; do not
infer its settings from the backend defaults.

Rustup's repository toolchain file selects a default, but explicit command-line and environment overrides can take precedence. The backend therefore uses an explicit versioned command instead of relying on the current working directory. See the [official Rustup override documentation](https://rust-lang.github.io/rustup/overrides.html).

Cargo's `rust-version` field expresses a minimum supported version; it is not an exact toolchain lock. No broader minimum-version compatibility promise is made by this prototype. See the [Cargo manifest reference](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field).

## Setup and verification

Contributors need Rustup, the pinned toolchain, a platform linker and the locked
dependency sources. Follow [contributor setup](../../../CONTRIBUTING.md#set-up)
for separate workspace/backend/portable graphs and targeted checks. Offline builds
require those sources to be prepared; this is not a dependency-free workspace.

Rustup may install missing tools when invoked under the repository pin. Ordinary
`tsrust check`, `emit`, `build` and `run` do not install tools or fetch sources.
Explicit `tsrust setup` can prepare the pinned toolchain and dependencies with
consent; see [native-tool setup](../../../website/src/content/docs/tools/rust-setup.md).

Tests cover agreement between the two pins, compilation outside the repository with conflicting directory/environment overrides, and missing-backend errors. Native acceptance and handwritten-witness checks also select the pinned Rust release explicitly.

## Upgrade policy

Upgrade Rust intentionally alongside a compiler change:

1. Install the candidate toolchain without replacing the user's default.
2. Update `rust-toolchain.toml` and `RUST_BACKEND_VERSION` together.
3. Run the full source acceptance, native-output, diagnostic, and witness suites; run formatting and Clippy.
4. Rerun frontend and native-build measurements separately. Record any changed diagnostics, output, or generated-code requirements.
5. Release the new backend requirement with the matching `tsrust` version. Keep previous toolchains available for older compiler releases.

The release pin also fixes the associated Rust standard library for the selected target. It does not alone make builds bit-for-bit reproducible: OS SDKs, system linkers/libraries, target settings, build flags, and future dependencies must also be controlled. Keep `Cargo.lock` with the source.

## Distribution boundaries

The [managed installer](../../../website/src/content/docs/tools/install-and-update.md)
installs and updates a verified CLI/LSP pair. It does not install Rust or change
Rustup defaults. `tsrust setup --toolchain --yes` explicitly prepares the pinned
backend through Rustup; a system linker/SDK remains a separate prerequisite.

Bundling Rust binaries/sysroots into a Tsrust-managed toolchain is not implemented.
Portable source packs contain verified build inputs and notices, not Rust compiler
binaries. Keep those two distribution concerns separate. Shipping only `rustc`
would not provide a complete native toolchain.
