# Server HTML and JSX

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Web proof.

**Scope:** Scope typed HTML, JSX components, escaping, streaming, and the native web showcase.

**Planning deliverable:** A typed HTML prototype with dynamic escaping and component composition; streaming as a separate increment.

**Dependencies to coordinate:** [http-and-hono](http-and-hono.md), [ownership-and-memory](ownership-and-memory.md), [error-handling](error-handling.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="web-19"></a>

## 19. JSX / TSX as a First-Class Backend Feature

Hono provides an important second test:
JSX-like server-rendered HTML.
Investigate supporting:
function Layout(props: { children: Html }) {
    return (
        <html>
            <body>
                {props.children}
            </body>
        </html>
    );
}
and:
app.get("/", ctx => {
    return ctx.html(
        <Layout>
            <h1>Hello</h1>
        </Layout>
    );
});
without requiring React.
This should be treated as:
native structured HTML syntax
rather than emulating React’s runtime.

<a id="web-20"></a>

## 20. JSX Compilation Strategy

Investigate compiling JSX directly into optimized response-generation code.
For example:
<h1>Hello {user.name}</h1>
could conceptually lower to:
write static "<h1>Hello "
escape user.name
write static "</h1>"
rather than creating an intermediate virtual DOM.
Potential advantages:
* very low allocation
* streaming
* automatic escaping
* compile-time HTML checks
* predictable memory behavior

<a id="web-21"></a>

## 21. HTML Type Safety

Potential built-in type:
Html
Distinguish:
string
from:
Html
so raw user strings cannot accidentally be treated as trusted markup.
Potential helpers:
html(...)
escapeHtml(...)
unsafeHtml(...)
unsafeHtml() should be explicit.
This could provide stronger XSS guarantees than ordinary string-based templating.

<a id="web-22"></a>

## 22. JSX Components Without React Semantics

Do not automatically reproduce:
* React state model
* client reconciliation
* hooks
* virtual DOM
* hydration
Server-side components could simply be functions:
function UserCard(user: User): Html {
    return (
        <article>
            <h2>{user.name}</h2>
            <p>{user.email}</p>
        </article>
    );
}
This retains React/JSX familiarity without introducing a React runtime.

<a id="web-23"></a>

## 23. Streaming HTML

Investigate:
return ctx.streamHtml(
    <Page>
        <AsyncComponent />
    </Page>
);
Potentially compile async HTML components into native streaming output.
This could become a powerful backend feature without recreating React Server Components wholesale.

<a id="web-37"></a>

## 37. Core Web Vision

One potential long-term differentiator deserves explicit investigation:
The language should feel like Hono, TypeScript, and server-side JSX were designed from the beginning for native compilation rather than layered on top of JavaScript.
A possible experience:
import { App } from "http";
import { z } from "validation";

const app = new App();

const UserInput = z.object({
    name: z.string().min(1),
    email: z.email(),
});

function UserPage(user: User): Html {
    return (
        <main>
            <h1>{user.name}</h1>
            <p>{user.email}</p>
        </main>
    );
}

app.post("/users", async ctx => {
    const input = await ctx.req.json(UserInput);

    const user = await db.users.create(input);

    return ctx.html(<UserPage {...user} />);
});

app.listen(3000);
The developer experience looks familiar.
The compiler/runtime underneath should be fundamentally native.
Potential internal behavior:
typed route table
+
compile-time schema
+
native JSON parser
+
ownership inference
+
native async runtime
+
zero/low-copy HTTP
+
specialized serialization
+
streaming HTML writer
+
no V8
+
no tracing GC
This is a more meaningful target than simply translating TypeScript syntax into Rust.
