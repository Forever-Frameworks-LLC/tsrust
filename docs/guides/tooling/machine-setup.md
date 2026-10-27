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

Without `--yes`, toolchain setup asks before invoking rustup; noninteractive use requires
that flag. Setup requests rustup's minimal profile for the exact pin and disables
rustup self-update for this invocation. It does not select a global default,
change directory overrides, run downloaded installation scripts, or install
platform SDKs/linkers. Windows requires the appropriate Visual Studio C++ build
tools; macOS requires its command-line build tools; Linux requires the platform
C toolchain and linker. Run `doctor --native` to test the actual host combination.

Prepare compiler-owned backend dependency sources explicitly:

```sh
tsrust setup --dependencies --yes
# Validate/populate from already cached archives without network access:
tsrust setup --dependencies --offline
# Prepare both tools and sources, with download consent:
tsrust setup --toolchain --dependencies --yes
```

Source preparation delegates to `cargo fetch --locked`. The backend profile excludes
legacy rendering dependencies. Ordinary backend builds use Cargo's shared home,
exact locks and `--offline`; they create no vendor copy or private Cargo home.
A missing source produces a setup instruction rather than an automatic download.
The compiler hashes Cargo configuration (including nested and optional includes)
and environment inputs into artifact keys;
configuration changes invalidate executable/runtime artifacts while the stable Cargo
target remains reusable. Inputs are checked again before publishing cached artifacts.
Raw configuration contents and environment credentials are
not written into identity files. Package resolution and source storage remain Cargo's
responsibility.

`--offline` applies to dependency preparation and cannot be combined with toolchain
installation. Offline preparation needs no download consent. The shared Cargo home
follows CARGO_HOME, otherwise the normal user .cargo directory. Its configuration
and ancestor configuration of TSRUST_CACHE_DIR (by default the project's
target/tsrust-native) apply to backend support builds. The pinned toolchain and
compiler-owned runtime profiles remain explicit. Existing global defaults and
configuration files are never rewritten.

Portable source exports continue to carry their vendors and can be generated without
Rust installed, then rebuilt after relocation with an empty registry cache. Retained
UI builds and resolved custom binding graphs still use the legacy vendor path. The
CLI still embeds the portable legacy bundle; eliminating that payload and migrating
binding graphs are separate unfinished steps. These commands are not yet a CLI/LSP
installer or self-update flow.

The [rustup profile documentation](https://rust-lang.github.io/rustup/concepts/profiles.html)
defines the minimal profile. Its [basic usage documentation](https://rust-lang.github.io/rustup/basics.html)
documents invocation-scoped `--no-self-update`.

[Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html) defines
configuration discovery and nested/optional includes. Tsrust uses the already locked
toml_edit parser to inspect those include paths; Cargo still parses and applies the
build configuration itself. [cargo fetch](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html)
provides explicit source acquisition before offline use.
