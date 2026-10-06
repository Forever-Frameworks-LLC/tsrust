# Rust toolchain policy

[Compiler architecture](./compiler-and-modules.md) · [Development workstream](./toolchain-and-development.md) · [Repository setup](../README.md)

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

Generated programs have two explicit compilation profiles. `tsrust build/run` defaults to `-C opt-level=0`; `--release` selects `-C opt-level=3`. Both pass `-C overflow-checks=on`. These settings are independent of the Cargo profile used to build the compiler executable. The release profile matches the runtime benchmark optimization level and preserves the language's checked arithmetic and bounds behavior. `check` and `emit` do not accept `--release`, since they do not compile a native program.

The [profile integration proof](../benchmarks/runtime/diagnostics/2026-10-01T21-40-05-681Z-cli-release-profile-proof.json) records all 22 passing CLI acceptance tests, formatting, Clippy, and the release compiler build. Fresh emission and adapter checks for all 50 runtime workloads match the published production run exactly. This CLI change supplies optimized native builds; it does not create new throughput measurements or resolve the remaining slice-storage gaps.

Rustup's repository toolchain file selects a default, but explicit command-line and environment overrides can take precedence. The backend therefore uses an explicit versioned command instead of relying on the current working directory. See the [official Rustup override documentation](https://rust-lang.github.io/rustup/overrides.html).

Cargo's `rust-version` field expresses a minimum supported version; it is not an exact toolchain lock. No broader minimum-version compatibility promise is made by this prototype. See the [Cargo manifest reference](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field).

## Setup and verification

Contributors need Rustup and the pinned toolchain:

```sh
rustup toolchain install 1.96.1 --profile minimal --component rustfmt --component clippy
rustup show active-toolchain
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
```

The install command may need network access. Once the toolchain and components are installed, this dependency-free workspace can build and test offline. Rustup itself may install missing components when a contributor invokes Cargo under the repository pin; that is separate from the `tsrust` CLI, which never requests installation.

Tests cover agreement between the two pins, compilation outside the repository with conflicting directory/environment overrides, and missing-backend errors. Native acceptance and handwritten-witness checks also select the pinned Rust release explicitly.

## Upgrade policy

Upgrade Rust intentionally alongside a compiler change:

1. Install the candidate toolchain without replacing the user's default.
2. Update `rust-toolchain.toml` and `RUST_BACKEND_VERSION` together.
3. Run the full source acceptance, native-output, diagnostic, and witness suites; run formatting and Clippy.
4. Rerun frontend and native-build measurements separately. Record any changed diagnostics, output, or generated-code requirements.
5. Release the new backend requirement with the matching `tsrust` version. Keep previous toolchains available for older compiler releases.

The release pin also fixes the associated Rust standard library for the selected target. It does not alone make builds bit-for-bit reproducible: OS SDKs, system linkers/libraries, target settings, build flags, and future dependencies must also be controlled. Keep `Cargo.lock` with the source.

## Bundling: planned distribution work

Do not vendor compiler binaries into this source repository. For the prototype, reuse Rustup's versioned installations. This avoids building a downloader, update mechanism, and platform-specific distribution system before the language is proven.

For an end-user release, prefer an installer-managed backend associated with each `tsrust` release. It could reuse a verified compatible installation or download a versioned backend to an application-managed directory. Offline distributions could include the same artifacts. The CLI would resolve that managed compiler directly rather than require users to manage Rustup.

That work must define artifact checksums/provenance, supported host and target combinations, standard-library/sysroot contents, linker/SDK requirements, redistribution notices, offline packaging, rollback, and cache cleanup. Bundling only the `rustc` executable would not provide a complete native toolchain.

Bundling is not implemented. The current contract requires Rustup for native builds; the compiler, analyzer, and tests remain Rust applications.
