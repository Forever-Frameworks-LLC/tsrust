import { View, Text, Button, signal, handle, add, Node } from 'std:ui';

interface CounterProps {}

function Counter(props: CounterProps): Node {
  const count = signal(0);
  const label = handle(count);
  const action = handle(count);
  const disabledAction = handle(count);
  return <View className="gap-2 p-4">
    <Text className="text-xl">Count: {label}</Text>
    <Button onPress={() => { add(action, 1); }} className="bg-primary text-primary-foreground rounded">Increment</Button>
    <Button disabled onPress={() => { add(disabledAction, 100); }}>Disabled</Button>
  </View>;
}

export function Screen(): Node {
  return <View className="gap-4 p-4 bg-background">
    <Text className="text-xl font-bold">Independent counters</Text>
    <Counter />
    <Counter />
  </View>;
}
