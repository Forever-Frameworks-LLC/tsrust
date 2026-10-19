import { View, Text, Node, signal, withTheme } from 'std:ui';
import { Button } from '../components/ui/button';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '../components/ui/card';
import { Kbd, KbdGroup } from '../components/ui/kbd';
import { Separator } from '../components/ui/separator';
import { Counter } from '../examples/counter';
import config from '../../tsrust.config.json';

export function Page(): Node {
  const mode = signal('light');
  const hidePreview = signal(false);
  return withTheme(
    <View id="docs" className="h-screen overflow-hidden bg-background text-foreground">
      <View windowDragRegion className="h-16 shrink-0 flex-row items-center justify-between gap-4 pl-24 pr-4 border-b">
        <Text className="font-semibold">Tsrust / Docs</Text>
        <Button variant="outline" size="sm" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Light / dark</Button>
      </View>
      <View className="flex-row grow min-h-0">
        <View id="sidebar" role="region" aria-label="Documentation index" tabIndex={0} className="w-32 md:w-56 shrink-0 overflow-auto p-4 gap-4 border-r">
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
          <View className="max-w-192 gap-6">
            <Text className="text-sm text-muted-foreground">UI / APPLICATION FOUNDATION</Text>
            <Text as="h1" className="text-3xl font-semibold">One application. Two targets.</Text>
            <Text className="text-muted-foreground">Editable local components, checked state and a shared layout. This is the beginning of the documentation application.</Text>
            <Card>
              <CardHeader>
                <CardTitle><Text as="h2" className="text-xl font-semibold">State stays with the example</Text></CardTitle>
                <CardDescription>Change the counter, edit the draft, switch themes and resize the window.</CardDescription>
              </CardHeader>
              <CardContent className="gap-4">
                <View id="preview" hidden={hidePreview.get()} className="h-40 overflow-auto"><Counter /></View>
                <Button variant="outline" onPress={() => { hidePreview.set(!hidePreview.get()); }}>Hide / show preview</Button>
              </CardContent>
            </Card>
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
            <Text>This foundation is not the complete component library. Persistent routes, exact source previews, Markdown rendering, offline search, generated API pages, benchmark charts and the remaining component families are still pending.</Text>
            <View className="p-6 rounded border bg-muted gap-4">
              <Text className="font-semibold">Keep exploring the actual behavior</Text>
              <Text>Scroll back to the example. Hidden preview state is retained. Conditional removal will remain a separate operation that releases the removed subtree.</Text>
            </View>
          </View>
        </View>
      </View>
    </View>, config.ui.theme, mode);
}
