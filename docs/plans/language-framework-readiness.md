# Language and framework readiness

Current assessment: 2026-10-18, source baseline `752e9e14`.

The owner requires continued work on basic language completeness, CLI usability,
Hono-style HTTP over Axum and the UI vision. These are active requirements, not
conditional on another application request. Accepted package sections do not
establish framework readiness. This inventory is an initial source audit and five
compiler probes, not an exhaustive compatibility certification.

## Language

Reuse the implemented generics, callable ownership, record/array destructuring,
spread, nullable values, narrowing, async arrows and checked resource support.
Their focused tests already exist; do not rebuild those features from historical
roadmap statements. The installed compiler accepts an inline async Router handler.

Current compiler probes establish these remaining gaps:

| Construct | Observed result | Follow-up |
| --- | --- | --- |
| `interface Named extends Box<int>` | Implemented after the baseline probe; focused acceptance passes | See [generic inheritance acceptance](generic-interface-inheritance-acceptance.md) |
| Interface method signature `name(): string` | Explicit unsupported diagnostic | Evaluate lowering to existing callable fields; define receiver behavior before claiming methods |
| Class declaration | Generic `Expected ";"` diagnostic | Improve unsupported-syntax diagnostics; classes are not silently provided by native constructors |
| `keyof Item` | Generic `Expected ";"` diagnostic | Inventory type operators needed by framework authors before a bounded implementation |
| `async c => c.text("ok")` | Accepted | Reuse existing async-arrow evidence |
| JSX `children={ownedArray}` | Accepted at `a0fca44c` with compiler/editor witnesses | Explicit attributes and body children cannot be combined; automatic `{array}` body flattening remains unsupported |
| `(props.children ?? []).join('|')` on optional string arrays | `join() requires an array` in a minimized content-card witness | Direct nonoptional array receivers work. Investigate expression receiver inference without changing coalescing or ownership contracts |

Probe sources, exact diagnostics and compiler hash are retained as
`tsrust-readiness-20261018.json` in canonical SQLite after evidence import. A
successful check is not native execution evidence. These probes do not establish
that mapped/conditional types, arbitrary TypeScript structural typing or classes
should receive JavaScript semantics. Preserve checked ownership and explicit
precision boundaries.

## CLI

`new`, project discovery, `check`, formatting, native tests, `dev`, package handling,
cache retention and offline export already exist. The dev-loop milestone is
accepted at `e8d68b27`; older guide sentences saying retained frontend/runtime
separation are pending are stale.

A concrete usability gap is `run` argument forwarding. This section adds
`tsrust run [entry.tsr] [compiler options] -- [program arguments]`, preserving
configured project-root execution and literal argument boundaries. The separator
must precede application flags. `std:env.args()` now supplies owned process arguments excluding the executable
name, with a checked `NotUnicode` error instead of lossy conversion. The CLI
forwarding path accepts Unicode arguments; direct native callers can exercise
the checked non-Unicode error. See [acceptance](process-arguments-acceptance.md).

Further CLI review should cover command-specific help, actionable backend setup
errors, external-import dev watching and target/config discoverability. These are
review candidates, not claims that every item is absent or already accepted.

## HTTP framework

Compare the actual `stdlib/http` declarations, analyzer and runtime with the
[Hono app](https://hono.dev/docs/api/hono),
[request](https://hono.dev/docs/api/request) and
[context](https://hono.dev/docs/api/context) APIs (reviewed 2026-10-18).
The public native name remains `Router`; the accepted server lifecycle stays.

| Area | Current support | Missing or divergent surface |
| --- | --- | --- |
| Routing | Method routes, ordered/scoped middleware, multiple callbacks, parameters | Route grouping/base paths, `all`/`on`, optional/regex patterns; current `route(method,path,...)` differs from Hono subrouter mounting |
| Request | Header/parameter lookup, named query/queries lookup, text, form and multipart bodies | No-argument query collections, direct typed JSON body workflow, complete header/parameter collection access |
| Context | Text/HTML/JSON, status/header changes, typed application and request state | Redirect/body helpers, custom not-found and application error hooks |
| Testing | Network/runtime fixtures | Convenient in-process Router request dispatch |
| Streaming | Owned inbound bodies and bounded response channels | Response-owned producer callbacks, SSE, WebSocket handling |
| UI integration | Separate route/lifetime experiments | Generated document/region serving, checked loader boundaries and renderer integration |

This is substantial remaining compatibility work. Existing native method calls
and typed context do not imply near-source compatibility. Keep authentication,
sessions and authorization in application code. Helpers must not become a policy
framework. Prioritize ordinary request/response and composition APIs before
additional infrastructure libraries.

## UI

The synchronous owner/handle and independent destruction continuation is accepted;
see `experiments/ui-tsrust-owner-handles/CHECKED-RESOURCES.md`. The experiment's
older README and original matrices retain historical B2 blockers; consult the
checked continuation rather than treating those snapshots as current failures.

Production readiness remains open: async/worker lifecycle integration, `.tsrx`
parsing/checking/formatting/editor support, checked UI lowering, browser execution,
paired web/native rendering, keyboard/input/accessibility, state updates and
navigation. The vision's generated static screen, independent counters and editable
list remain acceptance targets. Route manifests and manually constructed native
witnesses do not close them. Reconcile each against current experiment code before
assigning its implementation; do not alter unrelated owner plans.

## Implementation order

1. Finish and verify the bounded CLI argument section.
2. Address basic language gaps exposed by framework authoring, starting with
   generic interface inheritance and precise unsupported-syntax diagnostics.
3. Complete ordinary HTTP request/response helpers and Router composition/error
   handling, preserving middleware ordering and ownership.
4. Continue the UI dependency and generated paired-target milestones in the vision.

Track complete sections independently. Add regression witnesses while implementing,
then batch targeted verification. Full-family timing remains separate. Reuse all
accepted SQL, cache, queue and resource evidence unless the changed semantics
actually affect it. Redis/brokers remain demand-driven; that does not defer the
language, HTTP or UI requirements above.

## CLI section evidence

Argument-splitting/child-command inspection passes one unit test. One native CLI
integration test passes both explicit and configured entries, flags after `--`,
Unicode/space/empty arguments and rejection of an unknown compiler option before
`--`. The native witness is formatted and checked for idempotence before execution.
A test-source quoting error in the first integration build was repaired; retain
both logs. No compiler/LSP family, SQL suite or performance benchmark was rerun:
this change affects CLI invocation only. There is no new language built-in or
editor signature. Whole-project formatting and Git whitespace checks pass.
