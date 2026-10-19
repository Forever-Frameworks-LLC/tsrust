import { View, Text, Button, Input, Textarea, Node, signal, withTheme } from 'std:ui';
import config from '../../tsrust.config.json';

function Counter(): Node {
  const count = signal(0);
  return (
    <View id="counter" className="gap-4 p-4 rounded border bg-card text-card-foreground">
      <Text id="count">Count: {count}</Text>
      <Input id="draft" aria-label="Draft" defaultValue="Original" className="border rounded p-2" />
      <Button id="increment" className="bg-primary text-primary-foreground rounded" onPress={() => { count.set(count.get() + 1); }}>Increment</Button>
    </View>
  );
}

export function Page(): Node {
  const mode = signal('light');
  const showCode = signal(false);
  return withTheme(
    <View id="application" className="bg-background text-foreground gap-4 p-4">
      <View windowDragRegion className="flex-row flex-wrap items-center justify-between gap-4 pl-24 min-h-12">
        <Text as="h1" className="text-xl font-semibold">Application contracts</Text>
        <Button id="theme" className="bg-secondary text-secondary-foreground rounded" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Switch theme</Button>
      </View>
      <Text id="mode">Mode: {mode}</Text>
      <Button id="show-code" className="border rounded" onPress={() => { showCode.set(!showCode.get()); }}>Toggle preview and source</Button>
      <View hidden={showCode.get()}><Counter /></View>
      <View hidden={!showCode.get()}>
        <Textarea id="source" aria-label="Example source" readOnly defaultValue="<Button onPress={() => count.set(count.get() + 1)}>Increment</Button>" className="w-full h-24 border rounded p-4 font-mono" />
      </View>
      {withTheme(<View id="nested" className="p-4 rounded bg-background text-foreground"><Text>Independent light scope</Text></View>, config.ui.theme, 'light')}
    </View>, config.ui.theme, mode);
}
