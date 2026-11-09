# GPUI starter

A native desktop app in one `.tsrx` file: a sidebar gallery of GPUI Kit
components (display, inputs, data, overlays), live English/French switching,
light and dark themes, and an About page that updates the app from GitHub
releases.

```bash
tsrust run examples/gpui-starter/main.tsrx
```

Package it as a downloadable app (see the
[packaging guide](../../docs/guides/tooling/desktop-apps.md)):

```bash
tsrust package examples/gpui-starter/main.tsrx --name "Tsrust Starter" --id dev.tsrust.starter \
  --version 0.1.0 --icon examples/gpui-starter/icon.png --repo owner/name
```

Translations live in `locales/en.json` and `locales/fr.json`.
