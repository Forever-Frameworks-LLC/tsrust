#!/bin/sh
# Build this checkout's compiler, then run the starter with live reload:
# saving any file here rebuilds and restarts the app. Pass --release for an
# optimized build. Press Ctrl+C to stop.
set -e
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
CARGO_INCREMENTAL=0 cargo build -q --manifest-path "$root/Cargo.toml" -p tsrust-compiler --bin tsrust
# Installs the pinned Rust toolchain and downloads the locked crates once;
# later runs find both already present.
"$root/target/debug/tsrust" setup --toolchain --dependencies --yes >/dev/null
cd "$here"
exec "$root/target/debug/tsrust" dev "$@"
