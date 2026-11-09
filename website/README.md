# Public documentation

Astro Starlight source for the Tsrust documentation site. The migration and content
ownership rules are in [the plan](../docs/plans/public-documentation-site.md).

```sh
npm ci
npm run dev
npm run build
python3 tools/check_links.py dist
npm run preview
```

Run these commands inside `website/` with Node 24. Site generation needs no Rust,
Tsrust compiler or running backend. The performance page reads the repository
measurement database. The production URL and `/tsrust`
base path are configured once in `astro.config.mjs`. Generated files belong in
ignored `dist/`; commit source and `package-lock.json` only.

`npm run dev` and `npm run build` first run `tools/generate-api-reference.mjs`, which
renders the standard-library and built-in reference into the ignored
`src/content/docs/reference/` from `../crates/tsrust-lsp/src/builtins.json`. Edit
API documentation in that catalog, not in generated pages. A new catalog group must
be mapped in the generator before the build succeeds.

## Refresh a local preview

`npm run preview` serves the existing `dist/`; it neither regenerates the API
reference nor rebuilds changed Markdown. To refresh a static preview, stop the
preview you own, then run from `website/` with the pinned Node version:

```sh
npm run build
python3 tools/check_links.py dist
npm run preview -- --host 127.0.0.1
```

Open the displayed local URL under `/tsrust/`. `npm run build` already includes
`npm run reference`; a separate reference step is unnecessary. `npm run reference`
alone updates ignored API pages and the performance page, but does not update
`dist/`. For active editing, use `npm run dev` instead. After changing the API
catalog while the dev server is running, rerun `npm run reference` so its generated
source pages refresh too.

Reference generation reads `crates/tsrust-lsp/src/builtins.json` and its explicit
module/group mapping. Performance generation reads `benchmarks/measurements.sqlite`
and workload source metadata. Neither command publishes the site. The existing
Documentation workflow validates pull requests and deploys verified `main` output;
a local build or preview does not update the public hosted site.

Learn tracks (`src/content/docs/learn/`) load `public/learn.js`, which turns each
level-2 heading after the `data-learn-track` marker into a step. Add a step as a
new `##` section; do not create separate how-to pages.

Add reviewed Markdown to `src/content/docs/`, supply `title` and `description`, and
add its route to the explicit sidebar. Keep links relative within the site and
verify the built output under the project base path. Do not add placeholder pages
or copy internal milestones into the public reference.

Hosting activation and content migration are in progress; this source does not
establish that the production site is deployed.
