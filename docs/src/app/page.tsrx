import { View, Text, Node, Child, signal, location, Link, back, forward, withTheme } from 'std:ui';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { ExampleCard } from '../components/docs/example-card';
import { searchablePage } from '../components/docs/api-reference';
import { Markdown, docsPath, routePath } from '../components/docs/markdown';
import { loadCatalog, renderExample } from '../../target/docs-content/catalog';
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

export function Page(): Node {
  const mode = signal('light');
  const path = location();
  const query = signal('');
  const catalog = loadCatalog();
  let navigation: Child[] = [];
  let articles: Child[] = [];
  let known = '|/|';
  const groups = ['getting-started', 'foundations', 'standard-library', 'components', 'recipes', 'blocks'];
  for (const group of groups) {
    navigation.push(<Text className="text-xs font-semibold pt-6 pb-2 text-muted-foreground">{groupLabel(group)}</Text>);
    for (const page of catalog.pages) {
      if (page.metadata.group !== group) {
        continue;
      }

      const target = docsPath(page.route);
      const searchable = searchablePage(page, catalog);
      navigation.push(<View hidden={!searchable.includes(query.get().trim().toLowerCase())}><Link href={clone(target)} className="text-sm py-2 px-3 rounded hover:bg-muted focus:ring">{clone(page.document.title)}</Link></View>);
    }
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
            <Text className="text-xs text-muted-foreground">Status: {String(page.metadata.status)}</Text>
            {Markdown(page, catalog)}
            <View className="border-t pt-6 gap-3"><Text className="text-xs text-muted-foreground">SOURCE / {clone(page.source)}</Text><Link href="/docs/getting-started/installation" className="text-sm font-medium py-2">Get started {'↗'}</Link></View>
          </View>
        </View>
        <View as="nav" aria-label="On this page" tabIndex={0} className="h-24 lg:h-full w-full lg:w-48 shrink-0 overflow-auto border-l p-4 gap-2"><Text className="text-xs font-semibold pt-6 pb-2">ON THIS PAGE</Text><View children={contents}/></View>
      </View>
    );
  }

  let counterCode = '';
  for (const example of catalog.examples) {
    if (example.id === 'counter') {
      counterCode = clone(example.code);
    }
  }

  return withTheme(
    <View id="site" className="h-screen overflow-hidden bg-background text-foreground">
      <View as="header" windowDragRegion className="h-16 shrink-0 flex-row items-center justify-between gap-4 pl-24 pr-6 border-b">
        <Link href="/" className="font-mono font-semibold text-base">tsrust.</Link>
        <View className="flex-row items-center gap-2"><Link href="/docs" className="text-sm px-3 py-2 rounded hover:bg-muted">Docs</Link><Button variant="outline" size="sm" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Light / dark</Button></View>
      </View>
      <View hidden={routePath(path.get()) !== '/'} id="landing" as="main" aria-label="Tsrust home" tabIndex={0} className="grow min-h-0 overflow-auto">
        <View className="p-6 md:p-16 gap-12">
          <View className="gap-6 max-w-192 py-12">
            <Text className="text-xs font-mono text-muted-foreground">TYPESCRIPT EXPRESSION. RUST EXECUTION.</Text>
            <Text as="h1" className="text-5xl font-serif leading-snug">One language. Closer to the machine.</Text>
            <Text className="text-xl leading-normal text-muted-foreground">Build interfaces and services with checked ownership, native execution and components you can make your own.</Text>
            <View className="flex-row flex-wrap gap-3 pt-4"><Link href="/docs/getting-started/installation" className="px-6 py-3 rounded bg-primary text-primary-foreground focus:ring">Get started</Link><Link href="/docs" className="px-6 py-3 rounded border hover:bg-muted focus:ring">Read the documentation</Link></View>
          </View>
          <View className="grid md:grid-cols-3 gap-8 border-t border-b py-8">
            <View className="gap-3"><Text className="font-mono text-xs text-muted-foreground">01 / LANGUAGE</Text><Text as="h2" className="font-semibold">Familiar syntax. Explicit ownership.</Text><Text className="text-sm leading-normal text-muted-foreground">Write TypeScript-style source with checked values, resources and conversions.</Text><Link href="/docs/standard-library/strings" className="text-sm py-2">Explore the language {'↗'}</Link></View>
            <View className="gap-3"><Text className="font-mono text-xs text-muted-foreground">02 / INTERFACES</Text><Text as="h2" className="font-semibold">Web and native, together.</Text><Text className="text-sm leading-normal text-muted-foreground">Share application source, state and themes between browser and desktop targets.</Text><Link href="/docs/ui/foundations/state" className="text-sm py-2">Understand UI state {'↗'}</Link></View>
            <View className="gap-3"><Text className="font-mono text-xs text-muted-foreground">03 / COMPONENTS</Text><Text as="h2" className="font-semibold">The source belongs to you.</Text><Text className="text-sm leading-normal text-muted-foreground">Install editable components offline. Configure design tokens in your project.</Text><Link href="/docs/ui/components/button" className="text-sm py-2">Browse components {'↗'}</Link></View>
          </View>
          <ExampleCard id="home-counter" title="Small example. Real state." description="Try the shared web/native counter, or inspect its checked source." source="src/examples/counter.tsrx" code={counterCode}>{renderExample('counter')}</ExampleCard>
          <Text className="text-xs text-muted-foreground pb-6">Tsrust is in development. Documentation marks partial and pending support explicitly.</Text>
        </View>
      </View>
      <View hidden={routePath(path.get()) === '/'} id="documentation" className="grow min-h-0 flex-row">
        <View id="sidebar" as="nav" aria-label="Documentation navigation" tabIndex={0} className="w-32 md:w-64 shrink-0 overflow-auto border-r px-3 pb-6">
          <View className="pt-6 gap-3"><Input aria-label="Search documentation" placeholder="Search documentation…" value={query} onChange={(event) => { query.set(event.target.value); }}/><View className="flex-row flex-wrap gap-2"><Button variant="outline" size="sm" onPress={() => { back(); }}>Back</Button><Button variant="outline" size="sm" onPress={() => { forward(); }}>Next</Button></View><Text className="text-xs text-muted-foreground">Search page text, APIs and code. Pending entries are labeled on their pages.</Text></View>
          <View children={navigation}/>
        </View>
        <View className="grow min-w-0 flex-row" children={articles}/>
        <View hidden={known.includes('|' + routePath(path.get()) + '|')} className="grow p-8 gap-6"><Text as="h1" className="text-3xl font-semibold">Page not found</Text><Text className="text-muted-foreground">This path is not in the documentation catalog.</Text><Link href="/docs" className="py-3">Return to documentation</Link></View>
      </View>
    </View>, config.ui.theme, mode);
}
