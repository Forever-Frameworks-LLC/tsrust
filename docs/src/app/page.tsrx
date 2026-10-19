import { View, Text, Node, Child, signal, location, Link, withTheme } from 'std:ui';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { ExampleCard } from '../components/docs/example-card';
import { searchablePage } from '../components/docs/api-reference';
import { Markdown, docsPath, routePath } from '../components/docs/markdown';
import { loadCatalog, renderExample, DocCodeToken } from '../../target/docs-content/catalog';
import config from '../../tsrust.config.json';

function groupLabel(group: string): string {
  if (group === 'getting-started') {
    return 'Getting started';
  }

  if (group === 'foundations') {
    return 'Foundations';
  }

  if (group === 'standard-library') {
    return 'Standard library';
  }

  if (group === 'components') {
    return 'Components';
  }

  if (group === 'recipes') {
    return 'Recipes';
  }

  if (group === 'blocks') {
    return 'Blocks';
  }

  return clone(group);
}

function activeGroup(path: string): string {
  if (path.startsWith('/docs/standard-library')) {
    return 'standard-library';
  }

  if (path.startsWith('/docs/ui/components')) {
    return 'components';
  }

  if (path.startsWith('/docs/ui/blocks')) {
    return 'blocks';
  }

  if (path.startsWith('/docs/ui/recipes')) {
    return 'recipes';
  }

  if (path.startsWith('/docs/ui/foundations')) {
    return 'foundations';
  }

  return 'getting-started';
}

export function Page(): Node {
  const mode = signal('light');
  const path = location();
  const query = signal('');
  const catalog = loadCatalog();
  const matches = signal(catalog.pages.length);
  let terms: string[] = [];
  let navigation: Child[] = [];
  let articles: Child[] = [];
  let sections: Child[] = [];
  let known = '|/|';
  const groups = ['getting-started', 'foundations', 'standard-library', 'components', 'recipes', 'blocks'];
  for (const group of groups) {
    let entries: Child[] = [];
    let first = '';
    for (const page of catalog.pages) {
      if (page.metadata.group !== group) {
        continue;
      }

      const target = docsPath(page.route);
      if (first.length === 0) {
        first = clone(target);
      }

      const searchable = searchablePage(page, catalog);
      terms.push(clone(searchable));
      entries.push(<View hidden={!searchable.includes(query.get().trim().toLowerCase())}><Link href={clone(target)} className="text-sm py-2 px-3 rounded hover:bg-muted focus:ring">{clone(page.document.title)}</Link></View>);
    }

    sections.push(<Link href={first} className="text-xs md:text-sm px-3 py-3 rounded hover:bg-muted">{groupLabel(group)}</Link>);
    const selectedGroup = clone(group);
    const label = groupLabel(group);
    navigation.push(<View hidden={query.get().length === 0 && activeGroup(path.get()) !== selectedGroup} className="gap-1"><Text className="text-xs font-semibold pt-6 pb-2 text-muted-foreground">{label}</Text><View children={entries}/></View>);
  }

  for (const page of catalog.pages) {
    const target = docsPath(page.route);
    known += target + '|';
    let contents: Child[] = [];
    for (const item of page.document.headings) {
      if (item.level > 1) {
        contents.push(<Link href={clone(target) + '#' + page.metadata.id + '-' + item.anchor} className="text-xs py-2 text-muted-foreground hover:text-foreground">{clone(item.title)}</Link>);
      }
    }

    articles.push(
      <View hidden={routePath(path.get()) !== target} className="grow min-h-0 flex-col lg:flex-row">
        <View id={page.metadata.id + '-article'} data-slot="route-page" as="main" aria-label="Documentation article" tabIndex={0} className="grow min-w-0 overflow-auto p-6 md:p-8">
          <View className="max-w-192 gap-6">
            <Text className="text-xs text-muted-foreground">DOCUMENTATION / {groupLabel(page.metadata.group)}</Text>
            {Markdown(page, catalog)}

          </View>
        </View>
        <View as="nav" aria-label="On this page" tabIndex={0} className="h-24 lg:h-full w-full lg:w-48 shrink-0 overflow-auto border-l p-4 gap-2"><Text className="text-xs font-semibold pt-6 pb-2">ON THIS PAGE</Text><View children={contents}/></View>
      </View>
    );
  }

  let counterCode = '';
  let counterTokens: DocCodeToken[] = [];
  for (const example of catalog.examples) {
    if (example.id === 'counter') {
      counterCode = clone(example.code);
      counterTokens = clone(example.tokens);
    }
  }

  return withTheme(
    <View id="site" className="h-screen overflow-hidden bg-background text-foreground">
      <View as="header" windowDragRegion className="h-16 shrink-0 flex-row items-center justify-between gap-4 pl-24 pr-6 border-b">
        <Link href="/" className="font-mono font-semibold text-base">tsrust.</Link>
        <View className="flex-row items-center gap-2"><Link href="/docs" className="text-sm px-3 py-2 rounded hover:bg-muted">Docs</Link><Button variant="outline" size="sm" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Light / dark</Button></View>
      </View>
      <View hidden={routePath(path.get()) !== '/'} id="landing" as="main" aria-label="Tsrust home" tabIndex={0} className="grow min-h-0 overflow-auto">
        <View className="p-6 md:p-12 gap-12">
          <View className="grid lg:grid-cols-2 gap-8 py-8 md:py-12">
            <View className="gap-8 justify-center">
              <Text as="h1" className="text-4xl md:text-5xl font-semibold leading-snug">Stay expressive.{'\n'}<Text className="text-primary">Go native.</Text></Text>
              <Text className="text-xl leading-normal text-muted-foreground max-w-128">TypeScript-style syntax. Rust execution. Web and desktop interfaces from the same source.</Text>
              <View className="flex-row flex-wrap gap-3"><Link href="/docs/getting-started/installation" className="px-6 py-3 rounded bg-primary text-primary-foreground focus:ring">Get started</Link><Link href="/docs" className="px-6 py-3 rounded border hover:bg-muted focus:ring">Documentation</Link></View>
            </View>
            <ExampleCard id="home-counter" title="counter.tsrx" description="One component. Two targets." source="src/examples/counter.tsrx" code={counterCode} tokens={counterTokens}>{renderExample('counter')}</ExampleCard>
          </View>
          <View className="border-t pt-8 gap-8">
            <Text as="h2" className="text-3xl font-semibold max-w-160">Own the whole stack.{'\n'}Including the components.</Text>
            <View className="grid md:grid-cols-2 gap-8">
              <View className="gap-4"><Text className="text-xl font-semibold">The source is yours.</Text><Text className="leading-normal text-muted-foreground max-w-128">Add components offline. Edit their source. Set your design tokens in one project configuration.</Text><Link href="/docs/ui/components/button" className="font-medium py-3 text-primary">Explore components {'↗'}</Link></View>
              <View className="gap-4"><Text className="text-xl font-semibold">Ownership you can follow.</Text><Text className="leading-normal text-muted-foreground max-w-128">Local state stays with its component. Resource lifetimes and boundary conversions are checked before execution.</Text><Link href="/docs/ui/foundations/state" className="font-medium py-3 text-primary">State and ownership {'↗'}</Link></View>
            </View>
          </View>
        </View>
      </View>
      <View hidden={routePath(path.get()) === '/'} id="documentation" className="grow min-h-0">
        <View as="nav" aria-label="Documentation sections" className="shrink-0 flex-row flex-wrap border-b px-3 gap-1" children={sections}/>
        <View className="grow min-h-0 flex-row">
        <View id="sidebar" as="nav" aria-label="Documentation navigation" className="w-32 md:w-64 shrink-0 min-h-0 border-r">
          <View className="shrink-0 pt-6 px-3 gap-3"><Input aria-label="Search documentation" aria-describedby="search-feedback" placeholder="Search documentation…" value={query} onChange={(event) => { query.set(event.target.value); const needle = event.target.value.trim().toLowerCase(); let total = 0; for (const term of terms) { if (term.includes(needle)) { total++; } } matches.set(total); }}/></View>
          <View hidden={query.get().length === 0} className="shrink-0 pt-4 px-3"><Text id="search-feedback" role="status" className="text-xs text-muted-foreground">{matches.get() === 0 ? 'No pages match. Try another search.' : String(matches.get()) + ' matching pages'}</Text></View>
          <View id="navigation-pages" role="region" aria-label="Documentation pages" tabIndex={0} className="grow min-h-0 overflow-auto px-3 pb-6" children={navigation}/>
        </View>
        <View className="grow min-w-0 flex-row" children={articles}/>
        <View hidden={known.includes('|' + routePath(path.get()) + '|')} className="grow p-8 gap-6"><Text as="h1" className="text-3xl font-semibold">Page not found</Text><Text className="text-muted-foreground">This path is not in the documentation catalog.</Text><Link href="/docs" className="py-3">Return to documentation</Link></View>
        </View>
      </View>
    </View>, config.ui.theme, mode);
}
