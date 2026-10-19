<!-- tsrust-docs
{"id":"documentation-rendering","group":"foundations","order":40,"status":"partial","targets":["web","native"],"examples":[]}
-->
# Documentation rendering

This site is a Tsrust application. The landing page lives at `/`; the shared documentation catalog lives under `/docs`.

## Content structure

The checked CommonMark tree supplies headings, paragraphs, lists, blockquotes, tables and code fences. HTML remains literal text; content never executes application code.

| Content | Web | Native |
| --- | --- | --- |
| Headings and paragraphs | Semantic elements | Accessible headings and text |
| Lists and tables | List and table elements | List and table roles |
| Code blocks | Highlighted, selectable text | Highlighted, selectable text |
| Application links | Browser anchors | Accessible links |

## Try a sequence

3. Open a **documentation page**.
4. Follow a heading link in the contents rail.
5. Return to the landing page and check that its counter retains state.

## Unicode: café 文

Heading links preserve Unicode text through browser URL encoding, history and direct entry. Native uses the same decoded heading identity.

## Inline text and source

Use **strong emphasis**, *emphasis*, ~~deleted text~~ and `inline code` in the same paragraph. Follow [state and ownership](state.md) without leaving the sentence.

```tsrust
// Unicode and whitespace stay intact when copied.
const greeting = 'Hello, café 文';
console.log(greeting.trim());
```

Copy writes the original source, including line endings and trailing whitespace. The button reports success after the clipboard write completes. If permission is denied, it displays a retry message. Browser clipboard access requires HTTPS or localhost.

The `syntaxKeyword`, `syntaxString`, `syntaxNumber` and `syntaxType` tokens in each theme mode customize highlighting. Omitted tokens use built-in light/dark colors. Unknown languages remain plain text.

## Rendering limits

Images appear as labeled references. External links display their destination as text. Raw HTML is displayed literally. Native text selection stays within one paragraph or code block; cross-paragraph selection and accessibility text-range navigation are not supported yet.
