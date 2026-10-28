import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://forever-frameworks-llc.github.io',
  base: '/tsrust',
  trailingSlash: 'always',
  // Absorbed feature pages keep working URLs; see docs/plans/documentation-architecture.md.
  redirects: {
    '/language/borrowed-optional-fallback/': '/tsrust/language/optional-values/#borrowed-text-with-a-fallback',
    '/language/optional-chaining/': '/tsrust/language/optional-values/#optional-chaining',
    '/language/nullish-coalescing/': '/tsrust/language/optional-values/#nullish-fallback',
    '/language/nullish-assignment/': '/tsrust/language/optional-values/#nullish-assignment',
  },
  integrations: [starlight({
    title: 'Tsrust',
    description: 'TypeScript-style language. Native backends. Explicit ownership.',
    social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/Forever-Frameworks-LLC/tsrust' }],
    editLink: { baseUrl: 'https://github.com/Forever-Frameworks-LLC/tsrust/edit/main/website/' },
    expressiveCode: { shiki: { langAlias: { tsrust: 'typescript', tsr: 'typescript', tsrx: 'tsx' } } },
    customCss: ['./src/styles/custom.css'],
    sidebar: [
      { label: 'Start here', items: [{ label: 'Overview', slug: '' }, 'start/installation', 'start/quickstart', 'start/editor', 'start/project-structure'] },
      { label: 'Language', items: ['language/declarations', 'language/destructuring', 'language/pattern-defaults-rest', 'language/record-spread-assignment', 'language/modules', 'language/type-aliases', 'language/interfaces', 'language/optional-fields', 'language/optional-values', 'language/nullable-callables', 'language/optional-default-parameters', 'language/rest-parameters', 'language/call-spread', 'language/array-literal-spread', 'language/array-from', 'language/nested-functions', 'language/nested-function-captures', 'language/captured-function-parameters', 'language/generic-function-contexts', 'language/generic-function-values', 'language/generic-value-adapters', 'language/fresh-generic-factories', 'language/lexical-generic-functions', 'language/arrow-functions', 'language/function-expressions', 'language/callable-expressions', 'language/callable-locals', 'language/callable-fields', 'language/borrowed-callable-fields', 'language/callable-arrays', 'language/callable-array-methods', 'language/text-imports', 'language/async', 'language/strings', 'language/assignment-values', 'language/comma-expressions', 'language/statement-bodies', 'language/switch'] },
      { label: 'Build backends', items: ['backend/applications', 'backend/http', 'backend/static-files', 'backend/http-transport', 'backend/api-clients', 'backend/authorization', 'backend/api-compatibility', 'backend/deployment'] },
      { label: 'Standard library', items: [
        'library/overview',
        'library/globals',
        { label: 'Modules', collapsed: true, items: [{ autogenerate: { directory: 'reference/library' } }] },
        { label: 'Built-in types', collapsed: true, items: [{ autogenerate: { directory: 'reference/builtins' } }] },
        { label: 'Guides', collapsed: true, items: ['library/json', 'library/typed-json', 'library/json-files', 'library/futures'] },
      ] },
      { label: 'Tools', items: ['tools/cli', 'tools/testing', 'tools/rust-setup', 'tools/install-and-update', 'tools/rust-packages'] },
      { label: 'Project', items: ['project/status'] },
    ],
  })],
});
