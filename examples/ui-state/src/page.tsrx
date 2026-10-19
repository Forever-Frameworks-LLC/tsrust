import { View, Text, Button, Node, signal } from 'std:ui';

function Counter(): Node {
  const name = signal('Before');
  const ready = signal(false);
  const amount = signal(1.25);
  const count = signal(9007199254740993);
  const zero = signal(-0.0);
  const nan = signal(NaN);
  const infinity = signal(Infinity);
  const negative = signal(-Infinity);
  const large = signal(1e21);
  return (
    <View className="p-4 gap-4">
      <Text data-slot="state-text">{name} / {ready} / {amount} / {count}</Text>
      <Text data-slot="number-text">{zero} / {nan} / {infinity} / {negative} / {large}</Text>
      <Button onPress={() => {
        name.set(name.get() + '!');
        ready.set(!ready.get());
        amount.set(amount.get() + 0.5);
        count.set(count.get() + 1);
      }}>Change</Button>
      <Button disabled={!ready.get()} onPress={() => { name.set('Reset'); }}>Reset</Button>
    </View>
  );
}

export function Page(): Node {
  return (<View><Counter/><Counter/></View>);
}
