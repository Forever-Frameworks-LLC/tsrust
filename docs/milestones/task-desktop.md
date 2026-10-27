# Windows Electron task application

This section packages the native SQLite service and shared React/shadcn task UI.
It adds no compiler builtins and preserves the old UI source in place. Windows
package execution is accepted; full prototype completion, other platforms,
installed updates and release signing are not claimed.

## Integration and ownership

The normal user-owned pnpm project uses Forge 8.0.1's Vite template, Electron
45.0.0, fuses 2.1.3, Vite 8.3.5, React 19.3.1, Query 5.105.0, TypeScript 7.0.2
and Node 24.19.0. Exact versions/integrities live in native/pnpm-lock.yaml.
The template tarball integrity was checked and its MIT notice is shipped in
public/LICENSE-template.txt. The resolved advisory review reported zero findings
in target/desktop-audit.json; that is bounded advisory evidence, not certification.
Electron's reviewed installer used upstream checksum validation. Its upstream
executable is not Authenticode-signed; application/release signing remains pending.

Forge requires pnpm's hoisted node linker, configured in pnpm-workspace.yaml.
Vite preserveSymlinks is disabled for the shared web source's dependency resolution.
Application code is strict; skipLibCheck matches the upstream template because
upstream debug/listr2 declarations currently fail strict library checking.
Independent generated SDK consumers retain their separate strict checks.

Main starts an authenticated ephemeral loopback backend. A bounded readiness record
reveals no credential to the renderer. The sandboxed, isolated renderer receives
only manifest-derived named task calls through preload. The SDK still validates
arguments and responses. Main checks sender/frame/origin, request size and active
request count. Navigation, windows, webviews and permissions are denied. Packaged
assets use a restricted custom origin with no renderer network access. Production
fuses disable Node execution/options/inspection and file extra privileges, and
enable ASAR-only loading/integrity and cookie encryption.

Normal quit waits for authenticated server shutdown and child exit. A small
application-owned Rust stdin-pipe guard exits 74 after hard parent death; this
does not claim graceful drain after a crash. Startup/backend failure opens an
unprivileged failure window with normal OS chrome. Local abort does not prove
backend cancellation or write rollback; bounded cancellation remains required.

The editable TitleBar component supplies branding, drag region and native control
safe areas. The native overlay is one pixel shorter than the header, preserving
its full-width bottom border. Windows/Linux default menus are removed. Native
controls supply caption accessibility and snap behavior; physical interaction
and non-Windows behavior are not automated acceptance claims.

Editable lib/api.ts files centralize HTTP/desktop/cache setup, and ApiProvider /
useApi keep it out of components. This is application source, preserved across SDK
generation. Demo authentication is replaceable application policy. CLI starter
generation and scoped migrations are still pending, not implied by these examples.

Windows builds include unchanged Microsoft-signed Visual Studio redistributable
DLLs beside the native backend, with hashes, versions and license notice. Acceptance
checks the loaded vcruntime140.dll path belongs to the package. Builders need
licensed Microsoft build tools; future app updates must also refresh this runtime.
No system runtime installation, trust-store change or signing enrollment occurs.

## Focused acceptance

- Native supervisor witness: missing executable, readiness timeout, migration
  failure, SQLite CRUD/persistence, paths containing spaces/#/%, idempotent clean
  stop, backend crash, denied shutdown and parent-pipe EOF. Log:
  target/desktop-backend-acceptance.log.
- Native typecheck and Forge package pass: target/desktop-typecheck.log and
  target/desktop-package-build.log. Normal build scripts generate the SDK and
  compile desktop.tsr with the installed verified compiler and shared Cargo cache.
- External temporary package copy runs with only System32 on PATH and no Rust/
  Node toolchain environment. Actual Electron/native CRUD, restart persistence,
  renderer isolation, malformed arguments, denied navigation/windows, startup
  failure, backend crash, clean quit and orphan cleanup pass. Fuses/runtime hashes
  and local runtime loading are checked. Log: target/desktop-package-acceptance.log.
- Header checks cover 44px height, drag region, full viewport width and 43px native
  overlay. Renderer screenshot target/task-desktop.png was inspected; CDP screenshots
  exclude native controls. A separate visible app lets the owner test those controls.
- Shared web refactor: production build passes; all four development browser cases
  pass, including real-native CRUD/accessibility, errors, cursor pages, disconnect
  storage behavior and upstream HMR. Logs: target/task-web-bootstrap-build.log and
  target/task-web-bootstrap-browser.log.
- Clean external adoption still passes after the bootstrap refactor, including
  frozen offline install, regenerated SDK, build and an ordinary owner dependency
  addition: target/task-web-bootstrap-clean.log.

The ZIP maker has passed on the initial package. Final ZIP regeneration, Forge
development/HMR, setup/import commands, migration rollback,
optional secure updater/installer integration, Expo Android, long-operation cleanup
and backend-family regression/performance remain pending. No publication or remote
CI ran. The active prototype goal remains incomplete.
