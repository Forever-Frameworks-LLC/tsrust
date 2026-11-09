# GPUI starter

A native desktop app in one `.tsrx` file: a sidebar gallery of GPUI Kit
components (display, inputs, data, workspace, overlays), live English/French
switching, light and dark themes, and an About page with in-app updates.

## Run it with live reload

From the repository root:

```bash
./examples/gpui-starter/dev.sh
```

The script builds this checkout's compiler, then runs `tsrust dev` here. Saving
`main.tsrx` or a locale file rebuilds and restarts the window (about 20 seconds;
the first build prepares the GPUI runtime and takes a few minutes). Press
Ctrl+C to stop. Add `--release` for an optimized build.

With a current `tsrust` on your PATH, `tsrust dev` in this folder does the same.

## Package it

See the [packaging guide](../../docs/guides/tooling/desktop-apps.md):

```bash
tsrust package main.tsrx --name "Tsrust Starter" --id dev.tsrust.starter \
  --version 0.1.0 --icon icon.png --repo owner/name
```

Translations live in `locales/en.json` and `locales/fr.json`.
