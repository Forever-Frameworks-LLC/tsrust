import { View, Text, Button, Node, signal, run } from 'std:ui';

function checked(value: int): string {
  if (value > 1) {
    throw 'Computed text failed';
  }

  return String(value);
}

function Broken(): Node {
  const count = signal(1);
  return (<View><Text>{checked(count.get())}</Text><Button aria-label="Break text" onPress={() => { count.set(count.get() + 1); }}>Break</Button></View>);
}

function Healthy(): Node {
  const count = signal(10);
  return (<View><Text data-slot="healthy">{count.get() + 1}</Text><Button aria-label="Healthy text" onPress={() => { count.set(count.get() + 1); }}>Healthy</Button></View>);
}

function main() {
  unwrap(run(Broken(), 'Broken'));
  unwrap(run(Healthy(), 'Healthy'));
}
