// Renders the desktop UI guides into the site. docs/guides/ui and the desktop
// packaging guide are the single source: the GPUI starter app builds its guide
// pages from the same files (tools/generate-ui-gallery.py). Output is ignored by
// Git and rebuilt before every site build.
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const website = join(dirname(fileURLToPath(import.meta.url)), '..');
const repository = join(website, '..');
const outputRoot = join(website, 'src', 'content', 'docs', 'ui');

// Site slug, title and description per guide, in reading order.
const guides = [
  ['docs/guides/ui/01-getting-started.md', 'getting-started', 'Get started with desktop UI', 'Create a native desktop app with TSRX components, run it with live rebuilds and package it.'],
  ['docs/guides/ui/02-components.md', 'components', 'Components', 'Compose GPUI Kit components, write your own, and use icons and the native terminal.'],
  ['docs/guides/ui/03-state.md', 'state', 'State and reactivity', 'Keep state in signals, control form inputs, and render conditional content and lists.'],
  ['docs/guides/ui/04-styling.md', 'styling', 'Styling and themes', 'Style components with utility classes, theme colors, custom themes and inline text runs.'],
  ['docs/guides/ui/05-navigation.md', 'navigation', 'Navigation and windows', 'Route between pages, open more windows, and use notifications, shortcuts and links.'],
  ['docs/guides/ui/06-input-and-zoom.md', 'input-and-zoom', 'Input and zoom', 'Handle keyboard, pointer and wheel input, and zoom the interface.'],
  ['docs/guides/tooling/desktop-apps.md', 'packaging', 'Package and update desktop apps', 'Package desktop apps for each platform, sign them, publish releases and update in place.'],
];

// The packaging guide is referenced by name from the getting-started guide.
const links = [[/The packaging guide covers/g, 'The [packaging guide](../packaging/) covers']];

rmSync(outputRoot, { recursive: true, force: true });
mkdirSync(outputRoot, { recursive: true });
for (const [source, slug, title, description] of guides) {
  const text = readFileSync(join(repository, source), 'utf8');
  const heading = text.match(/^# (.+)\n/);
  if (!heading) throw new Error(`${source} has no title heading`);
  let body = text.slice(heading[0].length).trim();
  // Gallery-only fence markers (`tsx live Name`) are plain TSX examples here.
  body = body.replace(/^```tsx live \w+$/gm, '```tsx');
  for (const [pattern, replacement] of links) body = body.replace(pattern, replacement);
  const page = `---\ntitle: ${JSON.stringify(title)}\ndescription: ${JSON.stringify(description)}\n---\n\n${body}\n\nThe [std:ui reference](/tsrust/reference/library/ui/) lists every component, prop and function.\n`;
  writeFileSync(join(outputRoot, `${slug}.md`), page);
}
console.log(`Generated ${guides.length} desktop UI guides`);
