import { Node, View, Text, Button, signal } from 'std:ui';

interface MaybeProps {
  visible: boolean;
}

/** The condition selects initial content; dynamic branch replacement is separate. */
function MaybeCounter(props: MaybeProps): Node | null {
  const count = signal(1);
  const result = props.visible ? <Button onPress={() => { count.set(count.get() + 1); }}>Count: {count}</Button> : null;
  return result;
}

function MissingWithState(): null {
  const idle = signal('not mounted');
  return null;
}

function NoValueWithState(): undefined {
  const idle = signal(false);
  const absent = undefined;
  return absent;
}

function optionalText(show: boolean): string | null {
  return show ? 'shown' : null;
}

export function Page(): Node {
  return (
    <View className="p-6 gap-4">
      <Text as="h1">Nullable children</Text>
      <View data-slot="flow">Before<MaybeCounter visible={false}/>Between{null}<MissingWithState/>{undefined}<MaybeCounter visible/>After<NoValueWithState/><MaybeCounter visible/></View>
      <Text data-slot="text">A{null}B{undefined}{optionalText(false)}C</Text>
      <Button>{null}Ready{undefined}</Button>
    </View>
  );
}
