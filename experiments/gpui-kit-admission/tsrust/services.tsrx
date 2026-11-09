import { View, Text, Button, Node, signal, run, Icon, Animate, notify, setWindowTitle, openWindow, writeClipboard, readClipboard, pickFiles, systemAppearance, zoomWindow } from 'std:ui';

function Second(): Node {
  return <View className="flex-col size-full items-center justify-center"><Text>Second window</Text></View>;
}

function App(): Node {
  const pasted = signal('');
  const picked = signal('');
  const clicks = signal(0);
  return (
    <View className="flex-col size-full items-center justify-center gap-3">
      <View className="flex-row gap-2 items-center"><Icon name="search" /><Icon name="bell" size="lg" /><Text>Appearance: {systemAppearance()}</Text></View>
      <Animate kind="slide-up" duration={600} key={`${clicks.get()}`}><Text>Animated {clicks}</Text></Animate>
      <Animate kind="pulse" duration={1200} repeat={true}><Text>Pulsing</Text></Animate>
      <Button onPress={() => { clicks.set(clicks.get() + 1); notify('Saved', { title: 'Done', variant: 'success', delivery: 'both' }); }}>Notify</Button>
      <Button onPress={() => { setWindowTitle(`Clicked ${clicks.get()}`); }}>Rename window</Button>
      <Button onPress={() => { writeClipboard('from tsrust'); readClipboard((text) => { pasted.set(clone(text)); }); }}>Clipboard</Button>
      <Text>Clipboard: {pasted}</Text>
      <Button onPress={() => { openWindow(<Second />, 'Second', { width: 320, height: 200 }); }}>New window</Button>
      <Button onPress={() => { pickFiles((paths) => { picked.set(clone(paths)); }); }}>Pick file</Button>
      <Text>Picked: {picked}</Text>
    </View>
  );
}

function main() {
  unwrap(run(App(), 'Services', {
    width: 520, height: 560
  }));
}
