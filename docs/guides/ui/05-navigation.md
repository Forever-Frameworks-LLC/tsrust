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
shares your signals.

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
