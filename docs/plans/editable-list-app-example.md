# Editable-list app: concrete API design example

Status: illustrative design, 2026-10-11. All imports, framework types, JSX, async handlers, config and context helpers below are proposed APIs, not implemented or compile-verified Tsrust. The example makes contracts reviewable; it is not permission to bypass compiler ownership rules. See [HTTP contract](axum-hono-http-ui-contract.md) and [UI vision](../../UI_VISION.md).

## Example policy, not mandatory framework policy

This example deliberately chooses session authentication, record visibility checks, CSRF-protected forms and no-store responses. The author installs those helpers explicitly; another app can choose different policies or omit helpers it does not need. The framework must run the installed chain consistently, provide checked context and avoid implicit data exposure. The sample repository owns access and revision rules; the framework cannot infer them. See [the responsibility boundary](../../UI_VISION.md#application-policy-and-framework-guarantees).

## File tree and endpoint ownership

```text
app/
  middleware.tsr                request ID, safe error mapping, response policy
  layout.tsrx                   persistent header + named main outlet
  not-found.tsrx                explicit fallback UI
  error.tsrx                    explicit failure UI
  login/
    page.tsrx                   GET /login (public sign-in screen)
  health/
    route.tsr                   GET /health (route only)
  items/
    middleware.tsr              authenticated identity for the whole subtree
    loader.tsr                  special file: typed data injected into items/page
    item-types.tsr              colocated page-data contracts
    page.tsrx                   GET /items (list + create form)
    route.tsr                   POST /items (create)
    [id]/
      middleware.tsr            validate id; preserve parent auth
      loader.tsr                special file: typed data injected into [id]/page
      template.tsrx             new editor instance when item path changes
      page.tsrx                 GET /items/:id (edit screen)
      route.tsr                 POST /items/:id (rename/delete by explicit intent)
      not-found.tsrx            item-specific missing-item UI
services/
  items.tsr                     application repository contract + adapter
  sessions.tsr                  session verification contract + adapter
components/
  ui/
    button.tsrx                editable source installed from bundled catalog
    text-input.tsrx             editable source installed from bundled catalog
  item-form.tsrx               application-owned reusable component
ui-components.lock.json        proposed installed-component version/provenance file
```

No `views/` directory is required. `app/` owns routing and route-local UI/data modules; `components/` owns reusable UI source and creates no endpoints. Small route-specific types stay beside the route. Shared types can move to an ordinary shared module when actual reuse justifies it. Folder names outside the configured app root do not acquire special-file semantics.

The sketches below use `@platform/ui` for conceptual primitives. The intended higher-level Button/TextInput wrappers may instead be installed as visible `components/ui/*.tsrx` files and imported locally (for example `@/components/ui/button`). Primitive runtime support remains a separate pinned dependency; copying a wrapper alone cannot supply a renderer. Exact exports/imports await the primitive/component split. See UI_VISION section 19 for offline installation and immutable component versions.

Root has neither page nor route: its special files apply to descendants; GET `/` selects the root not-found handler unless a root endpoint is added. `health` has only a route, `login` only a page, and both item segments have both. `loader.tsr` is a special file discovered alongside its page; the page does not import or re-export it. For this web example it runs server-side, and the build must enforce that boundary without exporting a public loader endpoint. A loader without a colocated page never runs and is reported as unpaired. A page without a loader remains valid.

Recommended first route registration: relative `/` means exactly the containing segment. Catch-all/prefix registrations are outside this example. `page.tsrx` plus `router.get("/")` is a compile/build conflict, even if content types differ. Page HEAD handling is proposed to follow GET without the body; explicit conflicts also need a diagnostic. Automatic OPTIONS and 405 behavior must preserve applicable middleware and be documented before implementation.

## Root middleware and providers

```typescript
// app/middleware.tsr — proposed framework-owned adapter helpers
import { requestId, errorResponses, privateResponses } from "std:http";

export function middleware() {
  return [requestId(), errorResponses(), privateResponses()];
}
```

The list executes in source order on entry and reverse order on return. Helpers represent contracts to implement, not existing security guarantees. `errorResponses` converts downstream typed failures to sanitized responses; `privateResponses` ensures the private example has explicit no-store policy, including downstream error responses. Request-ID middleware must reject/replace invalid incoming IDs rather than trust arbitrary header content. Failure before error middleware still needs a safe outer server failure response.

At application composition, explicitly provide session and item-service adapters. `ctx.services.sessions` and `ctx.services.items` below refer to those checked registrations. They are not automatically discovered global singletons. Pool sharing, per-request borrows and destruction remain compiler/runtime contracts. A fake implementation can replace either service in tests. Browser bundles receive neither provider nor credentials.

## Authentication and route parameters

```typescript
// app/items/middleware.tsr
import { requireSession, protectFormMutations } from "std:http";

export function middleware() {
  return [requireSession(), protectFormMutations()];
}
```

`requireSession` verifies the session using the supplied service and establishes typed `ctx.user` for descendants. Absent/invalid sessions stop the chain. Full-page navigation can redirect to `/login`; enhanced navigation returns a typed navigation instruction, not login markup inserted into the item editor. API responses use an explicit 401 policy. Return destinations must be same-origin, validated app paths.

`protectFormMutations`, explicitly installed by this app, enforces its configured origin/CSRF policy for unsafe methods. For these HTML forms, it checks a synchronizer token against the session using one bounded form parse; the handler receives that same parsed form. This explicitly chooses bounded reuse of decoded form values, not two reads of the request stream. GET receives a token for form rendering but performs no mutation. The loader explicitly passes the token to page props, and HiddenInput submits it; injection alone does not validate a request. Omitting the helper removes this protection and makes its injected context unavailable. If code still accesses `ctx.csrfToken`, report a missing capability/type diagnostic; applications that do not request that capability remain valid. Streaming/upload bodies are outside this contract.

```typescript
// app/items/[id]/middleware.tsr
import { routeParams, opaqueId } from "std:http";

export function middleware() {
  return [routeParams({ id: opaqueId() })];
}
```

This turns decoded raw params into a declared validated ID type without changing authentication. It does not turn arbitrary strings into integers or authorize record access. Invalid parameter syntax produces a defined 400 outcome; absence/denial is an application result. The shape syntax and type propagation are design sketches requiring compiler acceptance.

The resulting order for a POST to `/items/4` is root → items auth/CSRF → id validation → action. GET document/fragment follows the same ancestry with read-only mutation protection. Authentication precedes private existence lookup.

## Explicit server loaders

```typescript
// app/items/loader.tsr
export default async function loader(ctx) {
  const items = await ctx.services.items.listVisible(ctx.user);
  return { items, csrfToken: ctx.csrfToken };
}

// app/items/[id]/loader.tsr
export default async function loader(ctx) {
  const item = await ctx.services.items.findVisible(ctx.user, ctx.params.id);
  if (item === null) return ctx.notFound();
  return { item, csrfToken: ctx.csrfToken };
}
```

Concrete types must be declared/inferred by the accepted implementation. This application projects item data to `{ id, title, revision }` and omits database/session internals. That is application logic, not automatic framework redaction. The values are visible to the authorized recipient, not necessarily to the public. Server-side loader injection alone does not serialize the whole result. `findVisible` deliberately returns the same absent outcome for missing and nonvisible records; infrastructure failures are typed failures, not null. `notFound` is a typed control outcome dispatched to a visible handler, not a hidden exception/panic or nullable page-data value. Exact syntax is still open.

Page requests run their colocated loader only after the inherited policy chain. Loaders return data, not a response representation. Full HTML and partial HTML invoke the same loader. POST/API handlers do not implicitly run it; a subsequent page GET after navigation does. Parent page loaders do not automatically run for descendants. No raw-data endpoint is generated by this convention. This loader cannot run unchanged at build time because it requires a live request/session and token. A static variant needs a build-capable provider and finite parameter enumeration; the application chooses its output data and deployment access. Static generation itself does not require the data to be publicly accessible.

The proposed default export supplies the entry function in the loader file; it is not imported by the page. The compiler injects the successful awaited result as typed `data`. For example `data.item.title` is checked against the item-loader result; `data.item.unknown` is a source error. Not-found/redirect/failure outcomes select control/error handling before the page renders. Changes to loader results update page diagnostics, completion and hover without executing loader code in the editor. Exact function annotation and context syntax still need implementation design.

## Layout and pages

```tsx
// app/layout.tsrx
import { View, Text, Link, Outlet } from "@platform/ui";

export default function Layout() {
  return (
    <View className="min-h-screen bg-background text-foreground">
      <View role="banner" className="border-b p-4">
        <Text>My items</Text>
        <Link href="/items">Items</Link>
      </View>
      <Outlet name="main" />
    </View>
  );
}

// app/items/page.tsrx
import { View, Text, Link, Form, TextInput, HiddenInput, Button } from "@platform/ui";

export default function Page({ data }) {
  return (
    <View className="p-4 gap-4">
      <Text role="heading">Your items</Text>
      <Form method="post" action="/items">
        <HiddenInput name="csrf" value={data.csrfToken} />
        <TextInput name="title" label="New item" required />
        <Button type="submit">Add item</Button>
      </Form>
      {data.items.map(item => (
        <View key={item.id}>
          <Link href={{ route: "/items/[id]", params: { id: item.id } }}>
            {item.title}
          </Link>
        </View>
      ))}
    </View>
  );
}
```

Named `Outlet`, structured href, conventional typed loader injection, keyed children and prop destructuring are proposed framework/language surfaces, not claims of present compiler support. Link generation uses the route manifest and encodes each parameter. Web Form produces a real form with normal submission as fallback; native uses an explicit action adapter rather than hosting HTTP. The example header is intentionally public constant content; a user-specific header would need its own data/invalidation policy.

```tsx
// app/items/[id]/template.tsrx
import { View } from "@platform/ui";

export default function Template({ children }) {
  return <View className="p-4">{children}</View>;
}

// app/items/[id]/page.tsrx
import { View, Text, Form, TextInput, HiddenInput, Button } from "@platform/ui";

export default function Page({ data }) {
  const target = { route: "/items/[id]", params: { id: data.item.id } };
  return (
    <View>
      <Text role="heading">Edit item</Text>
      <Form method="post" action={target}>
        <HiddenInput name="csrf" value={data.csrfToken} />
        <HiddenInput name="revision" value={data.item.revision} />
        <TextInput name="title" label="Title" defaultValue={data.item.title} required />
        <Button type="submit" name="intent" value="rename">Save</Button>
        <Button type="submit" name="intent" value="delete" formNoValidate>Delete</Button>
      </Form>
    </View>
  );
}
```

For this example, template identity changes when the matched route/decoded item ID changes. A refresh or validation error for the same item does not remount it by default. Unsaved text belongs to the local form instance; changing data must not blindly overwrite it. A full navigation naturally reconstructs the document. Visual wrapper/layout rules do not authorize HTTP requests or contain server credentials.

## Method routes: create, rename and delete

```typescript
// app/items/route.tsr
export function routes(router) {
  router.post("/", async ctx => {
    const values = ctx.form.validateCreateItem();
    const item = await ctx.services.items.createForUser(ctx.user, values.title);
    return ctx.navigate({ route: "/items/[id]", params: { id: item.id } });
  });
}

// app/items/[id]/route.tsr
export function routes(router) {
  router.post("/", async ctx => {
    const values = ctx.form.validateItemAction();
    if (values.intent === "delete") {
      await ctx.services.items.deleteForUser(ctx.user, ctx.params.id, values.revision);
      return ctx.navigate("/items");
    }
    await ctx.services.items.renameForUser(
      ctx.user, ctx.params.id, values.title, values.revision
    );
    return ctx.navigate({ route: "/items/[id]", params: { id: ctx.params.id } });
  });
}

// app/health/route.tsr — no page here, so GET / is valid
export function routes(router) {
  router.get("/", ctx => ctx.text("ok"));
}
```

The two validation methods represent application-defined schemas wired into the proposed typed form context. They are not universal magic built-ins. Create requires a bounded nonblank title; action validates the exact rename/delete intent and revision, and requires a title only for rename. Unknown intents are rejected rather than becoming rename. Submitted extra fields cannot set user/tenant/ownership. `required` on a UI input is convenience, never server validation. Delete skips browser title validation with `formNoValidate` while retaining server intent/revision/CSRF checks, so clearing the title does not prevent deleting the item.

To satisfy this example's policy, its application repository must atomically check actor access and revision at the write, not rely on an earlier GET. This is not an automatic framework check. Missing/nonvisible → declared not-found policy; stale revision → 409 conflict; invalid form → 422 with safe field feedback; service failure → safe 500. The action outcome contract must retain the submitted form values and field errors for validation/conflict rendering. Do not automatically replay failed POSTs or claim that disconnect/cancellation rolls back committed writes. Duplicate-create retry policy is an additional action contract before production use.

No handler registers GET at either item segment: its page already owns that method/path. Both route and page are optional; a loader file alone does not create an HTTP endpoint.

## Visible fallback files

```tsx
// app/not-found.tsrx
import { View, Text, Link } from "@platform/ui";
export default function NotFound() {
  return <View><Text>Page not found.</Text><Link href="/items">Open items</Link></View>;
}

// app/items/[id]/not-found.tsrx
import { View, Text, Link } from "@platform/ui";
export default function NotFound() {
  return <View><Text>This item is unavailable.</Text><Link href="/items">Back to items</Link></View>;
}

// app/error.tsrx
import { View, Text, Link } from "@platform/ui";
export default function ErrorPage({ error }) {
  return <View><Text>Something went wrong.</Text><Text>{error.reference}</Text><Link href="/items">Open items</Link></View>;
}
```

`error.reference` is an opaque public correlation ID, never a stack trace/token/database message. Nested not-found can handle an item loader's absent outcome; unmatched URLs use the defined matched-ancestry fallback rule. Policies still apply. If the root layout itself fails, render root error in a minimal document rather than trying that layout again; if the error handler also fails, return a minimal safe failure response. API errors remain typed HTTP/data responses, not automatically these UI components. The login page/authentication ceremony is outside this example; it must be implemented before deploying the protected app.

## Full document, partial navigation and form updates

Transport names below are illustrative and intentionally narrow. A future adapter may use htmx or another small transport without changing the application files.

| Request | Common work | Result |
|---|---|---|
| Direct `GET /items` | Root middleware → items middleware → list loader | 200 full document containing root layout/header and main outlet |
| Enhanced `GET /items/4` | Same ancestry, plus ID middleware → item loader | 200 declared main-outlet fragment; header stays mounted |
| Normal `POST /items/4` | Complete inherited chain → bounded validation → authorized revision-checked mutation | 303 to canonical GET URL on success |
| Enhanced `POST /items/4` | Exactly the same policy/validation/mutation | Typed navigation result, then a new protected GET for the destination fragment |
| Same-item data refresh | Complete GET chain → item loader | Region update reconciled against local form/dirty state; no silent overwrite |
| Missing item | Complete applicable chain → absent loader outcome | Explicit item not-found UI, 404 status, full or region representation |

For a concrete prototype, an enhanced GET can request a versioned region representation using an Accept value and a framework-owned outlet ID. The server only accepts the outlet valid for that route/active layout contract, never an arbitrary CSS selector/template filename. Fragment responses include or accompany title/navigation metadata. Separate representation caches (`Vary` if headers select content) and use no-store for this private example. A representation request cannot bypass authentication or force private fields into a raw-data response.

`ctx.navigate` is a typed action outcome, not raw HTML: normal submission maps it to 303; enhanced submission maps it to an explicit versioned navigation response containing a validated local URL. The client performs GET next, never resubmits POST. Define that response's status/content type before implementing transport. Validation/conflict replies instead render the appropriate page/form feedback with 422/409; native maps the same outcomes to local navigation/errors. Fragments with 404/422 must be handled intentionally rather than dropped as generic network failures.

Example sequence:

```text
1. Open /items          → full header + list
2. Click item 4         → fetch item 4 region; main becomes editor
3. Type a title         → local form state only; no request per keystroke
4. Save                 → POST /items/4 → authorized update → navigation result
5. Refresh item 4 GET   → accepted saved revision/data; main updates
6. Delete               → POST /items/4 → deletion → navigate /items
7. Back to deleted item → fresh protected GET → visible not-found screen
```

The header's instance stays mounted for steps 2–7 in enhanced navigation, but middleware still runs on every HTTP request. Navigation generations prevent a late response for item 4 replacing item 2 after the user moved on. Editing while a save is in flight needs an edit-generation check: preserve edits made after submission rather than resetting to the old saved title. The deleted-item back-navigation witness requires revalidation, not unconditional restoration from a history cache. Updating identity/tenant invalidates relevant layout data and outstanding responses.

## Acceptance and unresolved API details

Test page-only, route-only, both and neither; GET collision; exact middleware traces; early auth rejection; no body double-consumption; same policy on fragments; object authorization on writes; stale revisions; escaped titles; header instance retention; form composition/focus; validation values; 404/500; response order; native outcome mapping. These are acceptance witnesses for this explicitly protected example and its selected helpers, not mandatory policies for every application or tests executed for this document. Include a separate public/no-helper route witness and verify that unrendered loader fields are absent from response artifacts.

Still to decide: final imports/types, exact loader entry signature (default function illustrated), typed schema/provider registration, template keys beyond this example, exact region/action protocol, dirty-form navigation prompts and platform-specific action providers. Loader discovery and typed data injection without page imports/re-exports are now owner direction. The design deliberately exposes remaining choices instead of claiming the syntax is ready to compile.
