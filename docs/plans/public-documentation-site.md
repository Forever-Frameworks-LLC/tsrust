# Public documentation site: Starlight migration

Status: owner requested 2026-10-28; plan first, implementation next. The owner selected GitHub Pages after reviewing its size/bandwidth limits. This plan
extends the active language/CLI/docs polish goal; it does not revive the original
Tsrust UI product. Audience: developers installing Tsrust, learning its ownership
and language rules, building native backends, and maintaining deployed services.

## Outcome and priority

Publish a clean, accessible static Astro Starlight site with small, task-oriented
pages. Keep one authoritative source for each public explanation. Migrate reviewed
content and repository links incrementally, deleting superseded public Markdown
only after its replacement and inbound links are verified. Preserve source
fixtures, internal plans, milestones, historical evidence and owner edits.

1. Record this information architecture, ownership policy and acceptance checklist.
2. Build the Starlight foundation and the complete installation-to-first-service
   journey; validate it locally before connecting production hosting.
3. Migrate current backend/language/tooling guides in coherent sections. Audit
   every candidate page for supported behavior rather than importing stale plans.
4. Update README and repository entry points, then remaining active inbound links.
   Remove duplicate migrated prose and obsolete public-navigation machinery only
   after its consumers are replaced. Keep the retained UI implementation intact.
5. Publish through a dedicated static-docs workflow and verify actual public URLs.
6. Resume remaining backend platform/milestone/performance acceptance. Existing
   Windows, Linux PR CI and local regression runs may finish while this work proceeds;
   retain their results and repair failures without restarting completed groups.

The static site becomes the next implementation priority. It must not trigger
optimized compiler release builds or require Tsrust/Rust to render Markdown.
Executable example acceptance stays separate from static-site generation.

## Information architecture

Primary navigation uses six expandable sections, at most two sidebar levels.
Search and the page table of contents handle deeper detail. Use explicit sidebar
ordering; never expose internal documents through automatic directory discovery.

| Section | Reader's question | Initial routes (relative to the site base) |
| --- | --- | --- |
| Start here | How do I get a working program? | `/`, `/start/installation/`, `/start/quickstart/`, `/start/editor/`, `/start/project-structure/` |
| Language | How does supported Tsrust behave? | `/language/overview/`, `/language/types/`, `/language/ownership/`, `/language/functions/`, `/language/control-flow/`, `/language/errors/`, `/language/modules/`, `/language/async/` |
| Build backends | How do I build and operate a service? | `/backend/overview/`, `/backend/applications/`, `/backend/http/`, `/backend/request-bodies/`, `/backend/validation/`, `/backend/api-clients/`, `/backend/data/`, `/backend/deployment/` |
| Standard library | Which APIs can I use? | `/library/overview/`, followed by reviewed module pages for strings, collections, bytes, files, processes, HTTP, JSON, testing and databases |
| Tools | How do I manage my project and installation? | `/tools/cli/`, `/tools/testing/`, `/tools/formatting/`, `/tools/rust-setup/`, `/tools/rust-packages/`, `/tools/install-and-update/`, `/tools/troubleshooting/` |
| Project | What is supported and how do I contribute? | `/project/status/`, `/project/contributing/`, `/project/releases/`, `/project/security/` |

These are target destinations, not permission to publish empty placeholders.
Publish only completed pages in navigation. An overview explains supported scope
and links to available detail; it must not promise an unimplemented feature.
Add one page when a topic outgrows a single reader task, retaining stable URLs.
No version selector or translated content until there is maintained content for it.

### Reader journeys

- First use: overview → installation (release or source fallback) → native-tool
  setup → quickstart → editor → language ownership essentials.
- First service: quickstart → backend application → routes and checked bodies →
  independent client SDK → test → deployment and compatibility.
- Daily reference: search or section overview → exact behavior/signature/example →
  related operation; long API detail belongs in reference rather than onboarding.
- Troubleshooting: reported symptom → diagnostic command → cause-specific repair
  → verification. Include stale editor paths, missing Rust/linker/sources and
  unmanaged versus managed installations.

### Page contract

Each page has a unique title, concise description and one primary task. Start with
what the reader can accomplish and any actual prerequisites. Then show the shortest
working example, expected output/behavior, relevant limits and errors, and two or
three meaningful next links. Reference pages specify arguments, defaults, mutation,
return values and errors. Use short headings, readable code blocks and focused
callouts; do not fill pages with gate numbers or implementation provenance.

Use the labels “CLI”, “language server”, “backend application”, “SDK”, “managed
installation” and “native build” consistently. Distinguish downloading the CLI/LSP
from preparing Rust tools to compile a native application.

## Visual and technical foundation

Use Starlight's standard responsive layout, search, sidebar, table of contents,
code blocks, keyboard navigation and theme controls. Apply restrained branding,
comfortable text width and a consistent accent. No custom component system,
remote fonts, analytics, accounts, server functions or decorative animation.
Check mobile and desktop rendering, keyboard focus, contrast and long code lines.

Use a dedicated `website/` package with pinned dependencies and a committed lockfile.
Canonical public pages live at `website/src/content/docs/` as Markdown with Starlight
frontmatter; MDX is reserved for a concrete content need. Standard static output
makes the same site deployable to GitHub Pages, Cloudflare or Netlify. Keep the
host/base URL in one configuration location and handle project-subpath assets,
search and internal links. Do not scatter hardcoded hostnames through site pages.

Site navigation, page title/description, content layout, callouts and code rendering
reuse Starlight components. Keep custom CSS small and attributable to an observable
readability or branding need. Tsrust code highlighting may alias TypeScript at
first; explain that this does not imply full TypeScript compatibility.

## Content ownership and migration

| Existing source | Treatment |
| --- | --- |
| README.md | Keep concise product overview, install/quickstart pointer, status, contribution and license links; point public guidance at deployed docs after verification. |
| docs/content/getting-started/ | Transform into canonical Starlight onboarding; preserve facts, convert custom metadata to frontmatter and fix links. Remove old copies after the retained docs app's consumption is deliberately handled. |
| docs/guides/ current backend/language/tooling guides | Audit and migrate by topic. Split large mixed guides, remove superseded public copies, update active inbound links and tests. |
| editors/vscode/README.md | Keep extension installation/package/marketplace-specific essentials; link full editor and troubleshooting guidance to the site. |
| examples/README.md and per-example READMEs | Keep local run prerequisites and commands; link conceptual explanations to canonical site pages. |
| CONTRIBUTING.md / SECURITY.md | Retain GitHub-discoverable contribution and reporting entry points. Keep repository-specific instructions here; link complementary site explanations without duplicating them. |
| docs/plans/, docs/milestones/, docs/handoffs/, measurements and agent state | Keep internal/historical Markdown outside the public site. Do not turn incomplete plans into reference documentation or rewrite sealed evidence. |
| docs/src/, original UI references and fixtures | Preserve retained UI source and historical acceptance. Identify their content-generator dependencies before deleting any consumed Markdown. Avoid permanent duplicated backend guidance merely to support the old presentation. |
| crates/tsrust-lsp/src/builtins.json | Remains canonical compiler-owned IntelliSense data. Retain executable example verification; do not create a second handwritten API catalog. Reuse validated data for site reference where warranted. |

Create a tracked migration map during implementation: old file/anchor → new route/
anchor → disposition (moved, retained operational, retained historical, pending).
Use it to audit Markdown links across the repository and code/config references to
moved files. Preserve useful external deep links with static redirects where an old
public URL actually existed. Do not add placeholder files containing only redirects
throughout the repository. Historical evidence may retain original paths, clearly
identified as historical; active instructions must resolve.

Generated documentation checks currently consume custom metadata, API examples and
UI content. Separate these consumers before removing old pages: public rendering
moves to Starlight; compiler behavior/examples remain checked by existing focused
fixtures. Adapt distribution link checks and any tests that assume an old page path.
Do not delete coverage, loosen assertions or erase historical records to make the
migration green.

## Hosting and automation

GitHub Pages is feasible for this public repository with existing administrator
access, but is not unlimited: published site ≤1 GB, deployment timeout 10 minutes,
soft bandwidth 100 GB/month. The soft ten-builds/hour limit does not apply to custom
Actions builds. Standard hosted runner time is free for public repositories and
Pages; artifact/cache storage still has separate limits. Sources checked 2026-10-28:
[Pages limits](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits)
and [Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions).

Build provider-neutral static output. For the selected GitHub Pages host, use the repository project URL initially and official Pages upload/deploy
Actions pinned to reviewed commits. Build PRs with read-only permissions and no
production secrets. Only the deployment job gets pages:write/id-token:write through
the github-pages environment. Publish accepted main content; cancel superseded docs
builds, cache npm downloads, and retain deployment artifacts briefly. Trigger only
on relevant site/content/config changes; keep compiler release automation separate.

Respect the existing stag/main protections. A docs workflow must not bypass code
checks or publish unmerged PR content. Avoid requiring a docs job on unrelated PRs
if path filters would leave a required status missing. Once the cheap docs-only path
is proven, consider a narrowly verified documentation-only CI classification so
prose changes need not rebuild compiler consumers; do not weaken current gates as
an incidental migration step.

Cloudflare/Netlify remain alternatives if Pages limits or access prove unsuitable;
inspect their current free-tier limits and account access before selecting them.
No provider is assumed unlimited. Hosting activation follows a locally validated,
reviewable site and the selected provider's access requirements.

## Acceptance checklist

- [x] GitHub Pages selected with documented limits; repository is public and administrator access verified. Pages activation remains an implementation step.
- [x] Reproducible clean install and static production build; no Rust/compiler prerequisite. Documentation run 40748887897 at 86435b8846badcfaf836af9c36da3fa56379f3a6 passed npm ci, production build, website-tool tests and generated links. The workflow uses Node/Python only; production deployment was correctly skipped for the PR.
- [ ] Complete installation → first native program → first backend → editor journey.
- [ ] Every published page reviewed for current supported behavior and runnable examples.
- [ ] Logical sidebar, working search, stable routes, mobile/desktop and keyboard checks.
- [ ] Migration map covers public source pages, old anchors and retained consumers.
- [ ] Duplicate migrated Markdown removed; internal evidence and owner files preserved.
- [ ] Active repository links and site links resolve, including anchors and base paths.
- [ ] Existing affected compiler/content/distribution checks retain their coverage.
- [ ] Cheap docs PR validation and protected production deployment succeed.
- [ ] Public root, deep links, search and assets verified at the actual deployed URL.
- [ ] README and repository entry points link to the verified public documentation.
