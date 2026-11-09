# GPUI starter

A native desktop app in one `.tsrx` file: a sidebar gallery of GPUI Kit
components (display, inputs, data, workspace, overlays), live English/French
switching, light and dark themes, and an About page with in-app updates.

## Run it with live reload

From the repository root:

```bash
./examples/gpui-starter/dev.sh
```

The script builds this checkout's compiler, installs the pinned Rust toolchain
and downloads the locked crates if they are missing (first run only), then runs
`tsrust dev` here. Saving
`main.tsrx` or a locale file rebuilds and restarts the window (about 20 seconds;
the first build prepares the GPUI runtime and takes a few minutes). Press
Ctrl+C to stop. Add `--release` for an optimized build.

With a current `tsrust` on your PATH, `tsrust setup --toolchain --dependencies --yes`
once and then `tsrust dev` in this folder do the same.

System requirements, from GPUI Kit's setup guide:

- **macOS:** Xcode command line tools.
- **Windows 10+:** Visual Studio 2022 Build Tools with the "Desktop development
  with C++" workload (MSVC and a Windows SDK), and CMake on `PATH`.
- **Linux:** packages verified on Ubuntu 24.04, plus a Wayland or X11 session
  with a working Vulkan driver:

  ```bash
  sudo apt install -y gcc g++ clang libfontconfig-dev libwayland-dev libwebkit2gtk-4.1-dev libxkbcommon-x11-dev libx11-xcb-dev libssl-dev libzstd-dev vulkan-validationlayers libvulkan1
  ```

## Package it

See the [packaging guide](../../docs/guides/tooling/desktop-apps.md):

```bash
tsrust package main.tsrx --name "Tsrust Starter" --id dev.tsrust.starter \
  --version 0.1.0 --icon icon.png --repo owner/name
```

Translations live in `locales/en.json` and `locales/fr.json`.
