<!-- tsrust-docs
{"id":"documentation-rendering","group":"foundations","order":40,"status":"partial","targets":["web","native"],"examples":[]}
-->
# Documentation rendering

This site is a Tsrust application. The landing page lives at `/`; the shared documentation catalog lives under `/docs`.

## Navigation and search

- Choose a page from the sidebar.
- Search page text and code without a network service.
- Use Back and Next to traverse your browsing history.

The browser supports direct document URLs and normal modified link clicks. Native builds use the same paths and keep an application history. Each route retains its own example state while mounted in the fixed catalog.

> A pending component page is an implementation inventory entry. It does not claim a finished control or a working component preview.

## Content structure

The checked CommonMark tree supplies headings, paragraphs, lists, blockquotes, tables and code fences. HTML remains literal text; content never executes application code.

| Content | Web | Native |
| --- | --- | --- |
| Headings and paragraphs | Semantic elements | Accessible headings and text |
| Lists and tables | List and table elements | List and table roles |
| Code blocks | Read-only text control | Read-only text editor |
| Application links | Browser anchors | Accessible links |

## Try a sequence

3. Open a documentation page.
4. Follow a heading link in the contents rail.
5. Return to the landing page and check that its counter retains state.

## Unicode: café 文

Heading links preserve Unicode text through browser URL encoding, history and direct entry. Native uses the same decoded heading identity.

## Current rendering limits

Inline emphasis currently retains its text without rich formatting. Paragraph links appear as separate links after their paragraph. Images appear as labeled references. Dedicated syntax-colored code, an exact-byte copy command and rich inline selection remain unfinished. Read-only code controls normalize line endings.

- [x] Content is checked before publication.
- [x] Page navigation changes the selected document.
- [ ] Rich inline rendering is complete.

See [state and ownership](state.md) for the lifetime contract and [theming](theming.md) for project-owned design tokens.
