import { View, Text, Node } from 'std:ui';

// One list row: the label and its length in a muted second column.
export function Row(label: ReadOnly<string>): Node {
  return (
    <View className="flex flex-row justify-between px-3 py-1 border-b border-border">
      <Text className="text-sm">
        {clone(label)}
      </Text>
      <Text className="text-sm text-muted-foreground">
        {label.length.toString() + ' chars'}
      </Text>
    </View>
  );
}
