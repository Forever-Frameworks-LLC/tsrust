# Create an application project

`tsrust new hello --app` creates a native greeting API and a React/Vite/shadcn web
project. `tsrust new hello --app --target backend` creates the same checked API
without a frontend or JavaScript dependency installation. The default application
target is web; explicit desktop/mobile selections are rejected until implemented.
An existing destination is never replaced. Creation does not install packages.

The generated layout is:

```text
hello/
  tsrust.config.json
  tsrust.api.json
  tsrust.integration.json  Initial template version and editable-file hashes
  app/                 Native entry, public access policy and greeting operation
    generated-api/     Checked native dispatcher and ownership marker
  packages/api/        Checked SDK, codecs and compatibility history
  tests/               Native greeting assertions
  web/                 When web is selected: normal editable frontend project
    src/lib/api.ts      Application-owned API and query bootstrap
    src/components/ui/  Local editable upstream shadcn components
```

Run `tsrust dev` at the project root. In a second terminal, run `pnpm install`
and `pnpm dev` from web. Visit http://127.0.0.1:5173 and choose **Greet server**.
The native API listens on loopback port 4318; Vite proxies /api. Both ports are
explicit and can be edited in source/configuration. Development does not configure
a public deployment or automatically serve the production assets.

The greeting operation is deliberately public. Read app/access.tsr before adding
protected operations; authentication remains application policy. No token prompt
or bundled secret is required for a public greeting. Put an application's HTTP/
auth provider in web/src/lib/api.ts when required. Components use the named client
and generated query adapter rather than assembling transports or command names.

After editing an API signature, run `tsrust api generate`. It preserves ordinary
application files and checks ownership of generated output. Commit the regenerated
SDK, dispatcher and compatibility history. API generation is an explicit step in
this initial integration; coordinated regeneration/dev orchestration is pending.

Run `tsrust check`, `tsrust test` and `tsrust format --check` for backend source.
Run `pnpm build` in web for frontend types and production assets. Review compiler,
TypeScript and editor errors during development. Manage frontend dependencies and
scripts normally; importing an existing frontend and scoped migration commands
remain separate required work.

The existing `tsrust new hello` console scaffold is retained. `--ui` explicitly
selects the frozen legacy renderer. It is not the React/Electron/Expo application
path. This first application command does not claim full production readiness or
desktop/mobile, updater, migration and deployment acceptance.
