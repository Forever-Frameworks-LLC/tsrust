import { View, Text, Button, Input, Node, LazyChildren, signal } from 'std:ui';

interface DeferredPanelProps {
  children?: LazyChildren;
}

/** The factory survives a close; each opening creates fresh owned content. */
function DeferredPanel(props: DeferredPanelProps): Node {
  const open = signal(false);
  const render = props.children;
  return (
    <View className="rounded-lg border bg-card p-4 gap-4">
      <Button id="lazy-toggle" className="rounded border px-3 py-2 self-start" onPress={() => { open.set(!open.get()); }}>{open.get() ? 'Close details' : 'Open details'}</Button>
      {open.get() ? <View id="lazy-content" className="border-t pt-4 gap-3" children={render === undefined ? [] : render()}/> : null}
    </View>
  );
}

function DetailCounter(): Node {
  const count = signal(0);
  return (
    <View className="gap-3">
      <Text id="lazy-child-count">Detail count: {count.get()}</Text>
      <Button id="lazy-increment-child" className="rounded bg-primary text-primary-foreground px-3 py-2 self-start" onPress={() => { count.set(count.get() + 1); }}>Increment detail</Button>
    </View>
  );
}

export function LazyChildrenExample(): Node {
  const parent = signal(0);
  return (
    <View className="gap-4">
      <Text id="lazy-parent-count">Parent count: {parent.get()}</Text>
      <Button id="lazy-increment-parent" className="rounded border px-3 py-2 self-start" onPress={() => { parent.set(parent.get() + 1); }}>Increment parent</Button>
      <Input id="lazy-draft" aria-label="Retained parent draft" placeholder="This draft survives closing details" className="w-full rounded border px-3 py-2"/>
      <DeferredPanel><DetailCounter/></DeferredPanel>
    </View>
  );
}
