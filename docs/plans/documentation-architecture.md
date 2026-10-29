# Documentation architecture: reference first, guided learning second

Status: owner-directed 2026-10-28; structure implemented 2026-10-28. Supersedes the
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
| Standard library | Globals, every public module and built-in type | Generated reference plus overview and a globals introduction |
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
| `language/optional-values/` | borrowed-optional-fallback, optional-chaining, nullish-coalescing, nullish-assignment |
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
      JSDoc `@param`/`@returns` paragraphs render as parameter and return lists.
- [x] Catalog code fences formatted with the project formatter; `tools/format-project.mjs`
      keeps them formatted.
- [x] Library overview links each module; hand-written
      [global functions and types](../../website/src/content/docs/library/globals.md) page.
      `std:result` documented as a compatibility import of the global `unwrap`.
- [x] All language chapters merged (41 feature pages into 12 chapters). 138 chapter
      examples ran with the current debug compiler, including three multi-file
      examples with their companion files.
- [x] Learn tracks: `learn/first-service` (eight backend guides as steps) and
      `learn/data` (three JSON guides). `library/futures` folded into
      `language/async`. Step navigation in `website/public/learn.js` with
      per-browser progress; pages read top to bottom without JavaScript.
- [x] Every absorbed route redirects to its new anchor; link checker validates
      redirect targets.

## Next: per-entry reference audit

Owner direction 2026-10-28: every method or section needs its own relevant
example where possible, and the whole reference needs a factual audit.

Automated checks completed so far:

- All 351 complete programs in catalog documentation type-check.
- All 273 documented `std:` exports import successfully.
- Built-in examples (strings, arrays, numbers, Map/Set, globals) produce their
  commented output; this found and fixed a compiler defect where
  `String.fromCharCode`/`fromCodePoint` alone omitted their runtime helper.
- The generator shows each example once per page, links repeats to the first
  occurrence, and lists identical inherited methods (for example schema
  `optional`/`nullable`) once.

Progress 2026-10-28:

- Every reference entry now has its own example (98 added, each run with the
  current debug compiler; outputs are recorded as `// prints:` comments).
  Router verb entries now name their HTTP method; `route()` documents its
  seven accepted methods.
- Defects found and fixed with regression tests: `String.fromCharCode`/
  `fromCodePoint` runtime gate; nullable `unwrap` results rejected by annotated
  destinations; global `unwrap` not seen by native argument-alternative hints.
- Every error code named in catalog prose exists in compiler/runtime source, and
  every numeric limit or default cited in a module's prose appears in that
  module's source (e.g. schema issue limits 64/32/1024).
- Placeholder examples (`function main() {}`) on 15 type entries replaced with
  working ones; per-operation ownership notes written for std:date,
  std:decimal, std:numeric and Heap instead of one copied note.
- A third defect fixed: console inspection of Results from std:env,
  std:numeric, std:date, std:uuid, std:decimal and std:crypto bridges failed
  Rust code generation (missing record key order).
- Prose reviewed against source: std:env, std:os, std:crypto, std:time,
  std:numeric, std:uuid, std:signals, std:decimal, std:bytes, std:collections,
  std:path (POSIX behavior; Windows-specific claims unverified locally), HTTP
  server limits/status codes, SQL and SQLite connection defaults.
- Globals moved out of "Built-in types" into their own Standard library section:
  global functions, global types, values and operators, console, JSON, Math,
  Number and Object. Built-in types now hold only value methods and the static
  helpers named after a type (Array, String, Map, Set). Every globals entry was
  reviewed: typeof rewritten (an unrelated union/ownership passage removed);
  null/undefined corrected (union arrays such as `(int | null)[]` are supported);
  clone now lists Sets; String(null) wording; unwrap has a Result example;
  JSON.stringify's JsonRaw/JSONL sentences placed in context.
- A fourth defect fixed: `Number(x)` on a copy union such as `int | undefined`
  failed Rust code generation.
- Known limitations surfaced (documented behavior, not changed): reading
  `.success` directly on a `safeParse(...)` call result needs a binding first;
  refinement paths need literal indices; inline `asString(...)` views need named
  storage.

Completed 2026-10-29: whole-catalog output check (455 programs), signature
check against native bindings, and prose review of every module and built-in
against running programs and source. Corrections from the final pass:
HttpContext sentences that only described their snippet, error codes for
status/header failures, discriminatedUnion's missing-tag code
(`missing-field`), JSON treatment of `undefined` array elements, and
uncatchable UTF-8 boundary failures for byte-offset string methods.

Not verifiable locally: Windows-specific std:path behavior.

Completed 2026-10-29 (retained guide inventory): the last 24 user-facing guides
were merged or deleted. New chapters: `language/operators` (truthiness, logical
selection and assignment, non-null assertion, `in`, unary plus/Number, `as`/
`satisfies`) and `language/errors` (try/catch/finally, thrown values, run-time
failures, Result and `match`). `language/types` gained numbers, enums and
recursive records; `language/control-flow` gained `for...in` and labels;
`tools/cli` gained diagnostic suppression. The nested-function remainder went to
functions, function values and generics. Every new example ran with the current
compiler. Two misleading diagnostics and one formatter defect found on the way
were fixed with regression tests. Library-only guides were deleted where the
audited reference covers them. Research and developer guides stay, labeled.