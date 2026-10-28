import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://forever-frameworks-llc.github.io',
  base: '/tsrust',
  trailingSlash: 'always',
  integrations: [starlight({
    title: 'Tsrust',
    description: 'TypeScript-style language. Native backends. Explicit ownership.',
    social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/Forever-Frameworks-LLC/tsrust' }],
    editLink: { baseUrl: 'https://github.com/Forever-Frameworks-LLC/tsrust/edit/main/website/' },
    expressiveCode: { shiki: { langAlias: { tsrust: 'typescript', tsr: 'typescript', tsrx: 'tsx' } } },
    customCss: ['./src/styles/custom.css'],
    sidebar: [
      { label: 'Start here', items: [{ label: 'Overview', slug: '' }, 'start/installation', 'start/quickstart', 'start/editor', 'start/project-structure'] },
      { label: 'Language', items: ['language/declarations', 'language/destructuring', 'language/pattern-defaults-rest', 'language/record-spread-assignment', 'language/modules', 'language/type-aliases', 'language/text-imports', 'language/async', 'language/strings', 'language/assignment-values', 'language/comma-expressions', 'language/statement-bodies', 'language/switch'] },
      { label: 'Build backends', items: ['backend/applications', 'backend/http', 'backend/static-files', 'backend/http-transport', 'backend/api-clients', 'backend/authorization', 'backend/api-compatibility', 'backend/deployment'] },
      { label: 'Standard library', items: ['library/overview', 'library/json', 'library/typed-json', 'library/json-files', 'library/futures'] },
      { label: 'Tools', items: ['tools/cli', 'tools/testing', 'tools/rust-setup', 'tools/install-and-update', 'tools/rust-packages'] },
      { label: 'Project', items: ['project/status'] },
    ],
  })],
});
