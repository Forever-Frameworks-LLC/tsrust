import { View, Text, Node, signal } from 'std:ui';
import { Button } from '../components/ui/button';

export function Counter(): Node {
  const count = signal(0);
  return (
    <View className="gap-4">
      <Text id="example-count" className="text-xl font-semibold">Count: {count}</Text>
      <View className="flex-row flex-wrap gap-2">
        <Button onPress={() => { count.set(count.get() + 1); }}>Increment</Button>
        <Button variant="outline" onPress={() => { count.set(count.get() - 1); }}>Decrement</Button>
        <Button variant="secondary" disabled={count.get() < 5} onPress={() => { count.set(count.get() + 100); }}>Add 100</Button>
      </View>
    </View>
  );
}
