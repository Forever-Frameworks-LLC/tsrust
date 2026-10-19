import { run, View, Button, Text, Node, signal } from 'std:ui';

function Failing(): Node {
  throw 'Branch construction failed';
}

function Broken(): Node {
  const visible = signal(false);
  return (
    <View>
      <Button aria-label="Break branch" onPress={() => { visible.set(true); }}>Break</Button>
      {visible.get() ? <Failing/> : null}
    </View>
  );
}

function Healthy(): Node {
  const count = signal(10);
  return (
    <View>
      <Text data-slot="healthy">{count}</Text>
      <Button aria-label="Healthy branch" onPress={() => { count.set(count.get() + 1); }}>Healthy</Button>
    </View>
  );
}

function main() {
  run(Broken(), 'Broken branch')!;
  run(Healthy(), 'Healthy branch')!;
}
