# Navigation and Windows

## Routes

`location()` holds the current path and `navigate(path)` changes it. A
`Route` renders its page on the first visit and keeps it, with its state and
scroll position, while you are elsewhere.

```tsx
function App(): Node {
  const path = location();
  return (
    <View className="flex-row size-full">
      <Sidebar>
        <SidebarGroup label="Pages">
          <SidebarItem active={path.get() === '/'} onClick={() => { navigate('/'); }}>Home</SidebarItem>
          <SidebarItem active={path.get() === '/settings'} onClick={() => { navigate('/settings'); }}>Settings</SidebarItem>
        </SidebarGroup>
      </Sidebar>
      <Route path="/" render={() => <Home />} />
      <Route path="/settings" render={() => <Settings />} />
    </View>
  );
}
```

## Windows

`run(node, title, options)` opens the main window; `width`, `height`,
`minWidth`, `minHeight`, `fonts` and `titlebar: 'custom'` (with a `TitleBar`
component) configure it. `openWindow(node, title)` opens another window that
can own independent state. Signals created inside the window component belong to
that window. Passing a signal into another window shares its value; its owning
component must stay alive while the other window uses it.

## Notifications and shortcuts

```tsx live Feedback
function Feedback(): Node {
  const saves = signal(0);
  return (
    <View className="flex-row items-center gap-3">
      <Shortcut command="save-doc" keys="secondary-s" onTrigger={() => { saves.set(saves.get() + 1); notify('Saved', { variant: 'success' }); }} />
      <Button onPress={() => { notify('Upload finished', { title: 'Files', variant: 'info' }); }}>Show notification</Button>
      <Kbd>secondary-s</Kbd>
      <Text variant="muted">saves: {saves.get()}</Text>
    </View>
  );
}
```

`secondary` is Cmd on macOS and Ctrl elsewhere. The clipboard, file dialogs
and URLs have functions too: `writeClipboard`, `readClipboard`, `pickFiles`,
`saveFile` and `openUrl`.


## Path parameters

A route pattern can name one nonempty path segment with `:name`, for example
`/users/:id`. Names are unique ASCII identifiers. Use
`routeParam(path.get(), '/users/:id', 'id')` to read the value; the result is an
empty string when the full pattern does not match or the name is absent. Heading
fragments do not affect matching. Paths remain checked local paths: queries,
percent escapes and traversal throw.

A retained route constructs its page once, including when a parameter changes.
Read the location signal in displayed expressions to follow those changes.

## Independent window state

Use `createLocation(initialPath)` before constructing an additional window's
routes and links. Its navigation/history and component signals belong to that
window. Commands such as `closeWindow()` and notifications go to the window whose
handler issued them. Theme changes still apply to the whole application.

```tsx live IndependentWindows
function WindowCounter(): Node {
  const path = createLocation('/counter/first');
  const count = signal(0);
  return (
    <View className="flex-col gap-4 p-6 size-full">
      <Text className="text-xl font-semibold">Independent window</Text>
      <Text>Count: {count}</Text>
      <Text>Route: {routeParam(path.get(), '/counter/:name', 'name')}</Text>
      <Button onPress={() => { count.set(count.get() + 1); }}>Add one</Button>
      <Button onPress={() => { navigate('/counter/second'); }}>Next route</Button>
      <Button onPress={() => { back(); }}>Back</Button>
      <Button onPress={() => { notify('This window received the notification.'); }}>Notify this window</Button>
      <Button onPress={() => { closeWindow(); }}>Close this window</Button>
    </View>
  );
}

function IndependentWindows(): Node {
  return <Button onPress={() => { openWindow(WindowCounter(), 'Independent counter', { width: 480, height: 560, minHeight: 480 }); }}>Open an independent counter</Button>;
}
```

Open two counters: each starts at zero and `/counter/first`. Incrementing or
navigating one leaves the other unchanged. Closing one keeps the other and this
gallery open.

## Shared window state

Pass a signal explicitly to share it. Writes redraw every live window reading
that signal. The component that created it remains its owner; closing a window
that only received the signal leaves the owner alive.

```tsx live SharedWindows
function SharedCounter(count: Signal<int>): Node {
  const path = createLocation('/');
  return (
    <View className="flex-col gap-4 p-6">
      <Text>Shared count: {count.get()}</Text>
      <Button onPress={() => { count.set(count.get() + 1); }}>Increment shared count</Button>
      <Button onPress={() => { closeWindow(); }}>Close shared counter</Button>
    </View>
  );
}

function SharedWindows(): Node {
  const count = signal(0);
  return (
    <View className="flex-col gap-3">
      <Text>Shared count: {count.get()}</Text>
      <Button onPress={() => { count.set(count.get() + 1); }}>Increment shared owner</Button>
      <Button onPress={() => { openWindow(SharedCounter(count), 'Shared counter', { width: 480, height: 400 }); }}>Open shared counter</Button>
    </View>
  );
}
```
