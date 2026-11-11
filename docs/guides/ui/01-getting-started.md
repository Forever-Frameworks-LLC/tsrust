# Getting Started

Tsrust compiles `.tsrx` files, TypeScript with TSRX component markup, into a native desktop
application. The window is drawn by GPUI and the components come from GPUI
Kit, so a button, a table or a date picker is a real Rust widget, not a web
view. You write components like React or Solid; the compiler checks
ownership and emits Rust.

## Create an app

```bash
tsrust new my-app --ui
cd my-app
tsrust dev
```

`tsrust dev` builds the app, opens its window and rebuilds when you save a
file. The project holds `src/main.tsrx`, `src/locales/*.json` for
translations, `icon.png` and `tsrust.config.json`.

## Your first component

A component is a function that returns a `Node`. Import components from
`std:ui`, keep state in signals and pass handlers as props.

```tsx live Counter
function Counter(): Node {
  const count = signal(0);
  return (
    <View className="flex-row items-center gap-3">
      <Button variant="primary" onPress={() => { count.set(count.get() + 1); }}>Add one</Button>
      <Text>Clicked {count.get()} times</Text>
    </View>
  );
}
```

`main` opens the window with the root component:

```tsx
function main() {
  unwrap(run(<Counter />, 'Counter', { width: 480, height: 320 }));
}
```

## Add a component to your project

Every component is available from `std:ui`. To change one, copy its wrapper
into your project and import it from there:

```bash
tsrust ui add Button
```

`tsrust ui list` shows every component, and `tsrust ui add --all` copies them
all.

## Package the app

```bash
tsrust package src/main.tsrx --name "My App" --version 1.0.0 --icon icon.png
```

This writes a `.app` bundle with a zip and DMG on macOS. See Packaging for
signing, releases and self-update.
