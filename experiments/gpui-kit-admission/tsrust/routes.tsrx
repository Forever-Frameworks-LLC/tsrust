import { View, Text, Button, Node, run, Svg, Path, Route, Link, location, navigate, back, CopyButton } from 'std:ui';

function App(): Node {
  const path = location();
  return (
    <View className="flex-col size-full p-6 gap-3">
      <View className="flex-row gap-3 items-center">
        <Svg viewBox="0 0 24 24" width={48} height={48} stroke="primary" fill="none" strokeWidth={2}><Path d="M3 12h18M12 3v18" /><Path d="M5 5l14 14" stroke="destructive" /></Svg>
        <Svg viewBox="0 0 24 24" width={32} height={32} className="text-primary"><Path d="M12 2l3 7h7l-5.5 4.5 2 7.5-6.5-4.5-6.5 4.5 2-7.5L2 9h7z" /></Svg>
        <Text>Path {path}</Text>
      </View>
      <View className="flex-row gap-3">
        <Link href="/">Home</Link>
        <Link href="/docs">Docs</Link>
        <Button onPress={() => { navigate('/settings'); }}>Settings</Button>
        <Button onPress={() => { back(); }}>Back</Button>
        <CopyButton value="copied text" />
      </View>
      <Route path="/" render={() => <Text>Home page</Text>} />
      <Route path="/docs" render={() => <Text>Documentation page</Text>} />
      <Route path="/settings" render={() => <Text>Settings page</Text>} />
    </View>
  );
}

function main() {
  unwrap(run(App(), 'Routes', {
    width: 640, height: 400
  }));
}
