# Input and Zoom

Controls report what they mean (`onPress`, `onChange`). A `View` can also observe
raw input: keys, pointer buttons and movement, the wheel, focus and dropped files.
Handlers receive typed records, so `event.key` and `event.shiftKey` are checked.

## Keyboard

Key events reach a View while it or something inside it has keyboard focus. Give
the View `tabIndex={0}` to make it a focus stop of its own; Tab moves to it and a
click focuses it. `key` names the key (`a`, `enter`, `escape`, `left`, `f5`) and
`text` is what it would type, so Shift+A has key `a` and text `A`.

```tsx live KeyboardInput
function KeyboardInput(): Node {
  const last = signal('Click here, then type');
  const focused = signal(false);
  return (
    <View
      tabIndex={0}
      className="flex-col gap-2 w-96 p-4 rounded-lg border"
      onKeyDown={(event) => { last.set(`${event.key}${event.shiftKey ? ' with Shift' : ''}${event.repeat ? ' (repeat)' : ''}`); }}
      onFocus={() => { focused.set(true); }}
      onBlur={() => { focused.set(false); }}
    >
      <Text className="font-bold">{focused.get() ? 'Focused' : 'Not focused'}</Text>
      <Text>{last.get()}</Text>
    </View>
  );
}
```

`onKeyUp` reports releases. For app-wide commands such as save, prefer a
`Shortcut`: it works wherever focus is and shows its keys in menus and tooltips.

## Pointer and wheel

`onPointerDown`, `onPointerUp` and `onPointerMove` report the button (`left`,
`right`, `middle`, `back`, `forward`, or `none` while moving) and the position in
window pixels. `onDoubleClick`, `onContextMenu` and `onHover` cover the common
cases, and `onWheel` reports scroll amounts without stopping scrolling.

```tsx live PointerInput
function PointerInput(): Node {
  const status = signal('Move, click, right-click or scroll here');
  const inside = signal(false);
  return (
    <View
      className="flex-col gap-2 w-96 h-32 p-4 rounded-lg border"
      onPointerDown={(event) => { status.set(`${event.button} button at ${Math.round(event.x)}, ${Math.round(event.y)}`); }}
      onDoubleClick={(_event) => { status.set('Double click'); }}
      onContextMenu={(event) => { status.set(`Context menu at ${Math.round(event.x)}, ${Math.round(event.y)}`); }}
      onWheel={(event) => { status.set(`Wheel ${event.deltaY} (${event.deltaMode})`); }}
      onHover={(hovered) => { inside.set(hovered); }}
    >
      <Text className="font-bold">{inside.get() ? 'Pointer inside' : 'Pointer outside'}</Text>
      <Text>{status.get()}</Text>
    </View>
  );
}
```

`onFileDrop` receives the paths of files dropped from the system file manager.

## Zoom and text size

Sizes in class names are written in pixels at the default text size, and they
scale together: text, spacing and boxes grow with GPUI Kit's own components.
`zoomIn`, `zoomOut`, `resetZoom` and `setZoom(level)` change the zoom of every
window. The system text-size preference (Windows Settings > Accessibility > Text
size, GNOME text scaling) applies automatically and multiplies the zoom; macOS
has no system-wide text size for desktop apps, so zoom is the control there.

```tsx live ZoomControls
function ZoomControls(): Node {
  return (
    <View className="flex-row gap-2">
      <Shortcut command="zoom-in" keys="secondary-=" onTrigger={() => { zoomIn(); }} />
      <Shortcut command="zoom-out" keys="secondary--" onTrigger={() => { zoomOut(); }} />
      <Shortcut command="actual-size" keys="secondary-0" onTrigger={() => { resetZoom(); }} />
      <Button tooltipCommand="zoom-out" onPress={() => { zoomOut(); }}>Smaller</Button>
      <Button tooltipCommand="actual-size" onPress={() => { resetZoom(); }}>Actual size</Button>
      <Button tooltipCommand="zoom-in" onPress={() => { zoomIn(); }}>Larger</Button>
    </View>
  );
}
```

Zoom steps follow browsers: 50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200,
250 and 300 percent. Pointer positions stay in window pixels at any zoom. Borders
stay one pixel wide.
