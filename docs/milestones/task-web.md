# React/Vite and shadcn task application

Focused web acceptance passes on Windows x64 with Node v24.19.0 and pnpm 12.3.4.
Source is `examples/backend-tasks/web`. Baseline is `1132d76d`; native test service
remains the accepted executable from the task HTTP section. No new compiler or
native runtime behavior is introduced by this section.

## Upstream admission

Scaffold: official create-vite 9.3.3 `react-ts`. Integration follows the documented
[Vite setup](https://ui.shadcn.com/docs/installation/vite). Official shadcn CLI
4.22.2 supplies Base UI / Nova components from the `@shadcn` registry. Its `docs`
command and primary component pages were consulted before installation; copied
components were reviewed. The pinned CLI accepts `--preset nova --base base`,
whereas the skill's `base-nova` spelling is rejected. A missing Tailwind/alias
preflight was repaired through the documented existing-project setup before init.

`package.json` pins tested direct versions, `pnpm-lock.yaml` records registry
integrities and transitive versions, and `tsrust-template.json` records scaffold
integrities and copied component hashes. Important runtime pins: React/React DOM
19.3.1, TanStack Query 5.105.0, Base UI 1.9.0, shadcn CSS package 4.22.1, Tailwind
4.3.3. Build pins: Vite 8.3.5, plugin-react 6.1.2, TypeScript 7.0.2 and oxlint
1.89.0. Browser/accessibility pins: Playwright 1.64.1 and axe adapter 4.14.0.

`pnpm audit --json` reports zero advisories in the admitted graph (437 total
dependency entries). This is a dated resolved-graph result, not a permanent
security claim. `pnpm licenses list --json` reports MIT, Apache-2.0, BSD, ISC,
BlueOak, 0BSD and dual-license dependencies; Geist uses OFL-1.1, caniuse-lite
CC-BY-4.0 and axe development tools MPL-2.0. Its single Unknown entry is the
private generated Tsrust package, whose shipped TSRUST-LICENSE is MIT. Included
source license notices preserve the copied shadcn and Vite attribution. Registry
integrity and pnpm policy admission pass; a separate registry signature/
attestation audit of this entire pnpm graph is not claimed. Starter upgrades need
their own resolved advisory and compatibility review.

## Behavior and evidence

The app consumes only generated package types and named hooks. DTOs are derived
from Client return types; no handwritten transport command names, codecs or query
keys are needed. Connection credentials live in memory and never enter assets or
browser storage. Each connection gets a distinct cache scope and QueryClient.
Disconnect cancels queries and clears that cache; it does not promise write
rollback. Browser endpoints require HTTPS except on loopback. Server authorization
and exact Origin policy remain authoritative.

The real-native headless witnesses cover empty/loading/denied/network states,
CRUD, active invalidation, UTF-8 validation, keyboard checkbox/edit submission,
deletion confirmation, cursor pagination, disconnect credential/storage behavior,
mobile overflow and zero axe violations on connection and populated pages. An
intercepted network failure leaves an unconfirmed title for explicit recovery.
The service is real SQLite; intercepted requests only hold or fail network access,
never return fabricated task data. Every child drains and exits cleanly, and its
temporary database is deleted. Tests record browser page errors in the CRUD case.

Commands/results:

- `pnpm build`: TypeScript and production Vite build pass, logs
  `target/task-web-build.log` and final `target/task-web-final-build.log`.
- Development browser run: CRUD/accessibility and cursor tests pass in
  `target/task-web-browser-repair.log`. The remaining loading/error test passes
  separately in `target/task-web-errors-repair.log` after correcting an ambiguous
  test selector (visible loading text and live status have the same message).
- `TASK_WEB_PREVIEW=1 pnpm exec playwright test`: all three production browser
  cases pass, `target/task-web-production.log`.
- Development HMR check passes, `target/task-web-hmr.log`: edits/restores the
  actual heading, observes both changes, and proves the same document survives.
- `pnpm lint` passes with one upstream warning for button.tsx exporting its
  variants alongside the component. Original public shadcn API is preserved.
- `node tools/verify-task-web-clean.mjs` passes in
  `target/task-web-clean-repair.log`: copies source outside the repository,
  regenerates the API to an external folder, installs frozen/offline with scripts
  disabled, builds with normal frontend tools and adds a user-owned dependency.
  Existing scripts, package-manager selection and source are preserved. The first
  attempt found a cold C-volume store; the repair explicitly reuses the admitted
  D-volume store rather than downloading or changing versions. All temp files exit
  cleanly. This copies the current starter, not an unimplemented setup command.

Headless screenshot `target/task-web-mobile.png` was inspected at 375px width;
controls and content fit without horizontal scrolling. Chromium headless runtime
156.0.8078.4 is Playwright's pinned build 1248. No desktop interaction occurred.
Production JS is approximately 334 kB uncompressed / 107 kB gzip; this build output
is not a startup or throughput benchmark.

## Remaining scope

Native static serving and two isolated mounts/API fallback are next. Electron
preload/lifecycle/packaging, Expo Android, backend cancellation cleanup, convenience
commands/migrations, full family regression and measurements remain required.
This is a functioning web example, not full prototype completion or proof that
Tsrust setup can already generate the whole project. No publication or remote CI
ran. Owner docs and frozen UI remain unchanged.

The installed CLI was atomically refreshed and independently stale-checked the
task configuration. Installed/debug SHA-256 match:
`55a253932ef52ba1cbc5a367ffdb2f8ddd50bb1e3ac5094b2513eb6f4f14b762`.
No LSP source changed in these client/web sections; no editor restart or desktop
interaction was performed.
