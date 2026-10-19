import { View, Text, Node, signal, withTheme } from 'std:ui';
import { Button } from '../components/ui/button';
import { ExampleCard } from '../components/docs/example-card';
import { Kbd, KbdGroup } from '../components/ui/kbd';
import { Separator } from '../components/ui/separator';
import { loadCatalog, renderExample, DocCatalog } from '../../target/docs-content/catalog';
import config from '../../tsrust.config.json';

function introduction(catalog: ReadOnly<DocCatalog>, summary: boolean): string {
  for (const page of catalog.pages) {
    if (page.metadata.id === 'introduction') {
      return clone(summary ? page.document.summary : page.document.title);
    }
  }

  throw 'Documentation introduction is missing';
}

function counterCode(catalog: ReadOnly<DocCatalog>): string {
  for (const example of catalog.examples) {
    if (example.id === 'counter') {
      return clone(example.code);
    }
  }

  throw 'Counter source is missing';
}

export function Page(): Node {
  const mode = signal('light');
  const catalog = loadCatalog();
  return withTheme(
    <View id="docs" className="h-screen overflow-hidden bg-background text-foreground">
      <View windowDragRegion className="h-16 shrink-0 flex-row items-center justify-between gap-4 pl-24 pr-4 border-b">
        <Text className="font-semibold">Tsrust / Docs</Text>
        <Button variant="outline" size="sm" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Light / dark</Button>
      </View>
      <View className="flex-row grow min-h-0">
        <View id="sidebar" role="region" aria-label="Documentation index" tabIndex={0} className="w-32 md:w-48 shrink-0 overflow-auto p-4 gap-4 border-r">
          <Text className="font-semibold">Foundations</Text>
          <Text className="text-sm text-muted-foreground">Shared web and native application</Text>
          <Separator />
          <Text className="font-semibold">Available controls</Text>
          <Text className="text-sm">Button</Text>
          <Text className="text-sm">Card</Text>
          <Text className="text-sm">Input</Text>
          <Text className="text-sm">Textarea</Text>
          <Text className="text-sm">Label</Text>
          <Text className="text-sm">Keyboard</Text>
          <Text className="text-sm">Separator</Text>
          <Separator />
          <Text className="text-sm text-muted-foreground">Component pages, routing, Markdown and search are being built. This screen exercises the shared application foundation.</Text>
        </View>
        <View id="article" role="region" aria-label="Documentation article" tabIndex={0} className="grow min-w-0 overflow-auto p-4 md:p-8">
          <View className="max-w-full gap-6">
            <Text className="text-sm text-muted-foreground">UI / APPLICATION FOUNDATION</Text>
            <Text as="h1" className="text-xl font-semibold">{introduction(catalog, false)}</Text>
            <Text className="text-muted-foreground">{introduction(catalog, true)}</Text>
            <Text className="text-sm text-muted-foreground">{catalog.pages.length} page destinations / {catalog.examples.length} checked examples / {catalog.api.length} API declarations</Text>
            <ExampleCard id="counter" title="State stays with the example" description="Change the counter, edit the draft, switch themes or inspect its source." source="src/examples/counter.tsrx" code={counterCode(catalog)}>
              {renderExample('counter')}
            </ExampleCard>
            <Text as="h2" className="text-xl font-semibold">Independent panes</Text>
            <Text>Sidebar and article scroll separately. Wheel and trackpad input continue into an ancestor when an inner pane reaches its limit.</Text>
            <View className="flex-row flex-wrap items-center gap-2">
              <Text>Focus a pane with</Text><Kbd>Tab</Kbd><Text>then use</Text><KbdGroup><Kbd>Page Up</Kbd><Text>/</Text><Kbd>Page Down</Kbd></KbdGroup>
            </View>
            <Separator />
            <Text as="h2" className="text-xl font-semibold">Theme belongs to the project</Text>
            <Text>Light and dark tokens live in tsrust.config.json. Changing mode preserves the draft and counter. Local component source lives in src/components/ui.</Text>
            <Text as="h2" className="text-xl font-semibold">Native window chrome</Text>
            <Text>The macOS build uses a custom header with real system window controls. Empty header space can drag the window; the theme button remains interactive. Other desktop platforms currently require a native titlebar.</Text>
            <Text as="h2" className="text-xl font-semibold">Implementation status</Text>
            <Text>This foundation is not the complete component library. Persistent routes, exact highlighted code and copy actions, Markdown rendering, offline search, generated API pages, benchmark charts and the remaining component families are still pending.</Text>
            <View className="p-6 rounded border bg-muted gap-4">
              <Text className="font-semibold">Keep exploring the actual behavior</Text>
              <Text>Scroll back to the example. Hidden preview state is retained. Conditional removal will remain a separate operation that releases the removed subtree.</Text>
            </View>
          </View>
        </View>
      </View>
    </View>, config.ui.theme, mode);
}
