import { View, Text, Button, Input, Node, signal } from 'std:ui';

interface CounterProps {
  name: string;
  start: int;
}

function Counter(props: CounterProps): Node {
  const count = signal(props.start);
  return (<Button data-slot="counter" className="bg-primary text-primary-foreground rounded px-4 py-2" onPress={() => { count.set(count.get() + 1); }}>{props.name}: {count}</Button>);
}

export function Page(): Node {
  const shown = signal(true);
  const second = signal(false);
  const pulses = signal(0);
  const name = signal('stable');
  return (
    <View className="p-6 gap-4">
      <Text as="h1">Conditional mounting</Text>
      <Button aria-label="Toggle counter" onPress={() => { shown.set(!shown.get()); }}>Toggle</Button>
      <Button aria-label="Switch counter" onPress={() => { second.set(!second.get()); }}>Switch</Button>
      <View data-slot="row" className="flex-row items-center gap-4">
        {shown.get() ? '' : null}
        {shown.get() ? second.get() ? <Counter name="Second" start={100}/> : <Counter name="First" start={1}/> : null}
        <Input id="stable" aria-label="Stable name" value={name} onChange={(event) => { name.set(event.target.value); }}/>
        <Button aria-label="Pulse" onPress={() => { pulses.set(pulses.get() + 1); }}>Pulse {pulses}</Button>
      </View>
      <Text data-slot="status">{name} / {pulses}</Text>
      <View data-slot="caption">{pulses.get() > 0 ? 'Pulse active' : null}</View>
    </View>
  );
}
