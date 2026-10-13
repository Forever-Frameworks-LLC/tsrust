# First parallel UI task: nonexecuting route manifest

Status: tightened planning assignment, 2026-10-11; not dispatched. Supersedes the earlier combined routing/static-build/scaffolding/catalog task. Read [UI_VISION.md](../../UI_VISION.md), the [latest review](ui-feasibility-review.md#follow-up-review-after-app-directory-and-loader-decisions), and [parallel boundaries](parallel-ui-foundations-plan.md). This is optional supporting tooling; it does not establish UI runtime feasibility.

## One outcome

Given a small editable-list app tree and separately supplied handler declarations, produce a deterministic, inspectable route manifest and diagnostics **without executing or parsing application code**. Prove special-file association, middleware ancestry, page/API ownership and path identity. No async implementation is needed. The eventual compiler supplies checked declarations; fixture declarations stand in for that input here.

The prototype records application-declared middleware without requiring auth, CSRF or any other policy helper. It neither executes nor proves business policies. Follow [the vision's responsibility boundary](../../UI_VISION.md#application-policy-and-framework-guarantees).

This one task can start alongside compiler work in a separate checkout. Stop after it passes. Do not spend the waiting period building a parallel framework in Node. If only one development slot is available, finish compiler work first; this prototype is useful but not on the core UI feasibility critical path.

## Isolation and prerequisites

- One worker in a separate worktree from an agreed committed baseline. Current main has active compiler source reorganization; never copy its uncommitted source or edit its shared files. This document does not create a worktree or authorize dispatch.
- Own only `experiments/ui-route-contracts/`. No compiler/LSP/editor changes, root manifests or lockfiles, production CLI, installed consumers, main analytics store or shared build output changes.
- Use existing Node and built-in `node:test`, no downloaded packages, Cargo, server, database or renderer. Node is a development harness, not a generated-app runtime requirement.
- The task brief must be committed/available in the selected baseline or provided explicitly by the integration owner. Follow AGENTS.md and the development workflow, including whole-project formatting before source checkpoints using an agreed verified compiler. If unavailable, report that checkpoint blocker; do not rebuild/replace active main consumers.
- Tests may create disposable fixtures in temporary directories; the CLI reads a fixture and prints to stdout. Do not implement source generation or installation. Reject symlinks in app-tree traversal for this experiment; do not traverse outside the supplied root.

## Bounded convention contract

Recognize all eight agreed special files: `layout.tsrx`, `template.tsrx`, `page.tsrx`, `loader.tsr`, `not-found.tsrx`, `error.tsrx`, `route.tsr`, `middleware.tsr`. Support root, literal segments and `[id]` only. Diagnose catch-alls, groups, malformed brackets and unsupported reserved conventions; never interpret them as literal URLs. Ordinary colocated files and `components/` do not create endpoints.

Each manifest segment records a project-relative source identity, canonical pattern, ordered parameter names, special-file source references, ancestor middleware/layout/template/boundary references and the exact local page-loader pairing. Keep wrapper/boundary references as declarations; do not pretend a manifest proves mount/disposal, exception handling or middleware execution.

The prototype requires a visible root `not-found.tsrx`; missing source is a diagnostic with the required path, never synthesized hidden behavior. Nested not-found/error sources are recorded. A directory with no page or route can still contribute wrappers/middleware to descendants. A page without a loader is valid. An orphan loader is a diagnostic and contributes no executable binding. A parent page loader is not inherited by child pages or API handlers.

Handler method declarations come from a separate, explicit JSON fixture keyed by discovered `route.tsr` source path. They are mock **checked input**, not a new app configuration convention. Do not scrape `get(...)` using regex, import the route module or infer exports from filenames. Diagnose missing declarations for discovered route files and declarations referencing nonexistent route files. Start with current-path handlers only; reject route subpaths rather than silently bypassing descendant ancestry.

A page reserves GET; colocated routes can own POST and other nonconflicting methods. A route-only segment can own GET. Duplicate method/path ownership fails with both source references. As a provisional recommendation, page GET also reserves HEAD; explicit conflict is rejected. Automatic HEAD for route-only GET, OPTIONS, 405, CORS and fallback middleware execution are deliberately unimplemented and listed in the handoff.

## Provisional URL policy for the experiment

Specify this policy in the README and tests, without claiming it is frozen production behavior:

- Case-sensitive route matching; one terminal slash canonicalizes away except `/`. Reject repeated separators, backslashes, control characters and dot segments, including encoded forms. Separate query/fragment from pathname; they do not become parameters or route identity.
- Decode each URL segment exactly once using strict UTF-8 percent decoding. Reject malformed escapes, decoded separators and decoded percent signs in this first slice; the latter deliberately excludes literal-percent IDs until the end-to-end proxy/server policy is reviewed. Never decode again downstream.
- Single parameters are opaque nonempty strings. No integer coercion, range expansion or implicit Unicode normalization. Builders reject missing/extra parameters and invalid segment values, then encode per segment.
- Literals outrank parameters. Reject structurally equivalent patterns with different parameter names and repeated parameter names in one pattern. Collision checks use the canonical URL identity, not enumeration order.
- Manifest URLs remain independent of filesystem output paths. No static artifact writing or claim of Windows/Unicode filesystem portability.

## Deliverables and order

```text
experiments/ui-route-contracts/
  README.md          scope, provisional policy, capability boundaries and handoff
  routes.mjs         discovery, manifest, matcher and path builder
  cli.mjs            inspect/demo only; stdout, no application writes
  fixtures/          small app trees plus mock checked handler declarations
  tests/             independent behavioral assertions using node:test
```

1. Write focused failing tests from the acceptance table, with expected outcomes independent of discovery implementation.
2. Implement deterministic discovery and declaration validation; serialize the same manifest regardless of directory enumeration order. Store relative, not machine-specific absolute paths in output.
3. Implement match/build and ancestry selection for **matched declared endpoints only**. Keep route matching distinct from middleware execution and database record existence.
4. Run the focused suite and one demo; hand off the manifest schema, rejected cases, exact commands/results and compiler integration dependencies. Stop. No promotion of the prototype into production without review.

## Acceptance matrix

| Witness | Required observable result |
|---|---|
| App with no auth/CSRF declarations | Valid without invented helpers or security-policy diagnostics |
| Root with only middleware/layout/not-found | No root endpoint; descendants inherit declared ancestry |
| Page only; route only GET; page plus route POST | All valid; one owner for each method/path |
| Page plus route GET; proposed HEAD conflict | Deterministic duplicate-ownership diagnostic with both sources |
| Page plus loader; page without loader; orphan loader | Exact local binding; no data binding; diagnostic respectively |
| Parent loader plus child page and colocated API | Neither child page nor API acquires the parent/page loader |
| Root/items/[id] middleware plus sibling `/items-private` | Root-to-leaf references for matched route; sibling never inherits items middleware |
| Nested layout/template/error/not-found | Ordered source references only, with no runtime lifecycle claim |
| `/items/new` versus `/items/[id]` | Literal wins; equivalent parameter patterns conflict |
| IDs `1`, `2`, `4`, `3`, `5` | All syntactically match `/items/[id]`; no assertion that a database row or static artifact exists |
| Absent explicit root not-found | Error naming missing source; no hidden fallback component |
| Ordinary files; components outside app | No added endpoint |
| Source containing import-time throw/write sentinel | Discovery succeeds based on names; sentinel never executes; input bytes unchanged |
| Handler declaration missing or unknown source | Clear diagnostic, no guessed method ownership |
| URL/build adversarial cases | Policy rejects ambiguous paths; valid segment values round-trip |
| Permuted input ordering | Byte-identical manifest and stable diagnostics |
| Symlink app entry; unsupported convention | Bounded diagnostic, no traversal/accidental endpoint |

Commands to provide:

```text
node --test experiments/ui-route-contracts/tests/*.test.mjs
node experiments/ui-route-contracts/cli.mjs demo
```

The demo shows page-only, API-only and mixed segments, local loader associations, policy ancestry and at least one collision. Do not label JS fixture values as type-safe injected Tsrust data. Tests establish declaration wiring only, not execution, auth enforcement or cancellation.

## Explicit exclusions and follow-up gates

Removed from this first task: static enumeration/build publication, executing loaders, fake DI container/provider execution, scaffolding, component catalog/lockfile installation, rendering, browser transport, native adapters, JSX parsing/type inference, production middleware and HTTP integration. These remain product requirements and separately scoped follow-ups.

Core UI feasibility should next prioritize owner/state/disposal witnesses and one real paired browser/native text-input slice. Neither inherently needs async, but each needs its own bounded assignment and language/backend contract. Do not automatically proceed from this task into those experiments.

Actual async loaders and Axum middleware wait for the accepted relevant S13 async, S14 task/runtime and S15 HTTP capabilities, with their transitive prerequisites. Required witnesses include suspend-safe request ownership, at-most-once `next`, body consumption, cancellation/drop, request isolation and checked context propagation. Async syntax alone is insufficient. `.tsrx`, inferred loader-to-page types, metadata checking, editor invalidation and provider capabilities are additional integration dependencies, not delivered by S13–S15.

## Ready-to-use assignment

Implement only this nonexecuting route-manifest prototype in a separate worktree from the agreed committed baseline. Own `experiments/ui-route-contracts/` only. Use Node built-ins and focused failing tests. Discover the bounded special-file tree, consume explicit mock checked method declarations, validate page/API collisions and page-local loader pairing, and demonstrate deterministic matching/building and middleware ancestry. Do not parse or execute app source, implement runtime behavior, install components or generate project source. Follow project checkpoint/formatting requirements in the isolated tree. Hand off focused test results, one demo, a small versioned experimental schema and precise compiler dependencies, then stop; no merge or follow-on implementation is authorized by this brief.
