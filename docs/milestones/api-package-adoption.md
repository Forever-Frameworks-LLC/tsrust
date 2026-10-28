# Standalone client and npm workspace adoption

Focused acceptance passes from adapter checkpoint `c5f07a8a`. Dedicated Tsrust
setup/export/pack convenience commands and other package-manager matrices remain
pending; this section proves the underlying generated product is usable with
ordinary npm without compiler/runtime adoption.

Run `node tools/verify-api-package.mjs` using the current compiler. It generates
the query fixture to an explicit OS temporary directory outside the repository,
stale-checks it, and uses upstream npm pack with scripts disabled. The witness
adds `.env` and backend-source sentinels, then requires the tarball's exact files
to equal the generated public allowlist plus package.json. It checks name,
version and SHA-512 integrity metadata. Neither sentinels nor ownership markers
ship. All filesystem cleanup resolves and checks the expected temporary parent.

The first detached consumer installs the tarball offline and executes a named
call with neither React nor TanStack installed. The second starts as an existing
app with owner source, scripts, npm settings and the fixture's exact peers. It
installs the tarball, then the original generation directory is deleted. Plain
calls, real QueryClient options and React hooks rendered under the application's
Provider still pass. TypeScript 7.0.2 checks inferred numeric results and rejects
bad inputs/results. A subsequent ordinary npm dependency addition works and
preserves owner scripts, source bytes, configuration and existing devDependencies.
This is a local zero-dependency package, so the check needs no registry publication.

A separate ordinary npm workspace includes the generated package and a web app.
The app declares the package name/version; npm produces a real workspace link,
with application peers at the workspace root. The same QueryClient/React hook
witness passes without copying dependencies into the generated package. No
parallel cache, React runtime or custom package resolver is introduced.

Actual acceptance: `target/api-package-acceptance.log`, exit 0. The invoked npm CLI
reports 11.17.0; Node reports v24.19.0. Peer/type versions are the committed
`tools/api-client-tests/package.json` pins: React/React DOM 19.3.1, TanStack Query
5.105.0, TypeScript 7.0.2, React/React DOM types 19.3.0. All installs are offline,
with scripts disabled; registry provenance/advisory admission remains the prior
fixture's recorded check. This is not new advisory review or browser UI evidence.
All temporary consumers/tarballs are removed after the check.

The implementation follows upstream [npm pack](https://docs.npmjs.com/cli/v11/commands/npm-pack/)
and [npm install](https://docs.npmjs.com/cli/v11/commands/npm-install/).
Folder installs outside the project can be symlinks; `--install-links` requests
an installed copy. Workspace hoisting and tarball installation are separately
verified here. pnpm isolated dependencies, Expo Metro and packaged Electron still
require their actual target acceptance. No package publication occurred.
