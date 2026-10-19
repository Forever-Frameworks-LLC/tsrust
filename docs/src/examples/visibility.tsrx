import { View, Text, Button, Node, signal } from 'std:ui';

export function Visibility(): Node {
  const visible = signal(false);
  const notifications = signal(0);
  const hidden = signal(false);
  return (
    <View className="gap-4">
      <Text id="visibility-state">Visible: {visible.get() ? 'yes' : 'no'}; notifications: {notifications}</Text>
      <Button onPress={() => { hidden.set(!hidden.get()); }}>Toggle observed view</Button>
      <View id="visibility-pane" role="region" aria-label="Visibility example" tabIndex={0} className="h-24 overflow-auto border rounded">
        <View className="h-32 shrink-0"><Text>Scroll to the observed view.</Text></View>
        <View id="visibility-target" hidden={hidden.get()} className="h-16 shrink-0 border-t p-3" onVisibilityChange={(next) => { visible.set(next); notifications.set(notifications.get() + 1); }}><Text>Observed view</Text></View>
        <View className="h-32 shrink-0"/>
      </View>
    </View>
  );
}
