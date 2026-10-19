import { View, Text, Node } from 'std:ui';
import { Kbd, KbdGroup } from '../components/ui/kbd';

export function KeyboardShortcuts(): Node {
  return (
    <View className="gap-4 p-1">
      <View className="flex-row items-center gap-3"><Text>Command palette</Text><KbdGroup><Kbd>Ctrl</Kbd><Text>+</Text><Kbd>K</Kbd></KbdGroup></View>
      <View className="flex-row items-center gap-3"><Text>Close</Text><Kbd>Esc</Kbd></View>
    </View>
  );
}
