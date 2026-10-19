# Checked documentation content

The documentation project uses one build-time content source for web and native.
`tsrust docs content --cwd docs` reads the `documentation` section of
`tsrust.config.json`, validates authored Markdown, examples and public declarations,
and publishes a self-contained typed module under `target/docs-content/`.
It performs no package fetch or benchmark run. The pinned CommonMark dependency is
part of the installed compiler; runtime applications do not need that parser.

## Authoring pages

Keep authored pages in the configured `content` directory. Each begins with a JSON
metadata comment, followed by exactly one H1:

```markdown
<!-- tsrust-docs
{"id":"ui-state","group":"foundations","order":10,"status":"partial","targets":["web","native"],"examples":["counter"]}
-->
# State

Explain observable behavior here.
```

IDs and groups use lowercase ASCII letters/digits with single internal hyphens.
IDs are stable author-owned identities. Directory-derived routes are lowercase
hyphenated segments; `index.md` names its directory route. Duplicate IDs/routes,
missing headings, invalid metadata and unsupported targets fail the build. Explicit
status describes page readiness; pending inventory destinations are never working
component previews. Targets must be distinct and nonempty.

Optional `component` identifies a pinned component. Optional `api` entries such as
`{"receiver":"string","name":"trim"}` reuse the actual IntelliSense catalog.
Component declarations and inherited fields come from a successful compiler check,
with original source annotations and documentation. They do not infer public
interfaces from sample calls or independently maintained prop lists.

## Markdown and source policy

The parser is pinned pulldown-cmark 0.13.4, using CommonMark plus tables, task lists
and strikethrough. The output is a flat preorder typed tree with parent indices,
UTF-8 source ranges, heading IDs and exact original Markdown. Strong/emphasis,
inline code, links, lists, quotes, rules, images and fenced code retain structure.
This build contract is not yet acceptance of the full shared visual renderer.

Heading IDs lowercase Unicode letters, retain letters/digits/underscores and join
whitespace/hyphen runs with a hyphen. Other punctuation is omitted. Empty IDs use
`section`. Collisions get a unique numeric suffix, including collisions with an
already suffixed heading. No Unicode normalization is applied.

Relative `.md` links resolve within the content graph; absolute directory routes
and existing heading fragments also resolve. Missing pages, references, fragments
and local assets fail. Internal paths currently use literal directory paths and
Unicode fragments: percent-encoded paths, query variants and file-routing parameters
are unsupported here. This content-link contract is separate from the unfinished
application router. Local assets cannot traverse symlinks or escape the project.

HTTP/HTTPS and nonempty mailto links are permitted. Protocol-relative links,
unknown schemes, whitespace/control characters, backslashes and unsafe delimiters
reject. Raw HTML remains escaped literal content; it never becomes a browser
injection or executable native import. Images require nonblank alt text. Math,
footnotes, MDX, arbitrary HTML and extension execution are not enabled.

Fenced code keeps CommonMark display text and original source bytes separately.
CRLF normalization for display never overwrites copied source. Actual examples
store the complete exact file plus SHA-256 and contiguous token ranges. Tsrust/
TSRX highlighting uses the compiler lexer, including async syntax and closing JSX
tags. Unknown fence languages remain readable unhighlighted text. Other-language
highlighting and rich shared text rendering remain pending.

## Examples, output and ownership

Configure each example's stable `id`, local `source`, named `export` and `kind`
(`component` or `code`). Only checked configured component exports appear in the
generated preview registry. Code examples are source-only. Unknown IDs throw;
Markdown cannot invoke an arbitrary import. Native assertions verify executable
examples separately; the content build type-checks them without running them.

`catalog.json` is derived data. `catalog.tsr` contains the same content and shared
checked declarations, uses `JSON.parse<DocCatalog>` to validate/materialize an owned
snapshot and exposes the checked preview registry. This deliberate conversion
avoids pretending inferred JSON records implicitly become named records or that
empty arrays acquire arbitrary element types. Unknown build metadata fields are
skipped under the ordinary checked JSON limits; numeric/source offsets remain
exact signed integers. There is no unchecked cast or JSON execution.

A successful content build validates everything before publication. The typed
module is replaced atomically; unchanged output retains its mtime. A failed build
preserves the previous publication. Generated files are not source backups and
must not be edited. Checks/editor analysis stay read-only. Automatic build/dev
content preparation, shared Markdown rendering, navigation and offline full-text
search are subsequent application integration work.
