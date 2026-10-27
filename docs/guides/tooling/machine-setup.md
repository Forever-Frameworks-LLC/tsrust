# Inspect and prepare native build tools

`tsrust check` and ordinary Rust source emission do not need Rust installed.
Native build, run, test and dev use Tsrust's exact pinned Rust toolchain.

```sh
tsrust doctor
tsrust doctor --json
tsrust doctor --native --json
```

Doctor never installs tools or fetches dependencies. Its versioned JSON report
identifies the host OS/architecture, required Rust version, and available pinned
rustc/Cargo. `--native` additionally compiles, links and executes a tiny program
in an automatically removed temporary directory. This checks host build tools,
not application dependencies or every optional runtime feature. Those remain
explicitly unchecked. Exit status is zero when the requested tool checks pass,
one when they fail. It does not certify that every application is ready to build.

Install [rustup](https://rustup.rs) first if it is missing, then explicitly prepare
the pinned compiler:

```sh
tsrust setup --toolchain
# Unattended, with permission to download the pinned toolchain:
tsrust setup --toolchain --yes
```

Without `--yes`, setup asks before invoking rustup; noninteractive use requires
that flag. Setup requests rustup's minimal profile for the exact pin and disables
rustup self-update for this invocation. It does not select a global default,
change directory overrides, run downloaded installation scripts, or install
platform SDKs/linkers. Windows requires the appropriate Visual Studio C++ build
tools; macOS requires its command-line build tools; Linux requires the platform
C toolchain and linker. Run `doctor --native` to test the actual host combination.

This is toolchain preparation only. Application source-cache preparation and
portable vendoring are separate work; existing runtime builds still use the
bundled dependency path until that migration is accepted. It is not a CLI/LSP
installer or self-update command.

The [rustup profile documentation](https://rust-lang.github.io/rustup/concepts/profiles.html)
defines the minimal profile. Its [basic usage documentation](https://rust-lang.github.io/rustup/basics.html)
documents invocation-scoped `--no-self-update`.
