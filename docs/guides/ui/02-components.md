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
GPUI Kit ships, such as `search`, `settings` or `file-text`. The gallery's
Icon page lists them.
