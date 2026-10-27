# Generated API wrapper test dependencies

This private development fixture pins dependencies for the forthcoming generated
TanStack Query adapter tests. It is not an application starter or a requirement
for users of the plain generated client. No adapter acceptance is claimed yet.

Install normally in this directory with `npm ci --ignore-scripts`. No global
package settings or install scripts are needed. The committed npm lockfile records
registry tarball integrity, including TypeScript's platform-specific optional
packages. Other-platform optional packages are not installed on Windows.

Initial dependency review on 2026-10-26, Windows x64, Node 24.19.0/npm 12.0.2:

| Package | Version | License | Source |
| --- | --- | --- | --- |
| @tanstack/react-query | 5.105.0 | MIT | TanStack/query, packages/react-query |
| @tanstack/query-core | 5.105.0 | MIT | TanStack/query, packages/query-core |
| react | 19.3.1 | MIT | facebook/react |
| @types/react | 19.3.0 | MIT | DefinitelyTyped/DefinitelyTyped |
| csstype | 3.2.3 | MIT | frenic/csstype |
| typescript | 7.0.2 | Apache-2.0 | microsoft/TypeScript |
| @typescript/typescript-win32-x64 | 7.0.2 | Apache-2.0 | microsoft/TypeScript |

The direct packages were read from official npm registry metadata before install.
TanStack declares React and @types/react peers `^18 || ^19`. `npm install
--ignore-scripts --no-fund --audit` installed seven packages and reported zero
known vulnerabilities across eight audited entries. `npm audit signatures`
verified seven registry signatures and three attestations. `npm exec -- tsc
--version` returned 7.0.2, and `npm ls --depth=1` found no missing required peers.
These are admission observations; generated declarations, runtime cache behavior,
hooks and application compatibility still require executable tests.

The existing plain generated client's declarations pass `npm run check:plain`
under TypeScript 7.0.2, with positive named-call/result checks and expected errors
for undeclared operations, missing inputs, incorrect nullable inputs and incorrect
result types. From this directory, first generate the ignored fixture with
`tsrust api generate --config ../../examples/api-contract/tsrust.api.json`, then
run that check. This consumer uses only ES2022 types, without DOM globals.

Repeat advisory and signature checks when updating this lockfile. Keep package
upgrades and their integration acceptance explicit; users' application dependency
management remains independent of this development fixture.
