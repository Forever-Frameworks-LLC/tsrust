import { View, Text, Button, Input, Node, signal, run } from 'std:ui';

function Slice(): Node {
  const count = signal(0);
  const name = signal('');
  return (
    <View className="flex-col size-full items-center justify-center gap-2">
      <Text>Count: {count}</Text>
      <Button onPress={() => { count.set(count.get() + 1); }}>Increment</Button>
      <Input placeholder="Name" value={name} onChange={(event) => { name.set(event.target.value); }} />
      {name.get() !== '' ? <Text>Hello, {name}!</Text> : null}
    </View>
  );
}

function main() {
  unwrap(run(Slice(), 'Tsrust GPUI slice'));
}
