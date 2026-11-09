import { View, Text, Button, Node, signal, run, appInfo, checkForUpdates, installUpdate, relaunch } from 'std:ui';

function App(): Node {
  const info = appInfo();
  const status = signal('Not checked');
  const ready = signal(false);
  return (
    <View className="flex-col size-full items-center justify-center gap-3">
      <Text>{`${info.name} ${info.version} (${info.repo})`}</Text>
      <Text>{status}</Text>
      <Button onPress={() => { status.set('Checking...'); checkForUpdates((update) => { status.set(update.error !== '' ? `Error: ${update.error}` : update.available ? `Update ${update.latest} available` : `Up to date (${update.current})`); }); }}>Check for updates</Button>
      <Button onPress={() => { status.set('Installing...'); installUpdate((error) => { if (error === '') { ready.set(true); status.set('Installed; relaunch to finish'); } else { status.set(`Error: ${error}`); } }); }}>Install update</Button>
      <Button onPress={() => { relaunch(); }}>Relaunch</Button>
    </View>
  );
}

function main() {
  unwrap(run(App(), 'Updates', {
    width: 520, height: 360
  }));
}
