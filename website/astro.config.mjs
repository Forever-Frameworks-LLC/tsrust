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
    customCss: ['./src/styles/custom.css'],
    sidebar: [
      { label: 'Start here', items: [{ label: 'Overview', slug: '' }, 'start/quickstart'] },
      { label: 'Project', items: ['project/status'] },
    ],
  })],
});
