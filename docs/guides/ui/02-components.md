# Components

Components follow shadcn/ui's composition style: a component's text and
parts are its children, not props.

## Text goes in children

Text children fill a component's label. A `Text` with a variant styles part
of it.

```tsx live LabelledFields
function LabelledFields(): Node {
  return (
    <View className="flex-col gap-3 w-80">
      <Label>Company address <Text variant="secondary">(optional)</Text></Label>
      <Input placeholder="1 Infinite Loop" />
      <Checkbox defaultChecked={true}>Email me receipts</Checkbox>
    </View>
  );
}
```

## Compose parts

Larger components are built from named parts, such as `DialogTrigger` and
`DialogContent`, or `TableHeader` and `TableRow`. Each part takes its own
`className`.

```tsx live ComposedDialog
function ComposedDialog(): Node {
  return (
    <Dialog>
      <DialogTrigger><Button outline={true}>Edit profile</Button></DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit profile</DialogTitle>
          <DialogDescription>Changes save when you press Save.</DialogDescription>
        </DialogHeader>
        <Input placeholder="Name" />
        <DialogFooter>
          <DialogClose><Button outline={true}>Cancel</Button></DialogClose>
          <DialogClose><Button variant="primary">Save</Button></DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

## Your own components

Write a props interface and a function. `children?: Child[]` accepts nested
content, which you pass on to a container.

```tsx live CardExample
interface CardProps {
  title: string;
  children?: Child[];
}

function Card(props: CardProps): Node {
  return (
    <View className="flex-col gap-2 rounded-lg border border-border p-4 w-72">
      <Text className="font-semibold">{props.title}</Text>
      <View className="flex-col gap-1" children={props.children ?? []} />
    </View>
  );
}

function CardExample(): Node {
  return <Card title="Storage"><Progress value={64} /><Text variant="muted">64 GB of 100 GB used</Text></Card>;
}
```

## Icons

`icon` props and `<Icon name="..." />` take any of the 1,830 Lucide icon names
GPUI Kit ships, such as `search`, `settings` or `file-text`. The Icon page of the
GPUI starter app (`apps/gpui-starter`) lists them.

## Native terminal

`Terminal` opens a real PTY shell on its first visible native mount. Its `directory`
is read at mount; omit it to use the application's current directory. `fontSize`
accepts 8–40 logical pixels and resizes the shell's grid using measured monospace
font metrics. Give each session a unique `id` within the window.

```tsrx
<Terminal id="shell" directory="/tmp" fontSize={14} className="size-full" />
```

Use `readTerminal` for a copied snapshot of the visible grid, selection, dimensions,
exit status and session generation. `controlTerminal` addresses that same session:
`focus`, `write`, `paste`, `interrupt`, `copy`, `scroll`, `select`, `close` or `restart`.
Both APIs reply through a callback on the next frame. A successful action reply
means delivery, not completion of a shell command. Supply `expectedGeneration`
from a snapshot to reject actions aimed at a replaced session. `select` takes
an inclusive zero-based viewport range in `selection`:
`[startRow, startColumn, endRow, endColumn]`.

Hiding the component preserves its shell and scrollback. `interrupt` sends Ctrl+C;
`close` force-terminates the foreground job and shell. `restart` is accepted only
after the shell has exited and starts a fresh grid with a new generation. Unmounting
closes the owned session; application quit waits for its cleanup. Agent hosts must
enforce grants and isolation before sending input or terminating a session.

ANSI colors, local selection/copy, bracketed paste and committed Unicode input are
native capabilities. Clipboard escape sequences cannot access the system clipboard.
Dynamic OSC palette overrides are drawn, but their query replies currently use the
theme defaults. The HTML renderer displays an explicit unsupported message and
never starts a shell. Windows lifecycle and full IME interaction remain unverified.
