import { View, Text, CodeEditor, Button, Toolbar, StatusBar, StatusBarLeft, Node, signal, run, setTheme, controlInput } from 'std:ui';

function App(): Node {
  const text = signal('// Tsrust Studio\n// This buffer is not saved to disk.\n\nfunction main() {\n  console.log("Hello, Studio");\n}\n');
  const dark = signal(true);
  return (
    <View className="flex-col size-full">
      <Toolbar size="small">
        <Button onPress={() => { controlInput('source', 'openSearch'); }}>Find in file</Button>
        <Button onPress={() => { controlInput('source', 'openReplace'); }}>Replace</Button>
        <Button onPress={() => { dark.set(!dark.get()); setTheme(dark.get() ? 'dark' : 'light'); }}>{dark.get() ? 'Light theme' : 'Dark theme'}</Button>
      </Toolbar>
      <CodeEditor id="source" language="tsrust" value={text.get()} onChange={(value) => {
        if (text.get() !== value) {
          text.set(clone(value));
        }
      }} accessibilityLabel="Source editor" className="grow min-h-0" />
      <StatusBar>
        <StatusBarLeft><Text>Unsaved scratch buffer</Text></StatusBarLeft>
      </StatusBar>
    </View>
  );
}

function main() {
  setTheme('dark');
  unwrap(run(App(), 'Tsrust Studio', {
    width: 1120, height: 780, minWidth: 720, minHeight: 520
  }));
}
