import { View, Text, Button, Node, signal } from 'std:ui';

export function Screen(): Node {
  const count = signal(0);
  return (
    <View role="region" aria-label="Composed controls" className="p-4 gap-4">
      <View>Hello <Text as="code" className="font-bold">composed content</Text>!</View>
      <Button onPress={() => { count.set(count.get() + 1); }} className="rounded bg-primary text-primary-foreground">
        <View as="span" className="flex-row gap-2"><Text as="kbd" aria-hidden>+</Text>Count: {count}</View>
      </Button>
      <Button data-slot="passive">No application handler</Button>
      <Button data-slot="conditional" disabled={count.get() < 2}>Conditional without handler</Button>
    </View>
  );
}
