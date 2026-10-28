---
title: Serve public static files
description: Mount deployment-owned frontend assets with explicit path and fallback rules.
---

`Router.serveStatic(prefix, directory, fallback?)` serves deployment-owned public
files with streamed GET/HEAD responses, MIME types, ranges and conditional dates.
Use separate non-root prefixes, such as `/web` and `/admin`, without trailing
slashes. Literal URL segments accept ASCII letters, digits, hyphens and underscores.
Prefixes cannot overlap. Roots must already exist. Relative roots resolve at
registration; use absolute paths when launching outside the project.

```tsrust
app.serveStatic('/web', 'web/dist', 'index.html');
app.serveStatic('/admin', 'admin/dist', 'index.html');
```

Build each frontend for its public prefix. `/web` redirects to `/web/`.
Directories can serve `index.html`, without listings. The optional fallback must
be an existing relative `.html` file within its own root; omitted/empty disables
it. Fallback applies only to missing extensionless paths with explicit acceptable
`text/html`. Missing script/font/image paths stay 404. Mounted `/web` never handles
`/api`, `/admin` or `/webish`. Earlier matching routes retain registration priority.
Other methods pass to later application routes.

Invalid configuration throws HttpError code `static` before registration.
Malformed path escapes, hidden/dot segments, encoded separators, controls,
double encoding and unsafe Windows path forms reject. Canonical paths reject
outside-root symlinks. Roots are trusted read-only deployment assets: concurrent
local filesystem writers are outside this containment contract. Never mount
uploads, source, credentials or a directory with adversarial writers. Public
selection, cache/CSP headers and any access policy remain application-owned.
Configuration strings are borrowed/copied; the router retains no caller loans.

Use [Router](../http/) to register API handlers alongside these public mounts.
