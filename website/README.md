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
Tsrust compiler, database or server-side runtime. The production URL and `/tsrust`
base path are configured once in `astro.config.mjs`. Generated files belong in
ignored `dist/`; commit source and `package-lock.json` only.

`npm run dev` and `npm run build` first run `tools/generate-api-reference.mjs`, which
renders the standard-library and built-in reference into the ignored
`src/content/docs/reference/` from `../crates/tsrust-lsp/src/builtins.json`. Edit
API documentation in that catalog, not in generated pages. A new catalog group must
be mapped in the generator before the build succeeds.

Add reviewed Markdown to `src/content/docs/`, supply `title` and `description`, and
add its route to the explicit sidebar. Keep links relative within the site and
verify the built output under the project base path. Do not add placeholder pages
or copy internal milestones into the public reference.

Hosting activation and content migration are in progress; this source does not
establish that the production site is deployed.
