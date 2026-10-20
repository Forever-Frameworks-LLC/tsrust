import { View, Button, Text, Node, signal, computed, useProp, ReactiveValue, ViewProps } from 'std:ui';

interface NoticeProps extends ViewProps<ReactiveValue<string>, ReactiveValue<string>> {
  tone?: ReactiveValue<'default' | 'destructive'>;
}

function Notice(props: NoticeProps): Node {
  const tone = useProp<'default' | 'destructive'>(props.tone ?? 'default');
  const classes = useProp<string>(props.className ?? '');
  props.tone = undefined;
  props.className = undefined;
  return (
    <View {...props} data-slot="notice" data-variant={tone.get()}
      className={'rounded border p-4 ' + (tone.get() === 'destructive' ? 'text-destructive ' : 'text-foreground ') + classes.get()} />
  );
}

export function Page(): Node {
  const active = signal(false);
  const count = signal(0);
  const doubled = computed<int>(() => count.get() * 2);
  return (
    <View className="flex flex-col gap-4 p-6">
      <Text as="h1" className="text-xl">Reactive props and composition</Text>
      <Notice tone={active.get() ? 'destructive' : 'default'} className={active.get() ? 'bg-muted' : 'bg-background'}>
        <Text>Count: {count.get()} / doubled: {doubled.get()}</Text>
      </Notice>
      <Button onPress={() => { active.set(!active.get()); }} className="rounded bg-primary text-primary-foreground p-2">Toggle tone</Button>
      <Button onPress={() => { count.set(count.get() + 1); }} className="rounded bg-secondary text-secondary-foreground p-2"
        render={(props) => (<Button {...props} data-slot="replacement" />)}>Increment</Button>
    </View>
  );
}
