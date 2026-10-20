import { View, Text, Button, Link, Node, signal } from 'std:ui';
import { Badge } from '../components/ui/badge';

/** Six palettes, live props, and replacement controls share the same editable component. */
export function BadgeVariants(): Node {
  const destructive = signal(false);
  const neutral = signal(false);
  const count = signal(0);
  return (
    <View className="gap-6" data-slot="badge-gallery">
      <View className="flex-row flex-wrap gap-2 items-center">
        <Badge>Default</Badge><Badge variant="secondary">Secondary</Badge><Badge variant="destructive">Destructive</Badge>
        <Badge variant="outline">Outline</Badge><Badge variant="ghost">Ghost</Badge><Badge variant="link">Link</Badge>
      </View>
      <View className="border-t pt-4 gap-3">
        <Text className="text-sm text-muted-foreground">One node, live palette</Text>
        <Badge id="badge-live" variant={neutral.get() ? null : destructive.get() ? 'destructive' : 'default'} className={destructive.get() ? 'px-3' : 'px-2'}>Build {count.get()}</Badge>
        <View className="flex-row flex-wrap gap-2">
          <Button className="rounded border px-3 py-2 text-sm" onPress={() => { destructive.set(!destructive.get()); }}>Change badge palette</Button>
          <Button className="rounded border px-3 py-2 text-sm" onPress={() => { neutral.set(!neutral.get()); }}>Toggle neutral badge</Button>
        </View>
      </View>
      <View className="border-t pt-4 gap-3">
        <Text className="text-sm text-muted-foreground">Replacement controls</Text>
        <View className="flex-row flex-wrap gap-3 items-center">
          <Badge variant="outline" render={(props) => (<Link className={props.className} data-slot={props['data-slot']} data-variant={props['data-variant']} href="/docs/ui/components/badge">Read the Badge API</Link>)} />
          <Badge variant="secondary" render={(props) => (<Button className={props.className} data-slot={props['data-slot']} data-variant={props['data-variant']} onPress={() => { count.set(count.get() + 1); }}>Add build</Button>)} />
        </View>
      </View>
      <View className="border-t pt-4 gap-3">
        <Text className="text-sm text-muted-foreground">Caller styling</Text>
        <View className="flex-row flex-wrap gap-2 items-center"><Badge className="rounded-sm">Small corners</Badge><Badge variant="outline" className="h-auto whitespace-normal max-w-48">Long labels can wrap when you opt in.</Badge></View>
      </View>
    </View>
  );
}
