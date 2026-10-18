import { View, Text, Button, signal, Node } from 'std:ui';

interface CounterProps {}

function Counter(props: CounterProps): Node {
  const count = signal(0);
  const increment = () => {
    count.set(count.get() + 1);
  };
  return (
    <View className="gap-2 p-4">
      <Text className="text-xl">Count: {count}</Text>
      <Button onPress={increment} className="bg-primary text-primary-foreground rounded">Increment</Button>
      <Button disabled onPress={() => { count.set(count.get() + 100); }}>Disabled</Button>
    </View>
  );
}

export function Screen(): Node {
  return (
    <View className="gap-4 p-4 bg-background">
      <Text className="text-xl font-bold">Independent counters</Text>
      <Counter />
      <Counter />
    </View>
  );
}
