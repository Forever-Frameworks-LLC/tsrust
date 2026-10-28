# CLI application scaffold

Implementation in progress after the owner's production reset. This record covers
the real creation path, not full application-platform completion.

## Behavior

`tsrust new <path> --app` creates app/, packages/api, tests/ and web/, with a root
pnpm workspace. `--target backend` omits frontend manifests/tooling. Invalid,
repeated or unsupported target choices fail before writes. Existing destinations
are never replaced. New files use exclusive creation; a later write/generation
failure reports the partial destination rather than deleting it or claiming success.
Legacy console and explicit --ui creation remain available. Interactive creation
offers the backend/web application first and labels the old UI as legacy.

The default is a minimal landing page and typed public greeting, not the task demo.
The native API listens on loopback 4318 with bounded HTTP settings. Vite listens
on loopback 5173 with a same-origin /api proxy and strict port selection. Initial
backend/frontend dev commands are separate; automatic coordinated regeneration/
orchestration, desktop/mobile creation and import/migration commands are pending.

SDK and native dispatcher generation reuse the existing checked manifest, codecs,
compatibility history and output ownership guards. Application-owned lib/api.ts
centralizes the public HTTP/query setup. The greeting's string parameter is used
directly: api.greet(name), or its generated mutation hook. Root workspace linking
keeps SDK changes live without copied-package installation. Ordinary package,
script and source ownership remains with the application author.

tsrust.integration.json records template version, selected targets and initial
editable-file hashes for future scoped migration comparisons. It is not a policy
preventing user edits and is not an implemented migration command. Generated API
ownership markers remain separate and authoritative for their outputs.

Raw assets carry a .template suffix, preventing editor services from treating
pre-generation imports/configs as runnable projects. Generated files have ordinary
source extensions and must pass real compiler, TypeScript and LSP diagnostics;
diagnostics are not disabled. Tsrust assets are formatted during creation and
checked for stable formatted behavior by the fresh-project witnesses.

## Package admission

The web template reuses the admitted Vite/shadcn integration and exact versions:
Vite 8.3.5, React 19.3.1, Query 5.105.0, TypeScript 7.0.2, Tailwind 4.3.3,
Base UI 1.9.0 and pnpm 12.3.4, with Node >=24.19.0 <25. Original Vite/shadcn MIT
notices are retained. Geist's OFL notice is copied to public/ and reaches dist/.
See the prior task-web milestone for upstream template/registry integrity and
license context. The new root lockfile is embedded to pin the tested graph.

Initial offline installation exposed an uncached qs transitive dependency. The
online repair kept install scripts disabled, then reviewed the resolved graph.
`pnpm audit --json` reports zero advisories across its 421 dependency total:
target/application-preview-audit.json. This is bounded advisory evidence, not a
security certification or a registry-signature audit of every package.

Current upstream references: [Vite server/proxy configuration](https://vite.dev/config/server-options.html),
[shadcn Vite integration](https://ui.shadcn.com/docs/installation/vite) and
[TanStack mutation behavior](https://tanstack.com/query/latest/docs/framework/react/guides/mutations).

## Focused evidence and remaining checks

- Fresh CLI creation tests passed for web/backend paths, generated native/client
  output, formatting/checking, invalid options, no implicit install, no overwrite
  and preservation of customized lib/api.ts. Final raw-asset/lockfile rebuild passes
  both cases in target/application-scaffold-tests-final.log.
- Actual CLI-generated owner project: generated-projects/greeting-web. Native
  greeting assertions and configured server build pass in
  target/application-native-tests.log and target/application-native-build.log.
- TypeScript caught an incorrect object argument where the SDK expects a string.
  The raw template and unchanged preview App call were repaired, with an initial
  hash guard to preserve owner customizations. Web types/production build now pass:
  target/application-web-build.log. A second actual CLI project,
  generated-projects/greeting-web-locked, proves the corrected call and embedded
  lock are emitted: frozen offline install and production build pass in
  target/application-final-install.log and target/application-final-web-build.log.
- Real debug LSP protocol checks on that project pass: entry/operation have no
  diagnostics, an injected in-memory type error is reported and clears after
  repair, and disk source stays unchanged. Log: target/application-editor.log.
- Installed compiler/LSP were refreshed and VS Code cooperatively restarted only
  its Tsrust service. Temporary settings and the staging executable were removed.
  Real installed LSP clean/error/repair checks pass in
  target/application-installed-editor.log. No editor window/process was stopped.
- Live Tsrust and Vite watchers served the locked project. Separate-browser checks
  pass typed valid/invalid calls, actual native greeting/empty input, loading,
  failed call/recovery, zero axe violations, mobile layout and guarded source HMR
  without replacing the document (target/application-web-browser.log). Mobile
  screenshot target/application-web.png and the visible in-app browser were inspected.
- Compiler/LSP all-target typechecking passes on Windows in
  target/application-rust-check-platform.log. Fixes make the shared import-library
  witness independent of CLI-private temporary helpers and align POSIX file-runtime
  test selection with its explicit macOS/Linux support. Portable JSON reader tests
  remain enabled. These are test-harness fixes, not Windows file-close support.
- The exact Windows import restoration/round-trip/unexpected-byte witness executes
  and passes, target/application-import-test.log. The existing console creation
  regression also passes. Legacy UI creation/check/format passes, but its native
  page test hit the original 60-second bound; retain that failure in
  target/application-existing-new-tests.log. Do not call it passed or widen the
  bound without diagnosis. Interactive CLI regression and final post-harness LSP
  rebuild did not run because that command chain stopped at the timeout.

Shutdown inspection caught rejected Geist assets from the hoisted workspace
node_modules outside Vite's explicit allowlist. The template and unchanged preview
configs now allow that dependency directory, retaining a narrow allowlist rather
than exposing the whole backend root. A short-lived isolated Vite instance on 5187
proved the actual font loads and all page assets return without HTTP errors:
target/application-assets.log. The browser witness now checks fonts/asset failures.
This last template repair still needs proof through the next fresh CLI generation;
it was not emitted by the already-running earlier compiler build.

At the owner's request the owned backend/Vite watchers were stopped with Ctrl+C;
ports 4318/5173 were verified free. The owner will start their own server. The
temporary asset QA server was also closed. Never stop owner-started processes.

Remaining follow-up: diagnose/rerun the legacy UI timeout within frozen compatibility
scope, run interactive CLI and the changed native_json_build checks, rebuild/refresh
the tools after test-only source repairs, and prove the asset fix in the next fresh
CLI project. The owner approved root app/, generated/, web/ with app/{api,lib,tests},
consolidated config, manager choice and manager-aware multi-target tsrust dev.
Those are the next coherent implementation section, not capabilities delivered by
this initial starter. Full Electron/Expo, update, migrations, cancellation and
backend-family regression/performance scope remains active.
