import { View, Text, Node, signal, run, Button, Dock, DockPanel, List } from 'std:ui';

function App(): Node {
  const clicks = signal(0);
  return (
    <View className="flex-col size-full">
      <Dock className="grow">
        <DockPanel title="Files" size={220}><List items={['main.tsrx', 'api.tsr', 'README.md']} className="h-48" /></DockPanel>
        <DockPanel title="Editor"><Text>Clicks {clicks}</Text><Button onPress={() => { clicks.set(clicks.get() + 1); }}>Click</Button></DockPanel>
        <DockPanel title="Console" size={260}><Text>Ready</Text></DockPanel>
      </Dock>
    </View>
  );
}

function main() {
  unwrap(run(App(), 'Dock', {
    width: 1000, height: 600
  }));
}
