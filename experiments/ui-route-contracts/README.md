# Experimental UI route manifest

This Node-built-ins-only prototype discovers an app directory without reading,
parsing or executing its application source. It records declarations, not runtime
behavior. It is isolated from the compiler, installed consumers and main analytics
store. Baseline: `ac3ef15a6657fc4ad5a35ad9abd54d410d3f8fc2`.

## Run

From the worktree root:

```sh
node --test experiments/ui-route-contracts/tests/*.test.mjs
node experiments/ui-route-contracts/cli.mjs demo
node experiments/ui-route-contracts/cli.mjs inspect experiments/ui-route-contracts/fixtures/editable-list experiments/ui-route-contracts/fixtures/editable-list/handlers.json
```

The CLI writes JSON to stdout, never application files. Invalid manifests return
`manifest: null`, sorted diagnostics and exit status 1. Usage, I/O and JSON syntax
errors go to stderr with status 1. The demo includes a deliberately conflicting
GET declaration as a successful demonstration of rejection.

`handlers.json` is explicit mock checked input, not a proposed application file
convention. Each key is an exact project-relative `app/.../route.tsr` identity.
Each value is an array of `{ "method": "POST", "path": "/" }` declarations.
An empty array explicitly declares no methods. Methods are uppercase standard
HTTP method tokens; only the current segment `/` is supported. Missing declarations,
unknown source keys, extra declaration fields and subpaths are diagnostics.

## Experimental schema: ui-route-manifest/0.1

`discover(projectRoot, declarations)` walks only `projectRoot/app`.
`compileManifest(files, declarations)` accepts a filename inventory for deterministic
contract tests. Both return `{manifest, diagnostics}`. Any diagnostic prevents
manifest acceptance. Diagnostic fields are `code`, sorted relative `sources`, and
`message`. No absolute machine paths or source contents enter the manifest.

A valid manifest has:

- `schemaVersion`: `ui-route-manifest/0.1`.
- `appRoot`: `app`; `notFound`: `app/not-found.tsrx`.
- `segments`: source-identity-sorted records for the app root and directories
  containing discovered files or their ancestors. Empty directories have no effect.
- Each segment has `id` (relative directory), `pattern` (encoded literal URL segments
  and bracketed parameter names), `parts` (one `{literal}` or `{parameter}` per
  segment), and ordered `params` names.
- `sources` maps present special-file kinds to their relative source paths.
- `ancestry` has `middleware`, `layout`, `template`, `not-found`, and `error` arrays,
  each ordered root to leaf, including the current segment. These are references,
  not executed policies, selected runtime error handlers or mounted components.
- `page` is null or `{source, loader}`. The loader is null unless colocated with
  that page. Orphan loaders fail validation; parent loaders never become child
  loaders or API handler bindings.
- `endpoints` contains method-sorted `{method, kind, source}` records. Pages reserve
  GET and HEAD. Routes own explicitly declared methods. Duplicate ownership fails
  with both source references. A wrappers-only segment owns no endpoint.

All eight agreed files are recognized: `layout.tsrx`, `template.tsrx`, `page.tsrx`,
`loader.tsr`, `not-found.tsrx`, `error.tsrx`, `route.tsr`, `middleware.tsr`.
No auth or CSRF declaration is required. Ordinary files create no endpoints;
components outside `app/` are not visited. `loading`, `default`, `global-error`
files and incorrect `.tsr`/`.tsrx` extensions for reserved names are rejected.
Catch-alls, groups, slots and malformed parameter segments are rejected.

## URL contract

`matchRoute(manifest, url, method = 'GET')` returns
`{segmentId, pattern, endpoint, params}` or null. A null match means no declared
endpoint for that path/method, not a selected HTTP status or executed 404 handler.
Literal precedence is resolved before method selection; a missing literal-route
method never falls through to a parameter route. Precedence compares path segments
from left to right. Equivalent patterns and repeated parameter names are errors.
A route match does not establish row existence: IDs 1, 2, 3, 4 and 5 all match `[id]`.

`buildPath(manifest, segmentId, params = {})` accepts an endpoint segment identity
and exactly its string parameters. It rejects missing/extra parameters and paths
shadowed by a more specific endpoint. It encodes each segment independently.

Matching is case-sensitive. One trailing slash is removed; repeated separators,
backslashes, controls, dot segments, malformed percent escapes, invalid UTF-8,
decoded separators and decoded percent signs are rejected. Each segment is decoded
exactly once. Query and fragment are separated, not parsed or validated as route
parameters. Raw controls/backslashes anywhere in the input are rejected. URLs must
be path references starting with `/`, not absolute URLs.

IDs remain opaque nonempty strings. Leading zeros and `+` are preserved. There is
no numeric coercion or Unicode normalization. Percent-bearing IDs are intentionally
excluded until the proxy/server normalization contract is reviewed. URL identity
is not an output filesystem path or a cross-platform filename portability claim.

## Scope and limitations

The pure API expects ordinary data objects and an in-process accepted manifest;
it is not a hardened arbitrary-object deserializer. Discovery assumes a stable,
trusted local project root. Symlink entries (including `app`) and special filesystem
entries are rejected, but this synchronous prototype is not a sandbox against
concurrent hostile filesystem replacement or a hostile root ancestor. The manifest
is data-only; no source bytes are loaded by discovery. Tests use invalid source and
an executable write/throw sentinel to verify nonexecution and unchanged inputs.

No runtime fallback ancestry, 404/error execution, 405/CORS/preflight behavior,
automatic HEAD for route-only GET, provider execution, authorization, loader
execution, rendering, static generation, scaffolding, catalog installation or
production CLI integration is implemented. Explicit OPTIONS declarations are
ordinary method declarations, not automatic preflight handling. No library or
package downloads are required. No performance or compiled-UI claims are made.

## Compiler handoff

S13/S14 are accepted per the task assignment; do not reopen their implementation.
S15 HTTP remains pending. Integration still requires separately owned work:

1. Checked route-handler declarations from the compiler, including source spans,
   method/path ownership and configured middleware ordering. Replace fixture
   declarations without adding a second parser.
2. `.tsrx` syntax, props checking, formatting, project entry discovery, LSP/editor
   support and dependency invalidation for route changes.
3. Loader successful-result inference and checked page injection, control outcomes,
   middleware context refinement and build/server/native provider capabilities.
4. Production normalization shared with HTTP/proxy adapters; reviewed precedence,
   fallback/405/HEAD/OPTIONS rules and typed link diagnostics.
5. S15 adapter execution proofs: at-most-once suspend-safe `next`, body consumption,
   cancellation/drop, isolation, bounded decoding and consistent installed-policy
   execution for all representations. This manifest does not prove those properties.
6. UI ownership/lifecycle, browser execution and native text/input remain separate
   milestones. Do not proceed to them as part of this task.

## Evidence

Focused command outputs are retained in the ignored, worktree-local `evidence.sqlite`
when validation is run for handoff. It is isolated pending serial integration into
canonical SQLite, never written to the main store. The database records exact
commands, exit status, stdout/stderr bytes and hashes; it is not a benchmark.
Git holds source, fixtures and documentation. No source snapshots or binary archives
are created. The final handoff identifies the source commit and check results.

## Validation result (2026-10-15)

Node v24.11.1: all 15 focused tests pass; demo and inspect exit 0. The demo emits
seven segments and the expected GET conflict. Both whole-project formatting passes
using `/Users/seancotter/.cargo/bin/tsrust` report zero changed files and zero
fingerprint updates. Byte hashes of tracked files and prototype source remain
unchanged across those passes, proving token preservation and idempotence; the
focused tests and demo run after formatting. A final formatting pass precedes the
source checkpoint after this documentation/comment update.

The first formatter invocation failed before changes because this clean worktree
has no benchmark TypeScript dependency installed. Subsequent passes use a temporary
Node import hook to read the existing main-checkout TypeScript installation without
modifying it. No dependency was downloaded or linked into this checkout. The exact
hook and dependency/compiler hashes are recorded in isolated SQLite. This is a
formatter tooling dependency only; the prototype itself uses Node built-ins.

No compiler/native/editor regressions or performance measurements were run. No
shared consumer, Cargo output, main analytics database or main checkout file was
modified. The source commit is the handoff checkpoint, not a merge or framework
acceptance milestone. Keep the ignored SQLite file until serial evidence integration.
