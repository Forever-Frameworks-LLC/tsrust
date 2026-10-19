import { View, Text, Button, Node, signal } from 'std:ui';
import { describe } from './labels';

export function Page(): Node {
  const count = signal(1);
  const name = signal('Ada');
  const visible = signal(false);
  return (
    <View className="p-6 gap-4">
      <Text as="h1">Computed text</Text>
      <Text data-slot="arithmetic">{count.get() + 1} / {describe(count.get())}</Text>
      <Text data-slot="name">{'Hello ' + name.get().toUpperCase()}</Text>
      <Text data-slot="optional">A{visible.get() ? 'shown' : null}B</Text>
      <Text data-slot="exact">{9007199254740993} / {1.25} / {-0.0}</Text>
      <Button onPress={() => { count.set(count.get() + 1); name.set('Grace'); visible.set(!visible.get()); }}>Update {count.get()}</Button>
      <View data-slot="flow">Next: {count.get() * 10}</View>
    </View>
  );
}
