# Electron task application

This normal user-owned pnpm project adapts the official Forge 8.0.1 Vite template
and reuses ../web/src. Windows package/lifecycle acceptance is recorded in
docs/milestones/task-desktop.md; other platforms and updater integration remain pending.
Its named bridge derives from the generated SDK manifest. Main owns the native
backend credential and loopback transport; preload exposes no raw IPC or OS API.

Edit src/components/title-bar.tsx for header content, src/renderer.css for its
appearance, and src/lib/app-config.ts for title and native caption-overlay defaults.
The header spans the window; its content respects native caption-button safe areas.
Native caption controls retain platform accessibility and snap/resize behavior.
The bottom border sits below the overlay so it remains visible across the window.
Interactive additions need app-region: no-drag. Windows/Linux omit the default menu;
macOS keeps its application menu. Only Windows has execution acceptance so far.

src/lib/api.ts configures the desktop API once. Shared web/src/lib/api.ts owns
HTTP/auth/cache setup, and lib/api-provider.tsx exposes useApi() to components.
These are editable starter files, not SDK generator outputs. Regenerating the SDK
must preserve them. The bearer-token form belongs to this demo's access policy;
applications can replace it with their own session provider. Initial setup/import
scaffolding and scoped integration migrations are still required prototype work.

Use pnpm install, pnpm typecheck, pnpm start, pnpm package or pnpm make normally.
The prestart/prepackage/premake scripts compile ../desktop.tsr with an installed
Tsrust CLI (or TSRUST override). Rust is needed for building, not for running a
packaged application. For the repository acceptance run, set CARGO_TARGET_DIR to
the repository target and TSRUST_CACHE_DIR to target/tsrust-native; keep
CARGO_INCREMENTAL=0 and CARGO_BUILD_JOBS=1.

Windows builds copy the installed Visual Studio redistributable's unmodified
Microsoft-signed runtime DLLs beside the backend, with hashes/versions and a notice.
This avoids a separate end-user runtime installation. The builder needs licensed
Microsoft build tools with their redistributable files. App-local runtime security
updates must ship with later app updates; no system runtime/trust store is changed.

Development uses the Forge Vite plugin on loopback port 5183. Production assets
load through a restricted local tsrust://tasks origin with no renderer network
access. Native SQLite data lives in app.getPath('userData'), not the app bundle.
TSRUST_DATA_DIR provides a local data-directory override. TSRUST_HEADLESS=1 hides
the window for automated verification; it does not relax sandbox/IPC policy.

Normal quit requests authenticated server shutdown and waits for exit. A dedicated
parent-owned stdin pipe is used only for lifetime detection, not business RPC.
EOF/error terminates an orphan with code 74; hard parent death does not claim
graceful database/server drain. Aborting a renderer call stops waiting, not the
backend operation, and its completion can be unknown. Explicit bounded operation
cancellation/cleanup remains a later required prototype gate.

The ZIP maker is a portable packaging witness, not an installed auto-update path.
Optional updater setup and Squirrel/MSIX/signing/feed choices are tracked in
docs/plans/native-targets-and-secure-updates.md. No release/publishing is enabled.
Keep the upstream MIT notice in public/LICENSE-template.txt and user edits on integration
upgrades. Real dependency additions and upstream tool commands stay available.
