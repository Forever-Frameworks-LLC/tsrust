import { View, Text, Node, signal } from 'std:ui';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';

export function Counter(): Node {
  const count = signal(0);
  return (
    <View className="gap-4">
      <Text id="example-count" className="text-2xl font-semibold">Count: {count}</Text>
      <View className="flex-row flex-wrap gap-2">
        <Button onPress={() => { count.set(count.get() + 1); }}>Increment</Button>
        <Button variant="outline" onPress={() => { count.set(count.get() - 1); }}>Decrement</Button>
        <Button variant="secondary" disabled={count.get() < 5} onPress={() => { count.set(count.get() + 100); }}>Add 100</Button>
      </View>
      <Label htmlFor="example-draft">A draft that stays yours</Label>
      <Input id="example-draft" defaultValue="Edit this, then change theme or hide the preview." className="w-full" />
    </View>
  );
}
