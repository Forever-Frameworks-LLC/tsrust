import { View, Text, Node, Child, signal, location, Route, Link, withTheme, Button as ThemeButton } from 'std:ui';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Landing } from '../components/docs/landing';
import { Markdown, docsPath, routePath } from '../components/docs/markdown';
import { loadNavigation, loadSearch, loadPage, DocSearchEntry } from '../../target/docs-content/catalog';
import config from '../../tsrust.config.json';

function groupLabel(group: string): string {
  if (group === 'language') {
    return 'Language';
  }

  if (group === 'framework') {
    return 'Services';
  }

  if (group === 'cli') {
    return 'CLI';
  }

  if (group === 'charts') {
    return 'Charts';
  }

  if (group === 'themes') {
    return 'Themes';
  }

  if (group === 'performance') {
    return 'Performance';
  }

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
  if (path.startsWith('/docs/language')) {
    return 'language';
  }

  if (path.startsWith('/docs/framework')) {
    return 'framework';
  }

  if (path.startsWith('/docs/cli')) {
    return 'cli';
  }

  if (path.startsWith('/docs/ui/charts')) {
    return 'charts';
  }

  if (path.startsWith('/docs/ui/themes')) {
    return 'themes';
  }

  if (path.startsWith('/docs/performance')) {
    return 'performance';
  }

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

function Article(route: string): Node {
  const catalog = loadPage(route);
  const page = catalog.pages[0];
  const target = docsPath(page.route);
  let contents: Child[] = [];
  for (const item of page.document.headings) {
    if (item.level > 1) {
      contents.push(<Link href={clone(target) + '#' + page.metadata.id + '-' + item.anchor} className="text-xs py-2 text-muted-foreground hover:text-foreground">{clone(item.title)}</Link>);
    }
  }

  return (
    <View className="grow min-h-0 flex-col lg:flex-row">
      <View id={page.metadata.id + '-article'} data-slot="route-page" as="main" aria-label="Documentation article" tabIndex={0} className="grow min-w-0 overflow-auto p-6 md:p-8">
        <View className="max-w-192 gap-6"><Text className="text-xs text-muted-foreground">DOCUMENTATION / {groupLabel(page.metadata.group)}</Text>{Markdown(page,catalog)}</View>
      </View>
      <View as="nav" aria-label="On this page" tabIndex={0} className="h-24 lg:h-full w-full lg:w-48 shrink-0 overflow-auto border-l p-4 gap-2"><Text className="text-xs font-semibold pt-6 pb-2">ON THIS PAGE</Text><View children={contents}/></View>
    </View>
  );
}

export function Page(): Node {
  const mode = signal('light');
  const path = location();
  const query = signal('');
  const pages = loadNavigation();
  const pageCount = pages.length;
  const matches = signal(pageCount);
  const hits = signal('');
  let terms: DocSearchEntry[] = [];
  let navigation: Child[] = [];
  let searchNavigation: Child[] = [];
  let articles: Child[] = [];
  let sections: Child[] = [];
  let known = '|/|';
  const groups = ['getting-started', 'language', 'cli', 'standard-library', 'framework', 'foundations', 'components', 'recipes', 'blocks', 'charts', 'themes', 'performance'];
  for (const group of groups) {
    let entries: Child[] = [];
    let first = '/docs/' + group;
    if (group === 'foundations' || group === 'components' || group === 'recipes' || group === 'blocks' || group === 'charts' || group === 'themes') {
      first = '/docs/ui/' + group;
    }

    for (const page of pages) {
      if (page.group !== group) {
        continue;
      }

      const target = docsPath(page.route);
      if (first.length === 0) {
        first = clone(target);
      }

      const key = '|' + target + '|';
      entries.push(<Link href={clone(target)} className="text-sm py-2 px-3 rounded hover:bg-muted focus:ring aria-current:bg-primary/10 aria-current:text-primary aria-current:font-semibold">{clone(page.title)}</Link>);
      searchNavigation.push(<View hidden={!hits.get().includes(key)}><Link href={clone(target)} className="text-sm py-2 px-3 rounded hover:bg-muted focus:ring aria-current:bg-primary/10 aria-current:text-primary">{clone(page.title)}</Link></View>);
    }

    sections.push(<Link href={first} className="shrink-0 text-xs md:text-sm px-3 py-3 rounded hover:bg-muted focus:ring aria-current:text-primary aria-current:bg-primary/10">{groupLabel(group)}</Link>);
    const selectedGroup = clone(group);
    const label = groupLabel(group);
    navigation.push(<View hidden={query.get().trim().length !== 0 || activeGroup(path.get()) !== selectedGroup} className="grow min-h-0 gap-1"><Text className="shrink-0 text-xs font-semibold pt-6 pb-2 px-3 text-muted-foreground">{label}</Text><View role="region" aria-label={clone(label) + ' pages'} tabIndex={0} className="grow min-h-0 overflow-auto px-3 pb-6" children={entries}/></View>);
  }

  for (const page of pages) {
    const target = docsPath(page.route);
    known += target + '|';
    const route = clone(page.route);
    articles.push(<Route path={target} render={() => Article(route)}/>);
  }

  return withTheme(
    <View id="site" className="h-screen overflow-hidden bg-background text-foreground">
      <View as="header" windowDragRegion className="h-24 md:h-16 shrink-0 flex-col md:flex-row items-stretch md:items-center justify-between gap-1 md:gap-4 px-4 md:pl-24 md:pr-6 border-b">
        <Link href="/" className="bg-transparent font-mono font-semibold text-base pl-24 md:pl-0 py-2">tsrust.</Link>
        <View as="nav" aria-label="Main navigation" className="flex-row items-center justify-between md:justify-start gap-1 md:gap-2">
          <Link href="/docs" className="text-sm px-3 py-2 rounded hover:bg-muted focus:ring aria-current:text-primary">Docs</Link>
          <Link href="https://github.com/Forever-Frameworks-LLC/tsrust" className="text-sm px-3 py-2 rounded hover:bg-muted focus:ring">GitHub</Link>
          <Link href="https://github.com/Forever-Frameworks-LLC/tsrust/releases" className="text-sm px-3 py-2 rounded hover:bg-muted focus:ring">Releases</Link>
          <ThemeButton aria-label="Toggle light and dark theme" className="text-sm px-3 py-2 rounded border hover:bg-muted focus:ring" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Theme</ThemeButton>
        </View>
      </View>
      <Route path="/" render={() => <View id="landing" as="main" aria-label="Tsrust home" tabIndex={0} className="grow min-h-0 overflow-auto"><Landing/></View>}/>
      <View hidden={routePath(path.get()) === '/'} id="documentation" className="grow min-h-0">
        <View as="nav" aria-label="Documentation sections" className="h-12 shrink-0 flex-row overflow-auto border-b px-3 gap-1" children={sections}/>
        <View className="grow min-h-0 flex-row">
        <View id="sidebar" as="nav" aria-label="Documentation navigation" className="w-32 md:w-64 shrink-0 min-h-0 border-r">
          <View className="shrink-0 pt-6 px-3 gap-3"><Input aria-label="Search documentation" aria-describedby="search-feedback" placeholder="Search documentation…" value={query} onChange={(event) => { query.set(event.target.value); const needle = event.target.value.trim().toLowerCase(); if (needle.length === 0) { matches.set(pageCount); hits.set(''); } else { if (terms.length === 0) { terms = loadSearch(); } let selected = ''; let total = 0; for (const term of terms) { if (term.text.toLowerCase().includes(needle)) { selected += '|' + docsPath(term.route) + '|'; total++; } } hits.set(selected); matches.set(total); } }}/></View>
          <View hidden={query.get().trim().length === 0} className="shrink-0 pt-4 px-3"><Text id="search-feedback" role="status" className="text-xs text-muted-foreground">{matches.get() === 0 ? 'No pages match. Try another search.' : String(matches.get()) + ' matching pages'}</Text></View>
          <View id="navigation-pages" hidden={query.get().trim().length !== 0} className="grow min-h-0" children={navigation}/>
          <View hidden={query.get().trim().length === 0} role="region" aria-label="Search results" tabIndex={0} className="grow min-h-0 overflow-auto px-3 pb-6" children={searchNavigation}/>
        </View>
        <View className="grow min-w-0 flex-row" children={articles}/>
        <View hidden={known.includes('|' + routePath(path.get()) + '|')} className="grow p-8 gap-6"><Text as="h1" className="text-3xl font-semibold">Page not found</Text><Text className="text-muted-foreground">This path is not in the documentation catalog.</Text><Link href="/docs" className="py-3">Return to documentation</Link></View>
        </View>
      </View>
    </View>, config.ui.theme, mode);
}
