# Task application

This is an ordinary user-owned React/TypeScript project based on official
create-vite 9.3.3, with shadcn Base UI / Nova components. Run upstream tools,
edit scripts and install dependencies directly. Tsrust owns the generated API
contract, not the frontend dependency graph.

From the backend directory, generate the API with a current compiler:

```sh
tsrust api generate --config tsrust.api.json
cd web
pnpm install --frozen-lockfile
pnpm dev --host 127.0.0.1 --port 5173 --strictPort
```

Start the native service following the parent README. Set its `TASK_ORIGIN` to
`http://127.0.0.1:5173`. Enter its full `/api` URL and access token in the app.
Use HTTPS for a non-loopback service. Tokens stay in connection memory; refresh
or disconnect requires entering them again. This prototype uses application-owned
bearer access rather than inventing a login provider. Never embed a real token in
Vite environment variables or source. Desktop will supply a separate preload
transport; that integration remains pending.

The `file:../generated-client` dependency is an installed local package, so its
optional peers resolve with the app. After changing backend contracts, run
generation and `pnpm install --no-frozen-lockfile` to refresh that copy. Changes
to API types then appear in the normal TypeScript build. Regeneration does not
rewrite this project's manifest, scripts, source or dependency policy. The exact
version pins and lockfile record the initial tested matrix; users may change them.

```sh
pnpm build
pnpm preview --host 127.0.0.1 --port 5173 --strictPort
pnpm lint
pnpm test
```

Browser tests require the built native test witness at repository `target/` and
`pnpm exec playwright install chromium`. Build that witness with
`node tools/verify-task-http.mjs` from the repository root. Tests launch a private
temporary SQLite service, use only headless Chromium, and clean it up. Port 5173
must be free. Set `TASK_WEB_PREVIEW=1` when testing the production preview; the
development-only HMR witness then skips. Browser tests never control the desktop.

The UI uses generated named calls/hooks, shows domain and transport failures,
validates UTF-8 title bytes, supports editable/completed tasks, confirms deletion
and pages through native results. Ambiguous network writes are not automatically
retried. Query abort does not imply backend rollback. Disconnect clears the
session cache; already accepted writes may still complete.

See `docs/milestones/task-web.md` for exact acceptance and remaining requirements.
Native production asset serving and two mounts pass focused acceptance. Electron
and Expo remain pending. Build with TSRUST_WEB_BASE=/web/ and set the backend's
TASK_WEB_ROOT to the absolute dist directory to serve this app from native code.
The root upstream README is retained for Vite configuration guidance.
