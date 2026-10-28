# Documentation architecture: reference first, guided learning second

Status: owner-directed 2026-10-28, implementation in progress. Supersedes the
page-per-feature drift in the Language section of the public site. The routes,
page contract and hosting decisions in
[public-documentation-site.md](public-documentation-site.md) still apply except
where this plan replaces them.

## Problem

The owner reported that the site shows many example how-to articles instead of
documentation of the standard library and of how the language works. Forty-one
Language pages each cover one narrow feature (for example separate pages for
callable locals, callable fields, borrowed callable fields, callable arrays and
callable array methods). They crowd search results and do not answer "what are
the rules?" in one place. The standard library had five guide pages and no
reference for most of its 23 modules.

## Target structure

| Section | Purpose | Shape |
| --- | --- | --- |
| Start here | Install, first program, editor, project layout | Short task pages (unchanged) |
| Language | Rules of the language | About ten chapter-sized reference pages |
| Standard library | Every public module and built-in type | Generated reference plus overview |
| Learn | How-tos as one guided, interactive track | One page per track with step navigation; steps are not separate search results |
| Build backends | Operating a service: HTTP, auth, deployment | Task pages until folded into Learn tracks |
| Tools, Project | CLI, testing, installation, status | Unchanged |

### Language chapters

Each chapter states the rules first, then examples that demonstrate a rule.
Audited facts and examples from the old pages move into the chapter; the old
route redirects to the chapter anchor.

| Chapter | Absorbs |
| --- | --- |
| `language/declarations/` | declarations, assignment-values, comma-expressions |
| `language/types/` | type-aliases, interfaces, optional-fields |
| `language/optional-values/` (done) | borrowed-optional-fallback, optional-chaining, nullish-coalescing, nullish-assignment |
| `language/patterns/` | destructuring, pattern-defaults-rest, record-spread-assignment |
| `language/functions/` | arrow-functions, function-expressions, callable-expressions, optional-default-parameters, rest-parameters, call-spread, nested-functions, nested-function-captures, captured-function-parameters |
| `language/function-values/` | callable-locals, callable-fields, borrowed-callable-fields, callable-arrays, callable-array-methods, nullable-callables |
| `language/generics/` | generic-function-contexts, generic-function-values, generic-value-adapters, fresh-generic-factories, lexical-generic-functions |
| `language/arrays/` | array-literal-spread, array-from |
| `language/control-flow/` | statement-bodies, switch |
| `language/modules/` | modules, text-imports |
| `language/strings/`, `language/async/` | unchanged scope |

Order follows a reader's path: values, types, optional values, patterns,
functions, function values, generics, arrays, control flow, modules, strings, async.

### Standard-library reference

`website/tools/generate-api-reference.mjs` renders
`crates/tsrust-lsp/src/builtins.json` into `reference/library/<module>/` and
`reference/builtins/<topic>/` before every `dev` and `build`. Output is ignored by
Git. Rules:

- The catalog stays the single source. Editor hover and the site cannot disagree.
- Internal group keys (`SqlPool`, `HttpRouter`) map to public names (`Pool`,
  `Router`) in an explicit table. An unmapped or unknown group fails the build.
- `std:ui` (frozen UI product) and the compiler-internal testing runtime are
  excluded.
- Prose angle brackets are escaped; code fences are labelled `tsrust`. Minified
  regression witnesses in `examples` are not shown; readable ones with recorded
  output are.
- The docs workflow also runs when `builtins.json` changes.

Hand-written library pages (`library/json`, `typed-json`, `json-files`, `futures`)
remain as guides under the module reference until the Learn track absorbs them.

### Learn: interactive guide

One Starlight page per track (first track: "Build your first service"), using a
small inline script for step navigation, progress kept in `localStorage`, and
`<details>` fallbacks so the page works without JavaScript. Steps link into the
reference instead of restating it. Tracks set `pagefind: false` on nothing that
holds reference content; the track page itself is indexed once.

Candidate tracks: first service (applications, HTTP, request bodies, API client,
testing, deployment); data (JSON, typed JSON, schema, SQL); concurrency (async,
futures, threads).

## Migration rules

- Merge one chapter at a time. Keep every audited fact and runnable example that
  explains a rule; drop repetition and walkthrough narration.
- Add a redirect from each absorbed route to its chapter anchor in
  `astro.config.mjs`. Update inbound links in the site, README and repository docs.
- Delete absorbed source pages in the same change. No empty placeholders.
- A merge is not an audit. Facts copied from audited pages keep their audit status
  (see [public-documentation-content-audit.md](public-documentation-content-audit.md));
  new statements need source or compiler verification.
- Validate each change with `npm --prefix website run build` and
  `python3 website/tools/check_links.py website/dist`. Run examples with the
  current compiler when a statement about output changes.

## Progress

- [x] Generated standard-library and built-in reference (28 pages, 103 catalog groups).
- [x] Library overview links each module to its reference page.
- [x] `language/optional-values/` merged from four pages; examples and rejection
      diagnostics (`borrowed-view`, `view-conflict`, `??` with `&&`, `??=` type)
      re-verified with the current debug compiler; old routes redirect.
- [ ] Remaining language chapters (merge one at a time; see table).
- [ ] Learn track: first service.
- [ ] Fold remaining library guides and backend how-tos into Learn tracks.
