---
title: Serve public static files
description: Mount deployment-owned frontend assets with explicit path and fallback rules.
---

Serve a built frontend alongside your API using `Router.serveStatic`. Mount only
public deployment assets; the directory must exist before the server starts.

## Mount a frontend

Build your frontend into `web/dist`, configured for the URL prefix `/web/`.
Then register the mount before starting the listener:

```tsrust
import { Router } from 'std:http';

async function main(): Task<void> {
  const app = new Router();
  app.serveStatic('/web', 'web/dist', 'index.html');
  const server = await app.listen('127.0.0.1:3000');
  await server.wait();
}
```

Open `http://127.0.0.1:3000/web/`. The example requires `web/dist/index.html`.
Relative filesystem paths resolve when the mount is registered; use an absolute
path if the process may start from a different working directory.

| Argument | Meaning and constraints |
| --- | --- |
| `prefix` | A non-root URL path such as `/web` or `/admin`, without a trailing slash. Each segment permits ASCII letters, digits, hyphens and underscores. |
| `directory` | An existing directory containing public files. |
| `fallback` | Optional existing relative `.html` file inside that directory. Omit it or pass an empty string to disable fallback. |

Add another mount with `app.serveStatic('/admin', 'admin/dist', 'index.html')`.
Prefixes cannot overlap: `/web` and `/web/admin` cannot be mounted separately.

## Requests and fallback behavior

Files use streamed GET/HEAD responses with MIME types, range requests and
conditional dates. Earlier matching routes retain registration priority; other
HTTP methods continue to later application routes.

| Request | Result |
| --- | --- |
| `/web` | 307 redirect to `/web/`, preserving the query string. |
| `/web/` or an existing directory | Serve its `index.html` when present; no directory listing. |
| Existing file under `/web/` | Serve the file. |
| Missing extensionless path with `Accept: text/html` | Serve the configured fallback, if enabled and HTML is acceptable. |
| Missing path with a file extension, such as `.js`, `.woff2` or `.png` | 404; never the HTML fallback. |
| `/api`, `/admin` or `/webish` | Outside this mount. |

An absent Accept header or only `*/*` does not request the HTML fallback.
`text/html;q=0` also disables it. The extension check keeps typical missing asset
requests from receiving an HTML page they cannot use. An extensionless asset path
can receive the fallback if the request explicitly accepts HTML.

## Invalid paths and deployment safety

Invalid configuration throws HttpError code `static` before registration.
Malformed path escapes, hidden/dot segments, encoded separators, controls,
double encoding and unsafe Windows path forms return 400. Resolved paths outside
the root, including outside-root symlinks, return 404.

Roots are trusted read-only deployment assets: concurrent
local filesystem writers are outside this containment contract. Never mount
uploads, source, credentials or a directory with adversarial writers. Public
selection, cache/CSP headers and any access policy remain application-owned.
Configuration strings are borrowed/copied; the router retains no caller loans.

Use [Router](../http/) to register API handlers alongside these public mounts.
